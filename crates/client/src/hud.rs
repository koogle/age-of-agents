//! The mostly hidden interface, painted by the GPU: resource coins top-right,
//! a globe minimap with speed coins bottom-right, and, only while something is
//! selected, an info pill and a bar of command coins bottom-centre. Art is the
//! generated coin and icon kit under `assets/ui/`; text is Nunito.
use std::collections::HashMap;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use aoa_game::{
    RESEARCH_FOOD_COST, RESEARCH_WOOD_COST, ResourceKind, TOWN_CENTER_WOOD_COST, TechnologyKind,
    UnitAction, VILLAGER_FOOD_COST, WorldSnapshot,
};
use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::assets::{Assets, Rgba};

const ATLAS: u32 = 2048;
const GLYPH_PX: f32 = 40.0;
const INK: [f32; 4] = [0.24, 0.2, 0.157, 1.0];
const MUTED: [f32; 4] = [0.45, 0.4, 0.34, 1.0];
const GLASS: [f32; 4] = [0.98, 0.96, 0.92, 0.86];
const ACCENT: [f32; 4] = [0.784, 0.333, 0.227, 1.0];
const ICONS: [&str; 18] = [
    "resource_wood",
    "resource_food",
    "resource_stone",
    "resource_gold",
    "resource_iron",
    "resource_clay",
    "resource_fiber",
    "command_build",
    "command_cancel",
    "command_train",
    "tech_forestry",
    "tech_agriculture",
    "tech_masonry",
    "tech_mining",
    "tech_textiles",
    "portrait_villager",
    "portrait_group",
    "portrait_towncenter",
];
const COINS: [&str; 3] = ["coin_normal", "coin_hover", "coin_disabled"];

