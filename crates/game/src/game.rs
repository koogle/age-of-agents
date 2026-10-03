use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

mod domain;
mod economy;
#[cfg(test)]
mod economy_tests;
mod fields;
#[cfg(test)]
mod fields_tests;
#[cfg(test)]
mod fixture;
mod gathering;
#[cfg(test)]
mod gathering_tests;
#[cfg(test)]
mod group_move_tests;
mod movement;
mod occupancy;
#[cfg(test)]
mod slice_a_tests;
#[cfg(test)]
mod soundness_tests;
mod terrain_codec;
mod worldgen;

pub use domain::*;
pub use fields::{FIELD_COST, FIELD_FOOD, FIELD_WORK_SECONDS};
pub use gathering::NEXT_RESOURCE_RADIUS;
use movement::{Goal, Travel};
pub use worldgen::FISHING_BOAT_COST;

/// The grid is finer than a villager is tall (a villager stands about one and
/// a half cells high), so bodies stand right against what they work on and
/// resources pack into tight woodlines, berry patches and mine clumps.
pub const WORLD_COLUMNS: u16 = 120;
pub const WORLD_ROWS: u16 = 80;
/// Sight radii in cells.
pub const UNIT_SIGHT_RADIUS: f64 = 8.0;
pub const BUILDING_SIGHT_RADIUS: f64 = 12.0;
pub const TOWN_CENTER_WOOD_COST: f64 = 20.0;
/// Construction time per resource unit of a building's cost, so dearer
/// buildings take longer to raise (a 15-wood house takes 4.5 seconds).
pub const BUILD_SECONDS_PER_RESOURCE: f64 = 0.3;
pub const VILLAGER_FOOD_COST: f64 = 50.0;
pub const VILLAGER_PRODUCTION_SECONDS: f64 = 6.0;
pub const RESEARCH_FOOD_COST: f64 = 40.0;
pub const RESEARCH_WOOD_COST: f64 = 20.0;
pub const RESEARCH_SECONDS: f64 = 8.0;
pub const GATHERING_TECH_MULTIPLIER: f64 = 1.2;
/// Minimum gap between resource clusters and starting-base clearance, in cells.
/// Nodes within one cluster touch.
pub const RESOURCE_CLUSTER_SEPARATION: f64 = 5.0;
pub const STARTING_BASE_RESOURCE_CLEARANCE: f64 = 8.0;
/// Walking speed in cells per second.
const MOVE_SPEED: f64 = 3.0;
pub(crate) const GATHER_RATE: f64 = 2.0;
pub const VILLAGER_CARRY_CAPACITY: f64 = 20.0;
/// The island new worlds get unless a seed is given.
pub const DEFAULT_SEED: u64 = 0x00A6_E0F0_A6E7;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameWorld {
    /// The island this world was generated from.
    pub seed: u64,
    pub tick: u64,
    pub simulation_speed: f64,
    pub terrain: Vec<TerrainCell>,
    pub explored_cells: Vec<CellCoordinate>,
    pub units: Vec<Unit>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<Building>,
    pub stockpile: Stockpile,
    pub researched_technologies: Vec<TechnologyKind>,
    pub scenario: ScenarioState,
    next_building_id: u64,
    next_unit_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitView {
    #[serde(flatten)]
    pub unit: Unit,
    /// Interpolated body position in cell units.
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildingView {
    #[serde(flatten)]
    pub building: Building,
    pub columns: u16,
    pub rows: u16,
}

// Clients decode snapshots too; the catalog is static data they already have.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub columns: u16,
    pub rows: u16,
    pub tick: u64,
    pub simulation_speed: f64,
    #[serde(with = "terrain_codec")]
    pub terrain: Vec<SnapshotTerrainCell>,
    pub units: Vec<UnitView>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<BuildingView>,
    pub stockpile: Stockpile,
    pub researched_technologies: Vec<TechnologyKind>,
    #[serde(skip_deserializing, default = "DomainCatalog::roadmap")]
    pub catalog: DomainCatalog,
    pub scenario: ScenarioState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Move {
        unit_id: String,
        to: CellCoordinate,
    },
    GroupMove {
        unit_ids: Vec<String>,
        to: CellCoordinate,
    },
    Gather {
        unit_id: String,
        resource_id: String,
    },
    PlantField {
        unit_id: String,
        origin: CellCoordinate,
    },
    Cultivate {
        unit_id: String,
        resource_id: String,
    },
    /// Place a foundation of `kind` (a town center when omitted) with its
    /// north-west corner at `origin`.
    Build {
        unit_id: String,
        origin: CellCoordinate,
        #[serde(default = "town_center_kind")]
        kind: BuildingKind,
    },
    /// Join or resume work on an existing foundation.
    Construct {
        unit_id: String,
        building_id: String,
    },
    Produce {
        building_id: String,
        product: ProductKind,
    },
    Research {
        building_id: String,
        technology: TechnologyKind,
    },
    SetSimulationSpeed {
        multiplier: f64,
    },
    /// Take the unit's load to a drop site (a town center, or a granary for
    /// food and fiber) and unload it.
    Deposit {
        unit_id: String,
        building_id: String,
    },
    /// Abandon the current task. The unit finishes the step it is taking,
    /// keeps any cargo, and leaves foundation progress in place.
    Stop {
        unit_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    UnitNotFound,
    EmptyUnitGroup,
    DuplicateUnit,
    ResourceNotFound,
    ResourceDepleted,
    FarmRequired,
    FieldNotDepleted,
    InvalidDestination,
    DestinationOccupied,
    TargetUnreachable,
    InvalidBuildSite,
    InsufficientWood,
    InsufficientStone,
    NotBuildable,
    NeedsCoast,
    PopulationCapReached,
    NothingToDeposit,
    BuildingRefusesCargo,
    BuildingNotFound,
    BuildingBusy,
    BuildingUnderConstruction,
    BuildingAlreadyComplete,
    ProductUnavailable,
    InsufficientFood,
    InsufficientResources(ResourceKind),
    InsufficientProductionResources,
    VillagerRequired,
    TechnologyUnavailable,
    TechnologyAlreadyResearched,
    TechnologyInProgress,
    MissingTechnologyPrerequisite,
    InsufficientResearchResources,
    InvalidSimulationSpeed,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnitNotFound => "unit not found",
            Self::EmptyUnitGroup => "unit group is empty",
            Self::DuplicateUnit => "unit group contains a duplicate member",
            Self::ResourceNotFound => "resource not found",
            Self::ResourceDepleted => "resource is depleted",
            Self::FarmRequired => "a completed farm is required for fields",
            Self::FieldNotDepleted => "harvest the field before replenishing it",
            Self::InvalidDestination => "destination is outside the world",
            Self::DestinationOccupied => "destination cell is occupied",
            Self::TargetUnreachable => "target is unreachable",
            Self::InvalidBuildSite => "build site is blocked or outside the world",
            Self::InsufficientWood => "insufficient wood",
            Self::InsufficientStone => "insufficient stone",
            Self::NotBuildable => "villagers cannot build that",
            Self::NeedsCoast => "a dock must touch the sea",
            Self::PopulationCapReached => "population cap reached",
            Self::NothingToDeposit => "unit is not carrying anything",
            Self::BuildingRefusesCargo => "building does not take that cargo",
            Self::BuildingNotFound => "building not found",
            Self::BuildingBusy => "building is already producing",
            Self::BuildingUnderConstruction => "building is still under construction",
            Self::BuildingAlreadyComplete => "building is already complete",
            Self::ProductUnavailable => "building cannot produce that item",
            Self::InsufficientFood => "insufficient food",
            Self::InsufficientResources(kind) => return write!(f, "insufficient {}", kind.name()),
            Self::InsufficientProductionResources => "insufficient resources for production",
            Self::VillagerRequired => "only villagers can gather or build",
            Self::TechnologyUnavailable => "building cannot research that technology",
            Self::TechnologyAlreadyResearched => "technology is already researched",
            Self::TechnologyInProgress => "technology is already being researched",
            Self::MissingTechnologyPrerequisite => "technology prerequisite is not researched",
            Self::InsufficientResearchResources => "research requires 40 food and 20 wood",
            Self::InvalidSimulationSpeed => "simulation speed must be 0, 1, or 2",
        };
        f.write_str(message)
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::generate(DEFAULT_SEED)
    }
}

