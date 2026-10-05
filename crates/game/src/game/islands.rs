//! Persistent islands in one coordinate space.
use super::*;
use crate::navigation::offset;

const OCEAN_GAP: u16 = 64;

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

    pub(super) fn discover_island(&mut self) {
        let id = self.island_origins.len() as u64;
        let Some(origin) = island_origin(id) else {
            return;
        };
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

    /// Destination shortcuts issue real sailing orders.
    pub(super) fn voyage(&mut self, ship_id: &str, id: u64) -> Result<(), CommandError> {
        let ship = self.ship_index(ship_id)?;
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
        if self.island_id != 0
            || self.island_origins.is_empty()
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
        Ok(())
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