pub fn files() -> Vec<String> {
    let mut files: Vec<String> = ICONS
        .iter()
        .map(|name| format!("ui/icons/{name}.png"))
        .collect();
    files.extend(COINS.iter().map(|name| format!("ui/buttons/{name}.png")));
    files.push("fonts/Nunito-ExtraBold.ttf".into());
    files
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Quad {
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub color: [f32; 4],
    /// x = mode (0 texture, 1 rounded rect, 2 ring, 3 globe), y = radius or width.
    pub params: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Speed(f64),
    Build,
    Cancel,
    Stop,
    Train,
    Research(TechnologyKind),
    /// Globe click: look at this map point.
    LookAt(Vec2),
}

struct Region {
    rect: [f32; 4],
    action: Action,
    enabled: bool,
}

struct Glyph {
    uv: [f32; 4],
    size: Vec2,
    offset: Vec2,
    advance: f32,
}

pub struct Atlas {
    pub image: Rgba,
    sprites: HashMap<String, [f32; 4]>,
    /// Painted bounds of each icon (uv rect and pixel size), so icons whose
    /// art fills their square differently still line up inside the coins.
    content: HashMap<String, ([f32; 4], Vec2)>,
    glyphs: HashMap<char, Glyph>,
}

/// The rectangle of `image` whose alpha is visibly painted, in pixels.
fn painted_bounds(image: &Rgba) -> (u32, u32, u32, u32) {
    let (mut x0, mut y0, mut x1, mut y1) = (image.width, image.height, 0, 0);
    for y in 0..image.height {
        for x in 0..image.width {
            if image.pixels[((y * image.width + x) * 4 + 3) as usize] > 40 {
                (x0, y0) = (x0.min(x), y0.min(y));
                (x1, y1) = (x1.max(x + 1), y1.max(y + 1));
            }
        }
    }
    if x1 <= x0 || y1 <= y0 {
        (0, 0, image.width, image.height)
    } else {
        (x0, y0, x1, y1)
    }
}

/// Packs coins, icons and glyphs into one texture with a simple shelf packer.
pub fn build_atlas(assets: &Assets) -> Atlas {
    let mut image = Rgba {
        width: ATLAS,
        height: ATLAS,
        pixels: vec![0; (ATLAS * ATLAS * 4) as usize],
    };
    let (mut x, mut y, mut shelf) = (0u32, 0u32, 0u32);
    let mut place = |w: u32, h: u32| {
        if x + w > ATLAS {
            x = 0;
            y += shelf + 2;
            shelf = 0;
        }
        let at = (x, y);
        x += w + 2;
        shelf = shelf.max(h);
        at
    };
    let uv = |(px, py): (u32, u32), w: u32, h: u32| {
        [
            px as f32 / ATLAS as f32,
            py as f32 / ATLAS as f32,
            (px + w) as f32 / ATLAS as f32,
            (py + h) as f32 / ATLAS as f32,
        ]
    };
    let blit = |image: &mut Rgba, source: &Rgba, (px, py): (u32, u32)| {
        for row in 0..source.height {
            let from = (row * source.width * 4) as usize;
            let to = (((py + row) * ATLAS + px) * 4) as usize;
            image.pixels[to..to + (source.width * 4) as usize]
                .copy_from_slice(&source.pixels[from..from + (source.width * 4) as usize]);
        }
    };
    let mut sprites = HashMap::new();
    let mut content = HashMap::new();
    for name in COINS {
        let coin = assets
            .image(&format!("ui/buttons/{name}.png"))
            .resized(192, 192);
        let at = place(192, 192);
        blit(&mut image, &coin, at);
        sprites.insert(name.to_string(), uv(at, 192, 192));
    }
    for name in ICONS {
        let icon = assets.image(&format!("ui/icons/{name}.png"));
        let at = place(icon.width, icon.height);
        blit(&mut image, &icon, at);
        sprites.insert(name.to_string(), uv(at, icon.width, icon.height));
        let (x0, y0, x1, y1) = painted_bounds(&icon);
        content.insert(
            name.to_string(),
            (
                uv((at.0 + x0, at.1 + y0), x1 - x0, y1 - y0),
                Vec2::new((x1 - x0) as f32, (y1 - y0) as f32),
            ),
        );
    }
    let font = FontRef::try_from_slice(assets.bytes("fonts/Nunito-ExtraBold.ttf")).expect("font");
    let scaled = font.as_scaled(PxScale::from(GLYPH_PX));
    let mut glyphs = HashMap::new();
    for ch in (32u8..127).map(char::from).chain(['×']) {
        let id = scaled.glyph_id(ch);
        let advance = scaled.h_advance(id);
        let glyph = id.with_scale_and_position(GLYPH_PX, ab_glyph::point(0.0, 0.0));
        let Some(outline) = font.outline_glyph(glyph) else {
            glyphs.insert(
                ch,
                Glyph {
                    uv: [0.0; 4],
                    size: Vec2::ZERO,
                    offset: Vec2::ZERO,
                    advance,
                },
            );
            continue;
        };
        let bounds = outline.px_bounds();
        let (w, h) = (
            bounds.width().ceil() as u32 + 1,
            bounds.height().ceil() as u32 + 1,
        );
        let at = place(w, h);
        outline.draw(|gx, gy, coverage| {
            let index = (((at.1 + gy) * ATLAS + at.0 + gx) * 4) as usize;
            image.pixels[index..index + 4].copy_from_slice(&[
                255,
                255,
                255,
                (coverage * 255.0) as u8,
            ]);
        });
        glyphs.insert(
            ch,
            Glyph {
                uv: uv(at, w, h),
                size: Vec2::new(w as f32, h as f32),
                offset: Vec2::new(bounds.min.x, bounds.min.y),
                advance,
            },
        );
    }
    Atlas {
        image,
        sprites,
        content,
        glyphs,
    }
}

/// What the HUD needs from the app each frame.
pub struct Model<'a> {
    pub snapshot: Option<&'a WorldSnapshot>,
    pub units: &'a [String],
    pub building: Option<&'a str>,
    pub build_mode: bool,
    pub toast: Option<&'a str>,
    pub camera: Vec2,
}

