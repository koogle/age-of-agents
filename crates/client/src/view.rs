//! Turns authoritative snapshots into what is drawn. Nothing here decides
//! game outcomes; it only interpolates and dresses what the simulation says.
use std::collections::{HashMap, VecDeque};

use aoa_game::{
    CellVisibility, GatherPhase, ResourceKind, TerrainBiome, UnitAction, WorldSnapshot,
};
use glam::{Vec2, Vec3};
use serde::Deserialize;

use crate::camera::Rig;
use crate::render::{Decal, Sprite};
use crate::terrain::{self, Heights, random};

const VILLAGER_HEIGHT: f32 = 0.78;
/// Units are drawn this many ticks behind the newest snapshot so jittery
/// arrivals always have a sample to interpolate toward.
const PLAYOUT_TICKS: f64 = 1.6;
const TICK_SECONDS: f64 = 0.1;
const TEAM_BLUE: [f32; 4] = [0.184, 0.435, 0.878, 0.9];
const SHEET_RESOURCES: usize = 3;
const SHEET_TOWN_CENTER: usize = 4;
const SHEET_IDLE_HD: usize = 5;
const SHEET_BUILDINGS: usize = 6;
/// Rows of the HD idle sheet, in villager sheet order (0, 1, 2).
const PEOPLE: [&str; 3] = ["villager", "villager_woman", "villager_elder"];

#[derive(Deserialize)]
struct VillagerSheet {
    cell: [f32; 2],
    anchor: [f32; 2],
    #[serde(rename = "figureHeight")]
    figure_height: f32,
    fps: HashMap<String, f32>,
    animations: HashMap<String, HashMap<String, Vec<[f32; 4]>>>,
}

/// Generated buildings in four construction stages (foundation, walls, roof,
/// complete), one row per building, each drawn to fill its cell.
#[derive(Deserialize)]
struct BuildingSheet {
    size: [f32; 2],
    cell: [f32; 2],
    frames: HashMap<String, Vec<[f32; 4]>>,
}

/// Sheet row, world width of a sheet cell, and the pixel height (from the cell
/// top) of the footprint centre, for buildings other than the town center.
/// Widths are set so each door stands about 1.2 villagers tall (the dock's
/// rowboat is about two villagers long).
fn building_art(kind: aoa_game::BuildingKind) -> Option<(&'static str, f32, f32)> {
    use aoa_game::BuildingKind::*;
    Some(match kind {
        House => ("house", 4.1, 205.0),
        Granary => ("granary", 3.9, 199.0),
        Watchtower => ("watchtower", 5.0, 228.0),
        Dock => ("dock", 4.5, 193.0),
        _ => return None,
    })
}

/// Standing frames at twice the resolution of `villager.json`, with the same
/// anchor after scaling: idle villagers are on screen most of the time.
#[derive(Deserialize)]
struct IdleSheet {
    size: [f32; 2],
    people: HashMap<String, HashMap<String, Vec<[f32; 4]>>>,
}

#[derive(Deserialize)]
struct ResourceNodeSheet {
    stages: Vec<[f32; 4]>,
    #[serde(rename = "unitsPerPixel")]
    units_per_pixel: f32,
}

#[derive(Deserialize)]
struct ResourceSheet {
    size: [f32; 2],
    cell: [f32; 2],
    anchor: [f32; 2],
    nodes: HashMap<String, ResourceNodeSheet>,
}

#[derive(Deserialize)]
struct TownCenterSheet {
    size: [f32; 2],
    cell: [f32; 2],
    anchor: [f32; 2],
    #[serde(rename = "unitsPerPixel")]
    units_per_pixel: f32,
    frames: HashMap<String, [f32; 4]>,
    /// (frame, construction progress from which it shows).
    #[serde(rename = "constructionStages")]
    construction_stages: Vec<(String, f64)>,
}

pub struct Sheets {
    villager: VillagerSheet,
    idle: IdleSheet,
    buildings: BuildingSheet,
    resources: ResourceSheet,
    town_center: TownCenterSheet,
}

