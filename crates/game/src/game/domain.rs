use serde::{Deserialize, Serialize};

pub const DEFAULT_SCENARIO_TICK_LIMIT: u64 = 36_000;

/// A continuous point in cell units: `(column + 0.5, row + 0.5)` is a cell center.
/// Positions are derived for presentation and sight; they are never authoritative.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub(super) fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CellCoordinate {
    pub column: u16,
    pub row: u16,
}

impl CellCoordinate {
    pub const fn new(column: u16, row: u16) -> Self {
        Self { column, row }
    }

    pub fn center(self) -> Position {
        Position {
            x: f64::from(self.column) + 0.5,
            y: f64::from(self.row) + 0.5,
        }
    }

    /// Chebyshev adjacency: the eight cells around `self`.
    pub(super) fn touches(self, other: Self) -> bool {
        self != other
            && self.column.abs_diff(other.column) <= 1
            && self.row.abs_diff(other.row) <= 1
    }
}

/// An axis-aligned block of cells. Every static thing in the world claims one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Footprint {
    pub origin: CellCoordinate,
    pub columns: u16,
    pub rows: u16,
}

impl Footprint {
    pub fn cells(self) -> impl Iterator<Item = CellCoordinate> {
        (self.origin.row..self.origin.row + self.rows).flat_map(move |row| {
            (self.origin.column..self.origin.column + self.columns)
                .map(move |column| CellCoordinate::new(column, row))
        })
    }

    pub fn contains(self, cell: CellCoordinate) -> bool {
        (self.origin.column..self.origin.column + self.columns).contains(&cell.column)
            && (self.origin.row..self.origin.row + self.rows).contains(&cell.row)
    }

    /// A cell from which a villager can work on this footprint: outside it and
    /// touching it, including diagonally.
    pub(super) fn is_interaction_cell(self, cell: CellCoordinate) -> bool {
        !self.contains(cell) && self.cells().any(|inner| inner.touches(cell))
    }