impl GameWorld {
    /// A fresh world on the island grown from `seed`.
    pub fn generate(seed: u64) -> Self {
        let island = worldgen::generate(seed);
        let villager = |number: u64, cell: CellCoordinate| Unit {
            id: format!("villager-{number}"),
            kind: UnitKind::Villager,
            cell,
            step: None,
            action: UnitAction::Idle,
            cargo: None,
        };
        let mut world = Self {
            seed: island.seed,
            tick: 0,
            simulation_speed: 1.0,
            terrain: island.terrain,
            explored_cells: Vec::new(),
            units: vec![
                villager(1, island.villagers[0]),
                villager(2, island.villagers[1]),
            ],
            resources: island.resources,
            buildings: vec![town_center("base-1", island.town_center, None)],
            stockpile: Stockpile::default(),
            researched_technologies: Vec::new(),
            scenario: ScenarioState::default(),
            next_building_id: 2,
            next_unit_id: 3,
        };
        world.refresh_exploration();
        world
    }
}

fn town_center_kind() -> BuildingKind {
    BuildingKind::TownCenter
}

fn town_center(id: &str, origin: CellCoordinate, construction: Option<f64>) -> Building {
    building(BuildingKind::TownCenter, id, origin, construction)
}

fn building(
    kind: BuildingKind,
    id: &str,
    origin: CellCoordinate,
    construction: Option<f64>,
) -> Building {
    let town_center = kind == BuildingKind::TownCenter;
    Building {
        id: id.into(),
        kind,
        origin,
        construction,
        produces: kind.products().to_vec(),
        researches: if town_center {
            TechnologyKind::ALL.to_vec()
        } else {
            match kind {
                BuildingKind::MiningCamp => vec![TechnologyKind::Mining],
                BuildingKind::Farm => vec![TechnologyKind::Agriculture],
                BuildingKind::LumberMill => vec![TechnologyKind::Forestry],
                BuildingKind::Kiln => vec![TechnologyKind::Masonry],
                BuildingKind::Weaver => vec![TechnologyKind::Textiles],
                _ => Vec::new(),
            }
        },
        job: None,
    }
}