impl Sheets {
    pub fn parse(
        villager: &[u8],
        idle: &[u8],
        resources: &[u8],
        town_center: &[u8],
        buildings: &[u8],
    ) -> Self {
        Self {
            buildings: serde_json::from_slice(buildings).expect("buildings.json"),
            villager: serde_json::from_slice(villager).expect("villager.json"),
            idle: serde_json::from_slice(idle).expect("villager_idle_hd.json"),
            resources: serde_json::from_slice(resources).expect("resources.json"),
            town_center: serde_json::from_slice(town_center).expect("towncenter.json"),
        }
    }
}

struct UnitEntry {
    samples: VecDeque<(f64, Vec3)>,
    position: Vec3,
    velocity: Vec3,
    facing: f32,
    variant: usize,
    /// Eased offset toward the thing being worked on, so a working villager
    /// stands right against it rather than at its cell centre.
    lean: Vec2,
}

pub struct WorldView {
    pub snapshot: Option<WorldSnapshot>,
    units: HashMap<String, UnitEntry>,
    render_tick: Option<f64>,
    latest_tick: f64,
    pub cells_dirty: bool,
    /// Ground heights of explored cells; rebuilt only when exploration grows.
    pub heights: Heights,
    pub heights_dirty: bool,
    known_heights: usize,
}

fn seed_of(id: &str) -> u32 {
    id.bytes()
        .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32))
}

fn ground(heights: &Heights, x: f32, z: f32) -> Vec3 {
    Vec3::new(x, heights.at(x, z), z)
}

fn uv(rect: [f32; 4], size: [f32; 2], mirror: bool) -> [f32; 4] {
    let (u0, u1) = (rect[0] / size[0], (rect[0] + rect[2]) / size[0]);
    let (v0, v1) = (rect[1] / size[1], (rect[1] + rect[3]) / size[1]);
    if mirror {
        [u1, v0, u0, v1]
    } else {
        [u0, v0, u1, v1]
    }
}

fn activity_for(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Wood => "chop",
        ResourceKind::Stone | ResourceKind::Gold | ResourceKind::Iron | ResourceKind::Coal => {
            "mine"
        }
        ResourceKind::Clay => "dig",
        _ => "forage",
    }
}

fn node_for(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Food => "berry",
        ResourceKind::Gold => "gold",
        ResourceKind::Iron | ResourceKind::Coal => "iron",
        ResourceKind::Clay => "clay",
        ResourceKind::Fiber => "fiber",
        _ => "stone",
    }
}

