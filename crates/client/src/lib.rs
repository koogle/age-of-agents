//! Age of Agents client: one wgpu renderer for a native window (`cargo run`,
//! simulation in-process) and WebGL2 in the browser (hosted server, or
//! `?local` to simulate in the page).
mod assets;
mod camera;
mod hud;
mod render;
mod source;
mod terrain;
mod view;

use std::collections::VecDeque;
use std::sync::Arc;

use aoa_game::{CellCoordinate, Command, WorldSnapshot};
use glam::{Vec2, Vec3};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use assets::{Assets, Rgba};
use camera::Rig;
use render::{Globals, Gpu, Renderer};
use source::{CommandResult, Source};
use view::{Selection, Sheets, WorldView};

const DRAG_THRESHOLD: f32 = 8.0;

// Last frame's camera, for the browser test hook below.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static LAST_VIEW: std::cell::RefCell<(glam::Mat4, f32, f32, usize, usize)> = const { std::cell::RefCell::new((glam::Mat4::IDENTITY, 1.0, 1.0, 0, 0)) };
}

/// Test hook: screen pixel (CSS px) of a world point, plus selection counts.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn debug_screen_of(x: f32, y: f32, z: f32) -> Vec<f32> {
    LAST_VIEW.with(|view| {
        let (matrix, width, height, units, building) = *view.borrow();
        let clip = matrix * glam::Vec4::new(x, y, z, 1.0);
        let scale = web_sys::window()
            .map(|w| w.device_pixel_ratio() as f32)
            .unwrap_or(1.0);
        vec![
            (clip.x / clip.w + 1.0) * 0.5 * width / scale,
            (1.0 - clip.y / clip.w) * 0.5 * height / scale,
            units as f32,
            building as f32,
        ]
    })
}

struct Pointer {
    on_hud: bool,
    down_at: Vec2,
    last: Vec2,
    button: MouseButton,
    grabbed: Option<Vec3>,
    dragging: bool,
}

struct Game {
    gpu: Gpu,
    renderer: Renderer,
    window: Arc<Window>,
}

pub struct App {
    assets: Option<Assets>,
    sheets: Sheets,
    atlas: hud::Atlas,
    hud: hud::Hud,
    build_mode: bool,
    toast: Option<(String, f64)>,
    game: Option<Game>,
    proxy: Option<EventLoopProxy<Game>>,
    rig: Rig,
    view: WorldView,
    source: Source,
    selection: Selection,
    pointer: Option<Pointer>,
    cursor: Vec2,
    incoming: VecDeque<WorldSnapshot>,
    results: Vec<CommandResult>,
    clock: f64,
    last_frame: Option<f64>,
}

fn now_seconds() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now() / 1000.0)
            .unwrap_or(0.0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        static START: OnceLock<std::time::Instant> = OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
    }
}

impl App {
    fn new(assets: Assets, source: Source, proxy: EventLoopProxy<Game>) -> Self {
        let sheets = Sheets::parse(
            assets.bytes("sprites/villager.json"),
            assets.bytes("sprites/resources.json"),
            assets.bytes("sprites/towncenter.json"),
        );
        let atlas = hud::build_atlas(&assets);
        Self {
            assets: Some(assets),
            sheets,
            atlas,
            hud: hud::Hud::new(),
            build_mode: false,
            toast: None,
            game: None,
            proxy: Some(proxy),
            rig: Rig::new(),
            view: WorldView::new(),
            source,
            selection: Selection::default(),
            pointer: None,
            cursor: Vec2::ZERO,
            incoming: VecDeque::new(),
            results: Vec::new(),
            clock: 0.0,
            last_frame: None,
        }
    }

    fn send(&mut self, command: Command) {
        self.source.send(command, &mut self.results);
    }

    fn placement(&self, pixel: Vec2) -> Option<(CellCoordinate, bool)> {
        let snapshot = self.view.snapshot.as_ref()?;
        let point = self.rig.ground_at(pixel)?;
        let (column, row) = ((point.x - 0.5).floor(), (point.z - 0.5).floor());
        if column < 0.0 || row < 0.0 || column + 2.0 > terrain::COLUMNS || row + 2.0 > terrain::ROWS
        {
            return None;
        }
        let origin = CellCoordinate {
            column: column as u16,
            row: row as u16,
        };
        let covers = |c: CellCoordinate| {
            (origin.column..origin.column + 2).contains(&c.column)
                && (origin.row..origin.row + 2).contains(&c.row)
        };
        let blocked = snapshot
            .resources
            .iter()
            .any(|r| r.amount > 0.0 && covers(r.cell))
            || snapshot.units.iter().any(|u| covers(u.unit.cell))
            || snapshot.buildings.iter().any(|b| {
                let o = b.building.origin;
                o.column < origin.column + 2
                    && origin.column < o.column + b.columns
                    && o.row < origin.row + 2
                    && origin.row < o.row + b.rows
            })
            || (0..4).any(|i| {
                let cell = &snapshot.terrain[(origin.row + i / 2) as usize
                    * snapshot.columns as usize
                    + (origin.column + i % 2) as usize];
                cell.visibility == aoa_game::CellVisibility::Unseen
            });
        Some((origin, !blocked))
    }

