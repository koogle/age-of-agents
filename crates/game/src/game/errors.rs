//! Typed reasons a command is rejected; the world is unchanged when one is returned.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    AnimalNotVisible,
    CannotAttack,
    ShipStorageUnavailable,
    StorageUnavailable,
    InvalidCargoAmount,
    ShipHoldFull,
    ShipNotFound,
    ArtifactUnavailable,
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
            Self::ArtifactUnavailable => "the artifact has already left the temple",
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
