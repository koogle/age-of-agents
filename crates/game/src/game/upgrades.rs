//! One paid masonry upgrade per building, using its existing task queue and plot.
use super::*;

pub const BUILDING_UPGRADE_SECONDS: f64 = 20.0;

impl BuildingKind {
    pub const fn upgrade_cost(self) -> &'static [(ResourceKind, f64)] {
        use ResourceKind::{Bricks, Timber};
        match self {
            Self::TownCenter | Self::Monument => &[(Bricks, 30.0), (Timber, 15.0)],
            Self::Dock
            | Self::LumberMill
            | Self::Smelter
            | Self::Barracks
            | Self::Range
            | Self::Workshop => &[(Bricks, 20.0), (Timber, 10.0)],
            _ => &[(Bricks, 10.0), (Timber, 5.0)],
        }
    }
}

impl Building {
    pub fn upgrade_pending(&self) -> bool {
        self.jobs()
            .any(|job| matches!(job, BuildingJob::Upgrade { .. }))
    }
}

impl GameWorld {
    pub fn masonry_upgrades_available(&self) -> bool {
        self.economy_rules == EconomyRules::Unrestricted || self.discovered(ResourceKind::Clay)
    }

    pub(super) fn upgrade_building(&mut self, building_id: &str) -> Result<(), CommandError> {
        let index = self.building_index_with_queue_space(building_id)?;
        let building = &self.buildings[index];
        if !self.masonry_upgrades_available() {
            return Err(CommandError::UpgradeUnavailable);
        }
        if building.masonry || building.upgrade_pending() {
            return Err(CommandError::BuildingAlreadyUpgraded);
        }
        let cost = building.kind.upgrade_cost();
        self.spend_at(building.origin, cost)?;
        self.buildings[index].enqueue(BuildingJob::Upgrade {
            elapsed_seconds: 0.0,
        });
        Ok(())
    }
}
