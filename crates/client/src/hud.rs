//! The mostly hidden interface, painted by the GPU: resource coins top-right,
//! a globe minimap with speed coins bottom-right, and, only while something is
//! selected, an info pill and a bar of command coins bottom-centre. Art is the
//! generated coin and icon kit under `assets/ui/`; text is Nunito.
use std::collections::HashMap;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use aoa_game::{BUILDABLE, BuildingKind, ResourceKind, TechnologyKind, WorldSnapshot};
use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::assets::{Assets, Rgba};

mod build_menu;
mod layout;
pub use build_menu::BuildingGroup;
mod selection;
use selection::{building_info, selection_model};

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
    files.push("fonts/Alegreya-MediumItalic.ttf".into());
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

/// The villager's build flow: closed, choosing a building, or placing one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuildUi {
    Off,
    Categories,
    Group(BuildingGroup),
    Placing(BuildingKind),
    PlacingField,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Speed(f64),
    Grid,
    /// Open the build menu.
    Build,
    /// Start placing this building.
    Place(BuildingKind),
    PlaceField,
    Cancel,
    Stop,
    Produce(aoa_game::ProductKind),
    BuildGroup(BuildingGroup),
    Research(TechnologyKind),
    CancelQueuedJob(u64),
    /// Globe click: look at this map point.
    LookAt(Vec2),
    /// Opens the reset confirmation and seed input.
    Reset,
    /// A tap on an unavailable command: say why in the toast line.
    Explain(String),
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
    gain_glyphs: HashMap<char, Glyph>,
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
    // Building coins show the finished building from the generated sheet.
    let sheets: HashMap<_, _> = [
        "towncenter",
        "buildings_hd",
        "buildings_economy",
        "buildings_crafts",
        "buildings_civic",
    ]
    .into_iter()
    .map(|name| {
        let frames: serde_json::Value =
            serde_json::from_slice(assets.bytes(&format!("sprites/{name}.json")))
                .expect("building frames");
        (name, (assets.image(&format!("sprites/{name}.png")), frames))
    })
    .collect();
    let portraits = BUILDABLE
        .into_iter()
        .map(|kind| {
            let (icon, row, _, _) = building_info(kind);
            (
                icon,
                build_menu::atlas(kind),
                row,
                kind == BuildingKind::TownCenter,
            )
        })
        .chain(std::iter::once((
            "field",
            "buildings_economy",
            "field",
            false,
        )));
    for (icon, atlas, row, town_center) in portraits {
        let (sheet, frames) = &sheets[atlas];
        let rect = if town_center {
            &frames["frames"]["complete"]
        } else {
            &frames["frames"][row][3]
        };
        let [fx, fy, fw, fh] =
            [0, 1, 2, 3].map(|i| rect[i].as_u64().expect("frame rectangle") as u32);
        let cell = sheet.crop(fx, fy, fw, fh);
        let (x0, y0, x1, y1) = painted_bounds(&cell);
        let painted = cell.crop(x0, y0, x1 - x0, y1 - y0);
        let fit = 160.0 / (painted.width.max(painted.height) as f32);
        let (w, h) = (
            ((painted.width as f32 * fit) as u32).max(1),
            ((painted.height as f32 * fit) as u32).max(1),
        );
        let painted = painted.resized(w, h);
        let at = place(w, h);
        blit(&mut image, &painted, at);
        sprites.insert(icon.to_string(), uv(at, w, h));
        content.insert(
            icon.to_string(),
            (uv(at, w, h), Vec2::new(w as f32, h as f32)),
        );
    }
    let mut fonts = Vec::new();
    for (path, css_em) in [
        ("fonts/Nunito-ExtraBold.ttf", false),
        ("fonts/Alegreya-MediumItalic.ttf", true),
    ] {
        let font = FontRef::try_from_slice(assets.bytes(path)).expect("font");
        // Match Canvas CSS font sizing for the original italic gain label.
        let pixels = if css_em {
            GLYPH_PX * font.height_unscaled() / font.units_per_em().expect("font em")
        } else {
            GLYPH_PX
        };
        let scaled = font.as_scaled(PxScale::from(pixels));
        let mut glyphs = HashMap::new();
        for ch in (32u8..127).map(char::from).chain(['×', '·']) {
            let id = scaled.glyph_id(ch);
            let advance = scaled.h_advance(id);
            let glyph = id.with_scale_and_position(pixels, ab_glyph::point(0.0, 0.0));
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
        fonts.push(glyphs);
    }
    let gain_glyphs = fonts.pop().expect("gain font");
    let glyphs = fonts.pop().expect("HUD font");
    Atlas {
        image,
        sprites,
        content,
        glyphs,
        gain_glyphs,
    }
}

