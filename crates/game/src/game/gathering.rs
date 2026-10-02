use super::movement::{Goal, Travel};
use super::*;

/// How far (in cells, from the exhausted node) a gatherer looks for the next
/// node of the same kind before going idle.
pub const NEXT_RESOURCE_RADIUS: u32 = 10;

impl GameWorld {
    pub(super) fn tick_gather(
        &mut self,
        unit_index: usize,
        resource_id: String,
        phase: GatherPhase,
        dt: f64,
    ) {
        match phase {
            GatherPhase::ToResource => self.tick_to_resource(unit_index, resource_id, dt),
            GatherPhase::Gathering => self.tick_at_resource(unit_index, resource_id, dt),
            GatherPhase::Returning => self.tick_returning(unit_index, resource_id, dt),
            GatherPhase::Depositing => self.tick_depositing(unit_index, resource_id),
        }
    }

    fn tick_to_resource(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        let Some(resource_index) = self
            .resources
            .iter()
            .position(|resource| resource.id == resource_id)
        else {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        };
        if self.resources[resource_index].amount <= 0.0 {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        }
        // A full basket, or goods of another kind, are dropped off before
        // heading out, rather than walking to the node and turning back.
        let kind = self.resources[resource_index].kind;
        if self.units[unit_index].cargo.as_ref().is_some_and(|cargo| {
            cargo.kind != kind || cargo.amount + f64::EPSILON >= VILLAGER_CARRY_CAPACITY
        }) {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }

        let footprint = self.resources[resource_index].footprint();
        match self.travel(unit_index, Goal::Beside(footprint), dt) {
            Travel::EnRoute => {}
            Travel::Unreachable => self.finish_or_return_with_cargo(unit_index, resource_id),
            Travel::Arrived { remaining } => {
                self.set_gather_phase(unit_index, resource_id.clone(), GatherPhase::Gathering);
                if remaining > 0.0 {
                    self.tick_at_resource(unit_index, resource_id, remaining);
                }
            }
        }
    }

    fn tick_at_resource(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        let Some(resource_index) = self
            .resources
            .iter()
            .position(|resource| resource.id == resource_id)
        else {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        };
        if self.resources[resource_index].amount <= f64::EPSILON {
            self.resources[resource_index].amount = 0.0;
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        }

        let kind = self.resources[resource_index].kind;
        if !self.is_beside(unit_index, self.resources[resource_index].footprint()) {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::ToResource);
            return;
        }
        if self.units[unit_index]
            .cargo
            .as_ref()
            .is_some_and(|cargo| cargo.kind != kind)
        {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }
        let carried = self.units[unit_index]
            .cargo
            .as_ref()
            .map_or(0.0, |cargo| cargo.amount);
        let capacity_left = (VILLAGER_CARRY_CAPACITY - carried).max(0.0);
        let gathered = (GATHER_RATE * self.gather_multiplier(kind) * dt)
            .min(self.resources[resource_index].amount)
            .min(capacity_left);
        self.resources[resource_index].amount -= gathered;
        if gathered > 0.0 {
            let cargo = self.units[unit_index]
                .cargo
                .get_or_insert(CarriedResource { kind, amount: 0.0 });
            cargo.amount += gathered;
        }

