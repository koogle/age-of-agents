//! Seeded island generation. One seed always yields the same island: a single
//! large landmass ringed by sea, rolling hills cut by winding valleys, several
//! ranges of impassable peaks with highlands around them, rivers running from
//! there to the sea (crossed at sandbar fords), forests and wetland where it
//! is moist, beaches on the coast, and starter resources where they belong.
//! Only wood and berries grow near the start; stone requires exploration.
//! Each island supplies the raw settlement and proposed transport budget,
//! including wood to process into timber, reachable from the town center.
//! Destinations add complementary advanced materials. Failed seeds are re-rolled.
//!
//! Only integer hashing, `+ - * /` and `sqrt` (all exactly rounded under IEEE
//! 754) are used, so the result is bit-identical
//! on every platform, native and wasm.

use std::collections::VecDeque;

use super::*;

mod drainage;
mod shape;
/// How many mountain ranges an island has, at least and at most.
const RANGES: (usize, usize) = (3, 4);
const MAX_ATTEMPTS: u64 = 64;
/// Land above this height rank is bare mountain; above the next, highland.
const MOUNTAIN_RANK: f64 = 0.93;
const HIGHLAND_RANK: f64 = 0.8;
const RIVERS: usize = 4;
const MIN_RIVER_LENGTH: usize = 14;
const FORD_SPACING: usize = 9;

pub(super) struct Island {
    pub seed: u64,
    pub terrain: Vec<TerrainCell>,
    pub resources: Vec<ResourceNode>,
    pub town_center: CellCoordinate,
    pub villagers: [CellCoordinate; 2],
}

/// The first acceptable island for `seed`, re-rolling deterministically.
pub(super) fn generate(seed: u64) -> Island {
    generate_with_resources(seed, &STARTER_RESOURCES)
}

fn generate_with_resources(seed: u64, kinds: &[ResourceKind]) -> Island {
    (0..MAX_ATTEMPTS)
        .find_map(|attempt| attempt_island(seed, mix(seed, attempt), kinds))
        .expect("an acceptable island within the re-roll budget")
}

const COLUMNS: usize = WORLD_COLUMNS as usize;
const ROWS: usize = WORLD_ROWS as usize;

/// SplitMix64: a tiny, well-mixed, platform-independent hash.
pub(super) fn mix(a: u64, b: u64) -> u64 {
    let mut z = a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn unit(hash: u64) -> f64 {
    (hash >> 11) as f64 / (1u64 << 53) as f64
}

fn lattice(seed: u64, x: i64, y: i64) -> f64 {
    unit(mix(mix(seed, x as u64), y as u64))
}

/// Smooth value noise in [0, 1).
fn noise(seed: u64, x: f64, y: f64) -> f64 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (x - ix, y - iy);
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (ix, iy) = (ix as i64, iy as i64);
    let a = lattice(seed, ix, iy);
    let b = lattice(seed, ix + 1, iy);
    let c = lattice(seed, ix, iy + 1);
    let d = lattice(seed, ix + 1, iy + 1);
    a + (b - a) * ux + (c - a) * uy + (a - b - c + d) * ux * uy
}

fn fbm(seed: u64, x: f64, y: f64) -> f64 {
    let mut sum = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    for octave in 0..4 {
        sum += amplitude * noise(mix(seed, octave), x * frequency, y * frequency);
        amplitude *= 0.5;
        frequency *= 2.03;
    }
    sum / 0.9375
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(self.0, 0)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn range(&mut self, low: f64, high: f64) -> f64 {
        low + unit(self.next()) * (high - low)
    }
}

fn at(column: usize, row: usize) -> usize {
    row * COLUMNS + column
}

fn neighbours4(index: usize) -> impl Iterator<Item = usize> {
    let (column, row) = (index % COLUMNS, index / COLUMNS);
    [(0i64, -1i64), (-1, 0), (1, 0), (0, 1)]
        .into_iter()
        .filter_map(move |(dx, dy)| {
            let (c, r) = (column as i64 + dx, row as i64 + dy);
            (c >= 0 && r >= 0 && c < COLUMNS as i64 && r < ROWS as i64)
                .then(|| at(c as usize, r as usize))
        })
}

