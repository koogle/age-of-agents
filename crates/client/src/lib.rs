//! Age of Agents client: one wgpu renderer for a native window (`cargo run`,
//! simulation in-process) and WebGL2 in the browser (hosted server, or
//! `?local` to simulate in the page).
mod assets;
mod camera;
mod feedback;
mod gestures;
mod gpu;
use gestures::Pointer;
mod hud;
mod islands;
mod placement;
mod render;
mod reset;
mod source;
mod storage;
mod terrain;
mod view;
mod window;
use window::{loaded, physical_size};

use std::collections::VecDeque;
use std::sync::Arc;

use aoa_game::{CellCoordinate, Command, WorldSnapshot};
use glam::{Vec2, Vec3};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use assets::{Assets, Rgba};
use camera::Rig;
use gpu::Gpu;
use render::{Globals, Renderer};
use source::Source;
use view::{Selection, Sheets, WorldView};

/// Hover markers: warm gold over something to work on, soft white over ground.
const HOVER_WORK: [f32; 4] = [0.98, 0.8, 0.32, 0.95];
const HOVER_GROUND: [f32; 4] = [1.0, 0.98, 0.9, 0.85];

/// What lies under a pointer.
enum Target {
    Ship(String),
    Unit(String),
    Resource(String),
    Foundation(String),
    Building(String),
    Ground(CellCoordinate),
}

// Last frame's camera, for the browser test hook below.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static LAST_VIEW: std::cell::RefCell<(glam::Mat4, f32, f32, usize, usize, f32, Vec2)> = const { std::cell::RefCell::new((glam::Mat4::IDENTITY, 1.0, 1.0, 0, 0, 0.0, Vec2::ZERO)) };
    static LAST_HEIGHTS: std::cell::RefCell<Option<terrain::Heights>> = const { std::cell::RefCell::new(None) };
}

/// Test hook: the drawn ground height at a world point.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn debug_height_at(x: f32, z: f32) -> f32 {
    LAST_HEIGHTS.with(|heights| heights.borrow().as_ref().map_or(0.0, |h| h.at(x, z)))
}