impl WorldView {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            units: HashMap::new(),
            render_tick: None,
            latest_tick: 0.0,
            cells_dirty: true,
            heights: Heights::unknown(),
            heights_dirty: true,
            known_heights: 0,
        }
    }

    pub fn sync(&mut self, next: WorldSnapshot) {
        self.latest_tick = next.tick as f64;
        let render_tick = self
            .render_tick
            .get_or_insert(self.latest_tick - PLAYOUT_TICKS);
        if (self.latest_tick - PLAYOUT_TICKS - *render_tick).abs() > 8.0 {
            *render_tick = self.latest_tick - PLAYOUT_TICKS;
        }
        let known = next
            .terrain
            .iter()
            .filter(|cell| cell.elevation.is_some())
            .count();
        if known != self.known_heights {
            self.known_heights = known;
            self.heights = Heights::from_elevations(next.terrain.iter().map(|cell| cell.elevation));
            self.heights_dirty = true;
        }
        let mut seen = Vec::with_capacity(next.units.len());
        for view in &next.units {
            let at = terrain::world_of(view.position.x, view.position.y);
            let target = ground(&self.heights, at.x, at.y);
            let entry = self
                .units
                .entry(view.unit.id.clone())
                .or_insert_with(|| UnitEntry {
                    samples: VecDeque::new(),
                    position: target,
                    velocity: Vec3::ZERO,
                    facing: 0.0,
                    lean: Vec2::ZERO,
                    variant: (seed_of(&view.unit.id) % 3) as usize,
                });
            // A command snapshot repeats the current tick: replace that sample.
            match entry.samples.back_mut() {
                Some(last) if last.0 >= self.latest_tick => last.1 = target,
                _ => entry.samples.push_back((self.latest_tick, target)),
            }
            while entry.samples.len() > 6 {
                entry.samples.pop_front();
            }
            seen.push(view.unit.id.clone());
        }
        self.units.retain(|id, _| seen.contains(id));
        self.snapshot = Some(next);
        self.cells_dirty = true;
    }

    /// Advances the presentation clock and unit positions.
    pub fn frame(&mut self, dt: f32) {
        let Some(render_tick) = self.render_tick.as_mut() else {
            return;
        };
        // Run slightly fast or slow to hold the playout buffer; stop at the
        // newest tick when the simulation pauses.
        let lag = self.latest_tick - PLAYOUT_TICKS - *render_tick;
        let rate = (1.0 + lag * 0.35).clamp(0.6, 1.6);
        *render_tick = (*render_tick + dt as f64 / TICK_SECONDS * rate).min(self.latest_tick);
        let tick = *render_tick;
        for entry in self.units.values_mut() {
            let previous = entry.position;
            entry.position = sample_at(&entry.samples, tick);
            if dt > 0.0 {
                let instant = (entry.position - previous) / dt;
                entry.velocity = entry.velocity.lerp(instant, (dt * 10.0).min(1.0));
            }
        }
    }

    /// Fog-of-war cell colours (RGBA) and painted ground layers (255 = unknown).
    pub fn cell_data(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        let snapshot = self.snapshot.as_ref()?;
        let count = snapshot.columns as usize * snapshot.rows as usize;
        let (mut rgba, mut layers) = (vec![0u8; count * 4], vec![255u8; count]);
        for cell in &snapshot.terrain {
            let index = cell.row as usize * snapshot.columns as usize + cell.column as usize;
            let color = cell
                .biome
                .map(terrain::biome_color)
                .unwrap_or(terrain::UNSEEN_COLOR);
            let visibility = match cell.visibility {
                CellVisibility::Visible => 255,
                CellVisibility::Explored => 128,
                CellVisibility::Unseen => 0,
            };
            rgba[index * 4..index * 4 + 4]
                .copy_from_slice(&[color[0], color[1], color[2], visibility]);
            layers[index] = cell.biome.map(terrain::biome_layer).unwrap_or(255);
        }
        Some((rgba, layers))
    }

    fn biome_at(&self, column: u16, row: u16) -> Option<TerrainBiome> {
        let snapshot = self.snapshot.as_ref()?;
        snapshot
            .terrain
            .get(row as usize * snapshot.columns as usize + column as usize)?
            .biome
    }

    /// Everything to draw this frame: (sheet, sprite) pairs and ground decals.
    pub fn draw_list(
        &mut self,
        sheets: &Sheets,
        rig: &Rig,
        time: f32,
        dt: f32,
        selection: &Selection,
    ) -> (Vec<(usize, Sprite)>, Vec<Decal>) {
        let mut sprites = Vec::new();
        let mut decals = Vec::new();
        let heights = &self.heights;
        let Some(snapshot) = self.snapshot.as_ref() else {
            return (sprites, decals);
        };
        let resources = &sheets.resources;
        let pivot = [
            resources.anchor[0] / resources.cell[0],
            1.0 - resources.anchor[1] / resources.cell[1],
        ];
        let resource_sprite = |node: &str, rect: [f32; 4], at: Vec3, scale: f32| {
            let size = resources.cell[0] * resources.nodes[node].units_per_pixel * scale;
            Sprite {
                anchor: at.to_array(),
                size: [size, size],
                pivot,
                uv: uv(rect, resources.size, false),
            }
        };
        for resource in &snapshot.resources {
            let (column, row) = (resource.cell.column, resource.cell.row);
            let seed = seed_of(&resource.id) as f32;
            let fraction = if resource.capacity > 0.0 {
                (resource.amount / resource.capacity) as f32
            } else {
                0.0
            };
            let center = terrain::cell_center(resource.cell);
            if resource.kind == ResourceKind::Wood {
                // One tree per cell, so woodlines read as dense stands: dark
                // cypresses, or gnarled olives on the heath.
                let olive = self.biome_at(column, row) == Some(TerrainBiome::Heath);
                let (tree, base) = if olive {
                    ("olive", 0.62)
                } else {
                    ("cypress", 0.85)
                };
                let node = if fraction > 0.0 { tree } else { "stump" };
                let x = center.x + (random(seed) - 0.5) * 0.12;
                let z = center.y + (random(seed + 1.0) - 0.5) * 0.12;
                let scale = base * (0.9 + random(seed + 7.0) * 0.25);
                sprites.push((
                    SHEET_RESOURCES,
                    resource_sprite(
                        node,
                        resources.nodes[node].stages[0],
                        ground(heights, x, z),
                        scale,
                    ),
                ));
            } else if fraction > 0.0 {
                let node = node_for(resource.kind);
                let stages = &resources.nodes[node].stages;
                let stage = (((1.0 - fraction) * stages.len() as f32).floor() as usize)
                    .min(stages.len() - 1);
                let at = ground(heights, center.x, center.y);
                sprites.push((
                    SHEET_RESOURCES,
                    resource_sprite(node, stages[stage], at, 0.6 + random(seed) * 0.08),
                ));
            }
        }
        for building in &snapshot.buildings {
            let center = footprint_center(heights, building);
            if let Some((row, width, center_px)) = building_art(building.building.kind) {
                let sheet = &sheets.buildings;
                let stage = match building.building.construction {
                    None => 3,
                    Some(work) => {
                        let progress = work / aoa_game::BUILD_SECONDS;
                        if progress < 1.0 / 3.0 {
                            0
                        } else if progress < 2.0 / 3.0 {
                            1
                        } else {
                            2
                        }
                    }
                };
                sprites.push((
                    SHEET_BUILDINGS,
                    Sprite {
                        anchor: center.to_array(),
                        size: [width, width * sheet.cell[1] / sheet.cell[0]],
                        pivot: [0.5, 1.0 - center_px / sheet.cell[1]],
                        uv: uv(sheet.frames[row][stage], sheet.size, false),
                    },
                ));
                if selection.building.as_deref() == Some(building.building.id.as_str()) {
                    decals.push(Decal {
                        center: (center + Vec3::Y * 0.03).to_array(),
                        radius: building.columns as f32 * terrain::CELL * 0.75,
                        color: TEAM_BLUE,
                        ring: 1.0,
                    });
                }
                continue;
            }
            // The temple rises through its drawn stages, then glows while working.
            let tc = &sheets.town_center;
            let frame = town_center_frame(
                tc,
                building.building.construction,
                building.building.job.is_some(),
            );
            // The temple sheet is drawn for a 4-cell footprint; scale it to
            // the town center's real one.
            let size = tc.cell[0] * tc.units_per_pixel * building.columns as f32 / 4.0;
            sprites.push((
                SHEET_TOWN_CENTER,
                Sprite {
                    anchor: center.to_array(),
                    size: [size, size * tc.cell[1] / tc.cell[0]],
                    pivot: [tc.anchor[0] / tc.cell[0], 1.0 - tc.anchor[1] / tc.cell[1]],
                    uv: uv(tc.frames[frame], tc.size, false),
                },
            ));
            if selection.building.as_deref() == Some(building.building.id.as_str()) {
                decals.push(Decal {
                    center: (center + Vec3::Y * 0.03).to_array(),
                    radius: building.columns as f32 * terrain::CELL * 0.75,
                    color: TEAM_BLUE,
                    ring: 1.0,
                });
            }
        }

        let villager = &sheets.villager;
        let cell_size = VILLAGER_HEIGHT * villager.cell[1] / villager.figure_height;
        let pivot = [
            villager.anchor[0] / villager.cell[0],
            1.0 - villager.anchor[1] / villager.cell[1],
        ];
        let (right, _) = rig.basis();
        let forward = Vec3::new(rig.target.x - rig.eye().x, 0.0, rig.target.z - rig.eye().z)
            .normalize_or_zero();
        for unit in &snapshot.units {
            let Some(entry) = self.units.get_mut(&unit.unit.id) else {
                continue;
            };
            let moving = Vec2::new(entry.velocity.x, entry.velocity.z).length() > 0.05;
            let work = if moving {
                None
            } else {
                work_target(heights, snapshot, &unit.unit.action)
            };
            let carrying = unit.unit.cargo.is_some();
            // A villager holding goods shows its load even when it stops
            // (interrupted, or waiting for a drop site): the carry pose, held still.
            let mut name = if carrying {
                "carry"
            } else if moving {
                "walk"
            } else {
                "idle"
            };
            let mut desired = entry.facing;
            if moving {
                desired = entry.velocity.x.atan2(entry.velocity.z);
            }
            let mut lean = Vec2::ZERO;
            if let Some((target, activity)) = work {
                name = activity;
                desired = (target.x - entry.position.x).atan2(target.y - entry.position.z);
                let toward = target - Vec2::new(entry.position.x, entry.position.z);
                lean = toward.normalize_or_zero() * (toward.length() - 0.3).clamp(0.0, 0.2);
            }
            entry.lean = entry.lean.lerp(lean, (dt * 6.0).min(1.0));
            let position = ground(
                heights,
                entry.position.x + entry.lean.x,
                entry.position.z + entry.lean.y,
            );
            let turn = (desired - entry.facing)
                .sin()
                .atan2((desired - entry.facing).cos());
            entry.facing += turn * (dt * 12.0).min(1.0);
            let direction = Vec3::new(entry.facing.sin(), 0.0, entry.facing.cos());
            let animation = villager
                .animations
                .get(name)
                .unwrap_or(&villager.animations["idle"]);
            let toward_viewer = direction.dot(forward) < 0.0;
            let screen_right = direction.dot(Vec3::new(right.x, 0.0, right.z)) > 0.0;
            let view = if toward_viewer || !animation.contains_key("back") {
                "front"
            } else {
                "back"
            };
            // Front frames look toward viewer-left, back frames toward viewer-right.
            let mirror = if view == "front" {
                screen_right
            } else {
                !screen_right
            };
            let fps = if name == "carry" && !moving {
                0.0
            } else {
                villager.fps.get(name).copied().unwrap_or(4.0)
            };
            let (sheet, frames, sheet_size) = if name == "idle" {
                let idle = &sheets.idle;
                (
                    SHEET_IDLE_HD,
                    &idle.people[PEOPLE[entry.variant]][view],
                    idle.size,
                )
            } else {
                (entry.variant, &animation[view], [2048.0, 1280.0])
            };
            let rect = frames[(time * fps) as usize % frames.len()];
            sprites.push((
                sheet,
                Sprite {
                    anchor: position.to_array(),
                    size: [cell_size, cell_size],
                    pivot,
                    uv: uv(rect, sheet_size, mirror),
                },
            ));
            decals.push(Decal {
                center: (position + Vec3::Y * 0.015).to_array(),
                radius: 0.2,
                color: [0.165, 0.165, 0.118, 0.28],
                ring: 0.0,
            });
            if selection.units.contains(&unit.unit.id) {
                decals.push(Decal {
                    center: (position + Vec3::Y * 0.02).to_array(),
                    radius: 0.32,
                    color: TEAM_BLUE,
                    ring: 1.0,
                });
            }
        }
        (sprites, decals)
    }

    /// The unit whose sprite covers a screen pixel, nearest the camera first.
    pub fn unit_at(&self, rig: &Rig, pixel: Vec2) -> Option<String> {
        let (_, up) = rig.basis();
        let mut best: Option<(f32, &String)> = None;
        for (id, entry) in &self.units {
            let (Some(foot), Some(head)) = (
                rig.screen_of(entry.position),
                rig.screen_of(entry.position + up * VILLAGER_HEIGHT),
            ) else {
                continue;
            };
            let height = (foot.y - head.y).max(8.0);
            let half_width = height * 0.28;
            let inside = pixel.x > foot.x - half_width
                && pixel.x < foot.x + half_width
                && pixel.y < foot.y + 4.0
                && pixel.y > head.y;
            let depth = rig.eye().distance(entry.position);
            if inside && best.is_none_or(|(d, _)| depth < d) {
                best = Some((depth, id));
            }
        }
        best.map(|(_, id)| id.clone())
    }
}