fn neighbours8(index: usize) -> impl Iterator<Item = usize> {
    let (column, row) = (index % COLUMNS, index / COLUMNS);
    (-1i64..=1)
        .flat_map(|dy| (-1i64..=1).map(move |dx| (dx, dy)))
        .filter(|&(dx, dy)| (dx, dy) != (0, 0))
        .filter_map(move |(dx, dy)| {
            let (c, r) = (column as i64 + dx, row as i64 + dy);
            (c >= 0 && r >= 0 && c < COLUMNS as i64 && r < ROWS as i64)
                .then(|| at(c as usize, r as usize))
        })
}

fn coordinate(index: usize) -> CellCoordinate {
    CellCoordinate::new((index % COLUMNS) as u16, (index / COLUMNS) as u16)
}

/// Breadth-first distance (in steps) from every cell to the nearest source.
fn distance_from(
    sources: impl Iterator<Item = usize>,
    passable: impl Fn(usize) -> bool,
) -> Vec<u32> {
    let mut distance = vec![u32::MAX; COLUMNS * ROWS];
    let mut queue = VecDeque::new();
    for source in sources {
        distance[source] = 0;
        queue.push_back(source);
    }
    while let Some(cell) = queue.pop_front() {
        for next in neighbours4(cell) {
            if distance[next] == u32::MAX && passable(next) {
                distance[next] = distance[cell] + 1;
                queue.push_back(next);
            }
        }
    }
    distance
}

