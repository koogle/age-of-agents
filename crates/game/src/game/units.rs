//! NPC/unit identity, position and movement state. Orders live in `jobs`.

use serde::{Deserialize, Serialize};

use super::{CarriedResource, CellCoordinate, Position, UnitAction};

pub(super) mod movement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
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
    /// The body claims both ends of an unfinished step until it arrives.
    pub fn claimed_cells(&self) -> impl Iterator<Item = CellCoordinate> + '_ {
        std::iter::once(self.cell).chain(self.step.map(|step| step.to))
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

impl UnitKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Villager => "Villager",
            Self::Guard => "Guard",
            Self::Archer => "Archer",
            Self::Healer => "Healer",
            Self::SiegeCart => "Siege cart",
        }
    }
}

pub const ROADMAP_UNITS: [UnitKind; 5] = [
    UnitKind::Villager,
    UnitKind::Guard,
    UnitKind::Archer,
    UnitKind::Healer,
    UnitKind::SiegeCart,
];
