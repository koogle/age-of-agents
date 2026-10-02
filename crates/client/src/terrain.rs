//! The island as drawn: the simulation's per-cell elevation (known only for
//! explored cells) becomes a smooth height field, sea outside the map. Cells
//! nobody has explored sit flat under the fog clouds, so their shape stays
//! hidden.
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
const MARGIN: f32 = 4.0;
const SUBDIVISIONS: u32 = 4;

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
        TerrainBiome::Beach => 8,
        TerrainBiome::Water => 9,
        TerrainBiome::Meadow => 0,
        TerrainBiome::Forest => 1,
        TerrainBiome::Prairie => 2,
        TerrainBiome::Highland => 3,
        TerrainBiome::Wetland => 4,
        TerrainBiome::Scrubland => 5,
        TerrainBiome::Heath => 6,
        TerrainBiome::Clayland => 7,
        // Neither has a painted layer of its own: the shader lays running
        // water over the shallows paint, and rock and snow over the highland's.
        TerrainBiome::River => 10,
        TerrainBiome::Mountain => 11,
    }
}

pub fn biome_color(biome: TerrainBiome) -> [u8; 3] {
    match biome {
        TerrainBiome::Beach => [236, 209, 153],
        TerrainBiome::Water => [96, 172, 196],
        TerrainBiome::Meadow => [176, 204, 92],
        TerrainBiome::Forest => [108, 156, 74],
        TerrainBiome::Prairie => [230, 200, 104],
        TerrainBiome::Highland => [214, 206, 182],
        TerrainBiome::Wetland => [118, 184, 128],
        TerrainBiome::Scrubland => [206, 180, 112],
        TerrainBiome::Heath => [166, 176, 104],
        TerrainBiome::Clayland => [214, 142, 92],
        TerrainBiome::Mountain => [156, 148, 138],
        TerrainBiome::River => [86, 166, 200],
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

/// Ground height of explored cells nobody has seen yet: flat, under the clouds.
const UNKNOWN_HEIGHT: f32 = 0.06;
/// Open sea floor beyond the map.
const SEA_FLOOR: f32 = -1.1;

/// How far a river bed sits below its banks.
const RIVER_DEPTH: f32 = 0.14;

/// World-unit height of a simulation elevation: land rises gently from the
/// shore through rolling hills and steepens into peaks; water drops below the
/// sea surface.
fn world_height(elevation: f32) -> f32 {
    if elevation > 0.0 {
        let peak = elevation * elevation * elevation * elevation;
        0.04 + elevation * 0.8 + peak * 1.5
    } else {
        SEA_LEVEL - 0.15 + elevation * 0.6
    }
}

/// Per-cell heights, interpolated between cell centres.
#[derive(Clone)]
pub struct Heights {
    cells: Vec<f32>,
}

impl Heights {
    pub fn unknown() -> Self {
        Self {
            cells: vec![UNKNOWN_HEIGHT; usize::from(WORLD_COLUMNS) * usize::from(WORLD_ROWS)],
        }
    }

    /// Heights from snapshot elevations (`None` for unexplored cells); river
    /// beds sink below their banks.
    pub fn from_cells(cells: impl Iterator<Item = (Option<f32>, Option<TerrainBiome>)>) -> Self {
        Self {
            cells: cells
                .map(|(elevation, biome)| match elevation {
                    None => UNKNOWN_HEIGHT,
                    Some(e) if biome == Some(TerrainBiome::River) => world_height(e) - RIVER_DEPTH,
                    Some(e) => world_height(e),
                })
                .collect(),
        }
    }

    fn cell(&self, column: i32, row: i32) -> f32 {
        if column < 0
            || row < 0
            || column >= i32::from(WORLD_COLUMNS)
            || row >= i32::from(WORLD_ROWS)
        {
            return SEA_FLOOR;
        }
        self.cells[row as usize * usize::from(WORLD_COLUMNS) + column as usize]
    }

    /// Smooth ground height at a world point.
    pub fn at(&self, x: f32, z: f32) -> f32 {
        let (gx, gz) = (x / CELL - 0.5, z / CELL - 0.5);
        let (c, r) = (gx.floor(), gz.floor());
        let (fx, fz) = (gx - c, gz - r);
        let (fx, fz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
        let (c, r) = (c as i32, r as i32);
        let top = self.cell(c, r) + (self.cell(c + 1, r) - self.cell(c, r)) * fx;
        let bottom = self.cell(c, r + 1) + (self.cell(c + 1, r + 1) - self.cell(c, r + 1)) * fx;
        let base = top + (bottom - top) * fz;
        // A little painterly roll so slopes are never perfectly planar.
        base + (noise(x * 0.9, z * 0.9) - 0.5) * 0.05
    }
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

pub fn ground_mesh(heights: &Heights) -> Mesh<GroundVertex> {
    let (width, depth) = (COLUMNS + MARGIN * 2.0, ROWS + MARGIN * 2.0);
    let steps = (width as u32 * SUBDIVISIONS, depth as u32 * SUBDIVISIONS);
    let (points, indices) = grid(width, depth, (-MARGIN, -MARGIN), steps);
    let e = 0.05;
    let vertices = points
        .into_iter()
        .map(|(x, z)| {
            let n = [
                heights.at(x - e, z) - heights.at(x + e, z),
                2.0 * e,
                heights.at(x, z - e) - heights.at(x, z + e),
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            GroundVertex {
                position: [x, heights.at(x, z), z],
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
    fn land_stands_above_the_sea_and_water_below_it() {
        let land = Heights::from_cells((0..2400).map(|_| (Some(0.4), None)));
        let water = Heights::from_cells((0..2400).map(|_| (Some(-0.4), None)));
        assert!(land.at(15.0, 10.0) > SEA_LEVEL + 0.2);
        assert!(water.at(15.0, 10.0) < SEA_LEVEL - 0.1);
        assert!(
            land.at(-6.0, -6.0) < SEA_LEVEL,
            "beyond the map is open sea"
        );
    }

    #[test]
    fn peaks_tower_over_hills_and_rivers_sink_into_their_banks() {
        assert!(
            world_height(1.0) - world_height(0.8) > (world_height(0.4) - world_height(0.2)) * 1.5
        );
        let river = Heights::from_cells((0..2400).map(|_| (Some(0.4), Some(TerrainBiome::River))));
        let banks = Heights::from_cells((0..2400).map(|_| (Some(0.4), Some(TerrainBiome::Meadow))));
        assert!(river.at(15.0, 10.0) < banks.at(15.0, 10.0) - 0.1);
        assert!(river.at(15.0, 10.0) > SEA_LEVEL, "rivers run above the sea");
    }
}
