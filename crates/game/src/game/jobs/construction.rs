//! Shared construction work on existing foundations.
use crate::game::*;

impl GameWorld {
    pub(in crate::game) fn tick_build(&mut self, unit: usize, building_id: &str, dt: f64) {
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