    fn act(&mut self, action: hud::Action) {
        match action {
            hud::Action::Speed(multiplier) => self.send(Command::SetSimulationSpeed { multiplier }),
            hud::Action::Build => self.build_mode = true,
            hud::Action::Cancel => self.build_mode = false,
            hud::Action::Train => {
                if let Some(building_id) = self.selection.building.clone() {
                    self.send(Command::Produce {
                        building_id,
                        product: aoa_game::ProductKind::Villager,
                    });
                }
            }
            hud::Action::Research(technology) => {
                if let Some(building_id) = self.selection.building.clone() {
                    self.send(Command::Research {
                        building_id,
                        technology,
                    });
                }
            }
            hud::Action::LookAt(point) => self.rig.look_at(point.x, point.y),
        }
    }

    fn tap(&mut self, pixel: Vec2, additive: bool) {
        if self.build_mode {
            if let (Some((origin, _)), Some(unit_id)) =
                (self.placement(pixel), self.selection.units.first().cloned())
            {
                self.send(Command::Build { unit_id, origin });
                self.build_mode = false;
            }
            return;
        }
        let Some(snapshot) = self.view.snapshot.as_ref() else {
            return;
        };
        if let Some(id) = self.view.unit_at(&self.rig, pixel) {
            if !additive {
                self.selection.units.clear();
            }
            if !self.selection.units.contains(&id) {
                self.selection.units.push(id);
            }
            self.selection.building = None;
            return;
        }
        let Some(point) = self.rig.ground_at(pixel) else {
            return;
        };
        if point.x < 0.0 || point.z < 0.0 || point.x >= terrain::COLUMNS || point.z >= terrain::ROWS
        {
            return;
        }
        let cell = CellCoordinate {
            column: point.x as u16,
            row: point.z as u16,
        };
        let resource = snapshot
            .resources
            .iter()
            .find(|r| r.amount > 0.0 && r.cell == cell)
            .map(|r| r.id.clone());
        let building = snapshot
            .buildings
            .iter()
            .find(|b| {
                let o = b.building.origin;
                (o.column..o.column + b.columns).contains(&cell.column)
                    && (o.row..o.row + b.rows).contains(&cell.row)
            })
            .map(|b| (b.building.id.clone(), b.building.construction.is_some()));
        let units = self.selection.units.clone();
        if units.is_empty() {
            self.selection.building = building.map(|(id, _)| id);
            return;
        }
        if let Some(resource_id) = resource {
            for unit_id in units {
                self.send(Command::Gather {
                    unit_id,
                    resource_id: resource_id.clone(),
                });
            }
        } else if let Some((building_id, true)) = building {
            for unit_id in units {
                self.send(Command::Construct {
                    unit_id,
                    building_id: building_id.clone(),
                });
            }
        } else if let Some((building_id, false)) = building {
            self.selection.units.clear();
            self.selection.building = Some(building_id);
        } else if units.len() == 1 {
            self.send(Command::Move {
                unit_id: units[0].clone(),
                to: cell,
            });
        } else {
            self.send(Command::GroupMove {
                unit_ids: units,
                to: cell,
            });
        }
    }

    fn key(&mut self, key: &Key) {
        match key.as_ref() {
            Key::Character("q") | Key::Character("Q") => {
                self.rig.rotate(-std::f32::consts::FRAC_PI_4, true)
            }
            Key::Character("e") | Key::Character("E") => {
                self.rig.rotate(std::f32::consts::FRAC_PI_4, true)
            }
            Key::Character("w") | Key::Named(NamedKey::ArrowUp) => self.rig.nudge(0.0, 1.0),
            Key::Character("s") | Key::Named(NamedKey::ArrowDown) => self.rig.nudge(0.0, -1.0),
            Key::Character("a") | Key::Named(NamedKey::ArrowLeft) => self.rig.nudge(-1.0, 0.0),
            Key::Character("d") | Key::Named(NamedKey::ArrowRight) => self.rig.nudge(1.0, 0.0),
            Key::Character("t") => {
                if let Some(building_id) = self.selection.building.clone() {
                    self.send(Command::Produce {
                        building_id,
                        product: aoa_game::ProductKind::Villager,
                    });
                }
            }
            Key::Character("0") => self.send(Command::SetSimulationSpeed { multiplier: 0.0 }),
            Key::Character("1") => self.send(Command::SetSimulationSpeed { multiplier: 1.0 }),
            Key::Character("2") => self.send(Command::SetSimulationSpeed { multiplier: 2.0 }),
            Key::Named(NamedKey::Escape) if self.build_mode => self.build_mode = false,
            Key::Named(NamedKey::Escape) => self.selection = Selection::default(),
            _ => {}
        }
    }