    pub fn center(self) -> Position {
        Position {
            x: f64::from(self.origin.column) + f64::from(self.columns) / 2.0,
            y: f64::from(self.origin.row) + f64::from(self.rows) / 2.0,
        }
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnitAction {
    BuildRoad {
        cells: Vec<CellCoordinate>,
    },
    AttackAnimal {
        animal_id: String,
        elapsed_seconds: f64,
    },
    Board {
        ship_id: String,
    },
    Idle,
    /// Walk to `to`. The destination cell is reserved for this unit until it arrives.
    Move {
        to: CellCoordinate,
    },
    Gather {
        resource_id: String,
        phase: GatherPhase,
    },
    /// Explore the footprint before validating and paying for a foundation.
    ExploreBuild {
        origin: CellCoordinate,
        kind: BuildingKind,
    },
    /// Walk beside the foundation `building_id` and raise it.
    Build {
        building_id: String,
    },
    /// Prepare or replenish a cultivated food field.
    Cultivate {
        resource_id: String,
    },
    /// Carry the load to a compatible building or stopped shore ship,
    /// unload it there, and stand idle.
    Deposit {
        storage_id: String,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatherPhase {
    #[default]
    ToResource,
    Gathering,
    Returning,
    Depositing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarriedResource {
    pub kind: ResourceKind,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub health: f64,
    pub id: String,
    pub kind: UnitKind,
    /// The cell this unit stands in. It is exclusively claimed by this unit.
    pub cell: CellCoordinate,
    /// A step in progress. Its target cell is claimed before the unit leaves
    /// `cell`, so a unit always owns every cell its body overlaps.
    pub step: Option<Step>,
    pub action: UnitAction,
    pub cargo: Option<CarriedResource>,
}

impl Unit {
    /// Work starts only after the claimed step ends beside the target footprint.
    pub fn is_at_work_site(&self, footprint: Footprint) -> bool {
        self.step.is_none() && footprint.is_interaction_cell(self.cell)
    }

    pub fn position(&self) -> Position {
        let from = self.cell.center();
        match self.step {
            None => from,
            Some(step) => {
                let to = step.to.center();
                Position {
                    x: from.x + (to.x - from.x) * step.progress,
                    y: from.y + (to.y - from.y) * step.progress,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub to: CellCoordinate,
    /// Fraction of the step completed, in `[0, 1)`.
    pub progress: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: String,
    pub kind: ResourceKind,
    pub cell: CellCoordinate,
    pub amount: f64,
    pub capacity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<FieldState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldState {
    /// Reserved preparation work; None means harvestable or depleted.
    pub work: Option<f64>,
}

impl ResourceNode {
    pub fn footprint(&self) -> Footprint {
        Footprint {
            origin: self.cell,
            columns: if self.field.is_some() { 3 } else { 1 },
            rows: if self.field.is_some() { 3 } else { 1 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Water,
    Wood,
    Food,
    Stone,
    Gold,
    Iron,
    Coal,
    Clay,
    Fiber,
    Timber,
    Steel,
    Bricks,
    Cloth,
    Rations,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitKind {
    #[default]
    Villager,
    Guard,
    Archer,
    Healer,
    SiegeCart,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub kind: BuildingKind,
    /// Completed clay/brick material upgrade on the existing plot.
    pub masonry: bool,
    /// North-west cell of the footprint.
    pub origin: CellCoordinate,
    /// `Some(seconds of work done)` while this is a foundation; `None` once complete.
    pub construction: Option<f64>,
    pub produces: Vec<ProductKind>,
    pub researches: Vec<TechnologyKind>,
    pub job: Option<BuildingJob>,
    /// Paid tasks waiting behind the active job, in submission order.
    pub queue: Vec<QueuedBuildingJob>,
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
            Self::Workshop => &[(ResourceKind::Timber, 25.0), (ResourceKind::Stone, 15.0)],
            Self::Infirmary => &[(ResourceKind::Timber, 15.0), (ResourceKind::Cloth, 10.0)],
            Self::Monument => &[
                (ResourceKind::Timber, 30.0),
                (ResourceKind::Stone, 30.0),
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
            Self::TownCenter | Self::Dock => true,
            Self::LumberMill => matches!(resource, ResourceKind::Wood),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductKind {
    TransportShip,
    Villager,
    Guard,
    Archer,
    Healer,
    SiegeCart,
    Timber,
    Steel,
    Bricks,
    Cloth,
    Rations,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BuildingJob {
    Upgrade {
        elapsed_seconds: f64,
    },
    Produce {
        product: ProductKind,
        elapsed_seconds: f64,
    },
    Research {
        technology: TechnologyKind,
        elapsed_seconds: f64,
    },
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

    pub const fn requires_masonry(self) -> bool {
        !matches!(self, Self::Masonry)
    }

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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stockpile {
    pub water: f64,
    pub wood: f64,
    pub food: f64,
    pub stone: f64,
    pub gold: f64,
    pub iron: f64,
    pub coal: f64,
    pub clay: f64,
    pub fiber: f64,
    pub timber: f64,
    pub steel: f64,
    pub bricks: f64,
    pub cloth: f64,
    pub rations: f64,
}

impl Stockpile {
    pub(super) fn entries(&self) -> [(&'static str, f64); 14] {
        [
            ("water", self.water),
            ("wood", self.wood),
            ("food", self.food),
            ("stone", self.stone),
            ("gold", self.gold),
            ("iron", self.iron),
            ("coal", self.coal),
            ("clay", self.clay),
            ("fiber", self.fiber),
            ("timber", self.timber),
            ("steel", self.steel),
            ("bricks", self.bricks),
            ("cloth", self.cloth),
            ("rations", self.rations),
        ]
    }

    pub fn amount(&self, kind: ResourceKind) -> f64 {
        match kind {
            ResourceKind::Water => self.water,
            ResourceKind::Wood => self.wood,
            ResourceKind::Food => self.food,
            ResourceKind::Stone => self.stone,
            ResourceKind::Gold => self.gold,
            ResourceKind::Iron => self.iron,
            ResourceKind::Coal => self.coal,
            ResourceKind::Clay => self.clay,
            ResourceKind::Fiber => self.fiber,
            ResourceKind::Timber => self.timber,
            ResourceKind::Steel => self.steel,
            ResourceKind::Bricks => self.bricks,
            ResourceKind::Cloth => self.cloth,
            ResourceKind::Rations => self.rations,
        }
    }

    /// Whether every cost is covered.
    pub fn affords(&self, cost: &[(ResourceKind, f64)]) -> bool {
        cost.iter()
            .all(|(kind, amount)| self.amount(*kind) >= *amount)
    }

    pub(super) fn add(&mut self, kind: ResourceKind, amount: f64) {
        match kind {
            ResourceKind::Water => self.water += amount,
            ResourceKind::Wood => self.wood += amount,
            ResourceKind::Food => self.food += amount,
            ResourceKind::Stone => self.stone += amount,
            ResourceKind::Gold => self.gold += amount,
            ResourceKind::Iron => self.iron += amount,
            ResourceKind::Coal => self.coal += amount,
            ResourceKind::Clay => self.clay += amount,
            ResourceKind::Fiber => self.fiber += amount,
            ResourceKind::Timber => self.timber += amount,
            ResourceKind::Steel => self.steel += amount,
            ResourceKind::Bricks => self.bricks += amount,
            ResourceKind::Cloth => self.cloth += amount,
            ResourceKind::Rations => self.rations += amount,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RecipeCatalogEntry {
    pub building: BuildingKind,
    pub inputs: &'static [ResourceKind],
    pub output: ResourceKind,
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

pub const ROADMAP_RESOURCES: [ResourceKind; 14] = [
    ResourceKind::Water,
    ResourceKind::Wood,
    ResourceKind::Food,
    ResourceKind::Stone,
    ResourceKind::Gold,
    ResourceKind::Iron,
    ResourceKind::Coal,
    ResourceKind::Clay,
    ResourceKind::Fiber,
    ResourceKind::Timber,
    ResourceKind::Steel,
    ResourceKind::Bricks,
    ResourceKind::Cloth,
    ResourceKind::Rations,
];

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

pub const ROADMAP_UNITS: [UnitKind; 5] = [
    UnitKind::Villager,
    UnitKind::Guard,
    UnitKind::Archer,
    UnitKind::Healer,
    UnitKind::SiegeCart,
];

pub const ROADMAP_RECIPES: [RecipeCatalogEntry; 5] = [
    RecipeCatalogEntry {
        building: BuildingKind::LumberMill,
        inputs: &[ResourceKind::Wood],
        output: ResourceKind::Timber,
    },
    RecipeCatalogEntry {
        building: BuildingKind::Smelter,
        inputs: &[ResourceKind::Iron, ResourceKind::Coal],
        output: ResourceKind::Steel,
    },
    RecipeCatalogEntry {
        building: BuildingKind::Kiln,
        inputs: &[ResourceKind::Clay, ResourceKind::Wood],
        output: ResourceKind::Bricks,
    },
    RecipeCatalogEntry {
        building: BuildingKind::Weaver,
        inputs: &[ResourceKind::Fiber],
        output: ResourceKind::Cloth,
    },
    RecipeCatalogEntry {
        building: BuildingKind::Kitchen,
        inputs: &[ResourceKind::Food],
        output: ResourceKind::Rations,
    },
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
