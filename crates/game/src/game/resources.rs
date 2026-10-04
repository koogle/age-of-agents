//! Resource kinds, world nodes, carried goods, stockpiles and the resource-chain catalog.

use serde::{Deserialize, Serialize};

use super::{BuildingKind, CellCoordinate, Footprint};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarriedResource {
    pub kind: ResourceKind,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: String,
    pub kind: ResourceKind,
    pub cell: CellCoordinate,
    pub amount: f64,
    pub capacity: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stockpile {
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
    pub(super) fn entries(&self) -> [(&'static str, f64); 13] {
        [
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

impl ResourceKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wood => "wood",
            Self::Food => "food",
            Self::Stone => "stone",
            Self::Gold => "gold",
            Self::Iron => "iron",
            Self::Coal => "coal",
            Self::Clay => "clay",
            Self::Fiber => "fiber",
            Self::Timber => "timber",
            Self::Steel => "steel",
            Self::Bricks => "bricks",
            Self::Cloth => "cloth",
            Self::Rations => "rations",
        }
    }
}

pub const ROADMAP_RESOURCES: [ResourceKind; 13] = [
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