fn attempt_island(seed: u64, roll: u64, kinds: &[ResourceKind]) -> Option<Island> {
    let mut rng = Rng(roll);
    let (width, height) = (COLUMNS as f64, ROWS as f64);
    // Several mountain ranges stand out from the middle, each a ridge broken
    // into peaks by ridged noise, so the start is ringed by high ground but
    // never sits on it. Rolling hills and winding valleys cover the rest; the
    // coastline is generated independently of these heights.
    let ranges = RANGES.0 + rng.below(RANGES.1 - RANGES.0 + 1);
    let mut ridges: Vec<((f64, f64), (f64, f64))> = Vec::new();
    for _ in 0..ranges * 40 {
        if ridges.len() == ranges {
            break;
        }
        let (rx, ry) = (rng.range(-0.75, 0.75), rng.range(-0.75, 0.75));
        let reach = (rx * rx + ry * ry).sqrt();
        let peak = (
            width / 2.0 + rx * width * 0.44,
            height / 2.0 + ry * height * 0.42,
        );
        let apart = ridges.iter().all(|(a, b)| {
            let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
            ((mx - peak.0) * (mx - peak.0) + (my - peak.1) * (my - peak.1)).sqrt() >= width * 0.28
        });
        if !(0.35..=0.75).contains(&reach) || !apart {
            continue;
        }
        let (dx, dy) = (rng.range(-1.0, 1.0), rng.range(-1.0, 1.0));
        let norm = (dx * dx + dy * dy).sqrt().max(1e-6);
        let half = width * rng.range(0.06, 0.12);
        ridges.push((
            (peak.0 - dx / norm * half, peak.1 - dy / norm * half),
            (peak.0 + dx / norm * half, peak.1 + dy / norm * half),
        ));
    }
    if ridges.len() < RANGES.0 {
        return None;
    }
    let (shape_seed, detail_seed, hill_seed, crag_seed, moisture_seed, clay_seed) = (
        rng.next(),
        rng.next(),
        rng.next(),
        rng.next(),
        rng.next(),
        rng.next(),
    );
    let valley_seed = rng.next();
    let outline = shape::outline(seed, shape_seed);
    let raw: Vec<f64> = (0..COLUMNS * ROWS)
        .map(|index| {
            let (x, y) = (
                (index % COLUMNS) as f64 + 0.5,
                (index / COLUMNS) as f64 + 0.5,
            );
            let crags = 1.0 - (2.0 * noise(crag_seed, x / 5.0, y / 5.0) - 1.0).abs();
            let mountain = ridges
                .iter()
                .map(|&ridge| {
                    let d = distance_to_segment((x, y), ridge) / 4.0;
                    0.8 / (1.0 + d * d)
                })
                .fold(0.0, f64::max)
                * (0.45 + 0.9 * crags);
            let hills = (fbm(hill_seed, x / 13.0, y / 13.0) - 0.5) * 0.9;
            // Valleys follow the creases of ridged noise; rivers find them.
            let crease = 1.0 - (2.0 * fbm(valley_seed, x / 22.0, y / 22.0) - 1.0).abs();
            let valley = crease * crease * crease * crease * 0.3;
            0.7 + mountain + hills - valley + (fbm(detail_seed, x / 6.0, y / 6.0) - 0.5) * 0.2
        })
        .collect();

    // Choose a coastline quantile for this shape family, then keep only
    // the largest landmass so there is exactly one island.
    let mut sorted = outline.clone();
    sorted.sort_by(f64::total_cmp);
    let sea = sorted[((1.0 - shape::land_share(seed)) * sorted.len() as f64) as usize];
    let mut land: Vec<bool> = outline.iter().map(|&e| e > sea).collect();
    let mut component = vec![usize::MAX; land.len()];
    let mut sizes = Vec::new();
    for start in 0..land.len() {
        if !land[start] || component[start] != usize::MAX {
            continue;
        }
        let id = sizes.len();
        let mut queue = VecDeque::from([start]);
        component[start] = id;
        let mut size = 0;
        while let Some(cell) = queue.pop_front() {
            size += 1;
            for next in neighbours4(cell) {
                if land[next] && component[next] == usize::MAX {
                    component[next] = id;
                    queue.push_back(next);
                }
            }
        }
        sizes.push(size);
    }
    let main = (0..sizes.len()).max_by_key(|&id| (sizes[id], usize::MAX - id))?;
    // Land must not touch the map edge: the island is surrounded by sea.
    for (index, cell) in land.iter_mut().enumerate() {
        let (column, row) = (index % COLUMNS, index / COLUMNS);
        let edge = column == 0 || row == 0 || column == COLUMNS - 1 || row == ROWS - 1;
        *cell = *cell && component[index] == main && !edge;
    }

    // Coast first, then relief: ridges cannot accidentally fill a bay or create
    // offshore peaks. Coastal slopes taper continuously toward sea level.
    let shore = distance_from((0..land.len()).filter(|&i| !land[i]), |i| land[i]);
    let offshore = distance_from((0..land.len()).filter(|&i| land[i]), |i| !land[i]);
    let mut elevation: Vec<f32> = (0..land.len())
        .map(|i| {
            if land[i] {
                (raw[i].max(0.03) * (f64::from(shore[i]) / 9.0).min(1.0)) as f32
            } else {
                -(offshore[i] as f32 / 12.0).clamp(0.05, 1.0)
            }
        })
        .collect();
    let drainage = drainage::drain(&land, &mut elevation);
    let top = elevation.iter().copied().fold(0.0, f32::max);
    for (i, height) in elevation.iter_mut().enumerate() {
        if land[i] {
            *height /= top;
        }
    }
    // Rank of every land cell by height, so biome bands hold a steady share
    // of the island however steep its relief.
    let mut by_height: Vec<usize> = (0..land.len()).filter(|&i| land[i]).collect();
    by_height.sort_by(|&a, &b| elevation[a].total_cmp(&elevation[b]).then(a.cmp(&b)));
    let mut rank = vec![0.0; land.len()];
    for (place, &index) in by_height.iter().enumerate() {
        rank[index] = place as f64 / by_height.len() as f64;
    }
    let mountain = |i: usize| land[i] && rank[i] >= MOUNTAIN_RANK;

    let (river, ford) = drainage.rivers(&land, &rank);
    if river.iter().filter(|&&r| r).count() < MIN_RIVER_LENGTH {
        return None;
    }

    // Distance to the open sea (water reachable from the map edge) versus
    // inland lakes and rivers, and a moisture field that is wetter near water.
    let open_sea = distance_from((0..land.len()).filter(|&i| i < COLUMNS), |i| !land[i]);
    let coast = distance_from(
        (0..land.len()).filter(|&i| !land[i] && open_sea[i] != u32::MAX),
        |i| land[i],
    );
    let water_any = distance_from((0..land.len()).filter(|&i| !land[i] || river[i]), |i| {
        land[i]
    });
    let terrain: Vec<TerrainCell> = (0..land.len())
        .map(|index| {
            let (column, row) = (index % COLUMNS, index / COLUMNS);
            let r = rank[index];
            let wet = (fbm(moisture_seed, column as f64 / 8.0, row as f64 / 8.0) * 0.7
                + 0.3 / (1.0 + f64::from(water_any[index].min(40)) / 3.0))
                .clamp(0.0, 1.0);
            let biome = if !land[index] {
                TerrainBiome::Water
            } else if ford[index] {
                // Sandbars where the river runs shallow enough to wade.
                TerrainBiome::Beach
            } else if river[index] {
                TerrainBiome::River
            } else if mountain(index) {
                TerrainBiome::Mountain
            } else if coast[index] <= 1 && r < 0.45 {
                TerrainBiome::Beach
            } else if r >= HIGHLAND_RANK {
                TerrainBiome::Highland
            } else if water_any[index] <= 2 && wet > 0.5 {
                TerrainBiome::Wetland
            } else if r > 0.55 {
                if wet > 0.52 {
                    TerrainBiome::Heath
                } else {
                    TerrainBiome::Scrubland
                }
            } else if r < 0.35 && fbm(clay_seed, column as f64 / 5.0, row as f64 / 5.0) > 0.62 {
                // Clay beds lie in a few low patches.
                TerrainBiome::Clayland
            } else if wet > 0.55 {
                TerrainBiome::Forest
            } else if wet > 0.42 {
                TerrainBiome::Meadow
            } else {
                TerrainBiome::Prairie
            };
            TerrainCell {
                column: column as u16,
                row: row as u16,
                biome,
                elevation: elevation[index],
            }
        })
        .collect();

    let (town_center, villagers) = choose_start(&terrain, &coast)?;
    let resources = place_resources(&terrain, &ford, town_center, &mut rng, kinds);
    let resources = reachable_only(&terrain, resources, town_center, villagers[0]);
    let enough = STARTER_RESOURCE_BUDGET.iter().all(|&(kind, cost)| {
        let total: f64 = resources
            .iter()
            .filter(|r| r.kind == kind)
            .map(|r| r.amount)
            .sum();
        total >= cost * 1.5
    });
    let enough = enough
        && kinds.iter().all(|kind| {
            resources
                .iter()
                .filter(|r| r.kind == *kind)
                .map(|r| r.amount)
                .sum::<f64>()
                >= 120.0
        });
    let near_start = |kind: ResourceKind| {
        let (columns, rows) = BuildingKind::TownCenter.size();
        let base = Footprint {
            origin: town_center,
            columns,
            rows,
        }
        .center();
        resources.iter().any(|r| {
            r.kind == kind
                && r.cell.center().distance(base)
                    <= NEAR_START.min(BuildingKind::TownCenter.sight_radius())
        })
    };
    (enough && near_start(ResourceKind::Wood) && near_start(ResourceKind::Food)).then_some(Island {
        seed,
        terrain,
        resources,
        town_center,
        villagers,
    })
}

