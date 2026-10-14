//! Persistent islands in one coordinate space.
use super::*;
use crate::navigation::offset;

const OCEAN_GAP: u16 = 64;
/// Extra seeded offset per island site, so the archipelago is not a lattice.
const SITE_JITTER: u16 = 24;
/// A ship this close to an undiscovered site's region discovers it.
const DISCOVERY_REACH: u16 = 12;
const SITE_SALT: u64 = 0x6172_6368_6970_656c;

/// Planned rank 0 (home) is 120×80; each rank further adds this many columns
/// and rows, so the deepest islands have room for their monsters.
const GROWTH_PER_RANK: (u16, u16) = (8, 5);
/// The most sites a run plans; the grid pitch fits the largest of them.
const MAX_SITES: u16 = 8;

/// The region of the site at planned rank `rank`.
pub fn site_size(rank: usize) -> (u16, u16) {
    let rank = rank as u16;
    (
        WORLD_COLUMNS + GROWTH_PER_RANK.0 * rank,
        WORLD_ROWS + GROWTH_PER_RANK.1 * rank,
    )
}

pub(super) fn starting_islands() -> Vec<Footprint> {
    let (columns, rows) = site_size(0);
    vec![Footprint {
        origin: CellCoordinate::new(0, 0),
        columns,
        rows,
    }]
}

impl GameWorld {
    pub fn columns(&self) -> u16 {
        self.terrain
            .last()
            .map_or(WORLD_COLUMNS, |c| c.column.saturating_add(1))
    }

    pub fn rows(&self) -> u16 {
        self.terrain
            .last()
            .map_or(WORLD_ROWS, |c| c.row.saturating_add(1))
    }

    pub(super) fn in_bounds(&self, cell: CellCoordinate) -> bool {
        cell.column < self.columns() && cell.row < self.rows()
    }

    /// Discover the next planned site in run order (tests).
    #[cfg(test)]
    pub(super) fn discover_island(&mut self) {
        if let Some(origin) = self.undiscovered_sites().first().copied() {
            self.discover_site(origin);
        }
    }

    /// The run's planned sites without generated terrain yet, in run order.
    pub(super) fn undiscovered_sites(&self) -> Vec<Footprint> {
        archipelago_plan(self.seed)
            .into_iter()
            .filter(|site| !self.islands.contains(site))
            .collect()
    }

    fn discover_site(&mut self, site: Footprint) {
        let id = self.islands.len() as u64;
        let origin = site.origin;
        let columns = self.columns().max(origin.column + site.columns);
        let rows = self.rows().max(origin.row + site.rows);
        let grid = worldgen::Grid {
            columns: usize::from(site.columns),
            rows: usize::from(site.rows),
        };
        let generated = worldgen::destination(worldgen::mix(self.seed, id), id, grid);
        self.resize_ocean(columns, rows);
        let stride = usize::from(self.columns());
        let plan = archipelago_plan(self.seed);
        let burnt = plan[stronghold_rank(plan.len())] == site;
        for mut cell in generated.terrain {
            cell.column += origin.column;
            cell.row += origin.row;
            // The barbarians have burnt the green land around their stronghold.
            if burnt
                && matches!(
                    cell.biome,
                    TerrainBiome::Meadow
                        | TerrainBiome::Forest
                        | TerrainBiome::Prairie
                        | TerrainBiome::Heath
                        | TerrainBiome::Scrubland
                )
            {
                cell.biome = TerrainBiome::BurntForest;
            }
            self.terrain[usize::from(cell.row) * stride + usize::from(cell.column)] = cell;
        }
        for mut node in generated.resources {
            node.cell.column += origin.column;
            node.cell.row += origin.row;
            node.id = format!("island-{id}:{}", node.id);
            self.resources.push(node);
        }
        self.islands.push(site);
        if plan.last() == Some(&site) {
            self.place_temple(site);
        } else if self.stronghold_island() == Some(id as usize) {
            self.place_stronghold(site, id as usize);
        }
        // A course steered into the fog may now end on land: continue to the
        // reachable water nearest the tapped point, so the new coast comes into
        // sight, or stop at sea when none is reachable.
        for ship in 0..self.ships.len() {
            let Some(to) = self.ships[ship].destination.filter(|&to| !self.water(to)) else {
                continue;
            };
            let paths = self.sea_paths(ship, self.ships[ship].cell);
            let coast = site
                .cells()
                .filter(|&c| self.water_free(c, Some(ship)) && paths.cost(c).is_some())
                .min_by_key(|c| {
                    let (dx, dy) = (c.column.abs_diff(to.column), c.row.abs_diff(to.row));
                    (u32::from(dx).pow(2) + u32::from(dy).pow(2), c.row, c.column)
                });
            self.ships[ship].destination = coast;
        }
        self.populate_wildlife(id as usize);
        self.inventories.push(Stockpile::default());
    }

