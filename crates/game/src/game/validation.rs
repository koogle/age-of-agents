//! Whole-world invariants checked after commands, ticks and loading persisted state.

use super::spatial::in_bounds;
use super::*;

impl GameWorld {
    /// Checks every structural invariant of the world. Persisted worlds that
    /// fail this are corrupt and must not be loaded.
    pub fn validate(&self) -> Result<(), String> {
        let expected_terrain = (0..WORLD_ROWS)
            .flat_map(|row| (0..WORLD_COLUMNS).map(move |column| CellCoordinate::new(column, row)));
        if !self
            .terrain
            .iter()
            .map(|cell| cell.coordinate())
            .eq(expected_terrain)
        {
            return Err("terrain is not a complete row-major grid".into());
        }
        if !self.explored_cells.iter().all(|cell| in_bounds(*cell))
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
        self.try_occupancy().map(|_| ())
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
