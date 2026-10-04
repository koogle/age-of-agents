//! The one place that decides who owns which cell.
//!
//! Every building footprint (complete or foundation), every live resource node,
//! every unit cell, and every in-progress step target is an exclusive claim. A
//! move destination is a reservation: other units may walk through it, but no
//! one else may choose it as a place to stop. `GameWorld::validate` rebuilds
//! this map from scratch and fails if any two claims or reservations collide,
//! so the invariant is checked rather than assumed.

#[cfg(test)]
use std::collections::BTreeSet;

use super::in_bounds;
use crate::game::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::game) enum Claim {
    Building(usize),
    Resource(usize),
    Unit(usize),
}

pub(in crate::game) struct Occupancy {
    impassable: Vec<bool>,
    claims: Vec<Option<Claim>>,
    reservations: Vec<Option<usize>>,
}

impl Occupancy {
    fn index(cell: CellCoordinate) -> usize {
        usize::from(cell.row) * usize::from(WORLD_COLUMNS) + usize::from(cell.column)
    }

    pub(in crate::game) fn claim(&self, cell: CellCoordinate) -> Option<Claim> {
        self.claims[Self::index(cell)]
    }

    /// Water, peaks, rivers, buildings, foundations, and live resources: things nobody walks through.
    pub(in crate::game) fn is_static(&self, cell: CellCoordinate) -> bool {
        self.impassable[Self::index(cell)]
            || matches!(
                self.claim(cell),
                Some(Claim::Building(_) | Claim::Resource(_))
            )
    }

    pub(in crate::game) fn has_other_unit(&self, cell: CellCoordinate, unit: usize) -> bool {
        matches!(self.claim(cell), Some(Claim::Unit(other)) if other != unit)
    }

    pub(in crate::game) fn reservation(&self, cell: CellCoordinate) -> Option<usize> {
        self.reservations[Self::index(cell)]
    }

    pub(in crate::game) fn reserved_by_other(
        &self,
        cell: CellCoordinate,
        unit: Option<usize>,
    ) -> bool {
        self.reservations[Self::index(cell)].is_some_and(|owner| Some(owner) != unit)
    }

    /// A cell `unit` may stop in: unclaimed by anyone else and unreserved by anyone else.
    pub(in crate::game) fn is_free_for(&self, cell: CellCoordinate, unit: Option<usize>) -> bool {
        if self.impassable[Self::index(cell)] {
            return false;
        }
        let claimed = match self.claim(cell) {
            None => false,
            Some(Claim::Unit(owner)) => Some(owner) != unit,
            Some(_) => true,
        };
        !claimed && !self.reserved_by_other(cell, unit)
    }
}

impl GameWorld {
    /// The occupancy map of a valid world. Every mutation keeps the world valid,
    /// so a collision here is a programming error, not a gameplay outcome.
    pub(in crate::game) fn occupancy(&self) -> Occupancy {
        self.try_occupancy()
            .unwrap_or_else(|error| panic!("world invariant violated: {error}"))
    }

    pub(in crate::game) fn try_occupancy(&self) -> Result<Occupancy, String> {
        let count = usize::from(WORLD_COLUMNS) * usize::from(WORLD_ROWS);
        let impassable: Vec<bool> = self
            .terrain
            .iter()
            .map(|cell| !cell.biome.is_walkable())
            .collect();
        let mut occupancy = Occupancy {
            impassable: impassable.clone(),
            claims: vec![None; count],
            reservations: vec![None; count],
        };
        let mut claim = |cell: CellCoordinate, owner: Claim| {
            if !in_bounds(cell) {
                return Err(format!("{owner:?} claims out-of-bounds cell {cell:?}"));
            }
            if impassable[Occupancy::index(cell)] {
                return Err(format!("{owner:?} stands on impassable ground at {cell:?}"));
            }
            let slot = &mut occupancy.claims[Occupancy::index(cell)];
            if let Some(existing) = slot {
                return Err(format!("{owner:?} and {existing:?} both claim {cell:?}"));
            }
            *slot = Some(owner);
            Ok(())
        };
        for (index, building) in self.buildings.iter().enumerate() {
            for cell in building.footprint().cells() {
                claim(cell, Claim::Building(index))?;
            }
        }
        for (index, resource) in self.resources.iter().enumerate() {
            if resource.amount > 0.0 || resource.field.is_some() {
                let footprint = resource.footprint();
                if !footprint.fits_in(WORLD_COLUMNS, WORLD_ROWS) {
                    return Err(format!("{} footprint is outside the world", resource.id));
                }
                for cell in footprint.cells() {
                    claim(cell, Claim::Resource(index))?;
                }
            }
        }
        for (index, unit) in self.units.iter().enumerate() {
            for cell in unit.claimed_cells() {
                claim(cell, Claim::Unit(index))?;
            }
        }
        for (index, unit) in self.units.iter().enumerate() {
            if let UnitAction::Move { to } = unit.action {
                if !in_bounds(to) {
                    return Err(format!("{} moves out of bounds", unit.id));
                }
                let slot = Occupancy::index(to);
                if let Some(owner) = occupancy.reservations[slot] {
                    return Err(format!(
                        "{} and {} reserve the same destination",
                        self.units[owner].id, unit.id
                    ));
                }
                if occupancy.is_static(to) {
                    return Err(format!("{} reserves a blocked destination", unit.id));
                }
                occupancy.reservations[slot] = Some(index);
            }
        }
        Ok(occupancy)
    }

    /// Cells claimed by anything at all, for tests and diagnostics.
    #[cfg(test)]
    pub(in crate::game) fn claimed_cells(&self) -> BTreeSet<CellCoordinate> {
        let occupancy = self.occupancy();
        self.terrain
            .iter()
            .map(|cell| cell.coordinate())
            .filter(|cell| occupancy.claim(*cell).is_some())
            .collect()
    }
}