/// What the HUD needs from the app each frame.
pub struct Model<'a> {
    pub snapshot: Option<&'a WorldSnapshot>,
    pub units: &'a [String],
    pub building: Option<&'a str>,
    pub build: BuildUi,
    pub show_grid: bool,
    pub toast: Option<&'a str>,
    pub camera: Vec2,
}

pub struct Hud {
    pub quads: Vec<Quad>,
    regions: Vec<Region>,
    pub hover: Option<Vec2>,
    pressed: Option<Action>,
}

fn resource_icon(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Wood => "resource_wood",
        ResourceKind::Food => "resource_food",
        ResourceKind::Stone => "resource_stone",
        ResourceKind::Gold => "resource_gold",
        ResourceKind::Iron => "resource_iron",
        ResourceKind::Clay => "resource_clay",
        ResourceKind::Fiber => "resource_fiber",
        ResourceKind::Coal | ResourceKind::Steel => "resource_iron",
        ResourceKind::Timber => "resource_wood",
        ResourceKind::Bricks => "resource_clay",
        ResourceKind::Cloth => "resource_fiber",
        ResourceKind::Rations => "resource_food",
    }
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

    /// Physical-pixel overlay; it has no hit region and never intercepts input.
    pub fn selection_box(&mut self, from: Vec2, to: Vec2, scale: f32) {
        let min = from.min(to);
        let size = (to - from).abs();
        let edge = 2.0 * scale;
        self.shape(
            [min.x, min.y, size.x, size.y],
            [0.98, 0.8, 0.32, 0.12],
            1.0,
            0.0,
        );
        for rect in [
            [min.x, min.y, size.x, edge],
            [min.x, min.y + size.y - edge, size.x, edge],
            [min.x, min.y, edge, size.y],
            [min.x + size.x - edge, min.y, edge, size.y],
        ] {
            self.shape(rect, [0.98, 0.8, 0.32, 0.95], 1.0, 0.0);
        }
    }

    fn text_width(atlas: &Atlas, text: &str, size: f32) -> f32 {
        Self::glyph_width(&atlas.glyphs, text, size)
    }
    fn text(
        &mut self,
        atlas: &Atlas,
        text: &str,
        at: (f32, f32),
        size: f32,
        color: [f32; 4],
        center: bool,
    ) {
        self.glyph_text(&atlas.glyphs, text, at, size, color, center);
    }
    fn glyph_width(glyphs: &HashMap<char, Glyph>, text: &str, size: f32) -> f32 {
        text.chars()
            .map(|ch| glyphs.get(&ch).map(|g| g.advance).unwrap_or(0.0))
            .sum::<f32>()
            * size
            / GLYPH_PX
    }

    /// Draws `text` with its baseline at `y`, centred on `x` when `center`.
    fn glyph_text(
        &mut self,
        glyphs: &HashMap<char, Glyph>,
        text: &str,
        at: (f32, f32),
        size: f32,
        color: [f32; 4],
        center: bool,
    ) {
        let scale = size / GLYPH_PX;
        let (x, y) = at;
        let mut pen = if center {
            x - Self::glyph_width(glyphs, text, size) / 2.0
        } else {
            x
        };
        for ch in text.chars() {
            let Some(glyph) = glyphs.get(&ch) else {
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

    /// Centre the visible glyph bounds in a control, independent of baseline
    /// and advance padding (which differ between digits, × and pause bars).
    fn centered_label(
        &mut self,
        atlas: &Atlas,
        text: &str,
        center: Vec2,
        size: f32,
        color: [f32; 4],
    ) {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        let mut pen = 0.0;
        for ch in text.chars() {
            let Some(glyph) = atlas.glyphs.get(&ch) else {
                continue;
            };
            if glyph.size.x > 0.0 {
                let start = Vec2::new(pen, 0.0) + glyph.offset;
                min = min.min(start);
                // Atlas glyphs have one extra transparent row and column.
                max = max.max(start + glyph.size - Vec2::ONE);
            }
            pen += glyph.advance;
        }
        if min.is_finite() {
            let baseline = center - (min + max) * (0.5 * size / GLYPH_PX);
            self.glyph_text(
                &atlas.glyphs,
                text,
                (baseline.x, baseline.y),
                size,
                color,
                false,
            );
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
        // Unaffordable or unavailable: the icon is drawn in greyscale.
        let (tint, grey) = if enabled {
            ([1.0; 4], 0.0)
        } else {
            ([0.85, 0.85, 0.85, 0.8], 1.0)
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
            params: [0.0, 0.0, grey, 0.0],
        });
    }

    fn hovered(&self, rect: [f32; 4]) -> bool {
        self.hover.is_some_and(|p| {
            p.x >= rect[0] && p.x <= rect[0] + rect[2] && p.y >= rect[1] && p.y <= rect[1] + rect[3]
        })
    }

    /// Original Canvas label: medium italic serif, ivory fill, brown stroke.
    /// `pixels_per_world` keeps its scale consistent with the painted scene.
    pub fn gain_label(
        &mut self,
        atlas: &Atlas,
        text: &str,
        at: Vec2,
        pixels_per_world: f32,
        alpha: f32,
    ) {
        let size = 34.0 / 110.0 * pixels_per_world;
        let radius = 2.5 / 110.0 * pixels_per_world;
        for step in 0..8 {
            let angle = step as f32 * std::f32::consts::TAU / 8.0;
            let edge = at + Vec2::new(angle.cos(), angle.sin()) * radius;
            self.glyph_text(
                &atlas.gain_glyphs,
                text,
                (edge.x, edge.y),
                size,
                [74.0 / 255.0, 58.0 / 255.0, 42.0 / 255.0, alpha],
                true,
            );
        }
        self.glyph_text(
            &atlas.gain_glyphs,
            text,
            (at.x, at.y),
            size,
            [246.0 / 255.0, 240.0 / 255.0, 225.0 / 255.0, alpha],
            true,
        );
    }

    fn wrapped_lines(atlas: &Atlas, text: &str, size: f32, width: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in text.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if !line.is_empty() && Self::text_width(atlas, &candidate, size) > width {
                lines.push(std::mem::take(&mut line));
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines
    }

    fn toast(&mut self, atlas: &Atlas, toast: Option<&str>, width: f32, s: f32, top: f32) {
        if let Some(text) = toast {
            // Below the resource coins (and the paused pill), so it never
            // covers the counts on a narrow screen.
            let lines = Self::wrapped_lines(atlas, text, 14.0 * s, width - 60.0 * s);
            let w = lines
                .iter()
                .map(|line| Self::text_width(atlas, line, 14.0 * s))
                .fold(0.0_f32, f32::max)
                + 36.0 * s;
            self.shape(
                [
                    (width - w) / 2.0,
                    top,
                    w,
                    (18.0 + 18.0 * lines.len() as f32) * s,
                ],
                GLASS,
                1.0,
                17.0 * s,
            );
            for (index, line) in lines.iter().enumerate() {
                self.text(
                    atlas,
                    line,
                    (width / 2.0, top + (22.0 + index as f32 * 18.0) * s),
                    14.0 * s,
                    INK,
                    true,
                );
            }
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
