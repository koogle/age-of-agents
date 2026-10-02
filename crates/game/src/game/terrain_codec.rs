//! Compact snapshot terrain: one character per cell for visibility and biome,
//! one for elevation, instead of a JSON object per cell. A 60x40 map drops
//! from about 110 KB to under 5 KB per snapshot.
//!
//! `cells`: `.` unseen; explored `a`.., visible `A`.., offset by biome index.
//! `heights`: `.` unknown; otherwise one base64url digit, elevation quantized
//! over [-1, 1] in 64 steps (snapshots carry already-quantized values, so the
//! encoding round-trips exactly).

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::*;

const BIOMES: [TerrainBiome; 12] = [
    TerrainBiome::Meadow,
    TerrainBiome::Forest,
    TerrainBiome::Prairie,
    TerrainBiome::Highland,
    TerrainBiome::Wetland,
    TerrainBiome::Scrubland,
    TerrainBiome::Heath,
    TerrainBiome::Clayland,
    TerrainBiome::Beach,
    TerrainBiome::Water,
    TerrainBiome::Mountain,
    TerrainBiome::River,
];
const DIGITS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// The elevation a client sees: quantized to the 64 encodable steps.
pub(super) fn quantize(elevation: f32) -> f32 {
    let step = ((elevation.clamp(-1.0, 1.0) + 1.0) / 2.0 * 63.0).round();
    step / 63.0 * 2.0 - 1.0
}

#[derive(Serialize, Deserialize)]
struct Compact {
    columns: u16,
    cells: String,
    heights: String,
}

pub(super) fn serialize<S: Serializer>(
    terrain: &[SnapshotTerrainCell],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let cells = terrain
        .iter()
        .map(|cell| match (cell.visibility, cell.biome) {
            (CellVisibility::Unseen, _) | (_, None) => '.',
            (visibility, Some(biome)) => {
                let index = BIOMES
                    .iter()
                    .position(|b| *b == biome)
                    .expect("every biome is encodable") as u8;
                char::from(
                    if visibility == CellVisibility::Visible {
                        b'A'
                    } else {
                        b'a'
                    } + index,
                )
            }
        })
        .collect();
    let heights = terrain
        .iter()
        .map(|cell| match cell.elevation {
            None => '.',
            Some(e) => {
                char::from(DIGITS[((e.clamp(-1.0, 1.0) + 1.0) / 2.0 * 63.0).round() as usize])
            }
        })
        .collect();
    Compact {
        columns: WORLD_COLUMNS,
        cells,
        heights,
    }
    .serialize(serializer)
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<SnapshotTerrainCell>, D::Error> {
    let compact = Compact::deserialize(deserializer)?;
    if compact.columns == 0 || compact.cells.len() != compact.heights.len() {
        return Err(D::Error::custom("terrain cells and heights disagree"));
    }
    compact
        .cells
        .bytes()
        .zip(compact.heights.bytes())
        .enumerate()
        .map(|(index, (cell, height))| {
            let (visibility, biome) = match cell {
                b'.' => (CellVisibility::Unseen, None),
                b'a'..=b'l' => (
                    CellVisibility::Explored,
                    Some(BIOMES[usize::from(cell - b'a')]),
                ),
                b'A'..=b'L' => (
                    CellVisibility::Visible,
                    Some(BIOMES[usize::from(cell - b'A')]),
                ),
                other => return Err(D::Error::custom(format!("bad terrain cell {other}"))),
            };
            let elevation = match height {
                b'.' => None,
                digit => {
                    let step = DIGITS
                        .iter()
                        .position(|d| *d == digit)
                        .ok_or_else(|| D::Error::custom("bad height"))?;
                    Some(step as f32 / 63.0 * 2.0 - 1.0)
                }
            };
            Ok(SnapshotTerrainCell {
                column: (index % usize::from(compact.columns)) as u16,
                row: (index / usize::from(compact.columns)) as u16,
                biome,
                elevation,
                visibility,
            })
        })
        .collect()
}
