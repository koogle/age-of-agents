//! Resource discovery gates new games. Missing rules in old saves retain the
//! unrestricted economy; loading a save never deletes resources or buildings.
use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomyRules {
    #[default]
    Unrestricted,
    IslandProgression,
}

pub const STARTER_BUILDINGS: [BuildingKind; 6] = [
    BuildingKind::TownCenter,
    BuildingKind::House,
    BuildingKind::Granary,
    BuildingKind::Farm,
    BuildingKind::LumberMill,
    BuildingKind::Dock,
];
pub const STARTER_RESOURCES: [ResourceKind; 3] =
    [ResourceKind::Wood, ResourceKind::Food, ResourceKind::Stone];
/// First transport recipe; no metal or cloth prerequisite.
pub const FIRST_TRANSPORT_COST: [(ResourceKind, f64); 2] =
    [(ResourceKind::Wood, 60.0), (ResourceKind::Timber, 20.0)];
/// Reachable raw supplies for settlement, processing, research and departure,
/// before the generator adds its 50% safety margin. Timber is made from wood.
pub const STARTER_RESOURCE_BUDGET: [(ResourceKind, f64); 3] = [
    (ResourceKind::Wood, 400.0),
    (ResourceKind::Food, 200.0),
    (ResourceKind::Stone, 80.0),
];

impl GameWorld {
    fn discovered(&self, kind: ResourceKind) -> bool {
        self.resources
            .iter()
            .any(|r| r.kind == kind && self.explored_cells.binary_search(&r.cell).is_ok())
            || self.islands.iter().any(|i| {
                i.resources
                    .iter()
                    .any(|r| r.kind == kind && i.explored_cells.binary_search(&r.cell).is_ok())
            })
    }

    pub fn building_available(&self, kind: BuildingKind) -> bool {
        if self.economy_rules == EconomyRules::Unrestricted || STARTER_BUILDINGS.contains(&kind) {
            return true;
        }
        use BuildingKind::*;
        use ResourceKind as R;
        match kind {
            MiningCamp => [R::Iron, R::Coal, R::Gold]
                .iter()
                .any(|&r| self.discovered(r)),
            Smelter | Barracks | Watchtower => self.discovered(R::Iron) && self.discovered(R::Coal),
            Kiln => self.discovered(R::Clay),
            Weaver | Range => self.discovered(R::Fiber),
            Workshop => [R::Iron, R::Coal, R::Clay]
                .iter()
                .all(|&r| self.discovered(r)),
            Monument => [R::Iron, R::Coal, R::Clay, R::Fiber, R::Gold]
                .iter()
                .all(|&r| self.discovered(r)),
            // Provisioning and healing need a playable purpose before introducing rations.
            Kitchen | Infirmary => false,
            _ => false,
        }
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