pub struct Hud {
    pub quads: Vec<Quad>,
    regions: Vec<Region>,
    pub hover: Option<Vec2>,
    pressed: Option<Action>,
}

fn resource_icon(kind: ResourceKind) -> Option<&'static str> {
    Some(match kind {
        ResourceKind::Wood => "resource_wood",
        ResourceKind::Food => "resource_food",
        ResourceKind::Stone => "resource_stone",
        ResourceKind::Gold => "resource_gold",
        ResourceKind::Iron => "resource_iron",
        ResourceKind::Clay => "resource_clay",
        ResourceKind::Fiber => "resource_fiber",
        _ => return None,
    })
}

fn tech_info(tech: TechnologyKind) -> (&'static str, &'static str, &'static str) {
    match tech {
        TechnologyKind::Forestry => ("tech_forestry", "Forestry", "Wood +20%"),
        TechnologyKind::Agriculture => ("tech_agriculture", "Agriculture", "Food +20%"),
        TechnologyKind::Masonry => ("tech_masonry", "Masonry", "Stone and clay +20%"),
        TechnologyKind::Mining => ("tech_mining", "Mining", "Gold and iron +20%"),
        TechnologyKind::Textiles => ("tech_textiles", "Textiles", "Fiber +20%"),
    }
}

struct Command {
    icon: &'static str,
    label: String,
    detail: String,
    enabled: bool,
    action: Action,
}

impl Hud {
    pub fn new() -> Self {
        Self {
            quads: Vec::new(),
            regions: Vec::new(),
            hover: None,
            pressed: None,
        }
    }

    fn sprite(&mut self, atlas: &Atlas, name: &str, rect: [f32; 4], color: [f32; 4]) {
        if let Some(uv) = atlas.sprites.get(name) {
            self.quads.push(Quad {
                rect,
                uv: *uv,
                color,
                params: [0.0; 4],
            });
        }
    }

    fn shape(&mut self, rect: [f32; 4], color: [f32; 4], mode: f32, radius: f32) {
        self.quads.push(Quad {
            rect,
            uv: [0.0; 4],
            color,
            params: [mode, radius, 0.0, 0.0],
        });
    }

    fn text_width(atlas: &Atlas, text: &str, size: f32) -> f32 {
        text.chars()
            .map(|ch| atlas.glyphs.get(&ch).map(|g| g.advance).unwrap_or(0.0))
            .sum::<f32>()
            * size
            / GLYPH_PX
    }

    /// Draws `text` with its baseline at `y`, centred on `x` when `center`.
    fn text(
        &mut self,
        atlas: &Atlas,
        text: &str,
        at: (f32, f32),
        size: f32,
        color: [f32; 4],
        center: bool,
    ) {
        let scale = size / GLYPH_PX;
        let (x, y) = at;
        let mut pen = if center {
            x - Self::text_width(atlas, text, size) / 2.0
        } else {
            x
        };
        for ch in text.chars() {
            let Some(glyph) = atlas.glyphs.get(&ch) else {
                continue;
            };
            if glyph.size.x > 0.0 {
                let rect = [
                    pen + glyph.offset.x * scale,
                    y + glyph.offset.y * scale,
                    glyph.size.x * scale,
                    glyph.size.y * scale,
                ];
                self.quads.push(Quad {
                    rect,
                    uv: glyph.uv,
                    color,
                    params: [0.0; 4],
                });
            }
            pen += glyph.advance * scale;
        }
    }

