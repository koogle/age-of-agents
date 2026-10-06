use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

mod coast;
mod construction;
mod roads;
pub use roads::{ROAD_WORK_SECONDS, Road, RoadKind, road_line};
#[cfg(test)]
mod roads_tests;
pub use coast::{DockFacing, dock_facing};
#[cfg(test)]
mod construction_tests;
mod domain;
mod economy;
pub use economy::{housing, population};
#[cfg(test)]
mod economy_tests;
mod fields;
#[cfg(test)]
mod fields_tests;
#[cfg(test)]
mod fixture;
mod gathering;
mod islands;
mod storage;
pub use storage::{CargoDirection, SHIP_RESOURCE_CAPACITY, ShipConnection, island_at};
#[cfg(test)]
mod islands_tests;
use islands::starting_origins;
#[cfg(test)]
mod gathering_tests;
#[cfg(test)]
mod group_move_tests;
mod movement;
mod occupancy;
mod progression;
#[cfg(test)]
mod progression_tests;
#[cfg(test)]
mod queue_tests;
#[cfg(test)]
mod ship_tests;
mod ships;
#[cfg(test)]
mod slice_a_tests;
#[cfg(test)]
mod soundness_tests;
mod terrain_codec;
mod wildlife;
#[cfg(test)]
mod wildlife_tests;
mod worldgen;

pub use domain::*;
pub use fields::{FIELD_COST, FIELD_FOOD, FIELD_SIZE, FIELD_WORK_SECONDS};
pub use gathering::NEXT_RESOURCE_RADIUS;
use movement::{Goal, Travel};
pub use progression::*;
pub use ships::*;
pub use wildlife::*;

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
pub const RESEARCH_COST: &[(ResourceKind, f64)] = &[
    (ResourceKind::Food, RESEARCH_FOOD_COST),
    (ResourceKind::Wood, RESEARCH_WOOD_COST),
];
pub const GATHERING_TECH_MULTIPLIER: f64 = 1.2;
/// Minimum gap between resource clusters and starting-base clearance, in cells.
/// Nodes within one cluster touch.
pub const RESOURCE_CLUSTER_SEPARATION: f64 = 5.0;
pub const STARTING_BASE_RESOURCE_CLEARANCE: f64 = 8.0;
/// Walking speed in cells per second.
const MOVE_SPEED: f64 = 3.0;
pub(crate) const GATHER_RATE: f64 = 2.0;
pub const VILLAGER_CARRY_CAPACITY: f64 = 20.0;
pub const UNIT_HEALTH: f64 = 100.0;
/// The island new worlds get unless a seed is given.
pub const DEFAULT_SEED: u64 = 0x00A6_E0F0_A6E7;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameWorld {
    pub island_id: u64,
    pub island_origins: Vec<CellCoordinate>,
    /// The island this world was generated from.
    pub seed: u64,
    pub economy_rules: EconomyRules,
    pub tick: u64,
    pub simulation_speed: f64,
    pub terrain: Vec<TerrainCell>,
    pub explored_cells: Vec<CellCoordinate>,
    pub units: Vec<Unit>,
    pub ships: Vec<TransportShip>,
    pub animals: Vec<Animal>,
    pub roads: Vec<Road>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<Building>,
    pub inventories: Vec<Stockpile>,
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
    pub produces: Vec<ProductKind>,
    pub researches: Vec<TechnologyKind>,
}

