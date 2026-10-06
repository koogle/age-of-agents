//! Island inventories and movable storage share one spending/deposit boundary.
use super::*;

pub const SHIP_RESOURCE_CAPACITY: f64 = 50.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoDirection {
    Load,
    Unload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipConnection {
    pub ship_id: String,
    pub island_id: usize,
    pub docked: bool,
}

#[derive(Clone, Copy)]
pub(super) enum StorageSource {
    Island(usize),
    Ship(usize),
}

pub(super) struct StorageSite<'a> {
    pub id: &'a str,
    pub footprint: Footprint,
    pub source: StorageSource,
    pub space: f64,
}

pub fn island_at(origins: &[CellCoordinate], cell: CellCoordinate) -> Option<usize> {
    origins.iter().position(|origin| {
        cell.column >= origin.column
            && cell.row >= origin.row
            && cell.column - origin.column < WORLD_COLUMNS
            && cell.row - origin.row < WORLD_ROWS
    })
}

impl Stockpile {
    pub fn total(&self) -> f64 {
        self.entries().iter().map(|(_, amount)| amount).sum()
    }
}

impl GameWorld {
    pub fn island_at(&self, cell: CellCoordinate) -> Option<usize> {
        island_at(&self.island_origins, cell)
    }

    pub fn connected_island(&self, ship: usize) -> Option<usize> {
        if !self.ships[ship].stopped() {
            return None;
        }
        if let Some(dock) = self.dock_for_ship(ship) {
            return self.island_at(dock.origin);
        }
        self.landing_cells(ship).into_iter().find_map(|cell| {
            (self.in_bounds(cell)
                && self.terrain[usize::from(cell.row) * usize::from(self.columns())
                    + usize::from(cell.column)]
                .biome
                .is_walkable())
            .then(|| self.island_at(cell))
            .flatten()
        })
    }

    pub(super) fn ship_connections(&self) -> Vec<ShipConnection> {
        (0..self.ships.len())
            .filter_map(|ship| {
                Some(ShipConnection {
                    ship_id: self.ships[ship].id.clone(),
                    island_id: self.connected_island(ship)?,
                    docked: self.dock_for_ship(ship).is_some(),
                })
            })
            .collect()
    }

    pub fn available_on(&self, island: usize) -> Stockpile {
        let mut stock = self.inventories[island].clone();
        for (index, ship) in self.ships.iter().enumerate() {
            if self.connected_island(index) == Some(island) {
                for &kind in &ResourceKind::ALL {
                    stock.add(kind, ship.cargo.amount(kind));
                }
            }
        }
        stock
    }

    pub fn available_at(&self, cell: CellCoordinate) -> Stockpile {
        self.island_at(cell)
            .map_or_else(Stockpile::default, |id| self.available_on(id))
    }

    /// Reserve once, using onshore stock first, then connected ships in stable order.
    pub(super) fn spend_at(
        &mut self,
        cell: CellCoordinate,
        cost: &[(ResourceKind, f64)],
    ) -> Result<(), CommandError> {
        let island = self
            .island_at(cell)
            .ok_or(CommandError::InvalidDestination)?;
        let available = self.available_on(island);
        if let Some(kind) = available.missing_resource(cost) {
            return Err(CommandError::InsufficientResources(kind));
        }
        let ships: Vec<_> = (0..self.ships.len())
            .filter(|&i| self.connected_island(i) == Some(island))
            .collect();
        for &(kind, amount) in cost {
            let ashore = self.inventories[island].amount(kind).min(amount);
            self.inventories[island].add(kind, -ashore);
            let mut remaining = amount - ashore;
            for &ship in &ships {
                let take = self.ships[ship].cargo.amount(kind).min(remaining);
                self.ships[ship].cargo.add(kind, -take);
                remaining -= take;
            }
        }
        Ok(())
    }

    pub(super) fn credit_at(&mut self, cell: CellCoordinate, kind: ResourceKind, amount: f64) {
        let island = self
            .island_at(cell)
            .expect("land activity belongs to an island");
        self.inventories[island].add(kind, amount);
    }

    pub(super) fn transfer_ship_cargo(
        &mut self,
        id: &str,
        kind: ResourceKind,
        amount: f64,
        direction: CargoDirection,
    ) -> Result<(), CommandError> {
        if !amount.is_finite() || amount <= 0.0 {
            return Err(CommandError::InvalidCargoAmount);
        }
        let ship = self.ship_index(id)?;
        let island = self
            .connected_island(ship)
            .filter(|_| self.dock_for_ship(ship).is_some())
            .ok_or(CommandError::ShipStorageUnavailable)?;
        match direction {
            CargoDirection::Load => {
                if amount > SHIP_RESOURCE_CAPACITY - self.ships[ship].cargo.total() {
                    return Err(CommandError::ShipHoldFull);
                }
                if self.inventories[island].amount(kind) < amount {
                    return Err(CommandError::InsufficientResources(kind));
                }
                self.inventories[island].add(kind, -amount);
                self.ships[ship].cargo.add(kind, amount);
            }
            CargoDirection::Unload => {
                if self.ships[ship].cargo.amount(kind) < amount {
                    return Err(CommandError::InsufficientResources(kind));
                }
                self.ships[ship].cargo.add(kind, -amount);
                self.inventories[island].add(kind, amount);
            }
        }
        Ok(())
    }

    pub(super) fn storage_sites(&self, kind: Option<ResourceKind>) -> Vec<StorageSite<'_>> {
        let mut sites: Vec<_> = self
            .buildings
            .iter()
            .filter(|b| {
                b.is_complete()
                    && kind.map_or(b.kind == BuildingKind::TownCenter, |kind| {
                        b.kind.accepts(kind)
                    })
            })
            .filter_map(|b| {
                Some(StorageSite {
                    id: &b.id,
                    footprint: b.footprint(),
                    source: StorageSource::Island(self.island_at(b.origin)?),
                    space: f64::MAX,
                })
            })
            .collect();
        for (ship, vessel) in self.ships.iter().enumerate() {
            let space = SHIP_RESOURCE_CAPACITY - vessel.cargo.total();
            if space > f64::EPSILON && self.connected_island(ship).is_some() {
                sites.push(StorageSite {
                    id: &vessel.id,
                    footprint: self
                        .dock_for_ship(ship)
                        .map_or(vessel.footprint(), Building::footprint),
                    source: StorageSource::Ship(ship),
                    space,
                });
            }
        }
        sites
    }

    pub(super) fn deposit_here(&mut self, unit: usize, target: Option<&str>) -> bool {
        let Some(cargo) = self.units[unit].cargo.clone() else {
            return true;
        };
        let site = self
            .storage_sites(Some(cargo.kind))
            .into_iter()
            .find(|site| {
                target.is_none_or(|id| site.id == id) && self.is_beside(unit, site.footprint)
            });
        let Some(site) = site else {
            return false;
        };
        let source = site.source;
        let amount = cargo.amount.min(site.space);
        match source {
            StorageSource::Island(island) => self.inventories[island].add(cargo.kind, amount),
            StorageSource::Ship(ship) => self.ships[ship].cargo.add(cargo.kind, amount),
        }
        self.units[unit].cargo = (cargo.amount > amount).then_some(CarriedResource {
            kind: cargo.kind,
            amount: cargo.amount - amount,
        });
        self.units[unit].cargo.is_none()
    }
}
