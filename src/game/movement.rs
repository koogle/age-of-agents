//! Cell-by-cell movement. A unit claims the next cell before stepping into it
//! and releases its old cell only when the step completes, so two bodies never
//! share a cell, even mid-stride.

use std::collections::BTreeSet;

use super::occupancy::{Occupancy, in_bounds};
use super::*;
use crate::navigation::{PathTree, offset};

#[derive(Debug, Clone, Copy)]
pub(super) enum Goal {
    Cell(CellCoordinate),
    /// Any free cell touching the footprint, including diagonally.
    Beside(Footprint),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Travel {
    Arrived {
        remaining: f64,
    },
    EnRoute,
    /// No route exists even if every other unit stepped aside.
    Unreachable,
}

impl GameWorld {
    /// Walks `unit` toward `goal` for up to `dt` seconds.
    pub(super) fn travel(&mut self, unit: usize, goal: Goal, mut dt: f64) -> Travel {
        loop {
            if let Some(step) = self.units[unit].step {
                let length = self.units[unit].cell.center().distance(step.to.center());
                let progress = step.progress + dt * MOVE_SPEED / length;
                if progress < 1.0 {
                    self.units[unit].step = Some(Step {
                        to: step.to,
                        progress,
                    });
                    return Travel::EnRoute;
                }
                dt = (progress - 1.0) * length / MOVE_SPEED;
                self.units[unit].cell = step.to;
                self.units[unit].step = None;
            }
            let occupancy = self.occupancy();
            if self.has_arrived(unit, goal, &occupancy) {
                return Travel::Arrived { remaining: dt };
            }
            if dt <= 0.0 {
                return Travel::EnRoute;
            }
            match self.next_step(unit, goal, &occupancy) {
                NextStep::Step(to) => {
                    self.units[unit].step = Some(Step { to, progress: 0.0 });
                }
                NextStep::Wait => return Travel::EnRoute,
                NextStep::Unreachable => return Travel::Unreachable,
            }
        }
    }

    fn has_arrived(&self, unit: usize, goal: Goal, occupancy: &Occupancy) -> bool {
        let here = self.units[unit].cell;
        self.units[unit].step.is_none()
            && match goal {
                Goal::Cell(cell) => here == cell,
                Goal::Beside(footprint) => {
                    footprint.is_interaction_cell(here)
                        && !occupancy.reserved_by_other(here, Some(unit))
                }
            }
    }

    fn goal_cells(&self, unit: usize, goal: Goal, occupancy: &Occupancy) -> Vec<CellCoordinate> {
        match goal {
            Goal::Cell(cell) => vec![cell],
            Goal::Beside(footprint) => interaction_cells(footprint)
                .filter(|cell| !occupancy.is_static(*cell))
                .filter(|cell| !occupancy.reserved_by_other(*cell, Some(unit)))
                .collect(),
        }
    }

    /// Prefers a route around other units; if none exists, approaches along the
    /// route through them and waits for them to clear.
    fn next_step(&self, unit: usize, goal: Goal, occupancy: &Occupancy) -> NextStep {
        let start = self.units[unit].cell;
        let goals = self.goal_cells(unit, goal, occupancy);
        let clear = |cell| !occupancy.is_static(cell) && !occupancy.has_other_unit(cell, unit);
        let around = PathTree::search(WORLD_COLUMNS, WORLD_ROWS, start, clear);
        if let Some(to) = around
            .nearest(goals.iter().copied().filter(|cell| clear(*cell)))
            .and_then(|target| around.first_step(target))
        {
            return NextStep::Step(to);
        }
        let through = PathTree::search(WORLD_COLUMNS, WORLD_ROWS, start, |cell| {
            !occupancy.is_static(cell)
        });
        let Some(target) = through.nearest(goals) else {
            return NextStep::Unreachable;
        };
        match through.first_step(target) {
            Some(to) if step_is_clear(start, to, clear) => NextStep::Step(to),
            _ => NextStep::Wait,
        }
    }

    /// Path tree over cells that are not statically blocked.
    pub(super) fn static_paths(&self, unit: usize, occupancy: &Occupancy) -> PathTree {
        PathTree::search(WORLD_COLUMNS, WORLD_ROWS, self.units[unit].cell, |cell| {
            !occupancy.is_static(cell)
        })
    }

    /// Whether `unit` could stand beside `footprint` once other units move aside.
    pub(super) fn can_reach_beside(&self, unit: usize, footprint: Footprint) -> bool {
        let occupancy = self.occupancy();
        let goals = self.goal_cells(unit, Goal::Beside(footprint), &occupancy);
        self.static_paths(unit, &occupancy).nearest(goals).is_some()
    }

    pub(super) fn validate_move_destination(
        &self,
        unit: usize,
        to: CellCoordinate,
    ) -> Result<(), CommandError> {
        if !in_bounds(to) {
            return Err(CommandError::InvalidDestination);
        }
        let occupancy = self.occupancy();
        if !occupancy.is_free_for(to, Some(unit)) {
            return Err(CommandError::DestinationOccupied);
        }
        self.static_paths(unit, &occupancy)
            .cost(to)
            .map(|_| ())
            .ok_or(CommandError::TargetUnreachable)
    }