impl GameWorld {
    /// Applies a command atomically: on error the world is unchanged.
    pub fn apply_command(&mut self, command: Command) -> Result<(), CommandError> {
        // A new order replaces a unit's current task, so validate it against a
        // copy in which that unit has stopped; a rejected order leaves the
        // world, and the unit's old task, untouched.
        let mut next = self.clone();
        next.execute(command)?;
        *self = next;
        #[cfg(debug_assertions)]
        if let Err(error) = self.validate() {
            panic!("an accepted command broke a world invariant: {error}");
        }
        Ok(())
    }

    fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        match command {
            Command::Move { unit_id, to } => {
                let unit = self.ordered_unit(&unit_id)?;
                self.validate_move_destination(unit, to)?;
                self.units[unit].action = UnitAction::Move { to };
            }
            Command::GroupMove { unit_ids, to } => {
                if unit_ids.is_empty() {
                    return Err(CommandError::EmptyUnitGroup);
                }
                let mut seen = BTreeSet::new();
                let mut members = Vec::with_capacity(unit_ids.len());
                for unit_id in unit_ids {
                    if !seen.insert(unit_id.clone()) {
                        return Err(CommandError::DuplicateUnit);
                    }
                    let index = self.ordered_unit(&unit_id)?;
                    members.push((unit_id, index));
                }
                members.sort();
                let members: Vec<_> = members.into_iter().map(|(_, index)| index).collect();
                for (unit, destination) in self.group_move_assignments(&members, to)? {
                    self.units[unit].action = UnitAction::Move { to: destination };
                }
            }
            Command::Gather {
                unit_id,
                resource_id,
            } => {
                let unit = self.ordered_unit(&unit_id)?;
                if self.units[unit].kind != UnitKind::Villager {
                    return Err(CommandError::VillagerRequired);
                }
                let resource = self
                    .resources
                    .iter()
                    .find(|resource| resource.id == resource_id)
                    .ok_or(CommandError::ResourceNotFound)?;
                if resource.amount <= 0.0 {
                    return Err(CommandError::ResourceDepleted);
                }
                if !self.can_reach_beside(unit, resource.footprint()) {
                    return Err(CommandError::TargetUnreachable);
                }
                self.units[unit].action = UnitAction::Gather {
                    resource_id,
                    phase: if self.units[unit].cargo.is_some() {
                        GatherPhase::Returning
                    } else {
                        GatherPhase::ToResource
                    },
                };
            }
            Command::PlantField { unit_id, origin } => self.plant_field(&unit_id, origin)?,
            Command::Cultivate {
                unit_id,
                resource_id,
            } => self.cultivate(&unit_id, &resource_id)?,
            Command::Build {
                unit_id,
                origin,
                kind,
            } => {
                let unit = self.ordered_unit(&unit_id)?;
                if self.units[unit].kind != UnitKind::Villager {
                    return Err(CommandError::VillagerRequired);
                }
                if !BUILDABLE.contains(&kind) {
                    return Err(CommandError::NotBuildable);
                }
                let (columns, rows) = kind.size();
                let footprint = Footprint {
                    origin,
                    columns,
                    rows,
                };
                if !self.footprint_is_free(footprint) {
                    return Err(CommandError::InvalidBuildSite);
                }
                if kind.needs_coast() && !self.touches_sea(footprint) {
                    return Err(CommandError::NeedsCoast);
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
            }
            Command::Construct {
                unit_id,
                building_id,
            } => {
                let unit = self.ordered_unit(&unit_id)?;
                if self.units[unit].kind != UnitKind::Villager {
                    return Err(CommandError::VillagerRequired);
                }
                let building = self
                    .buildings
                    .iter()
                    .find(|building| building.id == building_id)
                    .ok_or(CommandError::BuildingNotFound)?;
                if building.is_complete() {
                    return Err(CommandError::BuildingAlreadyComplete);
                }
                if !self.can_reach_beside(unit, building.footprint()) {
                    return Err(CommandError::TargetUnreachable);
                }
                self.units[unit].action = UnitAction::Build { building_id };
            }
            Command::Produce {
                building_id,
                product,
            } => {
                let building = self.building_index_and_idle(&building_id)?;
                if !self.buildings[building].kind.products().contains(&product) {
                    return Err(CommandError::ProductUnavailable);
                }
                if product.unit_kind().is_some() && self.villagers_and_trainees() >= self.housing()
                {
                    return Err(CommandError::PopulationCapReached);
                }
                if !self.stockpile.affords(product.cost()) {
                    return Err(if product == ProductKind::Villager {
                        CommandError::InsufficientFood
                    } else {
                        CommandError::InsufficientProductionResources
                    });
                }
                for &(kind, amount) in product.cost() {
                    self.stockpile.add(kind, -amount);
                }
                self.buildings[building].job = Some(BuildingJob::Produce {
                    product,
                    elapsed_seconds: 0.0,
                });
            }
            Command::Research {
                building_id,
                technology,
            } => {
                let building = self.building_index_and_idle(&building_id)?;
                if !self.buildings[building].researches.contains(&technology) {
                    return Err(CommandError::TechnologyUnavailable);
                }
                if self.researched_technologies.contains(&technology) {
                    return Err(CommandError::TechnologyAlreadyResearched);
                }
                if self.buildings.iter().any(|b| matches!(b.job, Some(BuildingJob::Research { technology: t, .. }) if t == technology)) {
                    return Err(CommandError::TechnologyInProgress);
                }
                if technology
                    .prerequisite()
                    .is_some_and(|required| !self.researched_technologies.contains(&required))
                {
                    return Err(CommandError::MissingTechnologyPrerequisite);
                }
                if self.stockpile.food < RESEARCH_FOOD_COST
                    || self.stockpile.wood < RESEARCH_WOOD_COST
                {
                    return Err(CommandError::InsufficientResearchResources);
                }
                self.stockpile.food -= RESEARCH_FOOD_COST;
                self.stockpile.wood -= RESEARCH_WOOD_COST;
                self.buildings[building].job = Some(BuildingJob::Research {
                    technology,
                    elapsed_seconds: 0.0,
                });
            }
            Command::Deposit {
                unit_id,
                building_id,
            } => {
                let unit = self.ordered_unit(&unit_id)?;
                let cargo = self.units[unit]
                    .cargo
                    .as_ref()
                    .ok_or(CommandError::NothingToDeposit)?
                    .kind;
                let building = self
                    .buildings
                    .iter()
                    .find(|building| building.id == building_id)
                    .ok_or(CommandError::BuildingNotFound)?;
                if !building.is_complete() {
                    return Err(CommandError::BuildingUnderConstruction);
                }
                if !building.kind.accepts(cargo) {
                    return Err(CommandError::BuildingRefusesCargo);
                }
                if !self.can_reach_beside(unit, building.footprint()) {
                    return Err(CommandError::TargetUnreachable);
                }
                self.units[unit].action = UnitAction::Deposit { building_id };
            }
            Command::Stop { unit_id } => {
                let unit = self
                    .units
                    .iter()
                    .position(|unit| unit.id == unit_id)
                    .ok_or(CommandError::UnitNotFound)?;
                self.units[unit].action = UnitAction::Idle;
            }
            Command::SetSimulationSpeed { multiplier } => {
                if ![0.0, 1.0, 2.0].contains(&multiplier) {
                    return Err(CommandError::InvalidSimulationSpeed);
                }
                self.simulation_speed = multiplier;
            }
        }
        Ok(())
    }

