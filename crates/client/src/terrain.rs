//! The island: a gently rolling playable rectangle that falls away to beaches
//! and sea. Heights come from a fixed noise field (never from biome data) so
//! the shape of the land reveals nothing about unexplored terrain.
use aoa_game::{CellCoordinate, TerrainBiome, WORLD_COLUMNS, WORLD_ROWS};
use bytemuck::{Pod, Zeroable};
use glam::Vec2;

/// World units per simulation cell. The grid is finer than a villager is
/// tall, so the island keeps its size while cells shrink.
pub const CELL: f32 = 0.5;
/// The playable rectangle in world units.
pub const COLUMNS: f32 = WORLD_COLUMNS as f32 * CELL;
pub const ROWS: f32 = WORLD_ROWS as f32 * CELL;

/// World ground point of a simulation position given in cell units.
pub fn world_of(x: f64, y: f64) -> Vec2 {
    Vec2::new(x as f32, y as f32) * CELL
}

/// World ground point at the centre of a cell.
pub fn cell_center(cell: CellCoordinate) -> Vec2 {
    Vec2::new(cell.column as f32 + 0.5, cell.row as f32 + 0.5) * CELL
}

/// The cell under a world ground point, if it lies on the map.
pub fn cell_at(x: f32, z: f32) -> Option<CellCoordinate> {
    let (column, row) = ((x / CELL).floor(), (z / CELL).floor());
    (column >= 0.0 && row >= 0.0 && column < WORLD_COLUMNS as f32 && row < WORLD_ROWS as f32)
        .then(|| CellCoordinate::new(column as u16, row as u16))
}
pub const SEA_LEVEL: f32 = -0.32;
const MARGIN: f32 = 9.0;
const SUBDIVISIONS: u32 = 3;

/// The ten painted ground layers, in texture-array order.
pub const GROUND_LAYERS: [&str; 10] = [
    "meadow",
    "forest",
    "prairie",
    "highland",
    "wetland",
    "scrubland",
    "heath",
    "clayland",
    "beach",
    "shallows",
];

pub fn biome_layer(biome: TerrainBiome) -> u8 {
    match biome {
        TerrainBiome::Meadow => 0,
        TerrainBiome::Forest => 1,
        TerrainBiome::Prairie => 2,
        TerrainBiome::Highland => 3,
        TerrainBiome::Wetland => 4,
        TerrainBiome::Scrubland => 5,
        TerrainBiome::Heath => 6,
        TerrainBiome::Clayland => 7,
    }
}

pub fn biome_color(biome: TerrainBiome) -> [u8; 3] {
    match biome {
        TerrainBiome::Meadow => [176, 204, 92],
        TerrainBiome::Forest => [108, 156, 74],
        TerrainBiome::Prairie => [230, 200, 104],
        TerrainBiome::Highland => [214, 206, 182],
        TerrainBiome::Wetland => [118, 184, 128],
        TerrainBiome::Scrubland => [206, 180, 112],
        TerrainBiome::Heath => [166, 176, 104],
        TerrainBiome::Clayland => [214, 142, 92],
    }
}
pub const UNSEEN_COLOR: [u8; 3] = [233, 216, 176];

fn hash(x: f32, y: f32) -> f32 {
    let s = (x * 127.1 + y * 311.7).sin() * 43758.547;
    s - s.floor()
}

fn noise(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (x - ix, y - iy);
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (a, b, c, d) = (
        hash(ix, iy),
        hash(ix + 1.0, iy),
        hash(ix, iy + 1.0),
        hash(ix + 1.0, iy + 1.0),
    );
    a + (b - a) * ux + (c - a) * uy + (a - b - c + d) * ux * uy
}

/// Deterministic pseudo-random value in [0, 1) for decoration choices.
pub fn random(seed: f32) -> f32 {
    hash(seed * 0.731, seed * 1.173)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn outside_distance(x: f32, z: f32) -> f32 {
    let dx = (-x).max(x - COLUMNS).max(0.0);
    let dz = (-z).max(z - ROWS).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

pub fn height_at(x: f32, z: f32) -> f32 {
    let rolling = (noise(x * 0.18, z * 0.18) - 0.5) * 0.16
        + (noise(x * 0.6 + 9.0, z * 0.6 + 3.0) - 0.5) * 0.04;
    let coast = outside_distance(x, z) + (noise(x * 0.3 + 40.0, z * 0.3) - 0.5) * 1.6;
    let fall = smoothstep(0.8, 4.2, coast);
    let rocky = if fall > 0.6 {
        (noise(x * 0.9, z * 0.9) - 0.5) * 0.2
    } else {
        0.0
    };
    rolling * (1.0 - fall) - fall * 0.9 + rocky
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GroundVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub shade: f32,
}

pub struct Mesh<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}

fn grid(
    width: f32,
    depth: f32,
    origin: (f32, f32),
    steps: (u32, u32),
) -> (Vec<(f32, f32)>, Vec<u32>) {
    let mut points = Vec::new();
    for j in 0..=steps.1 {
        for i in 0..=steps.0 {
            points.push((
                origin.0 + width * i as f32 / steps.0 as f32,
                origin.1 + depth * j as f32 / steps.1 as f32,
            ));
        }
    }
    let mut indices = Vec::new();
    let stride = steps.0 + 1;
    for j in 0..steps.1 {
        for i in 0..steps.0 {
            let a = j * stride + i;
            indices.extend_from_slice(&[a, a + stride, a + 1, a + 1, a + stride, a + stride + 1]);
        }
    }
    (points, indices)
}

pub fn ground_mesh() -> Mesh<GroundVertex> {
    let (width, depth) = (COLUMNS + MARGIN * 2.0, ROWS + MARGIN * 2.0);
    let steps = (width as u32 * SUBDIVISIONS, depth as u32 * SUBDIVISIONS);
    let (points, indices) = grid(width, depth, (-MARGIN, -MARGIN), steps);
    let e = 0.05;
    let vertices = points
        .into_iter()
        .map(|(x, z)| {
            let n = [
                height_at(x - e, z) - height_at(x + e, z),
                2.0 * e,
                height_at(x, z - e) - height_at(x, z + e),
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            GroundVertex {
                position: [x, height_at(x, z), z],
                normal: [n[0] / len, n[1] / len, n[2] / len],
                shade: 0.9 + noise(x * 1.7, z * 1.7) * 0.2,
            }
        })
        .collect();
    Mesh { vertices, indices }
}

pub fn sea_mesh() -> Mesh<[f32; 3]> {
    let (points, indices) = grid(
        400.0,
        400.0,
        (COLUMNS / 2.0 - 200.0, ROWS / 2.0 - 200.0),
        (80, 80),
    );
    Mesh {
        vertices: points.into_iter().map(|(x, z)| [x, SEA_LEVEL, z]).collect(),
        indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_playable_rectangle_stays_above_the_sea() {
        for z in 0..ROWS as u32 {
            for x in 0..COLUMNS as u32 {
                assert!(height_at(x as f32 + 0.5, z as f32 + 0.5) > SEA_LEVEL + 0.1);
            }
        }
        assert!(height_at(-8.0, -8.0) < SEA_LEVEL);
    }
}