    fn coin(&mut self, atlas: &Atlas, icon: &str, rect: [f32; 4], enabled: bool, hot: bool) {
        let frame = if !enabled {
            "coin_disabled"
        } else if hot {
            "coin_hover"
        } else {
            "coin_normal"
        };
        let lift = if hot { -3.0 } else { 0.0 };
        let rect = [rect[0], rect[1] + lift, rect[2], rect[3]];
        self.sprite(atlas, frame, rect, [1.0; 4]);
        let tint = if enabled {
            [1.0; 4]
        } else {
            [0.75, 0.75, 0.75, 0.7]
        };
        // Fit the painted part of the icon into the same box on every coin.
        let Some(&(uv, size)) = atlas.content.get(icon) else {
            return;
        };
        let scale = rect[2] * 0.6 / size.x.max(size.y);
        let (w, h) = (size.x * scale, size.y * scale);
        self.quads.push(Quad {
            rect: [
                rect[0] + (rect[2] - w) / 2.0,
                rect[1] + (rect[3] - h) / 2.0,
                w,
                h,
            ],
            uv,
            color: tint,
            params: [0.0; 4],
        });
    }

    fn hovered(&self, rect: [f32; 4]) -> bool {
        self.hover.is_some_and(|p| {
            p.x >= rect[0] && p.x <= rect[0] + rect[2] && p.y >= rect[1] && p.y <= rect[1] + rect[3]
        })
    }

