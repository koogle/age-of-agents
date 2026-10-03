//! Seeded island generation. One seed always yields the same island: a single
//! landmass ringed by sea, rolling hills, a ridge of impassable peaks with
//! highlands around it, rivers running from there to the sea (crossed at
//! sandbar fords), forests and
//! wetland where it is moist, beaches on the coast, and resources where they
//! belong (stone and ore in the hills, clay by the water, berries at forest
//! edges). Every island is checked to hold enough of every resource to reach
//! a fishing boat, all of it reachable on foot from the starting town center;
//! a seed that fails is deterministically re-rolled.
//!
//! Only integer hashing, `+ - * /` and `sqrt` (all exactly rounded under IEEE
//! 754) are used, so the result is bit-identical
//! on every platform, native and wasm.

use std::collections::VecDeque;

use super::*;

/// What leaving the island will cost. The generator guarantees the island
/// holds at least this much of each kind (plus the founding economy's needs).
pub const FISHING_BOAT_COST: [(ResourceKind, f64); 6] = [
    (ResourceKind::Wood, 300.0),
    (ResourceKind::Food, 150.0),
    (ResourceKind::Stone, 80.0),
    (ResourceKind::Iron, 60.0),
    (ResourceKind::Fiber, 60.0),
    (ResourceKind::Clay, 40.0),
];

/// Share of the map that is land.
const LAND_SHARE: f64 = 0.55;
const MAX_ATTEMPTS: u64 = 64;
/// Land above this height rank is bare mountain; above the next, highland.
const MOUNTAIN_RANK: f64 = 0.93;
const HIGHLAND_RANK: f64 = 0.8;
const RIVERS: usize = 2;
const MIN_RIVER_LENGTH: usize = 12;
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
    (0..MAX_ATTEMPTS)
        .find_map(|attempt| attempt_island(seed, mix(seed, attempt)))
        .expect("an acceptable island within the re-roll budget")
}

const COLUMNS: usize = WORLD_COLUMNS as usize;
const ROWS: usize = WORLD_ROWS as usize;

