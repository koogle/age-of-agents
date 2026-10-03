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
        let through = self.static_paths(unit, occupancy);
        let Some(target) = through.nearest(goals) else {
            // Reservations are temporary. Keep the order when the geometry
            // still allows an approach once the reserving unit moves away.
            if let Goal::Beside(footprint) = goal
                && through
                    .nearest(
                        interaction_cells(footprint).filter(|cell| !occupancy.is_static(*cell)),
                    )
                    .is_some()
            {
                return NextStep::Wait;
            }
            return NextStep::Unreachable;
        };
        match through.first_step(target) {
            Some(to) if step_is_clear(start, to, clear) => {
                if self.yields_contested_cell(unit, to, occupancy) {
                    NextStep::Wait
                } else {
                    NextStep::Step(to)
                }
            }
            Some(to) => self
                .yield_step(unit, to, occupancy)
                .map_or(NextStep::Wait, NextStep::Step),
            None => NextStep::Wait,
        }
    }

    /// Breaks a head-on standoff: when the unit standing in our way is itself
    /// waiting to step into our cell, the lower-indexed unit side-steps.
    fn yield_step(
        &self,
        unit: usize,
        blocked: CellCoordinate,
        occupancy: &Occupancy,
    ) -> Option<CellCoordinate> {
        let Some(occupancy::Claim::Unit(other)) = occupancy.claim(blocked) else {
            return None;
        };
        let here = self.units[unit].cell;
        if unit > other || self.units[other].step.is_some() {
            return None;
        }
        let goal = self.walking_goal(other)?;
        let paths = self.static_paths(other, occupancy);
        let route = paths.path_to(paths.nearest(self.goal_cells(other, goal, occupancy))?);
        if route.first() != Some(&here) {
            return None;
        }
        // Prefer stepping off the other unit's route; otherwise back away along it.
        let clear = |cell| !occupancy.is_static(cell) && !occupancy.has_other_unit(cell, unit);
        let mut options: Vec<_> = [
            (0, -1),
            (-1, 0),
            (1, 0),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ]
        .into_iter()
        .filter_map(|(dx, dy)| offset(here, dx, dy, WORLD_COLUMNS, WORLD_ROWS))
        .filter(|cell| {
            *cell != blocked
                && !occupancy.reserved_by_other(*cell, Some(unit))
                && step_is_clear(here, *cell, clear)
        })
        .collect();
        options.sort_by_key(|cell| route.contains(cell));
        options.first().copied()
    }

    /// The first cell a waiting unit wants to step into, if it is walking.
    fn wanted_step(&self, unit: usize, occupancy: &Occupancy) -> Option<CellCoordinate> {
        let goal = self.walking_goal(unit)?;
        let paths = self.static_paths(unit, occupancy);
        paths.first_step(paths.nearest(self.goal_cells(unit, goal, occupancy))?)
    }

    /// When several stalled units want the same free cell, the highest index
    /// takes it, so a unit that just side-stepped cannot starve the one it let by.
    fn yields_contested_cell(
        &self,
        unit: usize,
        to: CellCoordinate,
        occupancy: &Occupancy,
    ) -> bool {
        self.units.iter().enumerate().any(|(other, body)| {
            other > unit
                && body.step.is_none()
                && body.cell.touches(to)
                && self.wanted_step(other, occupancy) == Some(to)
        })
    }

    /// Where a unit is currently trying to walk, if anywhere.
    fn walking_goal(&self, unit: usize) -> Option<Goal> {
        match &self.units[unit].action {
            UnitAction::Move { to } => Some(Goal::Cell(*to)),
            UnitAction::Cultivate { resource_id } => {
                let field_goal = || {
                    self.resources
                        .iter()
                        .find(|r| &r.id == resource_id)
                        .map(|r| Goal::Beside(r.footprint()))
                };
                if self.units[unit].cargo.is_some() {
                    self.nearest_drop_site(unit)
                        .map(Goal::Beside)
                        .or_else(field_goal)
                } else {
                    field_goal()
                }
            }
            UnitAction::Build { .. } if self.units[unit].cargo.is_some() => {
                // Dropping goods off first; see `drop_off_before_building`.
                self.nearest_drop_site(unit).map(Goal::Beside).or_else(|| {
                    let UnitAction::Build { building_id } = &self.units[unit].action else {
                        return None;
                    };
                    self.buildings
                        .iter()
                        .find(|building| &building.id == building_id)
                        .map(|building| Goal::Beside(building.footprint()))
                })
            }
            UnitAction::Build { building_id } | UnitAction::Deposit { building_id } => self
                .buildings
                .iter()
                .find(|building| &building.id == building_id)
                .map(|building| Goal::Beside(building.footprint())),
            UnitAction::Gather {
                resource_id,
                phase: GatherPhase::ToResource,
            } => self
                .resources
                .iter()
                .find(|resource| &resource.id == resource_id)
                .map(|resource| Goal::Beside(resource.footprint())),
            UnitAction::Gather {
                phase: GatherPhase::Returning,
                ..
            } => self.nearest_drop_site(unit).map(Goal::Beside),
            _ => None,
        }
    }

    /// Path tree over cells that are not statically blocked.
    pub(super) fn static_paths(&self, unit: usize, occupancy: &Occupancy) -> PathTree {
        PathTree::search(WORLD_COLUMNS, WORLD_ROWS, self.units[unit].cell, |cell| {
            !occupancy.is_static(cell)
        })
    }

    /// A foundation must not split any unit's reachable ground. Foundations
    /// already block the full completed footprint, so check before placing one.
    /// Ignore temporary unit claims and reservations, just like static routing.
    pub(super) fn placement_preserves_routes(&self, footprint: Footprint) -> bool {
        let occupancy = self.occupancy();
        let mut checked: Vec<PathTree> = Vec::new();
        self.units.iter().all(|unit| {
            let start = unit.step.map_or(unit.cell, |step| step.to);
            // Units sharing connected ground need only one pair of searches.
            if checked.iter().any(|paths| paths.cost(start).is_some()) {
                return true;
            }
            let before = PathTree::search(WORLD_COLUMNS, WORLD_ROWS, start, |cell| {
                !occupancy.is_static(cell)
            });
            let after = PathTree::search(WORLD_COLUMNS, WORLD_ROWS, start, |cell| {
                !footprint.contains(cell) && !occupancy.is_static(cell)
            });
            // Even if every lost cell belongs to the new building, a unit
            // that could walk before must still have somewhere to step.
            let body = Footprint {
                origin: start,
                columns: 1,
                rows: 1,
            };
            if before.nearest(interaction_cells(body)).is_some()
                && after.nearest(interaction_cells(body)).is_none()
            {
                return false;
            }
            let preserved = self.terrain.iter().all(|terrain| {
                let cell = terrain.coordinate();
                footprint.contains(cell)
                    || before.cost(cell).is_none()
                    || after.cost(cell).is_some()
            });
            checked.push(after);
            preserved
        })
    }

    /// Whether `unit` could stand beside `footprint` once other units move aside.
    pub(super) fn can_reach_beside(&self, unit: usize, footprint: Footprint) -> bool {
        let occupancy = self.occupancy();
        let goals = interaction_cells(footprint).filter(|cell| !occupancy.is_static(*cell));
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
        if !self.can_reserve(to, &[unit], &occupancy) {
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
            .filter(|cell| self.can_reserve(*cell, members, &occupancy))
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

    /// Whether `movers` may reserve `cell` as a stopping place. A cell is
    /// acceptable while another unit is merely walking through it: walkers never
    /// stop on a cell reserved by someone else, so the cell is guaranteed to clear.
    /// Units at rest, or idle units finishing a step, would never leave.
    fn can_reserve(&self, cell: CellCoordinate, movers: &[usize], occupancy: &Occupancy) -> bool {
        if !in_bounds(cell) || occupancy.is_static(cell) {
            return false;
        }
        if occupancy
            .reservation(cell)
            .is_some_and(|owner| !movers.contains(&owner))
        {
            return false;
        }
        match occupancy.claim(cell) {
            Some(occupancy::Claim::Unit(owner)) if !movers.contains(&owner) => {
                let other = &self.units[owner];
                other.step.is_some() && other.action != UnitAction::Idle
            }
            _ => true,
        }
    }

    /// An idle unit that came to rest on someone else's reserved destination
    /// steps to the nearest cell it may reserve, so every reservation stays reachable.
    pub(super) fn make_way(&mut self, unit: usize) {
        let occupancy = self.occupancy();
        let here = self.units[unit].cell;
        if self.units[unit].step.is_some() || !occupancy.reserved_by_other(here, Some(unit)) {
            return;
        }
        let paths = self.static_paths(unit, &occupancy);
        let spot = self
            .terrain
            .iter()
            .map(|cell| cell.coordinate())
            // Only fully free cells: yielding into someone's path could swap-deadlock.
            .filter(|cell| *cell != here && occupancy.is_free_for(*cell, None))
            .filter_map(|cell| Some((paths.cost(cell)?, cell)))
            .min();
        if let Some((_, to)) = spot {
            self.units[unit].action = UnitAction::Move { to };
        }
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

    /// The complete building that takes the unit's cargo (a town center takes
    /// anything, a granary food and fiber) with the cheapest available route.
    /// Prefer routes clear of other villagers; wait only when all sites are busy.
    pub(super) fn nearest_drop_site(&self, unit: usize) -> Option<Footprint> {
        let cargo = self.units[unit].cargo.as_ref().map(|cargo| cargo.kind);
        let occupancy = self.occupancy();
        let paths = self.static_paths(unit, &occupancy);
        let clear = PathTree::search(WORLD_COLUMNS, WORLD_ROWS, self.units[unit].cell, |cell| {
            !occupancy.is_static(cell) && !occupancy.has_other_unit(cell, unit)
        });
        self.buildings
            .iter()
            .filter(|building| {
                building.is_complete()
                    && match cargo {
                        Some(kind) => building.kind.accepts(kind),
                        None => building.kind == BuildingKind::TownCenter,
                    }
            })
            .filter_map(|building| {
                let footprint = building.footprint();
                let goals = self.goal_cells(unit, Goal::Beside(footprint), &occupancy);
                let available = clear
                    .nearest(
                        goals
                            .iter()
                            .copied()
                            .filter(|cell| !occupancy.has_other_unit(*cell, unit)),
                    )
                    .and_then(|cell| clear.cost(cell));
                let cost = available.or_else(|| {
                    paths
                        .nearest(
                            interaction_cells(footprint).filter(|cell| !occupancy.is_static(*cell)),
                        )
                        .and_then(|cell| paths.cost(cell))
                })?;
                Some(((available.is_none(), cost, &building.id), footprint))
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