    /// Lays out the whole interface for this frame.
    pub fn layout(&mut self, atlas: &Atlas, model: &Model, width: f32, height: f32, scale: f32) {
        self.quads.clear();
        self.regions.clear();
        let Some(snapshot) = model.snapshot else {
            return;
        };
        let s = scale;
        let stock = &snapshot.stockpile;

        // Resource coins with count tabs, top-right: wood and food always, others once owned.
        let shown: Vec<(ResourceKind, f64)> = [
            (ResourceKind::Wood, stock.wood),
            (ResourceKind::Food, stock.food),
            (ResourceKind::Stone, stock.stone),
            (ResourceKind::Gold, stock.gold),
            (ResourceKind::Iron, stock.iron),
            (ResourceKind::Clay, stock.clay),
            (ResourceKind::Fiber, stock.fiber),
        ]
        .into_iter()
        .filter(|(kind, amount)| {
            matches!(kind, ResourceKind::Wood | ResourceKind::Food) || *amount >= 1.0
        })
        .collect();
        let d = 46.0 * s;
        let step = d + 18.0 * s;
        for (index, (kind, amount)) in shown.iter().enumerate() {
            let x = width - 16.0 * s - (shown.len() - index) as f32 * step + (step - d) / 2.0;
            let icon = resource_icon(*kind).unwrap_or("resource_wood");
            self.coin(atlas, icon, [x, 10.0 * s, d, d], true, false);
            let text = format!("{}", amount.floor() as i64);
            let tab = (Self::text_width(atlas, &text, 13.0 * s) + 14.0 * s).max(26.0 * s);
            self.shape(
                [
                    x + d / 2.0 - tab / 2.0,
                    10.0 * s + d - 2.0 * s,
                    tab,
                    18.0 * s,
                ],
                GLASS,
                1.0,
                9.0 * s,
            );
            self.text(
                atlas,
                &text,
                (x + d / 2.0, 10.0 * s + d + 12.0 * s),
                13.0 * s,
                INK,
                true,
            );
        }

        // Globe minimap bottom-right, gold rim, speed coins on its shoulder.
        let r = 68.0 * s;
        let (gx, gy) = (width - 18.0 * s - r * 2.0, height - 18.0 * s - r * 2.0);
        let globe = [gx, gy, r * 2.0, r * 2.0];
        let span = 38.0;
        let (cx, cz) = (15.0, 10.0);
        self.quads.push(Quad {
            rect: globe,
            uv: [
                (cx - span / 2.0) / 30.0,
                (cz - span / 2.0) / 20.0,
                (cx + span / 2.0) / 30.0,
                (cz + span / 2.0) / 20.0,
            ],
            color: [1.0; 4],
            params: [3.0, 0.0, 0.0, 0.0],
        });
        self.shape(globe, [0.79, 0.59, 0.25, 1.0], 2.0, 6.0 * s);
        let camera = Vec2::new(
            gx + r + (model.camera.x - cx) / span * r * 2.0,
            gy + r + (model.camera.y - cz) / span * r * 2.0,
        );
        self.shape(
            [camera.x - 5.0 * s, camera.y - 5.0 * s, 10.0 * s, 10.0 * s],
            [1.0, 1.0, 1.0, 0.95],
            2.0,
            2.0 * s,
        );
        self.regions.push(Region {
            rect: globe,
            action: Action::LookAt(Vec2::new(cx - span / 2.0, cz - span / 2.0)),
            enabled: true,
        });
        let speeds = [(0.0, "II"), (1.0, "1×"), (2.0, "2×")];
        for (index, (speed, label)) in speeds.into_iter().enumerate() {
            let angle = std::f32::consts::PI * (1.0 + 0.16 + index as f32 * 0.17);
            let c = 30.0 * s;
            let center = Vec2::new(
                gx + r + angle.cos() * (r + 22.0 * s),
                gy + r + angle.sin() * (r + 22.0 * s),
            );
            let rect = [center.x - c / 2.0, center.y - c / 2.0, c, c];
            let active = (snapshot.simulation_speed - speed).abs() < 1e-6;
            self.sprite(
                atlas,
                if self.hovered(rect) {
                    "coin_hover"
                } else {
                    "coin_normal"
                },
                rect,
                [1.0; 4],
            );
            if active {
                self.shape(
                    [
                        rect[0] + 4.0 * s,
                        rect[1] + 4.0 * s,
                        c - 8.0 * s,
                        c - 8.0 * s,
                    ],
                    ACCENT,
                    1.0,
                    (c - 8.0 * s) / 2.0,
                );
            }
            self.text(
                atlas,
                label,
                (center.x, center.y + 4.5 * s),
                12.0 * s,
                if active { [1.0; 4] } else { INK },
                true,
            );
            self.regions.push(Region {
                rect,
                action: Action::Speed(speed),
                enabled: true,
            });
        }

        // Selection: info pill plus a glass bar of command coins.
        let Some((portrait, title, detail, progress, commands)) = selection_model(snapshot, model)
        else {
            self.toast(atlas, model.toast, width, s);
            return;
        };
        let m = 52.0 * s;
        let gap = 10.0 * s;
        let bar_width = commands.len() as f32 * (m + gap) - gap + 28.0 * s;
        let bar = [
            (width - bar_width) / 2.0,
            height - m - 30.0 * s,
            bar_width,
            m + 12.0 * s,
        ];
        let mut hover_text = None;
        if !commands.is_empty() {
            self.shape(bar, GLASS, 1.0, bar[3] / 2.0);
            for (index, command) in commands.iter().enumerate() {
                let rect = [
                    bar[0] + 14.0 * s + index as f32 * (m + gap),
                    bar[1] + 6.0 * s,
                    m,
                    m,
                ];
                // The hit area spans half the gap on each side, so sweeping
                // across the bar never falls back to the selection text.
                let hit = [rect[0] - gap / 2.0, bar[1], m + gap, bar[3]];
                let hot = self.hovered(hit);
                if hot {
                    hover_text = Some((command.label.clone(), command.detail.clone()));
                }
                self.coin(
                    atlas,
                    command.icon,
                    rect,
                    command.enabled,
                    hot && command.enabled,
                );
                self.regions.push(Region {
                    rect: hit,
                    action: command.action.clone(),
                    enabled: command.enabled,
                });
            }
        }
        // One width for every text this selection can show, so hovering
        // commands changes the words but never resizes the pill.
        let widest = std::iter::once((&title, &detail))
            .chain(commands.iter().map(|c| (&c.label, &c.detail)))
            .map(|(t, d)| {
                Self::text_width(atlas, t, 15.0 * s).max(Self::text_width(atlas, d, 12.0 * s))
            })
            .fold(0.0, f32::max);
        let info_width = (widest + 84.0 * s).max(200.0 * s);
        let (title, detail) = hover_text.unwrap_or((title, detail));
        let info = [
            (width - info_width) / 2.0,
            bar[1] - 64.0 * s,
            info_width,
            52.0 * s,
        ];
        self.shape(info, GLASS, 1.0, 26.0 * s);
        self.sprite(
            atlas,
            portrait,
            [info[0] - 2.0 * s, info[1] - 2.0 * s, 56.0 * s, 56.0 * s],
            [1.0; 4],
        );
        self.text(
            atlas,
            &title,
            (info[0] + 64.0 * s, info[1] + 23.0 * s),
            15.0 * s,
            INK,
            false,
        );
        self.text(
            atlas,
            &detail,
            (info[0] + 64.0 * s, info[1] + 40.0 * s),
            12.0 * s,
            MUTED,
            false,
        );
        if let Some(progress) = progress {
            let track = [
                info[0] + 64.0 * s,
                info[1] + 45.0 * s,
                info_width - 84.0 * s,
                3.0 * s,
            ];
            self.shape(track, [0.24, 0.2, 0.16, 0.15], 1.0, 1.5 * s);
            self.shape(
                [track[0], track[1], track[2] * progress.min(1.0), track[3]],
                ACCENT,
                1.0,
                1.5 * s,
            );
        }
        self.toast(atlas, model.toast, width, s);
    }