fn distance_to_segment(p: (f64, f64), (a, b): ((f64, f64), (f64, f64))) -> f64 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let t = (((p.0 - a.0) * abx + (p.1 - a.1) * aby) / (abx * abx + aby * aby).max(1e-9))
        .clamp(0.0, 1.0);
    let (x, y) = (p.0 - a.0 - abx * t, p.1 - a.1 - aby * t);
    (x * x + y * y).sqrt()
}

/// A flat, dry, central site for the town center, with two villager spots
/// in front of it.
fn choose_start(
    terrain: &[TerrainCell],
    coast: &[u32],
) -> Option<(CellCoordinate, [CellCoordinate; 2])> {
    let settles = |c: usize, r: usize| {
        let cell = &terrain[at(c, r)];
        matches!(
            cell.biome,
            TerrainBiome::Meadow
                | TerrainBiome::Prairie
                | TerrainBiome::Scrubland
                | TerrainBiome::Heath
        ) && coast[at(c, r)] >= 3
    };
    let center = (COLUMNS as f64 / 2.0, ROWS as f64 / 2.0);
    let size = usize::from(BuildingKind::TownCenter.size().0);
    let mut best: Option<(f64, usize, usize)> = None;
    for row in 1..ROWS - (size + 3) {
        for column in 1..COLUMNS - (size + 1) {
            // Footprint plus a one-cell apron and the villager row in front.
            let site = (column - 1..column + size + 1)
                .all(|c| (row - 1..row + size + 3).all(|r| settles(c, r)));
            if !site {
                continue;
            }
            let half = size as f64 / 2.0;
            let (x, y) = (column as f64 + half, row as f64 + half);
            let heights = (column..column + size).flat_map(|c| {
                (row..row + size).map(move |r| f64::from(terrain[at(c, r)].elevation))
            });
            let (lo, hi) = heights.fold((f64::MAX, f64::MIN), |(lo, hi), h| (lo.min(h), hi.max(h)));
            let score = (x - center.0).hypot((y - center.1) * 1.4) + (hi - lo) * 20.0;
            if best.is_none_or(|(s, _, _)| score < s) {
                best = Some((score, column, row));
            }
        }
    }
    let (_, column, row) = best?;
    let origin = CellCoordinate::new(column as u16, row as u16);
    let villagers = [
        CellCoordinate::new((column + 1) as u16, (row + size) as u16),
        CellCoordinate::new((column + size - 2) as u16, (row + size) as u16),
    ];
    Some((origin, villagers))
}

