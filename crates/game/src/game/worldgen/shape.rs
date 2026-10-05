//! Large-scale coastline masks, independent of the relief laid onto them.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Shape {
    Rounded,
    Square,
    Long,
    Bay,
    Lobed,
}

pub(super) fn kind(seed: u64) -> Shape {
    match mix(seed, 0x5348415045) % 5 {
        0 => Shape::Rounded,
        1 => Shape::Square,
        2 => Shape::Long,
        3 => Shape::Bay,
        _ => Shape::Lobed,
    }
}

pub(super) fn land_share(seed: u64) -> f64 {
    match kind(seed) {
        Shape::Square | Shape::Long => 0.42,
        Shape::Bay => 0.48,
        _ => 0.55,
    }
}

pub(super) fn outline(seed: u64, detail: u64) -> Vec<f64> {
    let shape = kind(seed);
    let flip = mix(seed, 81) & 1 == 0;
    (0..COLUMNS * ROWS)
        .map(|i| {
            let (x, y) = ((i % COLUMNS) as f64 + 0.5, (i / COLUMNS) as f64 + 0.5);
            let nx = (x - COLUMNS as f64 / 2.0) / (COLUMNS as f64 * 0.46);
            let ny = (y - ROWS as f64 / 2.0) / (ROWS as f64 * 0.44);
            // Low-frequency coordinate warping bends long coasts and bay mouths,
            // while the smaller mask noise roughens their shoreline.
            let nx = nx + (noise(mix(detail, 1), x / 35.0, y / 35.0) - 0.5) * 0.18;
            let ny = ny + (noise(mix(detail, 2), x / 30.0, y / 30.0) - 0.5) * 0.18;
            let nx = if flip { -nx } else { nx };
            let warp = (fbm(detail, x / 18.0, y / 18.0) - 0.5) * 0.38;
            let mask = match shape {
                Shape::Rounded => 1.0 - nx * nx - ny * ny,
                // Equal physical width and height, with gently irregular corners.
                Shape::Square => 1.0 - (nx.abs() * 1.45).max(ny.abs()),
                Shape::Long => 1.0 - nx * nx * 0.55 - (ny + nx * 0.18).powi(2) * 2.1,
                Shape::Bay => {
                    let outer = 1.0 - nx * nx - ny * ny;
                    let basin = (nx * nx / 0.25 + ny * ny / 0.32) - 1.0;
                    // An open inlet joins the central basin to the sea.
                    let inlet = (ny.abs() - 0.18).max(-nx);
                    outer.min(basin).min(inlet)
                }
                Shape::Lobed => {
                    let west = 1.0 - (nx + 0.38).powi(2) / 0.65 - (ny + 0.18).powi(2) / 0.65;
                    let east = 1.0 - (nx - 0.43).powi(2) / 0.5 - (ny - 0.26).powi(2) / 0.45;
                    west.max(east)
                }
            };
            // Protect the basin and its mouth from coastline noise and quantiles.
            if shape == Shape::Bay
                && (nx * nx / 0.25 + ny * ny / 0.32 < 1.0 || (nx > 0.0 && ny.abs() < 0.18))
            {
                -10.0
            } else {
                mask + warp
            }
        })
        .collect()
}
