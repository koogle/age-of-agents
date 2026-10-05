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
    rows: u16,
    cells: String,
    heights: String,
}

pub(super) fn serialize<S: Serializer>(
    terrain: &[SnapshotTerrainCell],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let cells: String = terrain
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
    let heights: String = terrain
        .iter()
        .map(|cell| match cell.elevation {
            None => '.',
            Some(e) => {
                char::from(DIGITS[((e.clamp(-1.0, 1.0) + 1.0) / 2.0 * 63.0).round() as usize])
            }
        })
        .collect();
    Compact {
        columns: terrain.iter().take_while(|c| c.row == 0).count() as u16,
        rows: terrain.last().map_or(0, |c| c.row + 1),
        cells: pack(&cells),
        heights: pack(&heights),
    }
    .serialize(serializer)
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<SnapshotTerrainCell>, D::Error> {
    let compact = Compact::deserialize(deserializer)?;
    let limit = usize::from(compact.columns) * usize::from(compact.rows);
    let cells = unpack(&compact.cells, limit).map_err(D::Error::custom)?;
    let heights = unpack(&compact.heights, limit).map_err(D::Error::custom)?;
    if compact.columns == 0 || cells.len() != limit || heights.len() != limit {
        return Err(D::Error::custom("terrain cells and heights disagree"));
    }
    cells
        .bytes()
        .zip(heights.bytes())
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

// Long unseen/ocean runs are common between islands. The marker does not occur
// in either alphabet, so literal characters and encoded runs are unambiguous.
fn pack(input: &str) -> String {
    use std::fmt::Write;
    let mut result = String::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let count = bytes[i..].iter().take_while(|&&b| b == bytes[i]).count();
        if count >= 8 {
            write!(result, "~{count}:{}", char::from(bytes[i])).unwrap();
        } else {
            result.push_str(&input[i..i + count]);
        }
        i += count;
    }
    result
}

fn unpack(input: &str, limit: usize) -> Result<String, &'static str> {
    if !input.is_ascii() {
        return Err("non-ASCII terrain");
    }
    let mut result = String::new();
    let mut rest = input;
    while !rest.is_empty() {
        let (count, value, used) = if let Some(run) = rest.strip_prefix('~') {
            let (count, tail) = run.split_once(':').ok_or("invalid terrain run")?;
            let length = count
                .parse::<usize>()
                .map_err(|_| "invalid terrain run length")?;
            if length == 0 || tail.is_empty() {
                return Err("empty terrain run");
            }
            (length, tail.as_bytes()[0], count.len() + 3)
        } else {
            (1, rest.as_bytes()[0], 1)
        };
        if count > limit.saturating_sub(result.len()) {
            return Err("terrain run exceeds map bounds");
        }
        result.extend(std::iter::repeat_n(char::from(value), count));
        rest = &rest[used..];
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runs_preserve_both_alphabets_and_reject_invalid_lengths() {
        for input in ["..AAaaJL", "................aBc", "1111111111--AZ_...."] {
            assert_eq!(unpack(&pack(input), input.len()).unwrap(), input);
        }
        for input in [
            "~0:.",
            "~999999999999999999999:.",
            "~10:",
            "~9:.",
            "~x:a",
            "é",
        ] {
            assert!(unpack(input, 8).is_err(), "{input}");
        }
    }
}