/// Only wood and food grow around the start; everything else lies at least
/// this many cells out, so the player has to explore to find it.
const FAR_FROM_START: f64 = 24.0;
/// The starting woodline and berries lie within this many cells.
const NEAR_START: f64 = 16.0;

/// Kind, id prefix, host biomes, clusters near the start, clusters further
/// out, cluster size, amount per node.
type Plan = (
    ResourceKind,
    &'static str,
    &'static [TerrainBiome],
    usize,
    usize,
    usize,
    f64,
);

const RESOURCE_PLAN: [Plan; 8] = [
    (
        ResourceKind::Wood,
        "tree",
        &[TerrainBiome::Forest, TerrainBiome::Heath],
        2,
        14,
        9,
        30.0,
    ),
    (
        ResourceKind::Food,
        "berries",
        &[
            TerrainBiome::Meadow,
            TerrainBiome::Forest,
            TerrainBiome::Heath,
        ],
        1,
        8,
        5,
        30.0,
    ),
    (
        ResourceKind::Stone,
        "stone",
        &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        0,
        6,
        4,
        40.0,
    ),
    (
        ResourceKind::Gold,
        "gold",
        &[TerrainBiome::Highland],
        0,
        3,
        3,
        40.0,
    ),
    (
        ResourceKind::Iron,
        "iron",
        &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        0,
        4,
        3,
        40.0,
    ),
    (
        ResourceKind::Coal,
        "coal",
        &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        0,
        4,
        3,
        40.0,
    ),
    (
        ResourceKind::Clay,
        "clay",
        &[
            TerrainBiome::Clayland,
            TerrainBiome::Wetland,
            TerrainBiome::Beach,
        ],
        0,
        4,
        3,
        40.0,
    ),
    (
        ResourceKind::Fiber,
        "fiber",
        &[
            TerrainBiome::Wetland,
            TerrainBiome::Prairie,
            TerrainBiome::Meadow,
        ],
        0,
        6,
        3,
        30.0,
    ),
];

/// The biomes `kind` grows in.
#[cfg(test)]
pub(super) fn grows_in(kind: ResourceKind) -> &'static [TerrainBiome] {
    RESOURCE_PLAN
        .iter()
        .find(|plan| plan.0 == kind)
        .map(|plan| plan.2)
        .unwrap_or(&[])
}

