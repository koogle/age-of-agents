//! The one place that decides who owns which cell.
//!
//! Every building footprint (complete or foundation), every live resource node,
//! every unit cell, and every in-progress step target is an exclusive claim. A
//! move destination is a reservation: other units may walk through it, but no
//! one else may choose it as a place to stop. `GameWorld::validate` rebuilds
//! this map from scratch and fails if any two claims or reservations collide,
//! so the invariant is checked rather than assumed.

use std::collections::BTreeSet;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Claim {
    Animal(usize),
    Building(usize),
    Resource(usize),
    Unit(usize),
}

pub(super) struct Occupancy {
    columns: u16,
    rows: u16,
    impassable: Vec<bool>,
    claims: Vec<Option<Claim>>,
    reservations: Vec<Option<usize>>,
}

impl Occupancy {
    fn index(&self, cell: CellCoordinate) -> usize {
        usize::from(cell.row) * usize::from(self.columns) + usize::from(cell.column)
    }

    pub(super) fn claim(&self, cell: CellCoordinate) -> Option<Claim> {
        if cell.column >= self.columns || cell.row >= self.rows {
            return None;
        }
        self.claims.get(self.index(cell)).copied().flatten()
    }

    /// Water, peaks, rivers, buildings, foundations, and live resources: things nobody walks through.
    pub(super) fn is_static(&self, cell: CellCoordinate) -> bool {
        (cell.column >= self.columns || cell.row >= self.rows || self.impassable[self.index(cell)])
            || matches!(
                self.claim(cell),
                Some(Claim::Building(_) | Claim::Resource(_) | Claim::Animal(_))
            )
    }

    pub(super) fn has_other_unit(&self, cell: CellCoordinate, unit: usize) -> bool {
        matches!(self.claim(cell), Some(Claim::Unit(other)) if other != unit)
    }

    pub(super) fn reservation(&self, cell: CellCoordinate) -> Option<usize> {
        if cell.column >= self.columns || cell.row >= self.rows {
            return None;
        }
        self.reservations.get(self.index(cell)).copied().flatten()
    }

    pub(super) fn reserved_by_other(&self, cell: CellCoordinate, unit: Option<usize>) -> bool {
        self.reservation(cell)
            .is_some_and(|owner| Some(owner) != unit)
    }