/// Test hook: screen pixel (CSS px) of a world point, plus selection counts.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn debug_screen_of(x: f32, y: f32, z: f32) -> Vec<f32> {
    LAST_VIEW.with(|view| {
        let (matrix, width, height, units, building, curve, center) = *view.borrow();
        let y = y - curve * (Vec2::new(x, z) - center).length_squared();
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
    /// The villager build menu, or the building being placed.
    build: hud::BuildUi,
    toast: Option<(String, f64)>,
    game: Option<Game>,
    proxy: Option<EventLoopProxy<Game>>,
    rig: Rig,
    view: WorldView,
    source: Source,
    selection: Selection,
    pointer: Option<Pointer>,
    cursor: Vec2,
    resource_island: usize,
    cargo_kind: aoa_game::ResourceKind,
    mouse_inside: bool,
    focused: bool,
    modifiers: ModifiersState,
    feedback: feedback::Feedback,
    incoming: VecDeque<WorldSnapshot>,
    clock: f64,
    last_frame: Option<f64>,
    /// Whether the camera has been moved to the starting town center yet.
    framed: bool,
    /// Whether the page's loading overlay has been dismissed.
    revealed: bool,
    show_grid: bool,
    /// When the ground mesh was last rebuilt (page seconds).
    ground_rebuilt_at: f64,
    /// Fingers currently down, by touch id.
    touches: Vec<(u64, Vec2)>,
    /// A two-finger pinch/twist is (or was, until every finger lifts) in progress,
    /// so lifting the last finger must not count as a tap.
    gesture: bool,
}

pub(crate) fn now_seconds() -> f64 {
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
            assets.bytes("sprites/villager_idle_hd.json"),
            assets.bytes("sprites/resources.json"),
            assets.bytes("sprites/towncenter.json"),
            assets.bytes("sprites/buildings_hd.json"),
            [
                "buildings_economy",
                "buildings_crafts",
                "buildings_civic",
                "units",
            ]
            .map(|name| assets.bytes(&format!("sprites/{name}.json"))),
            assets.bytes("sprites/villager_field_preparation.json"),
        );
        let atlas = hud::build_atlas(&assets);
        Self {
            assets: Some(assets),
            sheets,
            atlas,
            hud: hud::Hud::new(),
            build: hud::BuildUi::Off,
            toast: None,
            game: None,
            proxy: Some(proxy),
            rig: Rig::new(),
            view: WorldView::new(),
            source,
            selection: Selection::default(),
            pointer: None,
            cursor: Vec2::ZERO,
            resource_island: 0,
            cargo_kind: aoa_game::ResourceKind::Wood,
            mouse_inside: false,
            focused: true,
            modifiers: ModifiersState::empty(),
            feedback: feedback::Feedback::default(),
            incoming: VecDeque::new(),
            clock: 0.0,
            last_frame: None,
            framed: false,
            revealed: false,
            show_grid: false,
            ground_rebuilt_at: f64::MIN,
            touches: Vec::new(),
            gesture: false,
        }
    }

    /// The terrain point under a screen pixel.
    fn ground_at(&self, pixel: Vec2) -> Option<Vec3> {
        let heights = &self.view.heights;
        self.rig.ground_at(pixel, |x, z| heights.at(x, z))
    }

    fn send(&mut self, command: Command) {
        self.source.send(command);
    }

    /// What a tap at `pixel` would land on.
    fn target_at(&self, pixel: Vec2) -> Option<Target> {
        let snapshot = self.view.snapshot.as_ref()?;
        if let Some(id) = self.view.unit_at(&self.rig, pixel) {
            return Some(Target::Unit(id));
        }
        // A tree, rock or building is hit where it is drawn, not where the
        // ground behind it happens to be.
        match self.view.sprite_at(&self.rig, pixel) {
            Some(view::Pick::Ship(id)) => return Some(Target::Ship(id)),
            Some(view::Pick::Resource(id)) => return Some(Target::Resource(id)),
            Some(view::Pick::Building(id)) => {
                let building = snapshot.buildings.iter().find(|b| b.building.id == id)?;
                return Some(if building.building.construction.is_some() {
                    Target::Foundation(id)
                } else {
                    Target::Building(id)
                });
            }
            None => {}
        }
        let point = self.ground_at(pixel)?;
        let cell = terrain::cell_at(
            point.x,
            point.z,
            self.view.heights.columns,
            self.view.heights.rows,
        )?;
        if let Some(resource) = snapshot
            .resources
            .iter()
            .find(|r| (r.amount > 0.0 || r.field.is_some()) && r.footprint().contains(cell))
        {
            return Some(Target::Resource(resource.id.clone()));
        }
        if let Some(building) = snapshot.buildings.iter().find(|b| {
            let o = b.building.origin;
            (o.column..o.column + b.columns).contains(&cell.column)
                && (o.row..o.row + b.rows).contains(&cell.row)
        }) {
            let id = building.building.id.clone();
            return Some(if building.building.construction.is_some() {
                Target::Foundation(id)
            } else {
                Target::Building(id)
            });
        }
        Some(Target::Ground(cell))
    }

    /// Ground marker under the cursor while villagers are selected: what a
    /// tap would order them to do.
    fn hover_decal(&self) -> Option<render::Decal> {
        if self.selection.units.is_empty()
            || self.build != hud::BuildUi::Off
            || self.hud.covers(self.cursor)
            || self.pointer.as_ref().is_some_and(|p| p.dragging)
        {
            return None;
        }
        let snapshot = self.view.snapshot.as_ref()?;
        let (center, radius, color) = match self.target_at(self.cursor)? {
            Target::Resource(id) => {
                let r = snapshot.resources.iter().find(|r| r.id == id)?;
                let c = r.footprint().center();
                (
                    Vec2::new(c.x as f32, c.y as f32) * terrain::CELL,
                    if r.field.is_some() { 0.85 } else { 0.36 },
                    HOVER_WORK,
                )
            }
            Target::Foundation(id) => {
                let building = snapshot.buildings.iter().find(|b| b.building.id == id)?;
                let c = view::footprint_center(&self.view.heights, building);
                (Vec2::new(c.x, c.z), 1.2, HOVER_WORK)
            }
            Target::Ground(cell) => (terrain::cell_center(cell), 0.24, HOVER_GROUND),
            Target::Building(id) if !self.carriers_for(&id).is_empty() => {
                let building = snapshot.buildings.iter().find(|b| b.building.id == id)?;
                let c = view::footprint_center(&self.view.heights, building);
                (
                    Vec2::new(c.x, c.z),
                    building.columns as f32 * terrain::CELL * 0.6,
                    HOVER_WORK,
                )
            }
            Target::Unit(_) | Target::Ship(_) | Target::Building(_) => return None,
        };
        Some(render::Decal {
            center: [
                center.x,
                self.view.heights.at(center.x, center.y) + 0.035,
                center.y,
            ],
            radius,
            color,
            ring: 1.0,
        })
    }

    fn act(&mut self, action: hud::Action) {
        match action {
            hud::Action::SelectShip(id) => self.select_storage_ship(id),
            hud::Action::CargoKind => self.cycle_cargo(),
            hud::Action::TransferCargo(direction, amount) => self.transfer_cargo(direction, amount),
            hud::Action::Voyage(island_id) => {
                if let Some(ship_id) = self.selection.ship.clone() {
                    self.send(Command::Voyage { ship_id, island_id });
                }
            }
            hud::Action::Disembark => {
                if let Some(ship_id) = self.selection.ship.clone() {
                    self.send(Command::Disembark { ship_id });
                }
            }
            hud::Action::Speed(multiplier) => self.send(Command::SetSimulationSpeed { multiplier }),
            hud::Action::Grid => self.show_grid = !self.show_grid,
            hud::Action::Build => self.build = hud::BuildUi::Categories,
            hud::Action::PlaceField => self.build = hud::BuildUi::PlacingField,
            hud::Action::BuildGroup(group) => self.build = hud::BuildUi::Group(group),
            hud::Action::Place(kind) => self.build = hud::BuildUi::Placing(kind),
            hud::Action::Cancel => self.build = hud::BuildUi::Off,
            hud::Action::Stop => self.stop(),
            hud::Action::Produce(product) => {
                if let Some(building_id) = self.selection.building.clone() {
                    self.send(Command::Produce {
                        building_id,
                        product,
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
            hud::Action::CancelQueuedJob(queue_id) => {
                if let Some(building_id) = self.selection.building.clone() {
                    self.send(Command::CancelQueuedJob {
                        building_id,
                        queue_id,
                    });
                }
            }
            hud::Action::LookAt(point) => self.rig.look_at(point.x, point.y),
            hud::Action::Explain(reason) => self.toast = Some((reason, now_seconds() + 3.0)),
            hud::Action::Reset => {
                if let Some(seed) = reset::choose_seed() {
                    self.source.reset(seed);
                    self.view = WorldView::new();
                    self.incoming.clear();
                    self.selection = Selection::default();
                    self.build = hud::BuildUi::Off;
                    self.framed = false;
                }
                // Native dialogs can block for a while; don't catch up that time.
                self.last_frame = None;
            }
        }
    }

    fn tap(&mut self, pixel: Vec2, additive: bool) {
        if let Some(kind) = match self.build {
            hud::BuildUi::Placing(kind) => Some(kind),
            hud::BuildUi::PlacingField => Some(aoa_game::BuildingKind::Farm),
            _ => None,
        } {
            if let (Some((origin, true)), Some(unit_id)) = (
                self.placement(pixel, kind),
                self.selection.units.first().cloned(),
            ) {
                self.send(if self.build == hud::BuildUi::PlacingField {
                    Command::PlantField { unit_id, origin }
                } else {
                    Command::Build {
                        unit_id,
                        origin,
                        kind,
                    }
                });
                self.build = hud::BuildUi::Off;
            } else {
                self.toast = Some((
                    "That spot is not clear for building.".into(),
                    now_seconds() + 3.0,
                ));
            }
            return;
        }
        // A tap on the world closes the build menu.
        self.build = hud::BuildUi::Off;
        let Some(target) = self.target_at(pixel) else {
            return;
        };
        if let Target::Ship(id) = target {
            self.order_ship_storage(&id);
            self.selection.units.clear();
            self.selection.building = None;
            self.selection.ship = Some(id);
            return;
        }
        if let Target::Unit(id) = target {
            self.selection.select_unit(id, additive);
            return;
        }
        if let Some(ship_id) = self.selection.ship.clone() {
            if let Target::Building(ref building_id) = target
                && self.view.snapshot.as_ref().is_some_and(|s| {
                    s.buildings.iter().any(|b| {
                        &b.building.id == building_id
                            && b.building.kind == aoa_game::BuildingKind::Dock
                    })
                })
            {
                self.send(Command::DockShip {
                    ship_id,
                    building_id: building_id.clone(),
                });
                return;
            }
            if let Target::Ground(to) = target {
                self.send(Command::Sail { ship_id, to });
                return;
            }
            self.selection.ship = None;
        }
        let units = self.selection.units.clone();
        if units.is_empty() {
            self.selection.building = match target {
                Target::Foundation(id) | Target::Building(id) => Some(id),
                _ => None,
            };
            return;
        }
        match target {
            Target::Unit(_) | Target::Ship(_) => {}
            Target::Resource(resource_id) => {
                for unit_id in units {
                    let depleted_field = self.view.snapshot.as_ref().is_some_and(|s| {
                        s.resources
                            .iter()
                            .any(|r| r.id == resource_id && r.field.is_some() && r.amount <= 0.0)
                    });
                    self.send(if depleted_field {
                        Command::Cultivate {
                            unit_id,
                            resource_id: resource_id.clone(),
                        }
                    } else {
                        Command::Gather {
                            unit_id,
                            resource_id: resource_id.clone(),
                        }
                    });
                }
            }
            Target::Foundation(building_id) => {
                for unit_id in units {
                    self.send(Command::Construct {
                        unit_id,
                        building_id: building_id.clone(),
                    });
                }
            }
            Target::Building(building_id) => {
                // Selected villagers holding goods this building takes unload
                // there; otherwise the tap selects the building.
                let carriers = self.carriers_for(&building_id);
                if carriers.is_empty() {
                    self.selection.units.clear();
                    self.selection.building = Some(building_id);
                    return;
                }
                for unit_id in carriers {
                    self.send(Command::Deposit {
                        unit_id,
                        storage_id: building_id.clone(),
                    });
                }
                // The building's own menu stays one tap away: it is selected.
                self.selection.units.clear();
                self.selection.building = Some(building_id);
            }
            Target::Ground(cell) if units.len() == 1 => self.send(Command::Move {
                unit_id: units[0].clone(),
                to: cell,
            }),
            Target::Ground(cell) => self.send(Command::GroupMove {
                unit_ids: units,
                to: cell,
            }),
        }
    }

    /// Selected villagers carrying goods the complete building `id` accepts.
    fn carriers_for(&self, id: &str) -> Vec<String> {
        let Some(snapshot) = self.view.snapshot.as_ref() else {
            return Vec::new();
        };
        let Some(building) = snapshot
            .buildings
            .iter()
            .find(|b| b.building.id == id && b.building.construction.is_none())
        else {
            return Vec::new();
        };
        snapshot
            .units
            .iter()
            .filter(|u| self.selection.units.contains(&u.unit.id))
            .filter(|u| {
                u.unit
                    .cargo
                    .as_ref()
                    .is_some_and(|cargo| building.building.kind.accepts(cargo.kind))
            })
            .map(|u| u.unit.id.clone())
            .collect()
    }

    /// Stops every selected villager that is busy.
    fn stop(&mut self) {
        if let Some(ship_id) = self.selection.ship.clone() {
            self.send(Command::StopShip { ship_id });
            return;
        }
        let busy: Vec<String> = self
            .view
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .units
                    .iter()
                    .filter(|u| {
                        self.selection.units.contains(&u.unit.id)
                            && u.unit.action != aoa_game::UnitAction::Idle
                    })
                    .map(|u| u.unit.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        for unit_id in busy {
            self.send(Command::Stop { unit_id });
        }
    }

    fn key(&mut self, key: &Key) {
        match key.as_ref() {
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
            Key::Character("g") | Key::Character("G") => self.show_grid = !self.show_grid,
            Key::Character("x") | Key::Character("X") => self.stop(),
            Key::Character("0") => self.send(Command::SetSimulationSpeed { multiplier: 0.0 }),
            Key::Character("1") => self.send(Command::SetSimulationSpeed { multiplier: 1.0 }),
            Key::Character("2") => self.send(Command::SetSimulationSpeed { multiplier: 2.0 }),
            Key::Named(NamedKey::Escape) if self.build != hud::BuildUi::Off => {
                self.build = hud::BuildUi::Off
            }
            Key::Named(NamedKey::Escape) => self.selection = Selection::default(),
            _ => {}
        }
    }

    /// Keeps the surface, render targets and camera at the window's physical
    /// size, so the picture is sharp and pointer pixels match what is drawn.
    fn fit_surface(&mut self) {
        let Some(game) = self.game.as_mut() else {
            return;
        };
        let (width, height) = physical_size(&game.window);
        if (width, height) != (game.gpu.config.width, game.gpu.config.height) {
            game.gpu.resize(width, height);
            game.renderer.resize(&game.gpu.device, width, height);
        }
        self.rig.width = width as f32;
        self.rig.height = height as f32;
    }

    fn redraw(&mut self) {
        self.fit_surface();
        let now = now_seconds();
        let dt = self
            .last_frame
            .map(|last| (now - last).clamp(0.0, 0.25))
            .unwrap_or(0.0);
        self.last_frame = Some(now);
        self.clock += dt;
        self.source.poll(dt, &mut self.incoming);
        while let Some(snapshot) = self.incoming.pop_front() {
            self.feedback
                .observe(self.view.snapshot.as_ref(), &snapshot, self.clock);
            // The hosted server resets after a delay, so the view may have
            // framed the old island meanwhile: look again at the new one.
            if self.view.sync(snapshot) {
                self.selection.units.clear();
                self.selection.building = None;
                self.framed = false;
            }
            if !self.framed {
                self.frame_town_center();
            }
        }
        for result in self.source.take_results() {
            if let Err(error) = result {
                self.toast = Some((friendly(&error), now + 3.0));
            }
        }
        if self.toast.as_ref().is_some_and(|(_, until)| now > *until) {
            self.toast = None;
        }
        self.rig.map_size = Vec2::new(
            self.view.heights.columns as f32,
            self.view.heights.rows as f32,
        ) * terrain::CELL;
        self.edge_pan(dt as f32);
        self.view.frame(dt as f32);
        let ghost = match self.build {
            hud::BuildUi::PlacingField if !self.hud.covers(self.cursor) => self
                .placement(self.cursor, aoa_game::BuildingKind::Farm)
                .map(|(origin, ok)| (aoa_game::BuildingKind::Farm, origin, ok)),
            hud::BuildUi::Placing(kind) if !self.hud.covers(self.cursor) => self
                .placement(self.cursor, kind)
                .map(|(origin, ok)| (kind, origin, ok)),
            _ => None,
        };
        self.update_resource_island();
        let plots_changed = self.level_building_plots(ghost);
        let hover = self.hover_decal();
        let Some(game) = self.game.as_mut() else {
            return;
        };
        if std::mem::take(&mut self.view.cells_dirty)
            && let Some((rgba, layers)) = self.view.cell_data()
        {
            game.renderer.update_cells(
                &game.gpu,
                &rgba,
                &layers,
                self.view.heights.columns,
                self.view.heights.rows,
            );
        }
        // Rebuilding the ground mesh costs several milliseconds and a large
        // upload; while villagers explore, exploration grows every tick, so
        // the mesh catches up at most once a second.
        let region = game.renderer.ground_region(&self.rig);
        if plots_changed
            || game.renderer.ground_bounds != region
            || (self.view.heights_dirty && now - self.ground_rebuilt_at >= 1.0)
        {
            self.view.heights_dirty = false;
            self.ground_rebuilt_at = now;
            game.renderer
                .update_ground(&game.gpu.device, &self.view.heights, region);
            #[cfg(target_arch = "wasm32")]
            LAST_HEIGHTS.with(|heights| *heights.borrow_mut() = Some(self.view.heights.clone()));
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
            map_size: self.rig.map_size.to_array(),
            time: self.clock as f32,
            curve: self.rig.curve(),
            curve_center: [self.rig.target.x, self.rig.target.z],
            fog_near: self.rig.distance + 8.0,
            fog_far: self.rig.distance * 2.0 + 60.0,
            placement: ghost.map_or([0.0; 4], |(kind, origin, _)| {
                let (columns, rows) = kind.size();
                [
                    origin.column as f32 * terrain::CELL,
                    origin.row as f32 * terrain::CELL,
                    columns as f32 * terrain::CELL,
                    rows as f32 * terrain::CELL,
                ]
            }),
            placement_color: ghost.map_or([0.0; 4], |(_, _, ok)| {
                if ok {
                    [0.3, 0.8, 0.4, 1.0]
                } else {
                    [0.9, 0.25, 0.2, 1.0]
                }
            }),
            grid: [if self.show_grid { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
        };
        #[cfg(target_arch = "wasm32")]
        LAST_VIEW.with(|view| {
            *view.borrow_mut() = (
                self.rig.view_proj(),
                self.rig.width,
                self.rig.height,
                self.selection.units.len(),
                self.selection.building.is_some() as usize,
                self.rig.curve(),
                Vec2::new(self.rig.target.x, self.rig.target.z),
            )
        });
        let (mut sprites, mut decals) = self.view.draw_list(
            &self.sheets,
            &self.rig,
            self.clock as f32,
            dt as f32,
            &self.selection,
        );
        decals.extend(hover);
        if let Some((kind, origin, ok)) = ghost {
            let (columns, rows) = kind.size();
            let x = (origin.column as f32 + columns as f32 / 2.0) * terrain::CELL;
            let z = (origin.row as f32 + rows as f32 / 2.0) * terrain::CELL;
            let center = Vec3::new(x, self.view.heights.at(x, z), z);
            let (sheet, mut preview) = if self.build == hud::BuildUi::PlacingField {
                view::field_preview(&self.sheets, &self.view.heights, center)
            } else {
                view::building_sprite(&self.sheets, &self.view.heights, kind, center, None, false)
            };
            preview.tint = if ok {
                [0.75, 1.0, 0.8, 0.48]
            } else {
                [1.0, 0.45, 0.4, 0.48]
            };
            sprites.push((sheet, preview));
        }
        let model = hud::Model {
            resource_island: self.resource_island,
            cargo_kind: self.cargo_kind,
            snapshot: self.view.snapshot.as_ref(),
            units: &self.selection.units,
            building: self.selection.building.as_deref(),
            ship: self.selection.ship.as_deref(),
            build: self.build,
            show_grid: self.show_grid,
            toast: self.toast.as_ref().map(|(text, _)| text.as_str()),
            camera: Vec2::new(self.rig.target.x, self.rig.target.z),
        };
        let scale = game.window.scale_factor() as f32;
        self.hud
            .layout(&self.atlas, &model, self.rig.width, self.rig.height, scale);
        if let Some(pointer) = &self.pointer
            && pointer.box_select
            && pointer.dragging
        {
            self.hud.selection_box(pointer.down_at, self.cursor, scale);
        }
        self.feedback.draw(
            &mut self.hud,
            &self.atlas,
            &self.rig,
            &self.view.heights,
            self.clock,
        );
        game.renderer.render(
            &game.gpu,
            &globals,
            (camera::NEAR, camera::FAR),
            &mut sprites,
            &decals,
            &self.hud.quads,
        );
        if !self.revealed && self.view.snapshot.is_some() {
            self.revealed = true;
            loaded();
        }
        game.window.request_redraw();
    }
}

/// Reports startup progress (0 to 1) to the page's loading overlay.
#[cfg(target_arch = "wasm32")]
pub(crate) fn loading(fraction: f64, text: &str) {
    window::call_overlay(&[fraction.into(), text.into()]);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn loading(_fraction: f64, _text: &str) {}

/// Plain-language versions of the server's rejection reasons.
fn friendly(error: &str) -> String {
    match error {
        "destination cell is occupied" => "Something already stands there.".into(),
        "target is unreachable" => "No path leads there.".into(),
        "build site is blocked or outside the world" => {
            "That spot is not clear for building.".into()
        }
        "insufficient wood" => "Not enough wood for that building.".into(),
        "insufficient stone" => "Not enough stone for that building.".into(),
        "a dock must touch the sea" => "A dock must be built along the shore.".into(),
        "unit is not carrying anything" => "That villager has nothing to unload.".into(),
        "building does not take that cargo" => "That building does not take those goods.".into(),
        "population cap reached" => "Build a house to make room for more villagers.".into(),
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
        assets.image("sprites/villager_idle_hd.png"),
        assets.image("sprites/buildings_hd.png"),
        assets.image("sprites/buildings_economy.png"),
        assets.image("sprites/buildings_crafts.png"),
        assets.image("sprites/buildings_civic.png"),
        assets.image("sprites/units.png"),
        assets.image("sprites/transport.png"),
        assets.image("sprites/villager_field_preparation.png"),
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
        loading(0.92, "Preparing the world");
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
        loading(0.97, "Sailing to the island");
        game.window.request_redraw();
        self.game = Some(game);
        self.fit_surface();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => self.fit_surface(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_inside = true;
                self.moved(Vec2::new(position.x as f32, position.y as f32))
            }
            WindowEvent::CursorLeft { .. } => self.mouse_inside = false,
            WindowEvent::Focused(true) => self.focused = true,
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.press(self.cursor, button, self.modifiers.shift_key())
                }
                ElementState::Released => self.release(self.cursor, self.modifiers.shift_key()),
            },
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::Focused(false) => {
                self.focused = false;
                self.mouse_inside = false;
                self.modifiers = ModifiersState::empty();
                self.pointer = None;
                self.touches.clear();
                self.gesture = false;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 60.0,
                };
                let heights = &self.view.heights;
                self.rig
                    .zoom_at(self.cursor, (1.0 - lines * 0.1).clamp(0.5, 1.5), |x, z| {
                        heights.at(x, z)
                    });
            }
            WindowEvent::Touch(touch) => {
                let pixel = Vec2::new(touch.location.x as f32, touch.location.y as f32);
                self.touch(touch.id, touch.phase, pixel);
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
    let seed = std::env::var("AGE_OF_AGENTS_SEED")
        .ok()
        .and_then(|seed| seed.parse().ok())
        .unwrap_or(aoa_game::DEFAULT_SEED);
    run(assets, Source::local(seed));
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
        // `?local&seed=42` plays the island grown from seed 42 in the page.
        let seed = search
            .trim_start_matches('?')
            .split('&')
            .find_map(|pair| pair.strip_prefix("seed=")?.parse().ok())
            .unwrap_or(aoa_game::DEFAULT_SEED);
        let source = if search.contains("local") {
            Source::local(seed)
        } else {
            Source::Remote(source::remote::Remote::connect())
        };
        run(assets, source);
    });
}