    /// Villagers the complete buildings can house.
    pub fn housing(&self) -> usize {
        self.buildings
            .iter()
            .filter(|building| building.is_complete())
            .map(|building| building.kind.housing())
            .sum()
    }

    /// Living units plus those in training; processing jobs do not consume housing.
    pub fn villagers_and_trainees(&self) -> usize {
        self.units.len()
            + self
                .buildings
                .iter()
                .filter(|b| matches!(b.job, Some(BuildingJob::Produce { product, .. }) if product.unit_kind().is_some()))
                .count()
    }

    /// Whether any cell beside the footprint (sharing an edge) is water.
    fn touches_sea(&self, footprint: Footprint) -> bool {
        let Footprint {
            origin,
            columns,
            rows,
        } = footprint;
        let (c0, r0) = (i32::from(origin.column), i32::from(origin.row));
        let (c1, r1) = (c0 + i32::from(columns), r0 + i32::from(rows));
        let water = |column: i32, row: i32| {
            column >= 0
                && row >= 0
                && column < i32::from(WORLD_COLUMNS)
                && row < i32::from(WORLD_ROWS)
                && self.terrain[row as usize * usize::from(WORLD_COLUMNS) + column as usize].biome
                    == TerrainBiome::Water
        };
        (c0..c1).any(|c| water(c, r0 - 1) || water(c, r1))
            || (r0..r1).any(|r| water(c0 - 1, r) || water(c1, r))
    }