// Clients decode snapshots too; the catalog is static data they already have.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub island_id: u64,
    pub island_count: usize,
    pub island_origins: Vec<CellCoordinate>,
    pub available_buildings: Vec<BuildingKind>,
    pub columns: u16,
    pub rows: u16,
    pub tick: u64,
    pub simulation_speed: f64,
    #[serde(with = "terrain_codec")]
    pub terrain: Vec<SnapshotTerrainCell>,
    pub units: Vec<UnitView>,
    pub ships: Vec<TransportShip>,
    pub animals: Vec<Animal>,
    pub roads: Vec<Road>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<BuildingView>,
    /// Usable balances per island, including connected ship holds.
    pub inventories: Vec<Stockpile>,
    /// Onshore balances only, used by explicit dock transfers.
    pub stored_inventories: Vec<Stockpile>,
    pub ship_connections: Vec<ShipConnection>,
    pub researched_technologies: Vec<TechnologyKind>,
    #[serde(skip_deserializing, default = "DomainCatalog::roadmap")]
    pub catalog: DomainCatalog,
    pub scenario: ScenarioState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    BuildRoad {
        unit_id: String,
        start: CellCoordinate,
        end: CellCoordinate,
        kind: RoadKind,
    },
    AttackAnimal {
        unit_ids: Vec<String>,
        animal_id: String,
    },
    TransferShipCargo {
        ship_id: String,
        kind: ResourceKind,
        amount: f64,
        direction: CargoDirection,
    },
    Voyage {
        ship_id: String,
        island_id: u64,
    },
    Sail {
        ship_id: String,
        to: CellCoordinate,
    },
    DockShip {
        ship_id: String,
        building_id: String,
    },
    StopShip {
        ship_id: String,
    },
    Board {
        unit_id: String,
        ship_id: String,
    },
    Disembark {
        ship_id: String,
    },
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
    /// Build `kind` (a town center when omitted) at `origin`, exploring first
    /// if any footprint cell is outside current sight.
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
    /// Cancel a waiting task by stable ID; active work cannot be cancelled.
    CancelQueuedJob {
        building_id: String,
        queue_id: u64,
    },
    SetSimulationSpeed {
        multiplier: f64,
    },
    /// Take the unit's load to a drop site (a town center, or a granary for
    /// food and fiber) and unload it.
    Deposit {
        unit_id: String,
        storage_id: String,
    },
    /// Abandon the current task. The unit finishes the step it is taking,
    /// keeps any cargo, and leaves foundation progress in place.
    Stop {
        unit_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    AnimalNotVisible,
    CannotAttack,
    ShipStorageUnavailable,
    StorageUnavailable,
    InvalidCargoAmount,
    ShipHoldFull,
    ShipNotFound,
    ShipMustBeStopped,
    ShipFull,
    ShoreBlocked,
    DockRequired,
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
    BuildingQueueFull,
    QueuedJobNotFound,
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
    GamePaused,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::StorageUnavailable => "storage is unavailable, unreachable from shore, or full",
            Self::ShipStorageUnavailable => {
                "ship must be stopped beside a completed dock to transfer island storage"
            }
            Self::InvalidCargoAmount => "cargo amount must be finite and positive",
            Self::ShipHoldFull => {
                return write!(
                    f,
                    "ship can carry {SHIP_RESOURCE_CAPACITY} resources in total"
                );
            }
            Self::ShipNotFound => "transport ship not found",
            Self::ShipMustBeStopped => "stop the ship before boarding or unloading",
            Self::ShipFull => "transport passenger capacity is full",
            Self::ShoreBlocked => "no safe landing cells beside the ship or dock",
            Self::DockRequired => "select a completed dock",
            Self::AnimalNotVisible => "animal is not currently visible",
            Self::CannotAttack => "healers cannot attack",
            Self::UnitNotFound => "unit not found",
            Self::EmptyUnitGroup => "unit group is empty",
            Self::DuplicateUnit => "unit group contains a duplicate member",
            Self::ResourceNotFound => "resource not found",
            Self::ResourceDepleted => "resource is depleted",
            Self::FarmRequired => "a completed farm is required for fields",
            Self::FieldNotDepleted => "harvest the field before replenishing it",
            Self::InvalidDestination => "destination is outside the world",
            Self::DestinationOccupied => "Something already stands there.",
            Self::TargetUnreachable => "No path leads there.",
            Self::InvalidBuildSite => "That spot is not clear for building.",
            Self::InsufficientWood => "Not enough wood for that building.",
            Self::InsufficientStone => "Not enough stone for that building.",
            Self::NotBuildable => "villagers cannot build that",
            Self::NeedsCoast => "A dock must be built along the shore.",
            Self::PopulationCapReached => "Build a house to make room for more villagers.",
            Self::NothingToDeposit => "That villager has nothing to unload.",
            Self::BuildingRefusesCargo => "That building does not take those goods.",
            Self::BuildingNotFound => "building not found",
            Self::BuildingQueueFull => "building queue is full",
            Self::QueuedJobNotFound => "queued task is no longer waiting",
            Self::BuildingUnderConstruction => "building is still under construction",
            Self::BuildingAlreadyComplete => "building is already complete",
            Self::ProductUnavailable => "building cannot produce that item",
            Self::InsufficientFood => {
                return write!(f, "You need {VILLAGER_FOOD_COST} food to train a villager.");
            }
            Self::InsufficientResources(kind) => return write!(f, "insufficient {}", kind.name()),
            Self::InsufficientProductionResources => "insufficient resources for production",
            Self::VillagerRequired => "only villagers can gather or build",
            Self::TechnologyUnavailable => "building cannot research that technology",
            Self::TechnologyAlreadyResearched => "technology is already researched",
            Self::TechnologyInProgress => "technology is already being researched",
            Self::MissingTechnologyPrerequisite => "technology prerequisite is not researched",
            Self::InsufficientResearchResources => {
                return write!(
                    f,
                    "research requires {RESEARCH_FOOD_COST} food and {RESEARCH_WOOD_COST} wood"
                );
            }
            Self::GamePaused => "game is paused; resume to give orders",
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
            health: UNIT_HEALTH,
            id: format!("villager-{number}"),
            kind: UnitKind::Villager,
            cell,
            step: None,
            action: UnitAction::Idle,
            cargo: None,
        };
        let mut world = Self {
            island_id: 0,
            island_origins: starting_origins(),
            seed: island.seed,
            economy_rules: EconomyRules::IslandProgression,
            tick: 0,
            simulation_speed: 1.0,
            terrain: island.terrain,
            explored_cells: Vec::new(),
            units: vec![
                villager(1, island.villagers[0]),
                villager(2, island.villagers[1]),
            ],
            ships: Vec::new(),
            animals: Vec::new(),
            roads: Vec::new(),
            resources: island.resources,
            buildings: vec![town_center("base-1", island.town_center, None)],
            inventories: vec![Stockpile::default()],
            researched_technologies: Vec::new(),
            scenario: ScenarioState::default(),
            next_building_id: 2,
            next_unit_id: 3,
        };
        world.populate_wildlife(0);
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
    Building {
        id: id.into(),
        kind,
        origin,
        construction,
        job: None,
        queue: Vec::new(),
        next_queue_id: 0,
    }
}