fn place_resources(
    terrain: &[TerrainCell],
    ford: &[bool],
    town_center: CellCoordinate,
    rng: &mut Rng,
    kinds: &[ResourceKind],
) -> Vec<ResourceNode> {
    let (columns, rows) = BuildingKind::TownCenter.size();
    let base = Footprint {
        origin: town_center,
        columns,
        rows,
    }
    .center();
    // Fords stay open so both banks keep their crossing.
    let mut taken = ford.to_vec();
    let mut resources: Vec<ResourceNode> = Vec::new();
    for (kind, prefix, biomes, near, far, size, amount) in RESOURCE_PLAN {
        if !kinds.contains(&kind) {
            continue;
        }
        let mut number = 0;
        for cluster in 0..near + far {
            // Seeds suit the kind, keep the base clear and stay apart from
            // other clusters; ore prefers the highest ground. The first few
            // ring the start; the rest wait out in the unexplored island.
            let (closest, furthest) = if cluster < near {
                (
                    STARTING_BASE_RESOURCE_CLEARANCE + 2.0,
                    BuildingKind::TownCenter.sight_radius(),
                )
            } else if near > 0 {
                (STARTING_BASE_RESOURCE_CLEARANCE + 2.0, f64::MAX)
            } else {
                (FAR_FROM_START, f64::MAX)
            };
            let candidates: Vec<usize> = (0..terrain.len())
                .filter(|&i| {
                    let center = coordinate(i).center();
                    biomes.contains(&terrain[i].biome)
                        && (closest..=furthest).contains(&center.distance(base))
                        && resources.iter().all(|r| {
                            r.cell.center().distance(center) >= RESOURCE_CLUSTER_SEPARATION
                        })
                })
                .collect();
            if candidates.is_empty() {
                continue;
            }
            let site = if matches!(kind, ResourceKind::Gold | ResourceKind::Iron) {
                *candidates
                    .iter()
                    .max_by(|&&a, &&b| {
                        terrain[a]
                            .elevation
                            .total_cmp(&terrain[b].elevation)
                            .then(b.cmp(&a))
                    })
                    .expect("candidates is not empty")
            } else {
                candidates[rng.below(candidates.len())]
            };
            // Grow the cluster outward through suitable cells.
            let mut queue = VecDeque::from([site]);
            let mut seen = vec![site];
            let mut grown = 0;
            while let Some(cell) = queue.pop_front() {
                if grown == size {
                    break;
                }
                if !taken[cell] && biomes.contains(&terrain[cell].biome) {
                    taken[cell] = true;
                    grown += 1;
                    number += 1;
                    resources.push(ResourceNode {
                        field: None,
                        id: format!("{prefix}-{number}"),
                        kind,
                        cell: coordinate(cell),
                        amount,
                        capacity: amount,
                    });
                }
                let mut next: Vec<usize> =
                    neighbours8(cell).filter(|n| !seen.contains(n)).collect();
                // Shuffle so clusters grow into organic shapes, not squares.
                for i in (1..next.len()).rev() {
                    next.swap(i, rng.below(i + 1));
                }
                for n in next {
                    if biomes.contains(&terrain[n].biome)
                        && coordinate(n).center().distance(base) >= closest - 2.0
                    {
                        seen.push(n);
                        queue.push_back(n);
                    }
                }
            }
        }
    }
    resources
}

/// Drops nodes nobody could stand beside, walking from the villagers'
/// spot around water, peaks, rivers, the town center and other nodes.
fn reachable_only(
    terrain: &[TerrainCell],
    resources: Vec<ResourceNode>,
    town_center: CellCoordinate,
    start: CellCoordinate,
) -> Vec<ResourceNode> {
    let (columns, rows) = BuildingKind::TownCenter.size();
    let footprint = Footprint {
        origin: town_center,
        columns,
        rows,
    };
    let mut blocked: Vec<bool> = terrain.iter().map(|c| !c.biome.is_walkable()).collect();
    for cell in footprint.cells() {
        blocked[at(usize::from(cell.column), usize::from(cell.row))] = true;
    }
    for resource in &resources {
        blocked[at(
            usize::from(resource.cell.column),
            usize::from(resource.cell.row),
        )] = true;
    }
    let start = at(usize::from(start.column), usize::from(start.row));
    let walk = distance_from(std::iter::once(start), |i| !blocked[i]);
    resources
        .into_iter()
        .filter(|r| {
            neighbours4(at(usize::from(r.cell.column), usize::from(r.cell.row)))
                .any(|n| walk[n] != u32::MAX)
        })
        .collect()
}

#[cfg(test)]
mod tests;

/// Each destination supplies one complementary chain in its native biomes.
pub(super) fn destination(seed: u64, id: u64) -> Island {
    let special: &[ResourceKind] = match (id - 1) % 3 {
        0 => &[ResourceKind::Iron, ResourceKind::Coal],
        1 => &[ResourceKind::Clay],
        _ => &[ResourceKind::Fiber],
    };
    let mut kinds = STARTER_RESOURCES.to_vec();
    kinds.extend_from_slice(special);
    generate_with_resources(seed, &kinds)
}
