//! Persistent islands in one coordinate space.
use super::*;
use crate::navigation::offset;

const OCEAN_GAP: u16 = 64;
/// Extra seeded offset per island site, so the archipelago is not a lattice.
const SITE_JITTER: u16 = 24;
/// A ship this close to an undiscovered site's region discovers it.
const DISCOVERY_REACH: u16 = 12;
const SITE_SALT: u64 = 0x6172_6368_6970_656c;

pub(super) fn starting_origins() -> Vec<CellCoordinate> {
    vec![CellCoordinate::new(0, 0)]
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
    pub(super) fn undiscovered_sites(&self) -> Vec<CellCoordinate> {
        archipelago_plan(self.seed)
            .into_iter()
            .filter(|site| !self.island_origins.contains(site))
            .collect()
    }

    fn discover_site(&mut self, origin: CellCoordinate) {
        let id = self.island_origins.len() as u64;
        let columns = self.columns().max(origin.column + WORLD_COLUMNS);
        let rows = self.rows().max(origin.row + WORLD_ROWS);
        let generated = worldgen::destination(worldgen::mix(self.seed, id), id);
        self.resize_ocean(columns, rows);
        let stride = usize::from(self.columns());
        for mut cell in generated.terrain {
            cell.column += origin.column;
            cell.row += origin.row;
            self.terrain[usize::from(cell.row) * stride + usize::from(cell.column)] = cell;
        }
        for mut node in generated.resources {
            node.cell.column += origin.column;
            node.cell.row += origin.row;
            node.id = format!("island-{id}:{}", node.id);
            self.resources.push(node);
        }
        self.island_origins.push(origin);
        if archipelago_plan(self.seed).last() == Some(&origin) {
            self.place_temple(origin);
        }
        // A course steered into the fog may now end on land: continue to the
        // reachable water nearest the tapped point, so the new coast comes into
        // sight, or stop at sea when none is reachable.
        for ship in 0..self.ships.len() {
            let Some(to) = self.ships[ship].destination.filter(|&to| !self.water(to)) else {
                continue;
            };
            let paths = self.sea_paths(ship, self.ships[ship].cell);
            let coast = Footprint {
                origin,
                columns: WORLD_COLUMNS,
                rows: WORLD_ROWS,
            }
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
        let origin = *self
            .island_origins
            .get(id as usize)
            .ok_or(CommandError::InvalidDestination)?;
        if let Some(dock) = self
            .buildings
            .iter()
            .filter(|b| {
                b.kind == BuildingKind::Dock
                    && b.is_complete()
                    && b.origin.column >= origin.column
                    && b.origin.column < origin.column + WORLD_COLUMNS
                    && b.origin.row >= origin.row
                    && b.origin.row < origin.row + WORLD_ROWS
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
            .filter(|cell| {
                cell.column >= origin.column
                    && cell.column < origin.column + WORLD_COLUMNS
                    && cell.row >= origin.row
                    && cell.row < origin.row + WORLD_ROWS
            })
            .map(|cell| cell.coordinate())
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
        let distinct: BTreeSet<_> = self.island_origins.iter().collect();
        if self.island_id != 0
            || self.island_origins.first() != plan.first()
            || distinct.len() != self.island_origins.len()
            || self.island_origins.iter().any(|origin| {
                !plan.contains(origin)
                    || origin
                        .column
                        .checked_add(WORLD_COLUMNS)
                        .is_none_or(|end| end > self.columns())
                    || origin
                        .row
                        .checked_add(WORLD_ROWS)
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
pub fn archipelago_plan(seed: u64) -> Vec<CellCoordinate> {
    const GRID: u64 = 3;
    let hash = |salt: u64, value: u64| worldgen::mix(seed ^ SITE_SALT ^ salt, value);
    let count = 5 + (hash(0, 0) % 3) as usize;
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
                .expect("a 3×3 grid holds seven sites");
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
    order
        .into_iter()
        .map(|i| {
            let slot = slots[i];
            let jitter = |axis: u64| {
                if slot == 0 {
                    0
                } else {
                    (hash(axis << 40, slot) % u64::from(SITE_JITTER + 1)) as u16
                }
            };
            CellCoordinate::new(
                (slot % GRID) as u16 * (WORLD_COLUMNS + OCEAN_GAP + SITE_JITTER) + jitter(1),
                (slot / GRID) as u16 * (WORLD_ROWS + OCEAN_GAP + SITE_JITTER) + jitter(2),
            )
        })
        .collect()
}

/// The map size that contains every planned site.
pub fn plan_extent(plan: &[CellCoordinate]) -> (u16, u16) {
    plan.iter()
        .fold((WORLD_COLUMNS, WORLD_ROWS), |(c, r), site| {
            (
                c.max(site.column + WORLD_COLUMNS),
                r.max(site.row + WORLD_ROWS),
            )
        })
}

/// Chebyshev distance from a cell to a site's region (0 inside it).
pub(super) fn site_distance(site: CellCoordinate, cell: CellCoordinate) -> u16 {
    let gap = |value: u16, start: u16, length: u16| {
        if value < start {
            start - value
        } else {
            value.saturating_sub(start + length - 1)
        }
    };
    gap(cell.column, site.column, WORLD_COLUMNS).max(gap(cell.row, site.row, WORLD_ROWS))
}