impl GameWorld {
    /// Applies a command atomically: on error the world is unchanged.
    pub fn apply_command(&mut self, command: Command) -> Result<(), CommandError> {
        *self = self.clone().with_command(command)?;
        Ok(())
    }

    /// Consume an isolated candidate; callers commit it only after success (and saving).
    pub fn with_command(mut self, command: Command) -> Result<Self, CommandError> {
        if self.simulation_speed == 0.0 && !matches!(command, Command::SetSimulationSpeed { .. }) {
            return Err(CommandError::GamePaused);
        }
        self.execute(command)?;
        #[cfg(debug_assertions)]
        if let Err(error) = self.validate() {
            panic!("an accepted command broke a world invariant: {error}");
        }
        Ok(self)
    }

    fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        match command {
            Command::BuildRoad {
                unit_id,
                start,
                end,
                kind,
            } => self.order_road(&unit_id, start, end, kind)?,
            Command::AttackAnimal {
                unit_ids,
                animal_id,
            } => self.attack_animal(&unit_ids, &animal_id)?,
            Command::TransferShipCargo {
                ship_id,
                kind,
                amount,
                direction,
            } => {
                self.transfer_ship_cargo(&ship_id, kind, amount, direction)?;
            }
            Command::Voyage { ship_id, island_id } => self.voyage(&ship_id, island_id)?,
            Command::Sail { ship_id, to } => self.sail(&ship_id, to)?,
            Command::DockShip {
                ship_id,
                building_id,
            } => self.sail_to_dock(&ship_id, &building_id)?,
            Command::StopShip { ship_id } => {
                let index = self.ship_index(&ship_id)?;
                let to = self.ships[index]
                    .step
                    .map_or(self.ships[index].cell, |step| step.to);
                self.sail(&ship_id, to)?;
            }
            Command::Board { unit_id, ship_id } => self.board(&unit_id, &ship_id)?,
            Command::Disembark { ship_id } => self.disembark(&ship_id)?,
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
                let unit = self.ordered_villager(&unit_id)?;
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
                self.order_build(&unit_id, origin, kind)?;
            }
            Command::Construct {
                unit_id,
                building_id,
            } => {
                let unit = self.ordered_villager(&unit_id)?;
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
                let building = self.building_index_with_queue_space(&building_id)?;
                if !self.building_available(self.buildings[building].kind) {
                    return Err(CommandError::ProductUnavailable);
                }
                self.buildings[building].check_production(
                    product,
                    &self.available_at(self.buildings[building].origin),
                    self.villagers_and_trainees(),
                    self.housing(),
                )?;
                self.spend_at(self.buildings[building].origin, product.cost())?;
                self.buildings[building].enqueue(BuildingJob::Produce {
                    product,
                    elapsed_seconds: 0.0,
                });
            }
            Command::Research {
                building_id,
                technology,
            } => {
                let building = self.building_index_with_queue_space(&building_id)?;
                if !self.technology_available(technology) {
                    return Err(CommandError::TechnologyUnavailable);
                }
                let queued = self.buildings.iter().flat_map(Building::jobs).any(|job| matches!(job, BuildingJob::Research { technology: t, .. } if *t == technology));
                self.buildings[building].check_research(
                    technology,
                    &self.available_at(self.buildings[building].origin),
                    &self.researched_technologies,
                    queued,
                )?;
                self.spend_at(self.buildings[building].origin, RESEARCH_COST)?;
                self.buildings[building].enqueue(BuildingJob::Research {
                    technology,
                    elapsed_seconds: 0.0,
                });
            }
            Command::CancelQueuedJob {
                building_id,
                queue_id,
            } => {
                self.cancel_queued_job(&building_id, queue_id)?;
            }
            Command::Deposit {
                unit_id,
                storage_id,
            } => {
                let unit = self.ordered_unit(&unit_id)?;
                let cargo = self.units[unit]
                    .cargo
                    .as_ref()
                    .ok_or(CommandError::NothingToDeposit)?
                    .kind;
                if let Some(building) = self.buildings.iter().find(|b| b.id == storage_id) {
                    if !building.is_complete() {
                        return Err(CommandError::BuildingUnderConstruction);
                    }
                    if !building.kind.accepts(cargo) {
                        return Err(CommandError::BuildingRefusesCargo);
                    }
                }
                if !self.buildings.iter().any(|b| b.id == storage_id)
                    && !self.ships.iter().any(|s| s.id == storage_id)
                {
                    return Err(CommandError::BuildingNotFound);
                }
                let footprint = self
                    .storage_sites(Some(cargo))
                    .into_iter()
                    .find(|site| site.id == storage_id)
                    .map(|site| site.footprint)
                    .ok_or(CommandError::StorageUnavailable)?;
                if !self.can_reach_beside(unit, footprint) {
                    return Err(CommandError::TargetUnreachable);
                }
                self.units[unit].action = UnitAction::Deposit { storage_id };
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
        housing(self.buildings.iter())
    }