    fn press(&mut self, pixel: Vec2, button: MouseButton) {
        let on_hud = button == MouseButton::Left && self.hud.press(pixel);
        self.pointer = Some(Pointer {
            on_hud,
            down_at: pixel,
            last: pixel,
            button,
            grabbed: self.rig.ground_at(pixel),
            dragging: false,
        });
    }

    fn moved(&mut self, pixel: Vec2) {
        self.cursor = pixel;
        self.hud.hover = Some(pixel);
        let Some(pointer) = self.pointer.as_mut() else {
            return;
        };
        if pointer.on_hud {
            return;
        }
        if pointer.down_at.distance(pixel) > DRAG_THRESHOLD {
            pointer.dragging = true;
        }
        if pointer.dragging {
            if pointer.button == MouseButton::Right {
                self.rig.rotate((pixel.x - pointer.last.x) * 0.008, false);
            } else if let (Some(grabbed), Some(now)) = (pointer.grabbed, self.rig.ground_at(pixel))
            {
                self.rig.drag(grabbed, now);
            }
        }
        pointer.last = pixel;
    }

    fn release(&mut self, pixel: Vec2, additive: bool) {
        let Some(pointer) = self.pointer.take() else {
            return;
        };
        if pointer.on_hud {
            if let Some(action) = self.hud.release() {
                self.act(action);
            }
        } else if !pointer.dragging && pointer.button == MouseButton::Left {
            self.tap(pixel, additive);
        }
    }

    fn redraw(&mut self) {
        let now = now_seconds();
        let dt = self
            .last_frame
            .map(|last| (now - last).clamp(0.0, 0.25))
            .unwrap_or(0.0);
        self.last_frame = Some(now);
        self.clock += dt;
        self.source.poll(dt, &mut self.incoming, &mut self.results);
        while let Some(snapshot) = self.incoming.pop_front() {
            self.view.sync(snapshot);
        }
        for result in std::mem::take(&mut self.results) {
            if let Err(error) = result {
                self.toast = Some((friendly(&error), now + 3.0));
            }
        }
        if self.toast.as_ref().is_some_and(|(_, until)| now > *until) {
            self.toast = None;
        }
        self.rig.update(dt as f32);
        self.view.frame(dt as f32);
        let ghost = if self.build_mode {
            self.placement(self.cursor)
        } else {
            None
        };
        let Some(game) = self.game.as_mut() else {
            return;
        };
        if std::mem::take(&mut self.view.cells_dirty)
            && let Some((rgba, layers)) = self.view.cell_data()
        {
            game.renderer.update_cells(&game.gpu.queue, &rgba, &layers);
        }
        let (right, up) = self.rig.basis();
        let eye = self.rig.eye();
        let sun = Vec3::new(-11.0, 15.0, 7.0).normalize();
        let globals = Globals {
            view_proj: self.rig.view_proj().to_cols_array_2d(),
            camera_right: right.extend(0.0).to_array(),
            camera_up: up.extend(0.0).to_array(),
            camera_pos: eye.extend(1.0).to_array(),
            sun_dir: sun.extend(0.0).to_array(),
            map_size: [terrain::COLUMNS, terrain::ROWS],
            time: self.clock as f32,
            curve: self.rig.curve(),
            curve_center: [self.rig.target.x, self.rig.target.z],
            fog_near: self.rig.distance + 8.0,
            fog_far: self.rig.distance * 2.0 + 60.0,
        };
        #[cfg(target_arch = "wasm32")]
        LAST_VIEW.with(|view| {
            *view.borrow_mut() = (
                self.rig.view_proj(),
                self.rig.width,
                self.rig.height,
                self.selection.units.len(),
                self.selection.building.is_some() as usize,
            )
        });
        let (mut sprites, mut decals) = self.view.draw_list(
            &self.sheets,
            &self.rig,
            self.clock as f32,
            dt as f32,
            &self.selection,
        );
        if let Some((origin, ok)) = ghost {
            let (x, z) = (origin.column as f32 + 1.0, origin.row as f32 + 1.0);
            let color = if ok {
                [0.42, 0.78, 0.38, 0.85]
            } else {
                [0.85, 0.3, 0.25, 0.85]
            };
            decals.push(render::Decal {
                center: [x, terrain::height_at(x, z) + 0.04, z],
                radius: 1.35,
                color,
                ring: 1.0,
            });
        }
        let model = hud::Model {
            snapshot: self.view.snapshot.as_ref(),
            units: &self.selection.units,
            building: self.selection.building.as_deref(),
            build_mode: self.build_mode,
            toast: self.toast.as_ref().map(|(text, _)| text.as_str()),
            camera: Vec2::new(self.rig.target.x, self.rig.target.z),
        };
        let scale = game.window.scale_factor() as f32;
        self.hud
            .layout(&self.atlas, &model, self.rig.width, self.rig.height, scale);
        game.renderer.render(
            &game.gpu,
            &globals,
            (camera::NEAR, camera::FAR),
            &mut sprites,
            &decals,
            &self.hud.quads,
        );
        game.window.request_redraw();
    }
}

