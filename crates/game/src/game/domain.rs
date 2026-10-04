//! Buildings, terrain, research and scenario definitions.

use serde::{Deserialize, Serialize};

use super::{
    BuildingJob, CellCoordinate, Footprint, ProductKind, ROADMAP_RECIPES, ROADMAP_RESOURCES,
    ROADMAP_UNITS, RecipeCatalogEntry, ResourceKind, UnitKind,
};

pub const DEFAULT_SCENARIO_TICK_LIMIT: u64 = 36_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainBiome {
    Meadow,
    Forest,
    Prairie,
    Highland,
    Wetland,
    Scrubland,
    Heath,
    Clayland,
    /// Sand along the open sea.
    Beach,
    /// Sea and lakes: nobody walks, builds or gathers on water.
    Water,
    /// Bare rock and snow on the peaks: too steep to walk or build on.
    Mountain,
    /// Fresh water running from the hills to the sea, crossed only at fords.
    River,
}

impl TerrainBiome {
    /// Whether villagers may stand, walk and build here.
    pub fn is_walkable(self) -> bool {
        !matches!(self, Self::Water | Self::Mountain | Self::River)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TerrainCell {
    pub column: u16,
    pub row: u16,
    pub biome: TerrainBiome,
    /// Land (rivers included) in (0, 1] rising toward the peaks; sea and lakes
    /// in [-1, 0) by depth.
    pub elevation: f32,
}

impl TerrainCell {
    pub(super) fn coordinate(self) -> CellCoordinate {
        CellCoordinate {
            column: self.column,
            row: self.row,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellVisibility {
    Unseen,
    Explored,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SnapshotTerrainCell {
    pub column: u16,
    pub row: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biome: Option<TerrainBiome>,
    /// Only for explored cells, like the biome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f32>,
    pub visibility: CellVisibility,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub kind: BuildingKind,
    /// North-west cell of the footprint.
    pub origin: CellCoordinate,
    /// `Some(seconds of work done)` while this is a foundation; `None` once complete.
    pub construction: Option<f64>,
    pub produces: Vec<ProductKind>,
    pub researches: Vec<TechnologyKind>,
    pub job: Option<BuildingJob>,
    /// Paid tasks waiting behind the active job, in submission order.
    #[serde(default)]
    pub queue: Vec<QueuedBuildingJob>,
    #[serde(default)]
    pub next_queue_id: u64,
}

pub const MAX_QUEUED_JOBS: usize = 5;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueuedBuildingJob {
    pub id: u64,
    pub job: BuildingJob,
}

impl Building {
    pub fn jobs(&self) -> impl Iterator<Item = &BuildingJob> {
        self.job
            .iter()
            .chain(self.queue.iter().map(|entry| &entry.job))
    }

    pub(super) fn enqueue(&mut self, job: BuildingJob) {
        if self.job.is_none() {
            self.job = Some(job);
        } else {
            self.queue.push(QueuedBuildingJob {
                id: self.next_queue_id,
                job,
            });
            self.next_queue_id += 1;
        }
    }

    pub fn footprint(&self) -> Footprint {
        let (columns, rows) = self.kind.size();
        Footprint {
            origin: self.origin,
            columns,
            rows,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.construction.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingKind {
    TownCenter,
    MiningCamp,
    Farm,
    LumberMill,
    Smelter,
    Kiln,
    Weaver,
    Kitchen,
    Barracks,
    Range,
    Workshop,
    Infirmary,
    Watchtower,
    Monument,
    House,
    Granary,
    Dock,
}

/// What villagers can construct, in build-menu order.
pub const BUILDABLE: [BuildingKind; 17] = [
    BuildingKind::TownCenter,
    BuildingKind::House,
    BuildingKind::Granary,
    BuildingKind::Watchtower,
    BuildingKind::Dock,
    BuildingKind::MiningCamp,
    BuildingKind::Farm,
    BuildingKind::LumberMill,
    BuildingKind::Smelter,
    BuildingKind::Kiln,
    BuildingKind::Weaver,
    BuildingKind::Kitchen,
    BuildingKind::Barracks,
    BuildingKind::Range,
    BuildingKind::Workshop,
    BuildingKind::Infirmary,
    BuildingKind::Monument,
];

impl BuildingKind {
    /// Footprint in cells.
    pub const fn size(self) -> (u16, u16) {
        match self {
            // Compact rectangular plots; neighboring buildings can share an edge.
            Self::TownCenter => (5, 5),
            Self::Dock => (4, 4),
            Self::House | Self::Granary => (3, 3),
            Self::Watchtower => (2, 2),
            Self::Monument => (5, 5),
            Self::MiningCamp
            | Self::Farm
            | Self::Kiln
            | Self::Weaver
            | Self::Kitchen
            | Self::Infirmary => (3, 3),
            Self::LumberMill | Self::Smelter | Self::Barracks | Self::Range | Self::Workshop => {
                (4, 4)
            }
        }
    }

    /// Resources reserved when the foundation is placed.
    pub const fn cost(self) -> &'static [(ResourceKind, f64)] {
        match self {
            Self::TownCenter => &[(ResourceKind::Wood, super::TOWN_CENTER_WOOD_COST)],
            Self::House => &[(ResourceKind::Wood, 15.0)],
            Self::Granary => &[(ResourceKind::Wood, 25.0)],
            Self::Watchtower => &[(ResourceKind::Wood, 15.0), (ResourceKind::Stone, 15.0)],
            Self::Dock => &[(ResourceKind::Wood, 30.0)],
            Self::MiningCamp => &[(ResourceKind::Wood, 30.0), (ResourceKind::Stone, 10.0)],
            Self::Farm => &[(ResourceKind::Wood, 25.0)],
            Self::LumberMill => &[(ResourceKind::Wood, 40.0), (ResourceKind::Stone, 10.0)],
            Self::Smelter => &[(ResourceKind::Wood, 30.0), (ResourceKind::Stone, 30.0)],
            Self::Kiln => &[(ResourceKind::Wood, 30.0), (ResourceKind::Stone, 20.0)],
            Self::Weaver | Self::Kitchen => &[(ResourceKind::Wood, 30.0)],
            Self::Barracks | Self::Range => {
                &[(ResourceKind::Timber, 20.0), (ResourceKind::Stone, 20.0)]
            }
            Self::Workshop => &[(ResourceKind::Timber, 25.0), (ResourceKind::Bricks, 15.0)],
            Self::Infirmary => &[(ResourceKind::Timber, 15.0), (ResourceKind::Cloth, 10.0)],
            Self::Monument => &[
                (ResourceKind::Timber, 30.0),
                (ResourceKind::Bricks, 30.0),
                (ResourceKind::Cloth, 15.0),
                (ResourceKind::Gold, 20.0),
                (ResourceKind::Steel, 15.0),
            ],
        }
    }

    /// Seconds of villager work to raise the foundation, proportional to its cost.
    pub fn build_seconds(self) -> f64 {
        self.cost().iter().map(|(_, amount)| amount).sum::<f64>()
            * super::BUILD_SECONDS_PER_RESOURCE
    }

    /// Villagers this building houses once complete.
    pub const fn housing(self) -> usize {
        match self {
            Self::TownCenter | Self::House => 5,
            _ => 0,
        }
    }

    /// Whether gatherers may drop this resource here once it is complete.
    pub const fn accepts(self, resource: ResourceKind) -> bool {
        match self {
            Self::TownCenter => true,
            Self::Granary | Self::Farm => {
                matches!(resource, ResourceKind::Food | ResourceKind::Fiber)
            }
            Self::MiningCamp => matches!(
                resource,
                ResourceKind::Stone | ResourceKind::Gold | ResourceKind::Iron | ResourceKind::Coal
            ),
            _ => false,
        }
    }

    /// How far a complete building sees, in world units.
    pub const fn sight_radius(self) -> f64 {
        match self {
            Self::Watchtower => 20.0,
            Self::Monument => 24.0,
            _ => super::BUILDING_SIGHT_RADIUS,
        }
    }

    /// A dock must stand on land with open water along one side.
    pub const fn needs_coast(self) -> bool {
        matches!(self, Self::Dock)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnologyKind {
    Forestry,
    Agriculture,
    Masonry,
    Mining,
    Textiles,
}

impl TechnologyKind {
    pub const ALL: [Self; 5] = ROADMAP_TECHNOLOGIES;

    pub fn prerequisite(self) -> Option<Self> {
        match self {
            Self::Mining => Some(Self::Masonry),
            Self::Textiles => Some(Self::Agriculture),
            Self::Forestry | Self::Agriculture | Self::Masonry => None,
        }
    }

    pub(super) fn improves(self, resource: ResourceKind) -> bool {
        matches!(
            (self, resource),
            (Self::Forestry, ResourceKind::Wood)
                | (Self::Agriculture, ResourceKind::Food)
                | (Self::Masonry, ResourceKind::Stone | ResourceKind::Clay)
                | (Self::Mining, ResourceKind::Gold | ResourceKind::Iron)
                | (Self::Textiles, ResourceKind::Fiber)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DomainCatalog {
    pub resources: &'static [ResourceKind],
    pub buildings: &'static [BuildingKind],
    pub units: &'static [UnitKind],
    pub recipes: &'static [RecipeCatalogEntry],
    pub technologies: &'static [TechnologyKind],
}

impl DomainCatalog {
    pub const fn roadmap() -> Self {
        Self {
            resources: &ROADMAP_RESOURCES,
            buildings: &ROADMAP_BUILDINGS,
            units: &ROADMAP_UNITS,
            recipes: &ROADMAP_RECIPES,
            technologies: &ROADMAP_TECHNOLOGIES,
        }
    }
}

pub const ROADMAP_BUILDINGS: [BuildingKind; 17] = [
    BuildingKind::TownCenter,
    BuildingKind::MiningCamp,
    BuildingKind::Farm,
    BuildingKind::LumberMill,
    BuildingKind::Smelter,
    BuildingKind::Kiln,
    BuildingKind::Weaver,
    BuildingKind::Kitchen,
    BuildingKind::Barracks,
    BuildingKind::Range,
    BuildingKind::Workshop,
    BuildingKind::Infirmary,
    BuildingKind::Watchtower,
    BuildingKind::Monument,
    BuildingKind::House,
    BuildingKind::Granary,
    BuildingKind::Dock,
];

pub const ROADMAP_TECHNOLOGIES: [TechnologyKind; 5] = [
    TechnologyKind::Forestry,
    TechnologyKind::Agriculture,
    TechnologyKind::Masonry,
    TechnologyKind::Mining,
    TechnologyKind::Textiles,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioId {
    Prototype,
    FoundryTown,
    FrontierSurvey,
    MonumentWorks,
    HoldTheCoast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioOutcome {
    Running,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioObjectiveProgress {
    pub completed: u16,
    pub total: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioState {
    pub id: ScenarioId,
    pub tick_limit: u64,
    pub elapsed_ticks: u64,
    pub objective_progress: ScenarioObjectiveProgress,
    pub outcome: ScenarioOutcome,
}

impl Default for ScenarioState {
    fn default() -> Self {
        Self {
            id: ScenarioId::Prototype,
            tick_limit: DEFAULT_SCENARIO_TICK_LIMIT,
            elapsed_ticks: 0,
            objective_progress: ScenarioObjectiveProgress {
                completed: 0,
                total: 0,
            },
            outcome: ScenarioOutcome::Running,
        }
    }
}