    fn next_building_name(&self) -> String {
        format!("building-{}", self.next_building_id)
    }

    fn next_unit_name(&self) -> String {
        format!("villager-{}", self.next_unit_id)
    }

    fn building_index_and_idle(&self, building_id: &str) -> Result<usize, CommandError> {
        let index = self
            .buildings
            .iter()
            .position(|building| building.id == building_id)
            .ok_or(CommandError::BuildingNotFound)?;
        if !self.buildings[index].is_complete() {
            return Err(CommandError::BuildingUnderConstruction);
        }
        if self.buildings[index].job.is_some() {
            return Err(CommandError::BuildingBusy);
        }
        Ok(index)
    }

    /// The unit receiving an order, stopped first: like `Stop`, it finishes
    /// the step it is taking, keeps any cargo, and leaves foundation progress.
    fn ordered_unit(&mut self, unit_id: &str) -> Result<usize, CommandError> {
        let index = self
            .units
            .iter()
            .position(|unit| unit.id == unit_id)
            .ok_or(CommandError::UnitNotFound)?;
        self.units[index].action = UnitAction::Idle;
        Ok(index)
    }

    pub fn tick(&mut self, dt: f64) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let dt = dt * self.simulation_speed;
        if dt <= 0.0 {
            return;
        }
        self.tick += 1;
        for index in 0..self.units.len() {
            match self.units[index].action.clone() {
                // A unit whose order ended mid-stride finishes the step it claimed.
                UnitAction::Idle => {
                    if let Some(step) = self.units[index].step {
                        self.travel(index, Goal::Cell(step.to), dt);
                        self.make_way(index);
                    }
                }
                UnitAction::Move { to } => self.tick_move(index, to, dt),
                UnitAction::Gather { resource_id, phase } => {
                    self.tick_gather(index, resource_id, phase, dt)
                }
                UnitAction::Build { building_id } => self.tick_build(index, &building_id, dt),
                UnitAction::Cultivate { resource_id } => {
                    self.tick_cultivate(index, &resource_id, dt)
                }
                UnitAction::Deposit { building_id } => self.tick_deposit(index, &building_id, dt),
            }
        }
        for index in 0..self.buildings.len() {
            self.tick_building_job(index, dt);
        }
        self.refresh_exploration();
        #[cfg(debug_assertions)]
        if let Err(error) = self.validate() {
            panic!("tick {} broke a world invariant: {error}", self.tick);
        }
        // Slice A records terminal scenario state without enforcing Slice E's
        // command restrictions. Gameplay keeps simulating, but terminal scenario
        // progress is immutable once won or lost.
        if self.scenario.outcome == ScenarioOutcome::Running {
            self.scenario.elapsed_ticks = self.scenario.elapsed_ticks.saturating_add(1);
            if self.scenario.elapsed_ticks >= self.scenario.tick_limit {
                self.scenario.outcome = ScenarioOutcome::Lost;
            }
        }
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        let visible = self.visible_cells();
        let explored: BTreeSet<_> = self.explored_cells.iter().copied().collect();

