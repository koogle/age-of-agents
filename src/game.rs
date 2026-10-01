use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

mod domain;
mod gathering;
#[cfg(test)]
mod group_move_tests;
mod movement;
mod occupancy;
#[cfg(test)]
mod slice_a_tests;
#[cfg(test)]
mod soundness_tests;

pub use domain::*;
use movement::{Goal, Travel};

pub const WORLD_COLUMNS: u16 = 30;
pub const WORLD_ROWS: u16 = 20;
/// Sight radii in cells.
pub const UNIT_SIGHT_RADIUS: f64 = 4.0;
pub const BUILDING_SIGHT_RADIUS: f64 = 6.0;
pub const TOWN_CENTER_WOOD_COST: f64 = 20.0;
pub const BUILD_SECONDS: f64 = 4.0;
pub const VILLAGER_FOOD_COST: f64 = 50.0;
pub const VILLAGER_PRODUCTION_SECONDS: f64 = 6.0;
pub const RESEARCH_FOOD_COST: f64 = 40.0;
pub const RESEARCH_WOOD_COST: f64 = 20.0;
pub const RESEARCH_SECONDS: f64 = 8.0;
pub const GATHERING_TECH_MULTIPLIER: f64 = 1.2;
/// Resource spacing and starting-base clearance, in cells.
pub const RESOURCE_MIN_SEPARATION: f64 = 1.5;
pub const STARTING_BASE_RESOURCE_CLEARANCE: f64 = 2.5;
/// Walking speed in cells per second.
const MOVE_SPEED: f64 = 1.5;
pub(crate) const GATHER_RATE: f64 = 2.0;
pub const VILLAGER_CARRY_CAPACITY: f64 = 20.0;
const STARTING_TOWN_CENTER: CellCoordinate = CellCoordinate::new(14, 9);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameWorld {
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnitView {
    #[serde(flatten)]
    pub unit: Unit,
    /// Interpolated body position in cell units.
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BuildingView {
    #[serde(flatten)]
    pub building: Building,
    pub columns: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorldSnapshot {
    pub columns: u16,
    pub rows: u16,
    pub tick: u64,
    pub simulation_speed: f64,
    pub terrain: Vec<SnapshotTerrainCell>,
    pub units: Vec<UnitView>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<BuildingView>,
    pub stockpile: Stockpile,
    pub researched_technologies: Vec<TechnologyKind>,
    pub catalog: DomainCatalog,
    pub scenario: ScenarioState,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
    /// Place a town center foundation with its north-west corner at `origin`.
    Build {
        unit_id: String,
        origin: CellCoordinate,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    UnitNotFound,
    EmptyUnitGroup,
    DuplicateUnit,
    ResourceNotFound,
    ResourceDepleted,
    UnitBusy,
    InvalidDestination,
    DestinationOccupied,
    TargetUnreachable,
    InvalidBuildSite,
    InsufficientWood,
    BuildingNotFound,
    BuildingBusy,
    BuildingUnderConstruction,
    BuildingAlreadyComplete,
    ProductUnavailable,
    InsufficientFood,
    TechnologyUnavailable,
    TechnologyAlreadyResearched,
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
            Self::UnitBusy => "unit is busy",
            Self::InvalidDestination => "destination is outside the world",
            Self::DestinationOccupied => "destination cell is occupied",
            Self::TargetUnreachable => "target is unreachable",
            Self::InvalidBuildSite => "build site is blocked or outside the world",
            Self::InsufficientWood => "insufficient wood",
            Self::BuildingNotFound => "building not found",
            Self::BuildingBusy => "building is already producing",
            Self::BuildingUnderConstruction => "building is still under construction",
            Self::BuildingAlreadyComplete => "building is already complete",
            Self::ProductUnavailable => "building cannot produce that item",
            Self::InsufficientFood => "insufficient food",
            Self::TechnologyUnavailable => "building cannot research that technology",
            Self::TechnologyAlreadyResearched => "technology is already researched",
            Self::MissingTechnologyPrerequisite => "technology prerequisite is not researched",
            Self::InsufficientResearchResources => "research requires 40 food and 20 wood",
            Self::InvalidSimulationSpeed => "simulation speed must be 0, 1, or 2",
        };
        f.write_str(message)
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        let terrain = generate_terrain();
        let villager = |number: u64, column, row| Unit {
            id: format!("villager-{number}"),
            kind: UnitKind::Villager,
            cell: CellCoordinate::new(column, row),
            step: None,
            action: UnitAction::Idle,
            cargo: None,
        };
        let mut world = Self {
            tick: 0,
            simulation_speed: 1.0,
            terrain: terrain.clone(),
            explored_cells: Vec::new(),
            units: vec![villager(1, 14, 11), villager(2, 15, 11)],
            resources: generate_resources(&terrain),
            buildings: vec![town_center("base-1", STARTING_TOWN_CENTER, None)],
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

fn town_center(id: &str, origin: CellCoordinate, construction: Option<f64>) -> Building {
    Building {
        id: id.into(),
        kind: BuildingKind::TownCenter,
        origin,
        construction,
        produces: vec![ProductKind::Villager],
        researches: TechnologyKind::ALL.to_vec(),
        job: None,
    }
}

fn generate_terrain() -> Vec<TerrainCell> {
    const SITES: [(u16, u16, TerrainBiome); 8] = [
        (3, 3, TerrainBiome::Meadow),
        (11, 2, TerrainBiome::Forest),
        (21, 3, TerrainBiome::Prairie),
        (27, 7, TerrainBiome::Highland),
        (4, 14, TerrainBiome::Wetland),
        (12, 17, TerrainBiome::Scrubland),
        (20, 13, TerrainBiome::Heath),
        (27, 17, TerrainBiome::Clayland),
    ];

    let mut terrain = Vec::with_capacity(usize::from(WORLD_COLUMNS * WORLD_ROWS));
    for row in 0..WORLD_ROWS {
        for column in 0..WORLD_COLUMNS {
            let (_, _, biome) = SITES
                .iter()
                .min_by_key(|(site_column, site_row, _)| {
                    let dx = i32::from(column) - i32::from(*site_column);
                    let dy = i32::from(row) - i32::from(*site_row);
                    dx * dx + dy * dy
                })
                .expect("the fixed Voronoi map has sites");
            terrain.push(TerrainCell {
                column,
                row,
                biome: *biome,
            });
        }
    }
    terrain
}

fn compatible_biomes(kind: ResourceKind) -> &'static [TerrainBiome] {
    match kind {
        ResourceKind::Wood => &[TerrainBiome::Forest, TerrainBiome::Heath],
        ResourceKind::Food => &[TerrainBiome::Meadow, TerrainBiome::Prairie],
        ResourceKind::Stone => &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        ResourceKind::Gold => &[TerrainBiome::Highland],
        ResourceKind::Iron => &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        ResourceKind::Clay => &[TerrainBiome::Clayland, TerrainBiome::Wetland],
        ResourceKind::Fiber => &[TerrainBiome::Wetland, TerrainBiome::Prairie],
        ResourceKind::Coal
        | ResourceKind::Timber
        | ResourceKind::Steel
        | ResourceKind::Bricks
        | ResourceKind::Cloth
        | ResourceKind::Rations => &[],
    }
}

fn generate_resources(terrain: &[TerrainCell]) -> Vec<ResourceNode> {
    const SPECS: [(ResourceKind, &str, usize, f64); 7] = [
        (ResourceKind::Wood, "tree", 6, 25.0),
        (ResourceKind::Food, "berries", 4, 50.0),
        (ResourceKind::Stone, "stone", 4, 40.0),
        (ResourceKind::Gold, "gold", 2, 35.0),
        (ResourceKind::Iron, "iron", 2, 40.0),
        (ResourceKind::Clay, "clay", 2, 45.0),
        (ResourceKind::Fiber, "fiber", 2, 50.0),
    ];
    let base = Position { x: 15.0, y: 10.0 };

    let mut resources: Vec<ResourceNode> = Vec::new();
    for (kind, prefix, count, amount) in SPECS {
        for number in 1..=count {
            let cell = terrain
                .iter()
                .filter(|cell| compatible_biomes(kind).contains(&cell.biome))
                .filter(|cell| {
                    let center = cell.coordinate().center();
                    center.distance(base) >= STARTING_BASE_RESOURCE_CLEARANCE
                        && resources.iter().all(|resource| {
                            resource.cell.center().distance(center) + f64::EPSILON
                                >= RESOURCE_MIN_SEPARATION
                        })
                })
                .min_by_key(|cell| {
                    let dx = i32::from(cell.column) - i32::from(WORLD_COLUMNS / 2);
                    let dy = i32::from(cell.row) - i32::from(WORLD_ROWS / 2);
                    (dx * dx + dy * dy, cell.row, cell.column)
                })
                .expect("fixed terrain has enough separated biome-compatible resource cells");
            resources.push(ResourceNode {
                id: format!("{prefix}-{number}"),
                kind,
                cell: cell.coordinate(),
                amount,
                capacity: amount,
            });
        }
    }
    resources
}

impl GameWorld {
    /// Applies a command atomically: on error the world is unchanged.
    pub fn apply_command(&mut self, command: Command) -> Result<(), CommandError> {
        self.execute(command)?;
        #[cfg(debug_assertions)]
        if let Err(error) = self.validate() {
            panic!("an accepted command broke a world invariant: {error}");
        }
        Ok(())
    }

    fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        match command {
            Command::Move { unit_id, to } => {
                let unit = self.unit_index_and_idle(&unit_id)?;
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
                    let index = self.unit_index_and_idle(&unit_id)?;
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
                let unit = self.unit_index_and_idle(&unit_id)?;
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
                    phase: GatherPhase::ToResource,
                };
            }
            Command::Build { unit_id, origin } => {
                let unit = self.unit_index_and_idle(&unit_id)?;
                let (columns, rows) = BuildingKind::TownCenter.size();
                let footprint = Footprint {
                    origin,
                    columns,
                    rows,
                };
                if !self.footprint_is_free(footprint) {
                    return Err(CommandError::InvalidBuildSite);
                }
                if self.stockpile.wood < TOWN_CENTER_WOOD_COST {
                    return Err(CommandError::InsufficientWood);
                }
                // Validate reachability against the world as it will be, with
                // the foundation in place; roll back if the builder is cut off.
                let id = self.next_building_name();
                self.buildings.push(town_center(&id, origin, Some(0.0)));
                if !self.can_reach_beside(unit, footprint) {
                    self.buildings.pop();
                    return Err(CommandError::TargetUnreachable);
                }
                self.next_building_id += 1;
                self.stockpile.wood -= TOWN_CENTER_WOOD_COST;
                self.units[unit].action = UnitAction::Build { building_id: id };
            }
            Command::Construct {
                unit_id,
                building_id,
            } => {
                let unit = self.unit_index_and_idle(&unit_id)?;
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
                if !self.buildings[building].produces.contains(&product) {
                    return Err(CommandError::ProductUnavailable);
                }
                match product {
                    ProductKind::Villager if self.stockpile.food < VILLAGER_FOOD_COST => {
                        return Err(CommandError::InsufficientFood);
                    }
                    ProductKind::Villager => self.stockpile.food -= VILLAGER_FOOD_COST,
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
            Command::SetSimulationSpeed { multiplier } => {
                if ![0.0, 1.0, 2.0].contains(&multiplier) {
                    return Err(CommandError::InvalidSimulationSpeed);
                }
                self.simulation_speed = multiplier;
            }
        }
        Ok(())
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

    fn unit_index_and_idle(&self, unit_id: &str) -> Result<usize, CommandError> {
        let index = self
            .units
            .iter()
            .position(|unit| unit.id == unit_id)
            .ok_or(CommandError::UnitNotFound)?;
        if self.units[index].action != UnitAction::Idle {
            return Err(CommandError::UnitBusy);
        }
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
                    .map(|building| (building.footprint().center(), BUILDING_SIGHT_RADIUS)),
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
        if work + f64::EPSILON < BUILD_SECONDS {
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

    fn tick_building_job(&mut self, building_index: usize, dt: f64) {
        let Some(job) = self.buildings[building_index].job.clone() else {
            return;
        };
        match job {
            BuildingJob::Produce {
                product,
                mut elapsed_seconds,
            } => {
                elapsed_seconds += dt;
                if elapsed_seconds + f64::EPSILON < VILLAGER_PRODUCTION_SECONDS {
                    self.buildings[building_index].job = Some(BuildingJob::Produce {
                        product,
                        elapsed_seconds,
                    });
                    return;
                }
                match product {
                    ProductKind::Villager => {
                        let Some(cell) = self.spawn_cell(building_index) else {
                            self.buildings[building_index].job = Some(BuildingJob::Produce {
                                product,
                                elapsed_seconds,
                            });
                            return;
                        };
                        self.units.push(Unit {
                            id: self.next_unit_name(),
                            kind: UnitKind::Villager,
                            cell,
                            step: None,
                            action: UnitAction::Idle,
                            cargo: None,
                        });
                        self.next_unit_id += 1;
                    }
                }
            }
            BuildingJob::Research {
                technology,
                mut elapsed_seconds,
            } => {
                elapsed_seconds += dt;
                if elapsed_seconds + f64::EPSILON < RESEARCH_SECONDS {
                    self.buildings[building_index].job = Some(BuildingJob::Research {
                        technology,
                        elapsed_seconds,
                    });
                    return;
                }
                self.researched_technologies.push(technology);
                self.researched_technologies.sort_unstable();
            }
        }
        self.buildings[building_index].job = None;
    }
}

#[cfg(test)]
mod tests;
