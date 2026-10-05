//! A fogged build order explores first; only a visible, valid site gets a foundation.
use super::*;

impl GameWorld {
    pub(super) fn order_build(
        &mut self,
        unit_id: &str,
        origin: CellCoordinate,
        kind: BuildingKind,
    ) -> Result<(), CommandError> {
        let unit = self.ordered_unit(unit_id)?;
        if self.units[unit].kind != UnitKind::Villager {
            return Err(CommandError::VillagerRequired);
        }
        if !self.building_available(kind) {
            return Err(CommandError::NotBuildable);
        }
        let (columns, rows) = kind.size();
        let footprint = Footprint {
            origin,
            columns,
            rows,
        };
        if origin.column > WORLD_COLUMNS - columns || origin.row > WORLD_ROWS - rows {
            return Err(CommandError::InvalidBuildSite);
        }
        for &(resource, amount) in kind.cost() {
            if self.stockpile.amount(resource) < amount {
                return Err(match resource {
                    ResourceKind::Stone => CommandError::InsufficientStone,
                    ResourceKind::Wood => CommandError::InsufficientWood,
                    kind => CommandError::InsufficientResources(kind),
                });
            }
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
        for &(resource, amount) in kind.cost() {
            self.stockpile.add(resource, -amount);
        }
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
        if self.drop_off_before_building(unit, dt) {
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
            if self.apply_command(command).is_err() {
                self.units[unit].action = UnitAction::Idle;
            }
        } else if self.travel(unit, Goal::Beside(footprint), dt) == Travel::Unreachable {
            self.units[unit].action = UnitAction::Idle;
        }
    }

    pub(super) fn tick_build(&mut self, unit: usize, building_id: &str, dt: f64) {
        let Some(building) = self
            .buildings
            .iter()
            .position(|building| building.id == building_id && !building.is_complete())
        else {
            // Another builder finished it.
            self.units[unit].action = UnitAction::Idle;
            return;
        };
        if self.drop_off_before_building(unit, dt) {
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
        let work = self.buildings[building].construction.unwrap_or(0.0) + remaining;
        if work + f64::EPSILON < self.buildings[building].kind.build_seconds() {
            self.buildings[building].construction = Some(work);
        } else {
            // Completion releases every builder at once, so no unit is ever
            // left working on a building that is no longer a foundation.
            self.buildings[building].construction = None;
            for other in &mut self.units {
                if matches!(&other.action, UnitAction::Build { building_id: id } if id == building_id)
                {
                    other.action = UnitAction::Idle;
                }
            }
        }
    }
}
