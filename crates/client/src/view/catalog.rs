//! Fixed catalog atlas mapping shared by native and browser rendering.
use std::collections::HashMap;

use aoa_game::{BuildingKind, UnitKind};
use serde::Deserialize;

use super::{BuildingSheet, VillagerSheet};

pub(super) const SHEET_UNITS: usize = 10;

#[derive(Deserialize)]
pub(super) struct UnitSheet {
    pub size: [f32; 2],
    cell: [f32; 2],
    anchor: [f32; 2],
    pub units: HashMap<String, VillagerSheet>,
}

pub(super) struct Catalog {
    economy: BuildingSheet,
    crafts: BuildingSheet,
    civic: BuildingSheet,
    pub units: UnitSheet,
}

impl Catalog {
    pub fn parse([economy, crafts, civic, units]: [&[u8]; 4]) -> Self {
        let mut units: UnitSheet = serde_json::from_slice(units).expect("units.json");
        for art in units.units.values_mut() {
            art.size = units.size;
            art.cell = units.cell;
            art.anchor = units.anchor;
        }
        Self {
            economy: serde_json::from_slice(economy).expect("buildings_economy.json"),
            crafts: serde_json::from_slice(crafts).expect("buildings_crafts.json"),
            civic: serde_json::from_slice(civic).expect("buildings_civic.json"),
            units,
        }
    }

    pub fn field(&self) -> (usize, &BuildingSheet) {
        (7, &self.economy)
    }

    pub fn unit(&self, kind: UnitKind) -> &VillagerSheet {
        let name = match kind {
            UnitKind::Guard => "guard",
            UnitKind::Archer => "archer",
            UnitKind::Healer => "healer",
            UnitKind::SiegeCart => "siege_cart",
            UnitKind::Villager => unreachable!("villager uses its original sheet"),
        };
        &self.units.units[name]
    }

    pub fn building(&self, kind: BuildingKind) -> (usize, &BuildingSheet, &'static str) {
        match kind {
            BuildingKind::MiningCamp => (7, &self.economy, "mining_camp"),
            BuildingKind::Farm => (7, &self.economy, "farm"),
            BuildingKind::LumberMill => (7, &self.economy, "lumber_mill"),
            BuildingKind::Smelter => (7, &self.economy, "smelter"),
            BuildingKind::Kiln => (8, &self.crafts, "kiln"),
            BuildingKind::Weaver => (8, &self.crafts, "weaver"),
            BuildingKind::Kitchen => (8, &self.crafts, "kitchen"),
            BuildingKind::Barracks => (9, &self.civic, "barracks"),
            BuildingKind::Range => (9, &self.civic, "range"),
            BuildingKind::Workshop => (9, &self.civic, "workshop"),
            BuildingKind::Infirmary => (9, &self.civic, "infirmary"),
            BuildingKind::Monument => (8, &self.crafts, "monument"),
            _ => unreachable!("base building uses its original sheet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_trained_unit_has_idle_walk_and_action_frames_inside_the_atlas() {
        let catalog = Catalog::parse([
            include_bytes!("../../../../assets/sprites/buildings_economy.json"),
            include_bytes!("../../../../assets/sprites/buildings_crafts.json"),
            include_bytes!("../../../../assets/sprites/buildings_civic.json"),
            include_bytes!("../../../../assets/sprites/units.json"),
        ]);
        for kind in [
            UnitKind::Guard,
            UnitKind::Archer,
            UnitKind::Healer,
            UnitKind::SiegeCart,
        ] {
            let art = catalog.unit(kind);
            assert!(art.figure_height > 0.0);
            assert_eq!(art.cell, catalog.units.cell);
            assert_eq!(art.anchor, catalog.units.anchor);
            for action in ["idle", "walk", "action"] {
                let facings = &art.animations[action];
                assert!(!facings["front"].is_empty());
                for frames in facings.values() {
                    for [x, y, w, h] in frames {
                        assert!(*x >= 0.0 && *y >= 0.0 && *w > 0.0 && *h > 0.0);
                        assert!(x + w <= catalog.units.size[0] && y + h <= catalog.units.size[1]);
                    }
                }
            }
        }
    }
}