/// SplitMix64: a tiny, well-mixed, platform-independent hash.
fn mix(a: u64, b: u64) -> u64 {
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

fn attempt_island(seed: u64, roll: u64) -> Option<Island> {
    let mut rng = Rng(roll);
    let (width, height) = (COLUMNS as f64, ROWS as f64);
    // A mountain ridge runs through a point inland, broken into peaks by
    // ridged noise; rolling hills cover the rest; the coast is a warped ellipse.
    let peak = (width * rng.range(0.3, 0.7), height * rng.range(0.3, 0.7));
    let (dx, dy) = (rng.range(-1.0, 1.0), rng.range(-1.0, 1.0));
    let norm = (dx * dx + dy * dy).sqrt().max(1e-6);
    let half = width * rng.range(0.1, 0.2);
    let ridge = (
        (peak.0 - dx / norm * half, peak.1 - dy / norm * half),
        (peak.0 + dx / norm * half, peak.1 + dy / norm * half),
    );
    let (shape_seed, detail_seed, hill_seed, crag_seed, moisture_seed, clay_seed, meander_seed) = (
        rng.next(),
        rng.next(),
        rng.next(),
        rng.next(),
        rng.next(),
        rng.next(),
        rng.next(),
    );
    let raw: Vec<f64> = (0..COLUMNS * ROWS)
        .map(|index| {
            let (x, y) = (
                (index % COLUMNS) as f64 + 0.5,
                (index / COLUMNS) as f64 + 0.5,
            );
            let (nx, ny) = (
                (x - width / 2.0) / (width * 0.44),
                (y - height / 2.0) / (height * 0.42),
            );
            let warp = (fbm(shape_seed, x / 9.0, y / 9.0) - 0.5) * 0.55;
            let falloff = 1.0 - (nx * nx + ny * ny) - warp;
            let d = distance_to_segment((x, y), ridge) / 3.5;
            let crags = 1.0 - (2.0 * noise(crag_seed, x / 5.0, y / 5.0) - 1.0).abs();
            let mountain = 0.8 / (1.0 + d * d) * (0.45 + 0.9 * crags);
            let hills = (fbm(hill_seed, x / 11.0, y / 11.0) - 0.5) * 0.6;
            falloff * 0.9 + mountain + hills + (fbm(detail_seed, x / 6.0, y / 6.0) - 0.5) * 0.2
        })
        .collect();

    // Sea level at the quantile that leaves LAND_SHARE as land, then keep only
    // the largest landmass so there is exactly one island.
    let mut sorted = raw.clone();
    sorted.sort_by(f64::total_cmp);
    let sea = sorted[((1.0 - LAND_SHARE) * sorted.len() as f64) as usize];
    let top = sorted[sorted.len() - 1];
    let mut land: Vec<bool> = raw.iter().map(|&e| e > sea).collect();
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

    let mut elevation: Vec<f32> = raw
        .iter()
        .zip(&land)
        .map(|(&e, &is_land)| {
            if is_land {
                (((e - sea) / (top - sea)).clamp(0.02, 1.0)) as f32
            } else {
                (-((sea - e) / (sea - sorted[0])).clamp(0.05, 1.0)) as f32
            }
        })
        .collect();
    // Rank of every land cell by height, so biome bands hold a steady share
    // of the island however steep its relief.
    let mut by_height: Vec<usize> = (0..land.len()).filter(|&i| land[i]).collect();
    by_height.sort_by(|&a, &b| raw[a].total_cmp(&raw[b]).then(a.cmp(&b)));
    let mut rank = vec![0.0; land.len()];
    for (place, &index) in by_height.iter().enumerate() {
        rank[index] = place as f64 / by_height.len() as f64;
    }
    let mountain = |i: usize| land[i] && rank[i] >= MOUNTAIN_RANK;

    let (river, ford) = carve_rivers(&raw, &land, &rank, &mut elevation, meander_seed, &mut rng);

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
    let resources = place_resources(&terrain, &ford, town_center, &mut rng);
    let resources = reachable_only(&terrain, resources, town_center, villagers[0]);
    let enough = FISHING_BOAT_COST.iter().all(|&(kind, cost)| {
        let total: f64 = resources
            .iter()
            .filter(|r| r.kind == kind)
            .map(|r| r.amount)
            .sum();
        total >= cost * 1.5
    });
    let near_start = |kind: ResourceKind| {
        let (columns, rows) = BuildingKind::TownCenter.size();
        let base = Footprint {
            origin: town_center,
            columns,
            rows,
        }
        .center();
        resources
            .iter()
            .any(|r| r.kind == kind && r.cell.center().distance(base) <= 16.0)
    };
    (enough
        && resources
            .iter()
            .any(|r| r.kind == ResourceKind::Coal && r.amount >= 30.0)
        && near_start(ResourceKind::Wood)
        && near_start(ResourceKind::Food))
    .then_some(Island {
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

/// Rivers rise in the highlands below the peaks and run downhill to the sea
/// (or into an earlier river). Returns the river cells and the fords, shallow
/// sandbars spaced along each river so no bank is ever cut off. River beds
/// only ever descend toward the mouth.
fn carve_rivers(
    raw: &[f64],
    land: &[bool],
    rank: &[f64],
    elevation: &mut [f32],
    meander_seed: u64,
    rng: &mut Rng,
) -> (Vec<bool>, Vec<bool>) {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    // Priority flood from the water inward: every land cell learns the next
    // cell on its lowest route to the water. A little noise makes rivers meander.
    let level = |i: usize| {
        let (x, y) = ((i % COLUMNS) as f64, (i / COLUMNS) as f64);
        raw[i] + (fbm(meander_seed, x / 4.0, y / 4.0) - 0.5) * 0.3
    };
    let key = |value: f64| (value * 1e9) as i64;
    let mut downstream = vec![usize::MAX; land.len()];
    let mut done: Vec<bool> = land.iter().map(|&l| !l).collect();
    let mut heap = BinaryHeap::new();
    for i in (0..land.len()).filter(|&i| !land[i]) {
        if neighbours4(i).any(|n| land[n]) {
            heap.push(Reverse((i64::MIN, i)));
        }
    }
    while let Some(Reverse((current, cell))) = heap.pop() {
        for next in neighbours4(cell) {
            if !done[next] {
                done[next] = true;
                downstream[next] = cell;
                heap.push(Reverse((current.max(key(level(next))), next)));
            }
        }
    }

    let mut river = vec![false; land.len()];
    let mut ford = vec![false; land.len()];
    let mut sources: Vec<usize> = (0..land.len())
        .filter(|&i| (HIGHLAND_RANK..MOUNTAIN_RANK).contains(&rank[i]))
        .collect();
    let mut rivers = 0;
    while rivers < RIVERS && !sources.is_empty() {
        let source = sources.swap_remove(rng.below(sources.len()));
        let mut path = vec![source];
        while let Some(&last) = path.last() {
            let next = downstream[last];
            if next == usize::MAX || !land[next] || river[next] {
                break;
            }
            path.push(next);
        }
        let near_other = river
            .iter()
            .enumerate()
            .any(|(i, &r)| r && near(i, source, 10));
        if path.len() < MIN_RIVER_LENGTH || near_other {
            continue;
        }
        let mut bed = f32::MAX;
        let mut since_ford = FORD_SPACING / 2;
        for (step, &cell) in path.iter().enumerate() {
            river[cell] = true;
            bed = bed.min(elevation[cell]);
            elevation[cell] = bed.max(0.02);
            since_ford += 1;
            // A ford every few cells on a straight reach (a ford on a bend
            // would touch only one bank), never at the source or the mouth.
            let straight = step >= 3
                && step + 3 < path.len()
                && cell + cell == path[step - 1] + path[step + 1];
            if straight && since_ford >= FORD_SPACING {
                ford[cell] = true;
                since_ford = 0;
            }
        }
        sources.retain(|&s| !near(s, source, 10));
        rivers += 1;
    }
    (river, ford)
}

fn near(a: usize, b: usize, cells: usize) -> bool {
    (a % COLUMNS).abs_diff(b % COLUMNS) <= cells && (a / COLUMNS).abs_diff(b / COLUMNS) <= cells
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

/// (kind, id prefix, biomes it grows in, clusters, nodes per cluster, amount per node).
/// Kind, id prefix, host biomes, cluster count, cluster size, amount per node.
type Plan = (
    ResourceKind,
    &'static str,
    &'static [TerrainBiome],
    usize,
    usize,
    f64,
);

const RESOURCE_PLAN: [Plan; 8] = [
    (
        ResourceKind::Wood,
        "tree",
        &[TerrainBiome::Forest, TerrainBiome::Heath],
        6,
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
        4,
        5,
        30.0,
    ),
    (
        ResourceKind::Stone,
        "stone",
        &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        3,
        4,
        40.0,
    ),
    (
        ResourceKind::Gold,
        "gold",
        &[TerrainBiome::Highland],
        1,
        3,
        40.0,
    ),
    (
        ResourceKind::Iron,
        "iron",
        &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        2,
        3,
        40.0,
    ),
    (
        ResourceKind::Coal,
        "coal",
        &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        2,
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
        2,
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
        3,
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
    for (kind, prefix, biomes, clusters, size, amount) in RESOURCE_PLAN {
        let mut number = 0;
        for _ in 0..clusters {
            // Seeds suit the kind, keep the base clear and stay apart from
            // other clusters; ore prefers the highest ground.
            let candidates: Vec<usize> = (0..terrain.len())
                .filter(|&i| {
                    let center = coordinate(i).center();
                    biomes.contains(&terrain[i].biome)
                        && center.distance(base) >= STARTING_BASE_RESOURCE_CLEARANCE + 2.0
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
                        && coordinate(n).center().distance(base) >= STARTING_BASE_RESOURCE_CLEARANCE
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
mod tests {
    use super::*;

    fn index(cell: CellCoordinate) -> usize {
        at(usize::from(cell.column), usize::from(cell.row))
    }

    #[test]
    fn one_seed_always_grows_the_same_island_and_seeds_differ() {
        assert_eq!(GameWorld::generate(7), GameWorld::generate(7));
        assert_ne!(
            GameWorld::generate(7).terrain,
            GameWorld::generate(8).terrain
        );
        assert_eq!(GameWorld::default().seed, DEFAULT_SEED);
    }

    #[test]
    fn every_island_is_one_landmass_ringed_by_sea_and_beach() {
        for seed in 0..24 {
            let world = GameWorld::generate(seed);
            world.validate().unwrap();
            let land: Vec<bool> = world
                .terrain
                .iter()
                .map(|c| c.biome != TerrainBiome::Water)
                .collect();
            for (i, cell) in world.terrain.iter().enumerate() {
                let edge = cell.column == 0
                    || cell.row == 0
                    || cell.column == WORLD_COLUMNS - 1
                    || cell.row == WORLD_ROWS - 1;
                assert!(!(edge && land[i]), "seed {seed}: land touches the map edge");
                assert_eq!(
                    land[i],
                    cell.elevation > 0.0,
                    "seed {seed}: elevation disagrees with water"
                );
            }
            let first = land.iter().position(|&l| l).unwrap();
            let reached = distance_from(std::iter::once(first), |i| land[i]);
            assert!(
                land.iter()
                    .zip(&reached)
                    .all(|(&l, &d)| !l || d != u32::MAX),
                "seed {seed}: more than one island"
            );
            let share = land.iter().filter(|&&l| l).count() as f64 / land.len() as f64;
            assert!(
                (0.35..0.6).contains(&share),
                "seed {seed}: land share {share}"
            );
            assert!(world.terrain.iter().any(|c| c.biome == TerrainBiome::Beach));
            assert!(
                world
                    .terrain
                    .iter()
                    .any(|c| c.biome == TerrainBiome::Highland)
            );
        }
    }

    #[test]
    fn every_island_holds_enough_reachable_resources_to_reach_a_boat() {
        for seed in 0..24 {
            let world = GameWorld::generate(seed);
            assert!(
                world
                    .resources
                    .iter()
                    .filter(|r| r.kind == ResourceKind::Coal)
                    .map(|r| r.amount)
                    .sum::<f64>()
                    >= 30.0,
                "seed {seed}: missing smelter fuel"
            );
            for &(kind, cost) in &FISHING_BOAT_COST {
                let total: f64 = world
                    .resources
                    .iter()
                    .filter(|r| r.kind == kind)
                    .map(|r| r.amount)
                    .sum();
                assert!(total >= cost * 1.5, "seed {seed}: only {total} {kind:?}");
            }
            for (number, resource) in world.resources.iter().enumerate() {
                assert!(
                    grows_in(resource.kind).contains(&world.terrain[index(resource.cell)].biome)
                );
                // Villager 1 can walk to every node (the real pathfinder, not the generator's check).
                assert!(
                    world.can_reach_beside(0, resource.footprint()),
                    "seed {seed}: {} unreachable",
                    resource.id
                );
                assert!(
                    world.resources[number + 1..]
                        .iter()
                        .all(|other| other.id != resource.id)
                );
            }
            let base = world.buildings[0].footprint();
            assert!(world.units.iter().all(|u| base.is_interaction_cell(u.cell)));
            for kind in [ResourceKind::Wood, ResourceKind::Food] {
                assert!(
                    world
                        .resources
                        .iter()
                        .any(|r| r.kind == kind && r.cell.center().distance(base.center()) <= 16.0)
                );
            }
        }
    }

    #[test]
    fn every_island_has_peaks_and_rivers_that_reach_the_water() {
        for seed in 0..24 {
            let world = GameWorld::generate(seed);
            let biome = |i: usize| world.terrain[i].biome;
            let count = |b: TerrainBiome| world.terrain.iter().filter(|c| c.biome == b).count();
            assert!(count(TerrainBiome::Mountain) > 0, "seed {seed}: no peaks");
            assert!(
                count(TerrainBiome::River) >= MIN_RIVER_LENGTH,
                "seed {seed}: no river"
            );
            // Following river cells (and the fords across them) leads to the water.
            let wet = |i: usize| matches!(biome(i), TerrainBiome::River | TerrainBiome::Beach);
            let mouth = distance_from(
                (0..world.terrain.len()).filter(|&i| biome(i) == TerrainBiome::Water),
                wet,
            );
            for (i, cell) in world.terrain.iter().enumerate() {
                if cell.biome == TerrainBiome::River {
                    assert!(mouth[i] != u32::MAX, "seed {seed}: river cut off at {i}");
                    assert!(cell.elevation > 0.0);
                }
            }
            // Every peak towers over the land around it on average.
            let mean = |b: TerrainBiome| {
                let cells = world.terrain.iter().filter(|c| c.biome == b);
                cells.clone().map(|c| f64::from(c.elevation)).sum::<f64>() / cells.count() as f64
            };
            assert!(mean(TerrainBiome::Mountain) > mean(TerrainBiome::Highland));
            assert!(mean(TerrainBiome::Highland) > mean(TerrainBiome::Meadow));
        }
    }

    #[test]
    fn nobody_walks_or_builds_on_peaks_or_rivers() {
        let world = GameWorld::generate(1);
        for biome in [TerrainBiome::Mountain, TerrainBiome::River] {
            let cell = world
                .terrain
                .iter()
                .find(|c| c.biome == biome)
                .unwrap()
                .coordinate();
            let mut moved = world.clone();
            assert!(
                moved
                    .apply_command(Command::Move {
                        unit_id: "villager-1".into(),
                        to: cell
                    })
                    .is_err()
            );
            let mut built = world.clone();
            built.stockpile.wood = 1000.0;
            built.stockpile.stone = 1000.0;
            let origin =
                CellCoordinate::new(cell.column.saturating_sub(1), cell.row.saturating_sub(1));
            assert!(
                built
                    .apply_command(Command::Build {
                        kind: BuildingKind::House,
                        unit_id: "villager-1".into(),
                        origin
                    })
                    .is_err()
            );
            let mut stranded = world.clone();
            stranded.units[0].cell = cell;
            assert!(stranded.validate().is_err());
        }
    }

    #[test]
    fn nobody_walks_or_builds_on_water() {
        let mut world = GameWorld::generate(3);
        let water = world
            .terrain
            .iter()
            .find(|c| c.biome == TerrainBiome::Water && c.column > 2)
            .unwrap()
            .coordinate();
        let before = world.clone();
        let moved = world.apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to: water,
        });
        assert!(moved.is_err());
        world.stockpile.wood = 100.0;
        let built = world.apply_command(Command::Build {
            kind: BuildingKind::TownCenter,
            unit_id: "villager-1".into(),
            origin: water,
        });
        assert!(built.is_err());
        world.stockpile.wood = before.stockpile.wood;
        assert_eq!(world, before);
        let mut drowned = before.clone();
        drowned.units[0].cell = water;
        assert!(drowned.validate().is_err());
    }
}