    fn toast(&mut self, atlas: &Atlas, toast: Option<&str>, width: f32, s: f32) {
        if let Some(text) = toast {
            let w = Self::text_width(atlas, text, 14.0 * s) + 36.0 * s;
            self.shape(
                [(width - w) / 2.0, 16.0 * s, w, 34.0 * s],
                GLASS,
                1.0,
                17.0 * s,
            );
            self.text(atlas, text, (width / 2.0, 38.0 * s), 14.0 * s, INK, true);
        }
    }

    /// Whether a screen point lies on an interactive HUD element.
    pub fn covers(&self, at: Vec2) -> bool {
        self.regions.iter().any(|region| {
            let r = region.rect;
            at.x >= r[0] && at.x <= r[0] + r[2] && at.y >= r[1] && at.y <= r[1] + r[3]
        })
    }

    /// Pointer press: true when the HUD takes it.
    pub fn press(&mut self, at: Vec2) -> bool {
        self.pressed = None;
        for region in self.regions.iter().rev() {
            let r = region.rect;
            if at.x >= r[0] && at.x <= r[0] + r[2] && at.y >= r[1] && at.y <= r[1] + r[3] {
                if region.enabled {
                    self.pressed = Some(match &region.action {
                        Action::LookAt(origin) => {
                            let span = 38.0;
                            Action::LookAt(
                                *origin
                                    + Vec2::new(
                                        (at.x - r[0]) / r[2] * span,
                                        (at.y - r[1]) / r[3] * span,
                                    ),
                            )
                        }
                        action => action.clone(),
                    });
                }
                return true;
            }
        }
        false
    }

    /// Pointer release: the action pressed on this HUD element, if any.
    pub fn release(&mut self) -> Option<Action> {
        self.pressed.take()
    }
}