fn work_target(
    heights: &Heights,
    snapshot: &WorldSnapshot,
    action: &UnitAction,
) -> Option<(Vec2, &'static str)> {
    match action {
        UnitAction::Gather {
            resource_id,
            phase: GatherPhase::Gathering,
        } => {
            let resource = snapshot.resources.iter().find(|r| &r.id == resource_id)?;
            Some((
                terrain::cell_center(resource.cell),
                activity_for(resource.kind),
            ))
        }
        UnitAction::Build { building_id } => {
            let building = snapshot
                .buildings
                .iter()
                .find(|b| &b.building.id == building_id)?;
            let center = footprint_center(heights, building);
            let center = Vec2::new(center.x, center.z);
            Some((center, "build"))
        }
        _ => None,
    }
}

/// World point at the middle of a building's footprint.
pub fn footprint_center(heights: &Heights, building: &aoa_game::BuildingView) -> Vec3 {
    let origin = &building.building.origin;
    ground(
        heights,
        (origin.column as f32 + building.columns as f32 / 2.0) * terrain::CELL,
        (origin.row as f32 + building.rows as f32 / 2.0) * terrain::CELL,
    )
}

/// The temple's drawn stage: construction frames by progress, then complete or working.
fn town_center_frame(sheet: &TownCenterSheet, construction: Option<f64>, working: bool) -> &str {
    match construction {
        Some(work) => {
            let progress = work / aoa_game::BUILD_SECONDS;
            sheet
                .construction_stages
                .iter()
                .rev()
                .find(|(_, from)| progress >= *from)
                .map(|(name, _)| name.as_str())
                .unwrap_or("foundation")
        }
        None if working => "working",
        None => "complete",
    }
}

