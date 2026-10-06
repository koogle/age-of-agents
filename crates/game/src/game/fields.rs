//! Player-built food plots. Preparation reserves materials once; only labour
//! creates a harvest. Exhausted plots retain their footprint until replenished.
use super::*;

pub const FIELD_COST: &[(ResourceKind, f64)] = &[
    (ResourceKind::Wood, 10.0),
    (ResourceKind::Stone, 5.0),
    (ResourceKind::Water, 10.0),
];
pub const FIELD_FOOD: f64 = 120.0;
pub const FIELD_WORK_SECONDS: f64 = 12.0;
pub const GRANARY_FIELD_RADIUS: u16 = 6;
pub const GRANARY_FIELD_YIELD_MULTIPLIER: f64 = 1.5;

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

    fn afford_field(&self, origin: CellCoordinate) -> Result<(), CommandError> {
        let available = self.available_at(origin);
        for &(kind, amount) in FIELD_COST {
            if available.amount(kind) < amount {
                return Err(CommandError::InsufficientResources(kind));
            }
        }
        Ok(())
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
        if origin.column > self.columns() - footprint.columns
            || origin.row > self.rows() - footprint.rows
            || !self.footprint_is_free(footprint)
        {
            return Err(CommandError::InvalidBuildSite);
        }
        self.afford_field(origin)?;
        // A plot blocks its cells even after harvest. Preserve delivery and
        // escape routes just as we do when placing a building foundation.
        if !self.placement_preserves_routes(footprint) {
            return Err(CommandError::TargetUnreachable);
        }
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
        self.spend_at(origin, FIELD_COST)?;
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
            let origin = resource.cell;
            self.afford_field(origin)?;
            self.spend_at(origin, FIELD_COST)?;
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
        // Fix this harvest's yield once; nearby granaries never refill a live field.
        let footprint = self.resources[index].footprint();
        let boosted = self.buildings.iter().any(|building| {
            building.kind == BuildingKind::Granary
                && building.is_complete()
                && building.footprint().cells().any(|a| {
                    footprint.cells().any(|b| {
                        a.column.abs_diff(b.column).max(a.row.abs_diff(b.row))
                            <= GRANARY_FIELD_RADIUS
                    })
                })
        });
        self.resources[index].capacity = FIELD_FOOD
            * if boosted {
                GRANARY_FIELD_YIELD_MULTIPLIER
            } else {
                1.0
            };
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