type Selected = (&'static str, String, String, Option<f32>, Vec<Command>);

fn selection_model(snapshot: &WorldSnapshot, model: &Model) -> Option<Selected> {
    let stock = &snapshot.stockpile;
    if !model.units.is_empty() {
        let busy = snapshot
            .units
            .iter()
            .any(|u| model.units.contains(&u.unit.id) && u.unit.action != UnitAction::Idle);
        let commands = if model.build_mode {
            vec![Command {
                icon: "command_cancel",
                label: "Cancel placement".into(),
                detail: "Esc".into(),
                enabled: true,
                action: Action::Cancel,
            }]
        } else {
            let mut commands = vec![Command {
                icon: "command_build",
                label: "Build town center".into(),
                detail: format!("{} wood", TOWN_CENTER_WOOD_COST),
                enabled: stock.wood >= TOWN_CENTER_WOOD_COST,
                action: Action::Build,
            }];
            if busy {
                commands.push(Command {
                    icon: "command_cancel",
                    label: "Stop".into(),
                    detail: "Drop the current task · X".into(),
                    enabled: true,
                    action: Action::Stop,
                });
            }
            commands
        };
        if model.units.len() > 1 {
            let idle = snapshot
                .units
                .iter()
                .filter(|u| model.units.contains(&u.unit.id) && u.unit.action == UnitAction::Idle)
                .count();
            return Some((
                "portrait_group",
                format!("{} villagers", model.units.len()),
                format!("{idle} awaiting orders"),
                None,
                commands,
            ));
        }
        let unit = snapshot
            .units
            .iter()
            .find(|u| u.unit.id == model.units[0])?;
        let activity = match &unit.unit.action {
            UnitAction::Idle => "Awaiting orders".to_string(),
            UnitAction::Move { .. } => "Walking".into(),
            UnitAction::Build { .. } => "Building".into(),
            UnitAction::Gather { phase, .. } => match phase {
                aoa_game::GatherPhase::ToResource => "Heading out to gather".into(),
                aoa_game::GatherPhase::Gathering => "Gathering".into(),
                aoa_game::GatherPhase::Returning => "Carrying goods home".into(),
                aoa_game::GatherPhase::Depositing => "Unloading".into(),
            },
        };
        let cargo = unit
            .unit
            .cargo
            .as_ref()
            .map(|c| format!(" · {} carried", c.amount.floor()))
            .unwrap_or_default();
        let title = unit.unit.id.replace("villager-", "Villager ");
        return Some((
            "portrait_villager",
            title,
            format!("{activity}{cargo}"),
            None,
            commands,
        ));
    }
    let building = snapshot
        .buildings
        .iter()
        .find(|b| Some(b.building.id.as_str()) == model.building)?;
    if let Some(work) = building.building.construction {
        return Some((
            "portrait_towncenter",
            "Town center foundation".into(),
            "Villagers can help build it".into(),
            Some((work / aoa_game::BUILD_SECONDS) as f32),
            Vec::new(),
        ));
    }
    let job = building.building.job.as_ref();
    let busy = job.is_some();
    let (detail, progress) = match job {
        Some(aoa_game::BuildingJob::Produce {
            elapsed_seconds, ..
        }) => (
            "Training a villager".to_string(),
            Some((elapsed_seconds / aoa_game::VILLAGER_PRODUCTION_SECONDS) as f32),
        ),
        Some(aoa_game::BuildingJob::Research {
            technology,
            elapsed_seconds,
        }) => (
            format!("Researching {}", tech_info(*technology).1),
            Some((elapsed_seconds / aoa_game::RESEARCH_SECONDS) as f32),
        ),
        None => ("Ready".to_string(), None),
    };
    let mut commands = vec![Command {
        icon: "command_train",
        label: "Train villager".into(),
        detail: format!("{} food", VILLAGER_FOOD_COST),
        enabled: !busy && stock.food >= VILLAGER_FOOD_COST,
        action: Action::Train,
    }];
    let known = &snapshot.researched_technologies;
    for tech in TechnologyKind::ALL {
        let (icon, name, effect) = tech_info(tech);
        let done = known.contains(&tech);
        let blocked = tech.prerequisite().filter(|p| !known.contains(p));
        let detail = if done {
            "researched".to_string()
        } else if let Some(p) = blocked {
            format!("needs {}", tech_info(p).1)
        } else {
            format!(
                "{effect} · {} food, {} wood",
                RESEARCH_FOOD_COST, RESEARCH_WOOD_COST
            )
        };
        commands.push(Command {
            icon,
            label: name.into(),
            detail,
            enabled: !done
                && blocked.is_none()
                && !busy
                && stock.food >= RESEARCH_FOOD_COST
                && stock.wood >= RESEARCH_WOOD_COST,
            action: Action::Research(tech),
        });
    }
    Some((
        "portrait_towncenter",
        "Town center".into(),
        detail,
        progress,
        commands,
    ))
}
