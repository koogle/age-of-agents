//! Ordered building tasks. Inputs are paid at submission; each active job
//! completes once, or waits at the building when a unit has no free spawn cell.
use super::*;

impl ResourceKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wood => "wood",
            Self::Food => "food",
            Self::Stone => "stone",
            Self::Gold => "gold",
            Self::Iron => "iron",
            Self::Coal => "coal",
            Self::Clay => "clay",
            Self::Fiber => "fiber",
            Self::Timber => "timber",
            Self::Steel => "steel",
            Self::Bricks => "bricks",
            Self::Cloth => "cloth",
            Self::Rations => "rations",
        }
    }
}

impl UnitKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Villager => "Villager",
            Self::Guard => "Guard",
            Self::Archer => "Archer",
            Self::Healer => "Healer",
            Self::SiegeCart => "Siege cart",
        }
    }
}

impl BuildingKind {
    pub const fn products(self) -> &'static [ProductKind] {
        match self {
            Self::Dock => &[ProductKind::TransportShip],
            Self::TownCenter => &[ProductKind::Villager],
            Self::LumberMill => &[ProductKind::Timber],
            Self::Smelter => &[ProductKind::Steel],
            Self::Kiln => &[ProductKind::Bricks],
            Self::Weaver => &[ProductKind::Cloth],
            Self::Kitchen => &[ProductKind::Rations],
            Self::Barracks => &[ProductKind::Guard],
            Self::Range => &[ProductKind::Archer],
            Self::Workshop => &[ProductKind::SiegeCart],
            Self::Infirmary => &[ProductKind::Healer],
            _ => &[],
        }
    }
}

impl ProductKind {
    pub const fn unit_kind(self) -> Option<UnitKind> {
        match self {
            Self::Villager => Some(UnitKind::Villager),
            Self::Guard => Some(UnitKind::Guard),
            Self::Archer => Some(UnitKind::Archer),
            Self::Healer => Some(UnitKind::Healer),
            Self::SiegeCart => Some(UnitKind::SiegeCart),
            _ => None,
        }
    }

    pub const fn output(self) -> Option<(ResourceKind, f64)> {
        match self {
            Self::Timber => Some((ResourceKind::Timber, 5.0)),
            Self::Steel => Some((ResourceKind::Steel, 5.0)),
            Self::Bricks => Some((ResourceKind::Bricks, 5.0)),
            Self::Cloth => Some((ResourceKind::Cloth, 5.0)),
            Self::Rations => Some((ResourceKind::Rations, 5.0)),
            _ => None,
        }
    }

    pub const fn cost(self) -> &'static [(ResourceKind, f64)] {
        match self {
            Self::TransportShip => &FIRST_TRANSPORT_COST,
            Self::Villager => &[(ResourceKind::Food, VILLAGER_FOOD_COST)],
            Self::Guard => &[(ResourceKind::Food, 40.0), (ResourceKind::Steel, 2.0)],
            Self::Archer => &[(ResourceKind::Food, 40.0), (ResourceKind::Timber, 5.0)],
            Self::Healer => &[
                (ResourceKind::Food, 40.0),
                (ResourceKind::Cloth, 5.0),
                (ResourceKind::Rations, 5.0),
            ],
            Self::SiegeCart => &[(ResourceKind::Timber, 10.0), (ResourceKind::Steel, 10.0)],
            Self::Timber => &[(ResourceKind::Wood, 10.0)],
            Self::Steel => &[(ResourceKind::Iron, 5.0), (ResourceKind::Coal, 5.0)],
            Self::Bricks => &[(ResourceKind::Clay, 10.0), (ResourceKind::Wood, 5.0)],
            Self::Cloth => &[(ResourceKind::Fiber, 10.0)],
            Self::Rations => &[(ResourceKind::Food, 10.0)],
        }
    }

    pub const fn seconds(self) -> f64 {
        match self {
            Self::TransportShip => 20.0,
            Self::Villager => VILLAGER_PRODUCTION_SECONDS,
            Self::SiegeCart => 12.0,
            Self::Steel => 10.0,
            _ => 8.0,
        }
    }
}

impl GameWorld {
    pub(super) fn cancel_queued_job(
        &mut self,
        building_id: &str,
        queue_id: u64,
    ) -> Result<(), CommandError> {
        let building = self
            .buildings
            .iter_mut()
            .find(|b| b.id == building_id)
            .ok_or(CommandError::BuildingNotFound)?;
        let index = building
            .queue
            .iter()
            .position(|entry| entry.id == queue_id)
            .ok_or(CommandError::QueuedJobNotFound)?;
        let entry = building.queue.remove(index);
        match entry.job {
            BuildingJob::Produce { product, .. } => {
                for &(kind, amount) in product.cost() {
                    self.stockpile.add(kind, amount);
                }
            }
            BuildingJob::Research { .. } => {
                self.stockpile.food += RESEARCH_FOOD_COST;
                self.stockpile.wood += RESEARCH_WOOD_COST;
            }
        }
        Ok(())
    }

    /// A nearby working farm or mining camp improves matching gathering by 25%.
    /// Multiple buildings do not stack, and foundations confer no benefit.
    pub(super) fn extraction_multiplier(&self, kind: ResourceKind, cell: CellCoordinate) -> f64 {
        if self.buildings.iter().any(|b| {
            b.is_complete()
                && b.kind.accepts(kind)
                && matches!(b.kind, BuildingKind::Farm | BuildingKind::MiningCamp)
                && b.footprint()
                    .cells()
                    .any(|c| c.column.abs_diff(cell.column).max(c.row.abs_diff(cell.row)) <= 6)
        }) {
            1.25
        } else {
            1.0
        }
    }

    pub(super) fn tick_building_job(&mut self, index: usize, dt: f64) {
        let Some(job) = self.buildings[index].job.clone() else {
            return;
        };
        match job {
            BuildingJob::Produce {
                product,
                elapsed_seconds,
            } => {
                let elapsed_seconds = (elapsed_seconds + dt).min(product.seconds());
                if elapsed_seconds + f64::EPSILON < product.seconds() {
                    self.buildings[index].job = Some(BuildingJob::Produce {
                        product,
                        elapsed_seconds,
                    });
                    return;
                }
                if product == ProductKind::TransportShip {
                    if !self.spawn_ship(index) {
                        self.buildings[index].job = Some(BuildingJob::Produce {
                            product,
                            elapsed_seconds,
                        });
                        return;
                    }
                } else if let Some(kind) = product.unit_kind() {
                    let Some(cell) = self.spawn_cell(index) else {
                        self.buildings[index].job = Some(BuildingJob::Produce {
                            product,
                            elapsed_seconds,
                        });
                        return;
                    };
                    self.units.push(Unit {
                        id: self.next_unit_name(),
                        kind,
                        cell,
                        step: None,
                        action: UnitAction::Idle,
                        cargo: None,
                    });
                    self.next_unit_id += 1;
                } else if let Some((kind, amount)) = product.output() {
                    self.stockpile.add(kind, amount);
                }
            }
            BuildingJob::Research {
                technology,
                elapsed_seconds,
            } => {
                let elapsed_seconds = elapsed_seconds + dt;
                if elapsed_seconds + f64::EPSILON < RESEARCH_SECONDS {
                    self.buildings[index].job = Some(BuildingJob::Research {
                        technology,
                        elapsed_seconds,
                    });
                    return;
                }
                self.researched_technologies.push(technology);
                self.researched_technologies.sort_unstable();
            }
        }
        let building = &mut self.buildings[index];
        building.job = if building.queue.is_empty() {
            None
        } else {
            Some(building.queue.remove(0).job)
        };
    }
}
