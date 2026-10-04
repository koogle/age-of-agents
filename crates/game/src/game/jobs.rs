//! Activities that progress over simulation time: unit orders and building jobs.
//! Existing typed states and update order are preserved.

use serde::{Deserialize, Serialize};

use super::{CellCoordinate, TechnologyKind};

mod construction;
mod fields;
mod gathering;
mod production;

pub use fields::{FIELD_COST, FIELD_FOOD, FIELD_WORK_SECONDS};
pub use gathering::NEXT_RESOURCE_RADIUS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnitAction {
    Idle,
    /// Walk to `to`. The destination cell is reserved for this unit until it arrives.
    Move {
        to: CellCoordinate,
    },
    Gather {
        resource_id: String,
        phase: GatherPhase,
    },
    /// Walk beside the foundation `building_id` and raise it.
    Build {
        building_id: String,
    },
    /// Prepare or replenish a cultivated food field.
    Cultivate {
        resource_id: String,
    },
    /// Carry the load to the complete building `building_id`, unload it
    /// there, and stand idle.
    Deposit {
        building_id: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductKind {
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
    Produce {
        product: ProductKind,
        elapsed_seconds: f64,
    },
    Research {
        technology: TechnologyKind,
        elapsed_seconds: f64,
    },
}