    /// Living units plus active/waiting trainees, including ship passengers.
    pub fn villagers_and_trainees(&self) -> usize {
        population(
            self.units.len(),
            self.ships.iter().map(|s| s.passengers.len()).sum(),
            self.buildings.iter(),
        )
    }

    /// Whether any cell beside the footprint (sharing an edge) is water.
    fn touches_sea(&self, footprint: Footprint) -> bool {
        let water = |column: i32, row: i32| {
            column >= 0
                && row >= 0
                && column < i32::from(self.columns())
                && row < i32::from(self.rows())
                && self.terrain[row as usize * usize::from(self.columns()) + column as usize].biome
                    == TerrainBiome::Water
        };
        dock_facing(footprint, water).is_some()
    }

    fn next_building_name(&self) -> String {
        format!("building-{}", self.next_building_id)
    }

    fn next_unit_name(&self) -> String {
        format!("villager-{}", self.next_unit_id)
    }

    fn building_index_with_queue_space(&self, building_id: &str) -> Result<usize, CommandError> {
        let index = self
            .buildings
            .iter()
            .position(|building| building.id == building_id)
            .ok_or(CommandError::BuildingNotFound)?;
        self.buildings[index].check_queue_space()?;
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

    fn ordered_villager(&mut self, id: &str) -> Result<usize, CommandError> {
        let unit = self.ordered_unit(id)?;
        if self.units[unit].kind != UnitKind::Villager {
            return Err(CommandError::VillagerRequired);
        }
        Ok(unit)
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
                UnitAction::AttackAnimal {
                    animal_id,
                    elapsed_seconds,
                } => self.tick_attack_animal(index, &animal_id, elapsed_seconds, dt),
                UnitAction::Board { .. } => {}
                UnitAction::Move { to } => self.tick_move(index, to, dt),
                UnitAction::BuildRoad { cells } => self.tick_road(index, cells, dt),
                UnitAction::Gather { resource_id, phase } => {
                    self.tick_gather(index, resource_id, phase, dt)
                }
                UnitAction::ExploreBuild { origin, kind } => {
                    self.tick_explore_build(index, origin, kind, dt)
                }
                UnitAction::Build { building_id } => self.tick_build(index, &building_id, dt),
                UnitAction::Cultivate { resource_id } => {
                    self.tick_cultivate(index, &resource_id, dt)
                }
                UnitAction::Deposit { storage_id } => self.tick_deposit(index, &storage_id, dt),
            }
        }
        self.tick_wildlife(dt);
        self.tick_ships(dt);
        self.expand_archipelago();
        for index in (0..self.units.len()).rev() {
            if let UnitAction::Board { ship_id } = self.units[index].action.clone() {
                self.tick_board(index, &ship_id, dt);
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
        let mut visibility = vec![CellVisibility::Unseen; self.terrain.len()];
        let index = |cell: CellCoordinate| {
            usize::from(cell.row) * usize::from(self.columns()) + usize::from(cell.column)
        };
        for &cell in &self.explored_cells {
            visibility[index(cell)] = CellVisibility::Explored;
        }
        for &cell in &visible {
            visibility[index(cell)] = CellVisibility::Visible;
        }

        WorldSnapshot {
            roads: self
                .roads
                .iter()
                .filter(|r| explored.contains(&r.cell))
                .cloned()
                .collect(),
            animals: self
                .animals
                .iter()
                .filter(|a| {
                    visible.contains(&a.cell) && a.step.is_none_or(|s| visible.contains(&s.to))
                })
                .cloned()
                .collect(),
            island_id: self.island_id,
            island_count: self.island_origins.len(),
            island_origins: self.island_origins.clone(),
            available_buildings: self.available_buildings(),
            columns: self.columns(),
            rows: self.rows(),
            tick: self.tick,
            simulation_speed: self.simulation_speed,
            terrain: self
                .terrain
                .iter()
                .zip(visibility)
                .map(|(cell, visibility)| SnapshotTerrainCell {
                    column: cell.column,
                    row: cell.row,
                    biome: (visibility != CellVisibility::Unseen).then_some(cell.biome),
                    elevation: (visibility != CellVisibility::Unseen)
                        .then(|| terrain_codec::quantize(cell.elevation)),
                    visibility,
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
                    let produces = if self.building_available(building.kind) {
                        building.kind.products().to_vec()
                    } else {
                        Vec::new()
                    };
                    let researches = building
                        .kind
                        .researches()
                        .iter()
                        .copied()
                        .filter(|&t| self.technology_available(t))
                        .collect();
                    BuildingView {
                        building: building.clone(),
                        produces,
                        researches,
                        columns,
                        rows,
                    }
                })
                .collect(),
            ships: self.ships.clone(),
            inventories: (0..self.island_origins.len())
                .map(|id| self.available_on(id))
                .collect(),
            stored_inventories: self.inventories.clone(),
            ship_connections: self.ship_connections(),
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
                self.ships
                    .iter()
                    .map(|ship| (ship.position(), UNIT_SIGHT_RADIUS)),
            )
            .chain(
                self.buildings
                    .iter()
                    .filter(|building| building.is_complete())
                    .map(|building| (building.footprint().center(), building.kind.sight_radius())),
            )
            .collect();
        let mut visible = BTreeSet::new();
        for (eye, radius) in eyes {
            let min_column = (eye.x - radius).floor().max(0.0) as u16;
            let max_column = ((eye.x + radius).ceil() as u16).min(self.columns() - 1);
            let min_row = (eye.y - radius).floor().max(0.0) as u16;
            let max_row = ((eye.y + radius).ceil() as u16).min(self.rows() - 1);
            for row in min_row..=max_row {
                for column in min_column..=max_column {
                    let cell = CellCoordinate::new(column, row);
                    if eye.distance(cell.center()) <= radius {
                        visible.insert(cell);
                    }
                }
            }
        }
        visible
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
}

#[cfg(test)]
mod placement_tests;
#[cfg(test)]
mod tests;
