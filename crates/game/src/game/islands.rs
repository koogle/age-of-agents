//! Persistent islands in one coordinate space; archived maps are read only for save upgrades.
use super::*;
use crate::navigation::offset;

const OCEAN_GAP: u16 = 64;

pub(super) fn starting_origins() -> Vec<CellCoordinate> {
    vec![CellCoordinate::new(0, 0)]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IslandState {
    pub id: u64,
    pub terrain: Vec<TerrainCell>,
    pub explored_cells: Vec<CellCoordinate>,
    pub units: Vec<Unit>,
    pub ships: Vec<TransportShip>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<Building>,
}

impl IslandState {
    fn exchange(&mut self, world: &mut GameWorld) {
        use std::mem::swap;
        swap(&mut self.id, &mut world.island_id);
        swap(&mut self.terrain, &mut world.terrain);
        swap(&mut self.explored_cells, &mut world.explored_cells);
        swap(&mut self.units, &mut world.units);
        swap(&mut self.ships, &mut world.ships);
        swap(&mut self.resources, &mut world.resources);
        swap(&mut self.buildings, &mut world.buildings);
    }
}

impl GameWorld {
    pub(super) fn all_buildings(&self) -> impl Iterator<Item = &Building> {
        self.buildings
            .iter()
            .chain(self.islands.iter().flat_map(|i| i.buildings.iter()))
    }

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

    pub(super) fn discover_island(&mut self) {
        let id = self.island_origins.len() as u64;
        let Some(origin) = island_origin(id) else {
            return;
        };
        let columns = self.columns().max(origin.column + WORLD_COLUMNS);
        let rows = self.rows().max(origin.row + WORLD_ROWS);
        let generated = worldgen::destination(worldgen::mix(self.seed, id), id);
        let mut island = IslandState {
            id,
            terrain: generated.terrain,
            resources: generated.resources,
            explored_cells: Vec::new(),
            units: Vec::new(),
            ships: Vec::new(),
            buildings: Vec::new(),
        };
        island.translate(origin);
        self.resize_ocean(columns, rows);
        self.merge_island(island);
        self.island_origins.push(origin);
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

    fn merge_island(&mut self, island: IslandState) {
        let columns = usize::from(self.columns());
        for cell in island.terrain {
            self.terrain[usize::from(cell.row) * columns + usize::from(cell.column)] = cell;
        }
        self.explored_cells.extend(island.explored_cells);
        self.explored_cells.sort_unstable();
        self.units.extend(island.units);
        self.ships.extend(island.ships);
        self.resources.extend(island.resources);
        self.buildings.extend(island.buildings);
    }

    /// Discovery follows the frontier vessel; old coastlines never regenerate.
    pub(super) fn expand_archipelago(&mut self) {
        let id = self.island_origins.len() as u64;
        let Some(next) = island_origin(id) else {
            return;
        };
        let current = self.island_origins[id as usize - 1];
        let departing = |cell: CellCoordinate| {
            if next.column > current.column {
                cell.column >= current.column + WORLD_COLUMNS - 12
            } else if next.column < current.column {
                cell.column <= current.column + 12
            } else if next.row > current.row {
                cell.row >= current.row + WORLD_ROWS - 12
            } else {
                cell.row <= current.row + 12
            }
        };
        if self.ships.iter().any(|ship| {
            ship.cell.column >= current.column
                && ship.cell.column < current.column + WORLD_COLUMNS
                && ship.cell.row >= current.row
                && ship.cell.row < current.row + WORLD_ROWS
                && departing(ship.cell)
        }) {
            self.discover_island();
        }
    }

    /// Upgrade old local maps once, preserving orders, fog, queues and passengers.
    pub fn unify_islands(&mut self) -> Result<(), String> {
        if self.islands.is_empty() {
            return Ok(());
        }
        self.validate()?;
        let mut maps = self.islands.clone();
        // Check all offsets before touching a validated legacy save.
        if island_origin(maps.len() as u64).is_none() {
            return Err("archipelago exceeds coordinate range".into());
        }
        self.islands.clear();
        let mut current = IslandState {
            id: 0,
            terrain: Vec::new(),
            explored_cells: Vec::new(),
            units: Vec::new(),
            ships: Vec::new(),
            resources: Vec::new(),
            buildings: Vec::new(),
        };
        current.exchange(self);
        maps.push(current);
        maps.sort_by_key(|i| i.id);
        let origins: Vec<_> = maps
            .iter()
            .map(|i| island_origin(i.id).ok_or("archipelago exceeds coordinate range"))
            .collect::<Result<_, _>>()?;
        let columns = origins
            .iter()
            .map(|o| o.column + WORLD_COLUMNS)
            .max()
            .unwrap();
        let rows = origins.iter().map(|o| o.row + WORLD_ROWS).max().unwrap();
        self.resize_ocean(columns, rows);
        self.island_origins.clear();
        for mut island in maps {
            let origin = origins[island.id as usize];
            island.translate(origin);
            self.merge_island(island);
            self.island_origins.push(origin);
        }
        self.island_id = 0;
        self.validate()
    }

    /// Legacy destination commands now issue real sailing orders.
    pub(super) fn voyage(&mut self, ship_id: &str, id: u64) -> Result<(), CommandError> {
        let ship = self.ship_index(ship_id)?;
        if self.ships[ship].home_dock_id.is_none() {
            self.ships[ship].home_dock_id = self.dock_for_ship(ship).map(|dock| dock.id.clone());
        }
        if id == self.island_origins.len() as u64 {
            let next = island_origin(id).ok_or(CommandError::InvalidDestination)?;
            let current = self.island_origins[id as usize - 1];
            let mut frontier = CellCoordinate::new(
                current.column + WORLD_COLUMNS / 2,
                current.row + WORLD_ROWS / 2,
            );
            if next.column > current.column {
                frontier.column = current.column + WORLD_COLUMNS - 1;
            } else if next.column < current.column {
                frontier.column = current.column;
            } else if next.row > current.row {
                frontier.row = current.row + WORLD_ROWS - 1;
            } else {
                frontier.row = current.row;
            }
            return self.sail(ship_id, frontier);
        }
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
        let ids: BTreeSet<_> = self
            .islands
            .iter()
            .map(|i| i.id)
            .chain([self.island_id])
            .collect();
        if ids.len() != self.islands.len() + 1 || !ids.iter().copied().eq(0..ids.len() as u64) {
            return Err("island IDs are not a unique discovery sequence".into());
        }
        let mut research = BTreeSet::new();
        for job in self.all_buildings().flat_map(Building::jobs) {
            if let BuildingJob::Research { technology, .. } = job
                && !research.insert(technology)
            {
                return Err("research queued on multiple islands".into());
            }
        }
        let mut entities = BTreeSet::new();
        for (units, ships, buildings) in
            std::iter::once((&self.units, &self.ships, &self.buildings)).chain(
                self.islands
                    .iter()
                    .map(|i| (&i.units, &i.ships, &i.buildings)),
            )
        {
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
        }
        if self.islands.is_empty() {
            if self.island_origins.is_empty()
                || self.island_origins.iter().enumerate().any(|(id, origin)| {
                    island_origin(id as u64) != Some(*origin)
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
            return Ok(());
        }
        let mut local = self.clone();
        let islands = std::mem::take(&mut local.islands);
        for mut island in islands {
            island.exchange(&mut local);
            local.validate_local()?;
        }
        Ok(())
    }
}

impl IslandState {
    fn translate(&mut self, origin: CellCoordinate) {
        let shift = |cell: &mut CellCoordinate| {
            cell.column += origin.column;
            cell.row += origin.row;
        };
        let prefix = |id: &mut String| {
            *id = format!("island-{}:{id}", self.id);
        };
        let translate_unit = |unit: &mut Unit| {
            shift(&mut unit.cell);
            if let Some(step) = &mut unit.step {
                shift(&mut step.to);
            }
            match &mut unit.action {
                UnitAction::Move { to } => shift(to),
                UnitAction::ExploreBuild { origin, .. } => shift(origin),
                UnitAction::Gather { resource_id, .. } | UnitAction::Cultivate { resource_id } => {
                    prefix(resource_id)
                }
                _ => {}
            }
        };
        for cell in &mut self.terrain {
            cell.column += origin.column;
            cell.row += origin.row;
        }
        for cell in &mut self.explored_cells {
            shift(cell);
        }
        for unit in &mut self.units {
            translate_unit(unit);
        }
        for ship in &mut self.ships {
            shift(&mut ship.cell);
            if let Some(step) = &mut ship.step {
                shift(&mut step.to);
            }
            if let Some(to) = &mut ship.destination {
                shift(to);
            }
            for unit in &mut ship.passengers {
                translate_unit(unit);
            }
        }
        for node in &mut self.resources {
            shift(&mut node.cell);
            prefix(&mut node.id);
        }
        for building in &mut self.buildings {
            shift(&mut building.origin);
        }
    }
}

/// An expanding square spiral in the positive quadrant. Consecutive sites share
/// an ocean crossing, and the bounding rectangle stays proportional to island count.
fn island_origin(id: u64) -> Option<CellCoordinate> {
    let mut ring = 0_u64;
    while (ring + 1).checked_mul(ring + 1)? <= id {
        ring += 1;
    }
    let step = id - ring * ring;
    let (a, b) = if step <= ring {
        (step, ring)
    } else {
        (ring, 2 * ring - step)
    };
    let (x, y) = if ring.is_multiple_of(2) {
        (a, b)
    } else {
        (b, a)
    };
    let column = u16::try_from(x * u64::from(WORLD_COLUMNS + OCEAN_GAP)).ok()?;
    let row = u16::try_from(y * u64::from(WORLD_ROWS + OCEAN_GAP)).ok()?;
    column.checked_add(WORLD_COLUMNS)?;
    row.checked_add(WORLD_ROWS)?;
    Some(CellCoordinate::new(column, row))
}
