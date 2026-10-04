//! Player-built food plots. Preparation reserves materials once; only labour
//! creates a harvest. Exhausted plots retain their footprint until replenished.
use super::*;

pub const FIELD_COST: &[(ResourceKind, f64)] =
    &[(ResourceKind::Wood, 10.0), (ResourceKind::Stone, 5.0)];
pub const FIELD_FOOD: f64 = 120.0;
pub const FIELD_WORK_SECONDS: f64 = 12.0;

impl GameWorld {
    fn field_worker(&mut self, unit_id: &str) -> Result<usize, CommandError> {
        let unit = self.ordered_unit(unit_id)?;
        if self.units[unit].kind != UnitKind::Villager {
            return Err(CommandError::VillagerRequired);
        }
        if !self
            .buildings
            .iter()
            .any(|b| b.kind == BuildingKind::Farm && b.is_complete())
        {
            return Err(CommandError::FarmRequired);
        }
        Ok(unit)
    }

    fn afford_field(&self) -> Result<(), CommandError> {
        for &(kind, amount) in FIELD_COST {
            if self.stockpile.amount(kind) < amount {
                return Err(CommandError::InsufficientResources(kind));
            }
        }
        Ok(())
    }

    fn pay_for_field(&mut self) {
        for &(kind, amount) in FIELD_COST {
            self.stockpile.add(kind, -amount);
        }
    }

    pub(super) fn plant_field(
        &mut self,
        unit_id: &str,
        origin: CellCoordinate,
    ) -> Result<(), CommandError> {
        let unit = self.field_worker(unit_id)?;
        let footprint = Footprint {
            origin,
            columns: 3,
            rows: 3,
        };
        if origin.column > WORLD_COLUMNS - footprint.columns
            || origin.row > WORLD_ROWS - footprint.rows
            || !self.footprint_is_free(footprint)
        {
            return Err(CommandError::InvalidBuildSite);
        }
        self.afford_field()?;
        let id = format!("field-{}-{}", origin.column, origin.row);
        if self.resources.iter().any(|r| r.id == id) {
            return Err(CommandError::InvalidBuildSite);
        }
        self.resources.push(ResourceNode {
            id: id.clone(),
            kind: ResourceKind::Food,
            cell: origin,
            amount: 0.0,
            capacity: FIELD_FOOD,
            field: Some(FieldState { work: Some(0.0) }),
        });
        if !self.can_reach_beside(unit, footprint) {
            self.resources.pop();
            return Err(CommandError::TargetUnreachable);
        }
        self.pay_for_field();
        self.units[unit].action = UnitAction::Cultivate { resource_id: id };
        Ok(())
    }

    pub(super) fn cultivate(
        &mut self,
        unit_id: &str,
        resource_id: &str,
    ) -> Result<(), CommandError> {
        let unit = self.field_worker(unit_id)?;
        let index = self
            .resources
            .iter()
            .position(|r| r.id == resource_id && r.field.is_some())
            .ok_or(CommandError::ResourceNotFound)?;
        let resource = &self.resources[index];
        if resource.amount > 0.0 {
            return Err(CommandError::FieldNotDepleted);
        }
        if !self.can_reach_beside(unit, resource.footprint()) {
            return Err(CommandError::TargetUnreachable);
        }
        if resource.field.as_ref().unwrap().work.is_none() {
            self.afford_field()?;
            self.pay_for_field();
            self.resources[index].field.as_mut().unwrap().work = Some(0.0);
        }
        self.units[unit].action = UnitAction::Cultivate {
            resource_id: resource_id.into(),
        };
        Ok(())
    }

    pub(super) fn tick_cultivate(&mut self, unit: usize, resource_id: &str, dt: f64) {
        let Some(index) = self.resources.iter().position(|r| {
            r.id == resource_id && r.field.as_ref().is_some_and(|f| f.work.is_some())
        }) else {
            self.units[unit].action = UnitAction::Idle;
            return;
        };
        if self.drop_off_before_building(unit, dt) {
            return;
        }
        let remaining = match self.travel(unit, Goal::Beside(self.resources[index].footprint()), dt)
        {
            Travel::EnRoute => return,
            Travel::Unreachable => {
                self.units[unit].action = UnitAction::Idle;
                return;
            }
            Travel::Arrived { remaining } => remaining,
        };
        let field = self.resources[index].field.as_mut().unwrap();
        let work = field.work.unwrap() + remaining;
        if work + f64::EPSILON < FIELD_WORK_SECONDS {
            field.work = Some(work);
            return;
        }
        field.work = None;
        self.resources[index].amount = self.resources[index].capacity;
        for worker in &mut self.units {
            if matches!(&worker.action, UnitAction::Cultivate { resource_id: id } if id == resource_id)
            {
                worker.action = UnitAction::Gather {
                    resource_id: resource_id.into(),
                    phase: if worker.cargo.is_some() {
                        GatherPhase::Returning
                    } else {
                        GatherPhase::ToResource
                    },
                };
            }
        }
    }
}