/// Plain-language versions of the server's rejection reasons.
fn friendly(error: &str) -> String {
    match error {
        "unit is busy" => "That villager is busy with its current task.".into(),
        "destination cell is occupied" => "Something already stands there.".into(),
        "target is unreachable" => "No path leads there.".into(),
        "build site is blocked or outside the world" => {
            "The town center needs a clear 2×2 site.".into()
        }
        "insufficient wood" => "You need 20 wood to build.".into(),
        "insufficient food" => "You need 50 food to train a villager.".into(),
        other => other.to_string(),
    }
}

fn sheet_images(assets: &Assets) -> Vec<Rgba> {
    vec![
        assets.image("sprites/villager.png"),
        assets.image("sprites/villager_woman.png"),
        assets.image("sprites/villager_elder.png"),
        assets.image("sprites/resources.png"),
        assets.image("sprites/towncenter.png"),
    ]
}

impl ApplicationHandler<Game> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.game.is_some() || self.proxy.is_none() {
            return;
        }
        #[allow(unused_mut)]
        let mut attributes = Window::default_attributes().with_title("Age of Agents");
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("world"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
            attributes = attributes.with_canvas(canvas);
        }
        let window = Arc::new(event_loop.create_window(attributes).expect("window"));
        let proxy = self.proxy.take().expect("proxy");
        let assets = self.assets.take().expect("assets");
        let atlas = self.atlas.image.clone();
        let size = window.inner_size();
        let ready = async move {
            let gpu = Gpu::new(window.clone(), size.width, size.height).await;
            let renderer = Renderer::new(&gpu, &assets, &sheet_images(&assets), &atlas);
            let _ = proxy.send_event(Game {
                gpu,
                renderer,
                window,
            });
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(ready);
        #[cfg(not(target_arch = "wasm32"))]
        pollster::block_on(ready);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, game: Game) {
        let size = game.window.inner_size();
        self.rig.width = size.width as f32;
        self.rig.height = size.height as f32;
        game.window.request_redraw();
        self.game = Some(game);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(game) = self.game.as_mut() {
                    game.gpu.resize(size.width, size.height);
                    game.renderer
                        .resize(&game.gpu.device, size.width.max(1), size.height.max(1));
                }
                self.rig.width = size.width.max(1) as f32;
                self.rig.height = size.height.max(1) as f32;
            }
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => {
                self.moved(Vec2::new(position.x as f32, position.y as f32))
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => self.press(self.cursor, button),
                ElementState::Released => self.release(self.cursor, false),
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 60.0,
                };
                self.rig.zoom((1.0 - lines * 0.1).clamp(0.5, 1.5));
            }
            WindowEvent::Touch(touch) => {
                let pixel = Vec2::new(touch.location.x as f32, touch.location.y as f32);
                match touch.phase {
                    TouchPhase::Started => {
                        self.cursor = pixel;
                        self.press(pixel, MouseButton::Left);
                    }
                    TouchPhase::Moved => self.moved(pixel),
                    TouchPhase::Ended => self.release(pixel, false),
                    TouchPhase::Cancelled => self.pointer = None,
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.key(&event.logical_key)
            }
            _ => {}
        }
    }
}

fn run(assets: Assets, source: Source) {
    let event_loop = EventLoop::<Game>::with_user_event()
        .build()
        .expect("event loop");
    let app = App::new(assets, source, event_loop.create_proxy());
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        event_loop.run_app(&mut app).expect("run");
    }
}

/// Native entry point: the simulation runs in-process.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() {
    env_logger::init();
    let assets = pollster::block_on(Assets::load());
    run(assets, Source::local());
}

/// Browser entry point: the hosted server, or the in-page simulation with `?local`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
    wasm_bindgen_futures::spawn_local(async {
        let assets = Assets::load().await;
        let search = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default();
        let source = if search.contains("local") {
            Source::local()
        } else {
            Source::Remote(source::remote::Remote::connect())
        };
        run(assets, source);
    });
}