    /// A cell `unit` may stop in: unclaimed by anyone else and unreserved by anyone else.
    pub(super) fn is_free_for(&self, cell: CellCoordinate, unit: Option<usize>) -> bool {
        if cell.column >= self.columns || cell.row >= self.rows || self.impassable[self.index(cell)]
        {
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
    pub(super) fn occupancy(&self) -> Occupancy {
        self.try_occupancy()
            .unwrap_or_else(|error| panic!("world invariant violated: {error}"))
    }

    fn try_occupancy(&self) -> Result<Occupancy, String> {
        let count = usize::from(self.columns()) * usize::from(self.rows());
        let impassable: Vec<bool> = self
            .terrain
            .iter()
            .map(|cell| !cell.biome.is_walkable())
            .collect();
        let mut occupancy = Occupancy {
            columns: self.columns(),
            rows: self.rows(),
            impassable: impassable.clone(),
            claims: vec![None; count],
            reservations: vec![None; count],
        };
        let mut claim = |cell: CellCoordinate, owner: Claim| {
            if !self.in_bounds(cell) {
                return Err(format!("{owner:?} claims out-of-bounds cell {cell:?}"));
            }
            if impassable
                [usize::from(cell.row) * usize::from(self.columns()) + usize::from(cell.column)]
            {
                return Err(format!("{owner:?} stands on impassable ground at {cell:?}"));
            }
            let slot = &mut occupancy.claims
                [usize::from(cell.row) * usize::from(self.columns()) + usize::from(cell.column)];
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
                if footprint.origin.column > self.columns() - footprint.columns
                    || footprint.origin.row > self.rows() - footprint.rows
                {
                    return Err(format!("{} footprint is outside the world", resource.id));
                }
                for cell in footprint.cells() {
                    claim(cell, Claim::Resource(index))?;
                }
            }
        }
        for (index, animal) in self.animals.iter().enumerate() {
            claim(animal.cell, Claim::Animal(index))?;
            if let Some(step) = animal.step {
                claim(step.to, Claim::Animal(index))?;
            }
        }
        for (index, unit) in self.units.iter().enumerate() {
            claim(unit.cell, Claim::Unit(index))?;
            if let Some(step) = unit.step {
                claim(step.to, Claim::Unit(index))?;
            }
        }
        for (index, unit) in self.units.iter().enumerate() {
            if let UnitAction::Move { to } = unit.action {
                if !self.in_bounds(to) {
                    return Err(format!("{} moves out of bounds", unit.id));
                }
                let slot = occupancy.index(to);
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

    /// Checks every structural invariant of the world. Persisted worlds that
    /// fail this are corrupt and must not be loaded.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_wildlife()?;
        self.validate_islands()?;
        self.validate_local()
    }

    pub(super) fn validate_local(&self) -> Result<(), String> {
        let expected_terrain = (0..self.rows()).flat_map(|row| {
            (0..self.columns()).map(move |column| CellCoordinate::new(column, row))
        });
        if !self
            .terrain
            .iter()
            .map(|cell| cell.coordinate())
            .eq(expected_terrain)
        {
            return Err("terrain is not a complete row-major grid".into());
        }
        if !self.explored_cells.iter().all(|cell| self.in_bounds(*cell))
            || !self.explored_cells.is_sorted_by(|a, b| a < b)
        {
            return Err("explored cells are out of bounds or not strictly sorted".into());
        }
        if ![0.0, 1.0, 2.0].contains(&self.simulation_speed) {
            return Err("simulation speed is not 0, 1, or 2".into());
        }
        unique_ids(self.units.iter().map(|unit| unit.id.as_str()), "unit")?;
        unique_ids(self.buildings.iter().map(|b| b.id.as_str()), "building")?;
        unique_ids(self.resources.iter().map(|r| r.id.as_str()), "resource")?;
        if self
            .units
            .iter()
            .any(|unit| unit.id == self.next_unit_name())
            || self
                .buildings
                .iter()
                .any(|b| b.id == self.next_building_name())
        {
            return Err("an id counter would reissue an existing id".into());
        }
        for (name, amount) in self.stockpile.entries() {
            if !amount.is_finite() || amount < 0.0 {
                return Err(format!("stockpile {name} is {amount}"));
            }
        }
        for resource in &self.resources {
            if let Some(field) = &resource.field
                && (resource.kind != ResourceKind::Food
                    || field.work.is_some_and(|work| {
                        !(0.0..FIELD_WORK_SECONDS).contains(&work) || resource.amount != 0.0
                    }))
            {
                return Err(format!("{} has invalid field state", resource.id));
            }
            if !(resource.capacity.is_finite()
                && resource.capacity > 0.0
                && (0.0..=resource.capacity).contains(&resource.amount))
            {
                return Err(format!("{} has an invalid amount", resource.id));
            }
        }
        let mut queued_research = BTreeSet::new();
        for building in &self.buildings {
            if let Some(work) = building.construction
                && !(0.0..building.kind.build_seconds()).contains(&work)
            {
                return Err(format!("{} has invalid construction progress", building.id));
            }
            if building.queue.len() > MAX_QUEUED_JOBS
                || (!building.queue.is_empty() && building.job.is_none())
                || building
                    .queue
                    .windows(2)
                    .any(|pair| pair[0].id >= pair[1].id)
                || building.queue.iter().any(|entry| {
                    entry.id >= building.next_queue_id
                        || match entry.job {
                            BuildingJob::Produce {
                                elapsed_seconds, ..
                            }
                            | BuildingJob::Research {
                                elapsed_seconds, ..
                            } => elapsed_seconds != 0.0,
                        }
                })
            {
                return Err(format!("{} has an invalid task queue", building.id));
            }
            for job in building.jobs() {
                let elapsed = match job {
                    BuildingJob::Produce {
                        elapsed_seconds, ..
                    }
                    | BuildingJob::Research {
                        elapsed_seconds, ..
                    } => *elapsed_seconds,
                };
                if !(building.is_complete() && elapsed.is_finite() && elapsed >= 0.0) {
                    return Err(format!("{} has an invalid job", building.id));
                }
                if let BuildingJob::Research { technology, .. } = job
                    && !queued_research.insert(*technology)
                {
                    return Err("a technology is queued more than once".into());
                }
                match job {
                    BuildingJob::Produce { product, .. }
                        if !building.kind.products().contains(product) =>
                    {
                        return Err(format!("{} has an unavailable product", building.id));
                    }
                    BuildingJob::Research { technology, .. }
                        if !building.researches.contains(technology)
                            || self.researched_technologies.contains(technology) =>
                    {
                        return Err(format!("{} has invalid research", building.id));
                    }
                    _ => {}
                }
            }
        }
        for unit in &self.units {
            if let Some(step) = unit.step
                && (!unit.cell.touches(step.to) || !(0.0..1.0).contains(&step.progress))
            {
                return Err(format!("{} has an invalid step", unit.id));
            }
            if let Some(cargo) = &unit.cargo
                && !(cargo.amount > 0.0 && cargo.amount <= VILLAGER_CARRY_CAPACITY + 1e-9)
            {
                return Err(format!("{} carries an invalid load", unit.id));
            }
            match &unit.action {
                UnitAction::Gather { .. }
                | UnitAction::Build { .. }
                | UnitAction::ExploreBuild { .. }
                | UnitAction::Cultivate { .. }
                    if unit.kind != UnitKind::Villager =>
                {
                    return Err(format!("{} is not a worker", unit.id));
                }
                UnitAction::Gather { resource_id, .. }
                    if !self.resources.iter().any(|r| &r.id == resource_id) =>
                {
                    return Err(format!("{} gathers a missing resource", unit.id));
                }
                UnitAction::Cultivate { resource_id }
                    if !self.resources.iter().any(|r| {
                        &r.id == resource_id && r.field.as_ref().is_some_and(|f| f.work.is_some())
                    }) =>
                {
                    return Err(format!("{} cultivates no unfinished field", unit.id));
                }
                UnitAction::ExploreBuild { origin, kind }
                    if origin.column > self.columns() - kind.size().0
                        || origin.row > self.rows() - kind.size().1 =>
                {
                    return Err(format!("{} explores an invalid build site", unit.id));
                }
                UnitAction::Build { building_id }
                    if !self
                        .buildings
                        .iter()
                        .any(|b| &b.id == building_id && !b.is_complete()) =>
                {
                    return Err(format!("{} builds a missing foundation", unit.id));
                }
                UnitAction::Deposit { building_id }
                    if unit.cargo.is_none()
                        || !self
                            .buildings
                            .iter()
                            .any(|b| &b.id == building_id && b.is_complete()) =>
                {
                    return Err(format!("{} deposits nothing or nowhere", unit.id));
                }
                _ => {}
            }
        }
        self.validate_ships()?;
        self.try_occupancy().map(|_| ())
    }

    /// Cells claimed by anything at all, for tests and diagnostics.
    #[cfg(test)]
    pub(super) fn claimed_cells(&self) -> BTreeSet<CellCoordinate> {
        let occupancy = self.occupancy();
        self.terrain
            .iter()
            .map(|cell| cell.coordinate())
            .filter(|cell| occupancy.claim(*cell).is_some())
            .collect()
    }
}

fn unique_ids<'a>(ids: impl Iterator<Item = &'a str>, kind: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(format!("duplicate {kind} id {id}"));
        }
    }
    Ok(())
}