        if self.resources[resource_index].amount <= f64::EPSILON {
            self.resources[resource_index].amount = 0.0;
        }
        let full = self.units[unit_index]
            .cargo
            .as_ref()
            .is_some_and(|cargo| cargo.amount + f64::EPSILON >= VILLAGER_CARRY_CAPACITY);
        if full {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
        } else if self.resources[resource_index].amount == 0.0 {
            // Keep filling the basket at the next node when there is one.
            match self.next_resource(unit_index, &resource_id) {
                Some(next) => self.set_gather_phase(unit_index, next, GatherPhase::ToResource),
                None => self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning),
            }
        }
    }

    fn tick_returning(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        if self.units[unit_index].cargo.is_none() {
            self.resume_or_finish_gather(unit_index, resource_id);
            return;
        }
        // With no reachable drop site the load is kept until one exists.
        let Some(site) = self.nearest_drop_site(unit_index) else {
            return;
        };
        if let Travel::Arrived { .. } = self.travel(unit_index, Goal::Beside(site), dt) {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Depositing);
        }
    }

    fn tick_depositing(&mut self, unit_index: usize, resource_id: String) {
        let cargo = self.units[unit_index]
            .cargo
            .as_ref()
            .map(|cargo| cargo.kind);
        let beside_drop_site = self.buildings.iter().any(|building| {
            building.is_complete()
                && cargo.is_none_or(|kind| building.kind.accepts(kind))
                && self.is_beside(unit_index, building.footprint())
        });
        if !beside_drop_site {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }
        if let Some(cargo) = self.units[unit_index].cargo.take() {
            self.stockpile.add(cargo.kind, cargo.amount);
        }
        self.resume_or_finish_gather(unit_index, resource_id);
    }

    fn finish_or_return_with_cargo(&mut self, unit_index: usize, resource_id: String) {
        if self.units[unit_index].cargo.is_some() {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
        } else {
            self.continue_or_idle(unit_index, &resource_id);
        }
    }

    fn resume_or_finish_gather(&mut self, unit_index: usize, resource_id: String) {
        let resource_remains = self
            .resources
            .iter()
            .any(|resource| resource.id == resource_id && resource.amount > f64::EPSILON);
        if resource_remains {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::ToResource);
        } else {
            self.continue_or_idle(unit_index, &resource_id);
        }
    }

    /// Moves on to the next node of the same kind near an exhausted one, or idles.
    fn continue_or_idle(&mut self, unit_index: usize, resource_id: &str) {
        match self.next_resource(unit_index, resource_id) {
            Some(next) => self.set_gather_phase(unit_index, next, GatherPhase::ToResource),
            None => self.units[unit_index].action = UnitAction::Idle,
        }
    }

    /// The nearest live node of the same kind within `NEXT_RESOURCE_RADIUS`
    /// cells of the given node that the unit can reach; ties go to the lower id.
    fn next_resource(&self, unit_index: usize, resource_id: &str) -> Option<String> {
        let finished = self.resources.iter().find(|r| r.id == resource_id)?;
        let mut candidates: Vec<(u32, &ResourceNode)> = self
            .resources
            .iter()
            .filter(|r| r.kind == finished.kind && r.id != resource_id && r.amount > f64::EPSILON)
            .filter_map(|r| {
                let dc = u32::from(r.cell.column.abs_diff(finished.cell.column));
                let dr = u32::from(r.cell.row.abs_diff(finished.cell.row));
                let distance = dc * dc + dr * dr;
                (distance <= NEXT_RESOURCE_RADIUS * NEXT_RESOURCE_RADIUS).then_some((distance, r))
            })
            .collect();
        candidates.sort_by(|(da, a), (db, b)| da.cmp(db).then_with(|| a.id.cmp(&b.id)));
        candidates
            .into_iter()
            .find(|(_, r)| self.can_reach_beside(unit_index, r.footprint()))
            .map(|(_, r)| r.id.clone())
    }

    /// Walks a carrier to the building it was sent to and unloads there.
    pub(super) fn tick_deposit(&mut self, unit_index: usize, building_id: &str, dt: f64) {
        let site = self
            .buildings
            .iter()
            .find(|building| building.id == building_id && building.is_complete())
            .map(|building| (building.footprint(), building.kind));
        let (Some((footprint, kind)), Some(cargo)) = (site, self.units[unit_index].cargo.clone())
        else {
            self.units[unit_index].action = UnitAction::Idle;
            return;
        };
        if !kind.accepts(cargo.kind) {
            self.units[unit_index].action = UnitAction::Idle;
            return;
        }
        match self.travel(unit_index, Goal::Beside(footprint), dt) {
            Travel::EnRoute => {}
            Travel::Unreachable => self.units[unit_index].action = UnitAction::Idle,
            Travel::Arrived { .. } => {
                self.units[unit_index].cargo = None;
                self.stockpile.add(cargo.kind, cargo.amount);
                self.units[unit_index].action = UnitAction::Idle;
            }
        }
    }

    /// A builder carrying goods takes them to a drop site before building.
    /// Returns whether the unit is still on that errand. With no reachable
    /// drop site the goods are simply carried along.
    pub(super) fn drop_off_before_building(&mut self, unit_index: usize, dt: f64) -> bool {
        if self.units[unit_index].cargo.is_none() {
            return false;
        }
        let Some(site) = self.nearest_drop_site(unit_index) else {
            return false;
        };
        match self.travel(unit_index, Goal::Beside(site), dt) {
            Travel::EnRoute => true,
            Travel::Unreachable => false,
            Travel::Arrived { .. } => {
                if let Some(cargo) = self.units[unit_index].cargo.take() {
                    self.stockpile.add(cargo.kind, cargo.amount);
                }
                true
            }
        }
    }

    fn set_gather_phase(&mut self, unit_index: usize, resource_id: String, phase: GatherPhase) {
        self.units[unit_index].action = UnitAction::Gather { resource_id, phase };
    }
}