    fn resize_ocean(&mut self, columns: u16, rows: u16) {
        if !self.terrain.is_empty() && columns == self.columns() && rows == self.rows() {
            return;
        }
        let mut terrain = Vec::with_capacity(usize::from(columns) * usize::from(rows));
        terrain.extend((0..rows).flat_map(|row| {
            (0..columns).map(move |column| TerrainCell {
                column,
                row,
                biome: TerrainBiome::Water,
                elevation: -0.5,
            })
        }));
        for cell in self.terrain.drain(..) {
            terrain[usize::from(cell.row) * usize::from(columns) + usize::from(cell.column)] = cell;
        }
        self.terrain = terrain;
    }

    /// Once a ship exists the ocean spans the whole planned archipelago; a ship
    /// nearing an undiscovered site generates that island before it arrives.
    /// Ticks run this after building jobs, so a newly launched ship never
    /// waits a tick on the starting island's map.
    pub(super) fn expand_archipelago(&mut self) {
        if self.ships.is_empty() {
            return;
        }
        let (columns, rows) = plan_extent(&archipelago_plan(self.seed));
        self.resize_ocean(columns.max(self.columns()), rows.max(self.rows()));
        for site in self.undiscovered_sites() {
            if self
                .ships
                .iter()
                .any(|ship| site_distance(site, ship.cell) <= DISCOVERY_REACH)
            {
                self.discover_site(site);
            }
        }
    }

    /// Shortcuts back to discovered islands issue real sailing orders;
    /// undiscovered islands are reached only by steering into the fog.
    pub(super) fn voyage(&mut self, ship_id: &str, id: u64) -> Result<(), CommandError> {
        let ship = self.ship_index(ship_id)?;
        let island = *self
            .islands
            .get(id as usize)
            .ok_or(CommandError::InvalidDestination)?;
        if let Some(dock) = self
            .buildings
            .iter()
            .filter(|b| {
                b.kind == BuildingKind::Dock && b.is_complete() && island.contains(b.origin)
            })
            .min_by_key(|dock| Some(&dock.id) != self.ships[ship].home_dock_id.as_ref())
        {
            return self.sail_to_dock(ship_id, &dock.id.clone());
        }
        let occupancy = self.occupancy();
        let ship = self.ship_index(ship_id)?;
        let paths = self.sea_paths(ship, self.ships[ship].cell);
        let goals = self
            .terrain
            .iter()
            .map(|cell| cell.coordinate())
            .filter(|&cell| island.contains(cell))
            .filter(|&cell| {
                self.water_free(cell, Some(ship))
                    && [(0, -1), (-1, 0), (1, 0), (0, 1)]
                        .into_iter()
                        .any(|(dx, dy)| {
                            offset(cell, dx, dy, self.columns(), self.rows())
                                .is_some_and(|land| occupancy.is_free_for(land, None))
                        })
            });
        let target = paths.nearest(goals).ok_or(CommandError::ShoreBlocked)?;
        self.sail(ship_id, target)
    }

    pub(super) fn validate_islands(&self) -> Result<(), String> {
        let mut entities = BTreeSet::new();
        let (units, ships, buildings) = (&self.units, &self.ships, &self.buildings);
        for id in units
            .iter()
            .map(|u| &u.id)
            .chain(ships.iter().map(|s| &s.id))
            .chain(
                ships
                    .iter()
                    .flat_map(|s| s.passengers.iter().map(|u| &u.id)),
            )
            .chain(buildings.iter().map(|b| &b.id))
        {
            if !entities.insert(id) {
                return Err(format!("duplicate entity across islands: {id}"));
            }
        }
        let plan = archipelago_plan(self.seed);
        let distinct: BTreeSet<_> = self.islands.iter().map(|i| i.origin).collect();
        if self.island_id != 0
            || self.islands.first() != plan.first()
            || distinct.len() != self.islands.len()
            || self.islands.iter().any(|island| {
                !plan.contains(island)
                    || island
                        .origin
                        .column
                        .checked_add(island.columns)
                        .is_none_or(|end| end > self.columns())
                    || island
                        .origin
                        .row
                        .checked_add(island.rows)
                        .is_none_or(|end| end > self.rows())
            })
        {
            return Err("invalid archipelago origins".into());
        }
        Ok(())
    }
}

