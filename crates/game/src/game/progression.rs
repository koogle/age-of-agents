//! Resource discovery gates the starter economy.
use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomyRules {
    #[default]
    Unrestricted,
    IslandProgression,
}

pub const STARTER_BUILDINGS: [BuildingKind; 14] = [
    BuildingKind::TownCenter,
    BuildingKind::House,
    BuildingKind::Granary,
    BuildingKind::Farm,
    BuildingKind::LumberMill,
    BuildingKind::Dock,
    BuildingKind::Watchtower,
    BuildingKind::MiningCamp,
    BuildingKind::Smelter,
    BuildingKind::Kiln,
    BuildingKind::Weaver,
    BuildingKind::Kitchen,
    BuildingKind::Barracks,
    BuildingKind::Range,
];
pub const STARTER_RESOURCES: [ResourceKind; 4] = [
    ResourceKind::Wood,
    ResourceKind::Food,
    ResourceKind::Stone,
    ResourceKind::Water,
];
/// First transport recipe; no metal or cloth prerequisite.
pub const FIRST_TRANSPORT_COST: [(ResourceKind, f64); 2] =
    [(ResourceKind::Wood, 60.0), (ResourceKind::Timber, 20.0)];
/// Reachable raw supplies for settlement, processing, research and departure,
/// before the generator adds its 50% safety margin. Timber is made from wood.
pub const STARTER_RESOURCE_BUDGET: [(ResourceKind, f64); 4] = [
    (ResourceKind::Wood, 400.0),
    (ResourceKind::Food, 200.0),
    (ResourceKind::Stone, 80.0),
    (ResourceKind::Water, 10.0),
];

impl GameWorld {
    fn discovered(&self, kind: ResourceKind) -> bool {
        self.resources
            .iter()
            .any(|r| r.kind == kind && self.explored_cells.binary_search(&r.cell).is_ok())
    }

    /// Unlock construction from discoverable cost inputs, independently of current
    /// stock and the recipes a completed building can run.
    pub fn building_available(&self, kind: BuildingKind) -> bool {
        self.economy_rules == EconomyRules::Unrestricted
            || kind.cost().iter().all(|&(resource, _)| {
                use ResourceKind::*;
                match resource {
                    Wood | Food | Stone | Timber | Rations => true,
                    Steel => self.discovered(Iron) && self.discovered(Coal),
                    Bricks => self.discovered(Clay),
                    Cloth => self.discovered(Fiber),
                    raw => self.discovered(raw),
                }
            })
    }

    pub fn technology_available(&self, technology: TechnologyKind) -> bool {
        self.economy_rules == EconomyRules::Unrestricted
            || match technology {
                TechnologyKind::Forestry
                | TechnologyKind::Agriculture
                | TechnologyKind::Masonry => true,
                TechnologyKind::Mining => {
                    self.discovered(ResourceKind::Iron) || self.discovered(ResourceKind::Gold)
                }
                TechnologyKind::Textiles => self.discovered(ResourceKind::Fiber),
            }
    }

    pub fn available_buildings(&self) -> Vec<BuildingKind> {
        BUILDABLE
            .into_iter()
            .filter(|&kind| self.building_available(kind))
            .collect()
    }
}
