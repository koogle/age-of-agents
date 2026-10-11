//! A fogged build order explores first; only a visible, valid site gets a foundation.
use super::*;

impl GameWorld {
    pub(super) fn order_build(
        &mut self,
        unit_id: &str,
        origin: CellCoordinate,
        kind: BuildingKind,
    ) -> Result<(), CommandError> {
        let unit = self.ordered_villager(unit_id)?;
        if !self.building_available(kind) {
            return Err(CommandError::NotBuildable);
        }
        let (columns, rows) = kind.size();
        let footprint = Footprint {
            origin,
            columns,
            rows,
        };
        if origin.column > self.columns() - columns || origin.row > self.rows() - rows {
            return Err(CommandError::InvalidBuildSite);
        }
        let available = self.available_at(origin);
        if let Some(resource) = available.missing_resource(kind.cost()) {
            return Err(match resource {
                ResourceKind::Stone => CommandError::InsufficientStone,
                ResourceKind::Wood => CommandError::InsufficientWood,
                kind => CommandError::InsufficientResources(kind),
            });
        }
        if !self.build_site_visible(footprint) {
            self.units[unit].action = UnitAction::ExploreBuild { origin, kind };
            return Ok(());
        }
        if !self.footprint_is_free(footprint) {
            return Err(CommandError::InvalidBuildSite);
        }
        if kind.needs_coast() && !self.touches_sea(footprint) {
            return Err(CommandError::NeedsCoast);
        }
        if !self.placement_preserves_routes(footprint) {
            return Err(CommandError::TargetUnreachable);
        }
        // Validate reachability against the world as it will be, with
        // the foundation in place; roll back if the builder is cut off.
        let id = self.next_building_name();
        self.buildings.push(building(kind, &id, origin, Some(0.0)));
        if !self.can_reach_beside(unit, footprint) {
            self.buildings.pop();
            return Err(CommandError::TargetUnreachable);
        }
        self.next_building_id += 1;
        self.spend_at(origin, kind.cost())?;
        self.units[unit].action = UnitAction::Build { building_id: id };
        Ok(())
    }

    fn build_site_visible(&self, footprint: Footprint) -> bool {
        let visible = self.visible_cells();
        footprint.cells().all(|cell| visible.contains(&cell))
    }

    pub(super) fn tick_explore_build(
        &mut self,
        unit: usize,
        origin: CellCoordinate,
        kind: BuildingKind,
        dt: f64,
    ) {
        if self.unload_before_work(unit, dt) {
            return;
        }
        let (columns, rows) = kind.size();
        let footprint = Footprint {
            origin,
            columns,
            rows,
        };
        if self.build_site_visible(footprint) {
            // Reuse atomic command validation: terrain, occupancy, routes and
            // affordability may all have changed while the villager walked.
            let command = Command::Build {
                unit_id: self.units[unit].id.clone(),
                origin,
                kind,
            };
            if let Err(error) = self.apply_command(command) {
                self.give_up(unit, &error.to_string());
            }
        } else if self.travel(unit, Goal::Beside(footprint), dt) == Travel::Unreachable {
            self.give_up(unit, &CommandError::TargetUnreachable.to_string());
        }
    }

    /// Abandon an order the world no longer allows, saying why.
    fn give_up(&mut self, unit: usize, reason: &str) {
        self.units[unit].action = UnitAction::Idle;
        self.units[unit].notice = Some(UnitNotice {
            message: format!("Cannot build here. {reason}"),
            tick: self.tick,
        });
    }

    pub(super) fn tick_build(&mut self, unit: usize, building_id: &str, dt: f64) {
        let Some(building) = self
            .buildings
            .iter()
            .position(|building| building.id == building_id && building.needs_work())
        else {
            // Another builder finished it.
            self.units[unit].action = UnitAction::Idle;
            return;
        };
        if self.unload_before_work(unit, dt) {
            return;
        }
        let remaining =
            match self.travel(unit, Goal::Beside(self.buildings[building].footprint()), dt) {
                Travel::EnRoute => return,
                Travel::Unreachable => {
                    // The foundation stays; any villager can resume it with Construct.
                    self.units[unit].action = UnitAction::Idle;
                    return;
                }
                Travel::Arrived { remaining } => remaining,
            };
        if self.buildings[building].is_complete() {
            let damage = self.buildings[building].damage - REPAIR_PER_SECOND * remaining;
            self.buildings[building].damage = damage.max(0.0);
            if damage <= 0.0 {
                self.release_builders(building_id);
            }
            return;
        }
        let work = self.buildings[building].construction.unwrap_or(0.0) + remaining;
        if work + f64::EPSILON < self.buildings[building].kind.build_seconds() {
            self.buildings[building].construction = Some(work);
        } else {
            // Completion releases every builder at once, so no unit is ever
            // left working on a building that needs no more work; raiders'
            // damage to the foundation keeps them on as repairers.
            self.buildings[building].construction = None;
            if self.buildings[building].damage == 0.0 {
                self.release_builders(building_id);
            }
        }
    }

    fn release_builders(&mut self, building_id: &str) {
        for other in &mut self.units {
            if matches!(&other.action, UnitAction::Build { building_id: id } if id == building_id) {
                other.action = UnitAction::Idle;
            }
        }
    }
}

pub(super) fn building(
    kind: BuildingKind,
    id: &str,
    origin: CellCoordinate,
    construction: Option<f64>,
) -> Building {
    Building {
        id: id.into(),
        kind,
        origin,
        construction,
        job: None,
        queue: Vec::new(),
        next_queue_id: 0,
        damage: 0.0,
    }
}