/// The run's island sites, chosen from the seed before any island beyond the
/// first is generated. Sites occupy a 3×3 grid of regions with seeded offsets,
/// grown outward from the start so the archipelago spreads in two directions.
/// Index 0 is the start at the origin; the last site, the farthest by crossings,
/// holds the temple. The rest are ordered by crossings from the start.
pub fn archipelago_plan(seed: u64) -> Vec<Footprint> {
    const GRID: u64 = 3;
    let hash = |salt: u64, value: u64| worldgen::mix(seed ^ SITE_SALT ^ salt, value);
    // Five to seven ordinary sites plus the barbarians' stronghold island.
    let count = 6 + (hash(0, 0) % 3) as usize;
    let neighbors = |slot: u64| {
        let (x, y) = (slot % GRID, slot / GRID);
        [
            (x > 0).then(|| slot - 1),
            (x + 1 < GRID).then(|| slot + 1),
            (y > 0).then(|| slot - GRID),
            (y + 1 < GRID).then(|| slot + GRID),
        ]
        .into_iter()
        .flatten()
    };
    let mut best: Option<(Vec<u64>, Vec<u32>)> = None;
    for attempt in 1..=32_u64 {
        let mut slots = vec![0_u64];
        while slots.len() < count {
            let next = slots
                .iter()
                .flat_map(|&slot| neighbors(slot))
                .filter(|slot| !slots.contains(slot))
                .min_by_key(|&slot| hash(attempt, slot))
                .expect("a 3×3 grid holds eight sites");
            slots.push(next);
        }
        // Crossings from the start through chosen sites (breadth-first).
        let mut crossings = vec![u32::MAX; slots.len()];
        crossings[0] = 0;
        let mut frontier = vec![0_usize];
        while let Some(at) = frontier.pop() {
            for (index, slot) in slots.iter().enumerate() {
                if neighbors(slots[at]).any(|n| n == *slot) && crossings[index] > crossings[at] + 1
                {
                    crossings[index] = crossings[at] + 1;
                    frontier.push(index);
                }
            }
        }
        let farthest = crossings.iter().max().copied().unwrap_or(0);
        if best
            .as_ref()
            .is_none_or(|(_, c)| farthest > c.iter().max().copied().unwrap_or(0))
        {
            best = Some((slots, crossings));
        }
        if farthest >= 3 {
            break;
        }
    }
    let (slots, crossings) = best.expect("at least one attempt");
    let mut order: Vec<usize> = (0..slots.len()).collect();
    order.sort_by_key(|&i| (crossings[i], hash(1 << 32, slots[i])));
    let (pitch_columns, pitch_rows) = site_size(usize::from(MAX_SITES) - 1);
    order
        .into_iter()
        .enumerate()
        .map(|(rank, i)| {
            let slot = slots[i];
            let jitter = |axis: u64| {
                if slot == 0 {
                    0
                } else {
                    (hash(axis << 40, slot) % u64::from(SITE_JITTER + 1)) as u16
                }
            };
            let (columns, rows) = site_size(rank);
            Footprint {
                origin: CellCoordinate::new(
                    (slot % GRID) as u16 * (pitch_columns + OCEAN_GAP + SITE_JITTER) + jitter(1),
                    (slot / GRID) as u16 * (pitch_rows + OCEAN_GAP + SITE_JITTER) + jitter(2),
                ),
                columns,
                rows,
            }
        })
        .collect()
}

/// The barbarian stronghold's own island: the middle site by crossings, so
/// never home, the temple, or the first island a ship reaches.
pub fn stronghold_rank(sites: usize) -> usize {
    sites / 2
}

/// The map size that contains every planned site.
pub fn plan_extent(plan: &[Footprint]) -> (u16, u16) {
    plan.iter()
        .fold((WORLD_COLUMNS, WORLD_ROWS), |(c, r), site| {
            (
                c.max(site.origin.column + site.columns),
                r.max(site.origin.row + site.rows),
            )
        })
}

/// Chebyshev distance from a cell to a site's region (0 inside it).
pub(super) fn site_distance(site: Footprint, cell: CellCoordinate) -> u16 {
    let gap = |value: u16, start: u16, length: u16| {
        if value < start {
            start - value
        } else {
            value.saturating_sub(start + length - 1)
        }
    };
    gap(cell.column, site.origin.column, site.columns).max(gap(
        cell.row,
        site.origin.row,
        site.rows,
    ))
}
