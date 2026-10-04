//! Persistent local maps. Only the viewed island advances; voyages carry one
//! stopped ship, while the departing settlement and its orders remain intact.
use super::*;
use crate::navigation::{PathTree, offset};

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
    pub stockpile: Stockpile,
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
        swap(&mut self.stockpile, &mut world.stockpile);
    }
}

impl GameWorld {
    pub(super) fn all_buildings(&self) -> impl Iterator<Item = &Building> {
        self.buildings
            .iter()
            .chain(self.islands.iter().flat_map(|i| i.buildings.iter()))
    }

    pub(super) fn discover_island(&mut self) {
        let id = self.islands.len() as u64 + 1;
        let island = worldgen::destination(worldgen::mix(self.seed, id), id);
        self.islands.push(IslandState {
            id,
            terrain: island.terrain,
            resources: island.resources,
            explored_cells: Vec::new(),
            units: Vec::new(),
            ships: Vec::new(),
            buildings: Vec::new(),
            stockpile: Stockpile::default(),
        });
    }

    pub(super) fn voyage(&mut self, ship_id: &str, id: u64) -> Result<(), CommandError> {
        let ship = self.ship_index(ship_id)?;
        if !self.ships[ship].stopped() {
            return Err(CommandError::ShipMustBeStopped);
        }
        // Only the next discovery can be requested; arbitrary IDs never allocate maps.
        if id == self.island_id || id > self.islands.len() as u64 + 1 {
            return Err(CommandError::InvalidDestination);
        }
        // A lake-bound vessel cannot depart for the open ocean.
        let sea = PathTree::search(WORLD_COLUMNS, WORLD_ROWS, self.ships[ship].cell, |c| {
            self.water_free(c, Some(ship))
        });
        let edge = (0..WORLD_COLUMNS)
            .flat_map(|x| {
                [
                    CellCoordinate::new(x, 0),
                    CellCoordinate::new(x, WORLD_ROWS - 1),
                ]
            })
            .chain((0..WORLD_ROWS).flat_map(|y| {
                [
                    CellCoordinate::new(0, y),
                    CellCoordinate::new(WORLD_COLUMNS - 1, y),
                ]
            }));
        if sea.nearest(edge).is_none() {
            return Err(CommandError::TargetUnreachable);
        }
        if id == self.islands.len() as u64 + 1 {
            self.discover_island();
        }
        let destination = self
            .islands
            .iter()
            .position(|i| i.id == id)
            .ok_or(CommandError::InvalidDestination)?;
        // Cancel outstanding boarding orders before archiving the departing map.
        for unit in &mut self.units {
            if matches!(&unit.action, UnitAction::Board { ship_id: target } if target == ship_id) {
                unit.action = UnitAction::Idle;
            }
        }
        let mut vessel = self.ships.remove(ship);
        let mut island = self.islands.remove(destination);
        island.exchange(self);
        self.islands.push(island);
        self.islands.sort_by_key(|i| i.id);
        let occupancy = self.occupancy();
        let sea = PathTree::search(WORLD_COLUMNS, WORLD_ROWS, CellCoordinate::new(0, 0), |c| {
            self.water_free(c, None)
        });
        let arrival = self
            .terrain
            .iter()
            .map(|c| c.coordinate())
            .find(|&c| {
                self.water_free(c, None)
                    && sea.cost(c).is_some()
                    && [(0, -1), (-1, 0), (1, 0), (0, 1)]
                        .into_iter()
                        .any(|(dx, dy)| {
                            offset(c, dx, dy, WORLD_COLUMNS, WORLD_ROWS).is_some_and(|land| {
                                occupancy.is_free_for(land, None)
                                    && PathTree::search(WORLD_COLUMNS, WORLD_ROWS, land, |c| {
                                        occupancy.is_free_for(c, None)
                                    })
                                    .nearest(
                                        self.resources
                                            .iter()
                                            .filter(|r| r.kind == ResourceKind::Wood)
                                            .flat_map(|r| {
                                                movement::interaction_cells(Footprint {
                                                    origin: r.cell,
                                                    columns: 1,
                                                    rows: 1,
                                                })
                                            }),
                                    )
                                    .is_some()
                            })
                        })
            })
            .ok_or(CommandError::ShoreBlocked)?;
        vessel.cell = arrival;
        self.ships.push(vessel);
        self.refresh_exploration();
        Ok(())
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