fn sample_at(samples: &VecDeque<(f64, Vec3)>, tick: f64) -> Vec3 {
    let mut i = samples.len() - 1;
    while i > 0 && samples[i - 1].0 > tick {
        i -= 1;
    }
    let b = samples[i];
    if i == 0 || tick >= b.0 {
        return b.1;
    }
    let a = samples[i - 1];
    if tick <= a.0 {
        return a.1;
    }
    a.1.lerp(b.1, ((tick - a.0) / (b.0 - a.0)) as f32)
}

#[derive(Default)]
pub struct Selection {
    pub units: Vec<String>,
    pub building: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::GameWorld;

    fn sheet() -> TownCenterSheet {
        serde_json::from_str(include_str!("../../../assets/sprites/towncenter.json")).unwrap()
    }

    #[test]
    fn the_temple_rises_through_its_drawn_stages() {
        let sheet = sheet();
        let at = |seconds: f64| town_center_frame(&sheet, Some(seconds), false);
        assert_eq!(at(0.0), "foundation");
        assert_eq!(at(aoa_game::BUILD_SECONDS * 0.2), "build33");
        assert_eq!(at(aoa_game::BUILD_SECONDS * 0.6), "build66");
        assert_eq!(town_center_frame(&sheet, None, false), "complete");
        assert_eq!(town_center_frame(&sheet, None, true), "working");
        for frame in ["foundation", "build33", "build66", "complete", "working"] {
            assert!(sheet.frames.contains_key(frame));
        }
    }

    #[test]
    fn samples_interpolate_between_ticks_and_hold_at_the_ends() {
        let samples: VecDeque<_> = [(1.0, Vec3::ZERO), (2.0, Vec3::X)].into();
        assert_eq!(sample_at(&samples, 0.5), Vec3::ZERO);
        assert!((sample_at(&samples, 1.25) - Vec3::new(0.25, 0.0, 0.0)).length() < 1e-6);
        assert_eq!(sample_at(&samples, 3.0), Vec3::X);
    }

    #[test]
    fn the_presentation_clock_trails_the_newest_tick_and_never_passes_it() {
        let mut world = GameWorld::default();
        let mut view = WorldView::new();
        for _ in 0..20 {
            world.tick(0.1);
            view.sync(world.snapshot());
            view.frame(0.1);
        }
        let render = view.render_tick.unwrap();
        assert!(render <= view.latest_tick);
        assert!(view.latest_tick - render < PLAYOUT_TICKS + 1.0);
        // A paused simulation stops the clock at the newest tick.
        for _ in 0..50 {
            view.frame(0.1);
        }
        assert_eq!(view.render_tick.unwrap(), view.latest_tick);
    }
}