        WorldSnapshot {
            columns: WORLD_COLUMNS,
            rows: WORLD_ROWS,
            tick: self.tick,
            simulation_speed: self.simulation_speed,
            terrain: self
                .terrain
                .iter()
                .map(|cell| {
                    let coordinate = cell.coordinate();
                    let visibility = if visible.contains(&coordinate) {
                        CellVisibility::Visible
                    } else if explored.contains(&coordinate) {
                        CellVisibility::Explored
                    } else {
                        CellVisibility::Unseen
                    };
                    SnapshotTerrainCell {
                        column: cell.column,
                        row: cell.row,
                        biome: (visibility != CellVisibility::Unseen).then_some(cell.biome),
                        elevation: (visibility != CellVisibility::Unseen)
                            .then(|| terrain_codec::quantize(cell.elevation)),
                        visibility,
                    }
                })
                .collect(),
            units: self
                .units
                .iter()
                .filter(|unit| visible.contains(&unit.cell))
                .map(|unit| UnitView {
                    unit: unit.clone(),
                    position: unit.position(),
                })
                .collect(),
            resources: self
                .resources
                .iter()
                .filter(|resource| explored.contains(&resource.cell))
                .cloned()
                .collect(),
            buildings: self
                .buildings
                .iter()
                .filter(|building| {
                    building
                        .footprint()
                        .cells()
                        .any(|cell| explored.contains(&cell))
                })
                .map(|building| {
                    let (columns, rows) = building.kind.size();
                    BuildingView {
                        building: building.clone(),
                        columns,
                        rows,
                    }
                })
                .collect(),
            stockpile: self.stockpile.clone(),
            researched_technologies: self.researched_technologies.clone(),
            catalog: DomainCatalog::roadmap(),
            scenario: self.scenario.clone(),
        }
    }

    fn refresh_exploration(&mut self) {
        let mut explored: BTreeSet<_> = self.explored_cells.iter().copied().collect();
        explored.extend(self.visible_cells());
        self.explored_cells = explored.into_iter().collect();
    }

    fn visible_cells(&self) -> BTreeSet<CellCoordinate> {
        let eyes: Vec<_> = self
            .units
            .iter()
            .map(|unit| (unit.position(), UNIT_SIGHT_RADIUS))
            .chain(
                self.buildings
                    .iter()
                    .filter(|building| building.is_complete())
                    .map(|building| (building.footprint().center(), building.kind.sight_radius())),
            )
            .collect();
        self.terrain
            .iter()
            .map(|cell| cell.coordinate())
            .filter(|cell| {
                let center = cell.center();
                eyes.iter()
                    .any(|(eye, radius)| eye.distance(center) <= *radius)
            })
            .collect()
    }

    pub(super) fn gather_multiplier(&self, resource: ResourceKind) -> f64 {
        if self
            .researched_technologies
            .iter()
            .any(|technology| technology.improves(resource))
        {
            GATHERING_TECH_MULTIPLIER
        } else {
            1.0
        }
    }

    fn tick_build(&mut self, unit: usize, building_id: &str, dt: f64) {
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

#[cfg(test)]
mod placement_tests;
#[cfg(test)]
mod tests;