    /// Distinct, reachable, unreserved stopping cells for a group, nearest the
    /// target first. Members may take each other's current cells because every
    /// member is about to move.
    pub(super) fn group_move_assignments(
        &self,
        members: &[usize],
        target: CellCoordinate,
    ) -> Result<Vec<(usize, CellCoordinate)>, CommandError> {
        if !in_bounds(target) {
            return Err(CommandError::InvalidDestination);
        }
        let occupancy = self.occupancy();
        let mut candidates: Vec<_> = self
            .terrain
            .iter()
            .map(|cell| cell.coordinate())
            .filter(|cell| match occupancy.claim(*cell) {
                None => !occupancy.reserved_by_other(*cell, None),
                Some(occupancy::Claim::Unit(owner)) => members.contains(&owner),
                Some(_) => false,
            })
            .collect();
        candidates.sort_by_key(|cell| {
            let dx = i32::from(cell.column) - i32::from(target.column);
            let dy = i32::from(cell.row) - i32::from(target.row);
            (dx * dx + dy * dy, *cell)
        });
        let mut taken = BTreeSet::new();
        let mut assignments = Vec::with_capacity(members.len());
        for &unit in members {
            let paths = self.static_paths(unit, &occupancy);
            let destination = candidates
                .iter()
                .copied()
                .filter(|cell| !taken.contains(cell))
                .find(|cell| paths.cost(*cell).is_some())
                .ok_or(CommandError::TargetUnreachable)?;
            taken.insert(destination);
            assignments.push((unit, destination));
        }
        Ok(assignments)
    }

    pub(super) fn tick_move(&mut self, unit: usize, to: CellCoordinate, dt: f64) {
        // An unreachable destination (walled off after the order) is abandoned
        // rather than leaving the unit permanently busy.
        if self.travel(unit, Goal::Cell(to), dt) != Travel::EnRoute {
            self.units[unit].action = UnitAction::Idle;
        }
    }

    pub(super) fn is_beside(&self, unit: usize, footprint: Footprint) -> bool {
        self.units[unit].step.is_none() && footprint.is_interaction_cell(self.units[unit].cell)
    }

    /// The complete town center with the cheapest reachable drop-off cell.
    pub(super) fn nearest_reachable_town_center(&self, unit: usize) -> Option<Footprint> {
        let occupancy = self.occupancy();
        let paths = self.static_paths(unit, &occupancy);
        self.buildings
            .iter()
            .filter(|building| building.kind == BuildingKind::TownCenter && building.is_complete())
            .filter_map(|building| {
                let footprint = building.footprint();
                let goals = self.goal_cells(unit, Goal::Beside(footprint), &occupancy);
                let cost = paths.cost(paths.nearest(goals)?)?;
                Some(((cost, &building.id), footprint))
            })
            .min_by(|left, right| left.0.cmp(&right.0))
            .map(|(_, footprint)| footprint)
    }

    /// A free cell beside the building for a newly trained unit, if any.
    pub(super) fn spawn_cell(&self, building: usize) -> Option<CellCoordinate> {
        let occupancy = self.occupancy();
        let mut cells: Vec<_> = interaction_cells(self.buildings[building].footprint())
            .filter(|cell| occupancy.is_free_for(*cell, None))
            .collect();
        // Prefer the side facing the default camera (south, then east).
        cells.sort_by_key(|cell| (std::cmp::Reverse(cell.row), std::cmp::Reverse(cell.column)));
        cells.first().copied()
    }

    /// Whether a new footprint fits: in bounds and over cells nobody claims or reserves.
    pub(super) fn footprint_is_free(&self, footprint: Footprint) -> bool {
        let occupancy = self.occupancy();
        footprint
            .cells()
            .all(|cell| in_bounds(cell) && occupancy.is_free_for(cell, None))
    }
}

enum NextStep {
    Step(CellCoordinate),
    Wait,
    Unreachable,
}

/// The ring of in-bounds cells touching `footprint`, in cell order.
pub(super) fn interaction_cells(footprint: Footprint) -> impl Iterator<Item = CellCoordinate> {
    let origin = footprint.origin;
    let columns = i32::from(footprint.columns);
    let rows = i32::from(footprint.rows);
    (-1..=rows).flat_map(move |dy| {
        (-1..=columns).filter_map(move |dx| {
            let ring = dx == -1 || dy == -1 || dx == columns || dy == rows;
            ring.then(|| offset(origin, dx, dy, WORLD_COLUMNS, WORLD_ROWS))
                .flatten()
        })
    })
}

/// A diagonal step must not brush past an occupied corner.
fn step_is_clear(
    from: CellCoordinate,
    to: CellCoordinate,
    clear: impl Fn(CellCoordinate) -> bool,
) -> bool {
    clear(to)
        && (from.column == to.column
            || from.row == to.row
            || (clear(CellCoordinate::new(to.column, from.row))
                && clear(CellCoordinate::new(from.column, to.row))))
}
