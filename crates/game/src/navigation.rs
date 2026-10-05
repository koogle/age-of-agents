//! Deterministic grid search. Callers decide which cells are passable; this
//! module only knows geometry and step costs.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::game::CellCoordinate;

/// Orthogonal and diagonal step costs, an integer approximation of 1 : sqrt(2).
const ORTHOGONAL_COST: u32 = 2;
const DIAGONAL_COST: u32 = 3;

/// Shortest-path tree from one start cell over an 8-connected grid.
pub struct PathTree {
    columns: u16,
    start: CellCoordinate,
    cost: Vec<u32>,
    previous: Vec<Option<CellCoordinate>>,
}

impl PathTree {
    /// Dijkstra from `start`. A diagonal step is allowed only when both cells it
    /// cuts past are passable, so bodies never clip a blocked corner. The start
    /// cell itself never needs to be passable.
    pub fn search(
        columns: u16,
        rows: u16,
        start: CellCoordinate,
        passable: impl Fn(CellCoordinate) -> bool,
    ) -> Self {
        Self::search_to(columns, rows, start, None, &[], passable)
    }

    /// A* to one target; unlike a complete tree, unrelated ocean is not flooded.
    pub fn route(
        columns: u16,
        rows: u16,
        start: CellCoordinate,
        target: CellCoordinate,
        passable: impl Fn(CellCoordinate) -> bool,
    ) -> Self {
        Self::search_to(columns, rows, start, Some(target), &[], passable)
    }

    /// Stop Dijkstra when the cheapest goal is settled. Uses the same cell-order
    /// tie breaks and exact path as a complete search, without flooding the rest
    /// of the map. Returns the cost and path; an empty path means the start
    /// is already a goal.
    pub fn route_to_nearest(
        columns: u16,
        rows: u16,
        start: CellCoordinate,
        goals: &[CellCoordinate],
        passable: impl Fn(CellCoordinate) -> bool,
    ) -> Option<(u32, Vec<CellCoordinate>)> {
        if goals.is_empty() {
            return None;
        }
        let tree = Self::search_to(columns, rows, start, None, goals, passable);
        tree.nearest(goals.iter().copied())
            .map(|goal| (tree.cost(goal).unwrap(), tree.path_to(goal)))
    }

    fn search_to(
        columns: u16,
        rows: u16,
        start: CellCoordinate,
        target: Option<CellCoordinate>,
        goals: &[CellCoordinate],
        passable: impl Fn(CellCoordinate) -> bool,
    ) -> Self {
        let estimate = |cell: CellCoordinate| {
            target.map_or(0, |goal| {
                let x = u32::from(cell.column.abs_diff(goal.column));
                let y = u32::from(cell.row.abs_diff(goal.row));
                2 * x.max(y) + x.min(y)
            })
        };
        let count = usize::from(columns) * usize::from(rows);
        let mut tree = Self {
            columns,
            start,
            cost: vec![u32::MAX; count],
            previous: vec![None; count],
        };
        if start.column >= columns || start.row >= rows {
            return tree;
        }
        let mut open = BinaryHeap::new();
        let start_index = tree.index(start);
        tree.cost[start_index] = 0;
        open.push(Reverse((estimate(start), 0_u32, start)));
        while let Some(Reverse((_, cost, current))) = open.pop() {
            let current_index = tree.index(current);
            if cost != tree.cost[current_index] {
                continue;
            }
            if target == Some(current) || goals.contains(&current) {
                break;
            }
            for (next, step_cost) in steps(current, columns, rows, &passable) {
                let candidate = cost + step_cost;
                let index = tree.index(next);
                if candidate < tree.cost[index] {
                    tree.cost[index] = candidate;
                    tree.previous[index] = Some(current);
                    open.push(Reverse((candidate + estimate(next), candidate, next)));
                }
            }
        }
        tree
    }

    pub fn cost(&self, cell: CellCoordinate) -> Option<u32> {
        if cell.column >= self.columns {
            return None;
        }
        self.cost
            .get(self.index(cell))
            .copied()
            .filter(|cost| *cost != u32::MAX)
    }

    /// The cheapest reachable goal, ties broken by cell order.
    pub fn nearest(
        &self,
        goals: impl IntoIterator<Item = CellCoordinate>,
    ) -> Option<CellCoordinate> {
        goals
            .into_iter()
            .filter_map(|goal| Some((self.cost(goal)?, goal)))
            .min()
            .map(|(_, goal)| goal)
    }

    /// The cells after the start on the way to `goal`, ending at the goal;
    /// empty if the goal is the start or unreachable.
    pub fn path_to(&self, goal: CellCoordinate) -> Vec<CellCoordinate> {
        let mut path = Vec::new();
        if self.cost(goal).is_none() {
            return path;
        }
        let mut current = goal;
        while current != self.start {
            path.push(current);
            current = self.previous[self.index(current)].expect("reached cells have a predecessor");
        }
        path.reverse();
        path
    }

    /// The first cell to step into on the way to `goal`; `None` if the goal is
    /// the start or unreachable.
    pub fn first_step(&self, goal: CellCoordinate) -> Option<CellCoordinate> {
        self.path_to(goal).first().copied()
    }

    fn index(&self, cell: CellCoordinate) -> usize {
        usize::from(cell.row) * usize::from(self.columns) + usize::from(cell.column)
    }
}

/// Passable neighbors of `cell` with their step costs, in a fixed order.
fn steps(
    cell: CellCoordinate,
    columns: u16,
    rows: u16,
    passable: &impl Fn(CellCoordinate) -> bool,
) -> Vec<(CellCoordinate, u32)> {
    let mut result = Vec::with_capacity(8);
    for (dx, dy) in [
        (0_i32, -1_i32),
        (-1, 0),
        (1, 0),
        (0, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
    ] {
        let Some(next) = offset(cell, dx, dy, columns, rows) else {
            continue;
        };
        if !passable(next) {
            continue;
        }
        if dx != 0 && dy != 0 {
            let corners = [
                offset(cell, dx, 0, columns, rows),
                offset(cell, 0, dy, columns, rows),
            ];
            if !corners
                .into_iter()
                .all(|corner| corner.is_some_and(passable))
            {
                continue;
            }
            result.push((next, DIAGONAL_COST));
        } else {
            result.push((next, ORTHOGONAL_COST));
        }
    }
    result
}

pub fn offset(
    cell: CellCoordinate,
    dx: i32,
    dy: i32,
    columns: u16,
    rows: u16,
) -> Option<CellCoordinate> {
    let column = i32::from(cell.column) + dx;
    let row = i32::from(cell.row) + dy;
    ((0..i32::from(columns)).contains(&column) && (0..i32::from(rows)).contains(&row))
        .then(|| CellCoordinate::new(column as u16, row as u16))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(column: u16, row: u16) -> CellCoordinate {
        CellCoordinate::new(column, row)
    }

    #[test]
    fn diagonal_steps_never_cut_a_blocked_corner() {
        let tree = PathTree::search(3, 3, cell(0, 0), |c| c != cell(1, 0));
        assert_eq!(tree.first_step(cell(1, 1)), Some(cell(0, 1)));
        assert_eq!(tree.cost(cell(1, 1)), Some(ORTHOGONAL_COST * 2));
    }

    #[test]
    fn open_diagonal_is_preferred_and_deterministic() {
        for _ in 0..10 {
            let tree = PathTree::search(4, 4, cell(0, 0), |_| true);
            assert_eq!(tree.first_step(cell(3, 3)), Some(cell(1, 1)));
            assert_eq!(tree.cost(cell(3, 3)), Some(DIAGONAL_COST * 3));
        }
    }

    #[test]
    fn enclosed_goal_is_unreachable() {
        let walls = [cell(1, 0), cell(0, 1), cell(1, 1)];
        let tree = PathTree::search(3, 3, cell(0, 0), |c| !walls.contains(&c));
        assert_eq!(tree.cost(cell(2, 2)), None);
        assert_eq!(tree.first_step(cell(2, 2)), None);
        assert_eq!(tree.nearest([cell(2, 2), cell(0, 0)]), Some(cell(0, 0)));
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;
    #[test]
    fn nearest_routes_preserve_complete_search_paths_and_goal_ties() {
        let cell = CellCoordinate::new;
        // All obstacle layouts on a small grid, including blocked corners,
        // unreachable goals, equal-cost goals, and a start that is itself a goal.
        for walls in 0_u16..512 {
            let clear = |c: CellCoordinate| walls & (1 << (c.row * 3 + c.column)) == 0;
            for start in [cell(0, 0), cell(1, 1), cell(2, 2)] {
                let all = PathTree::search(3, 3, start, clear);
                for goals in [
                    vec![],
                    vec![cell(2, 0), cell(0, 2)],
                    vec![cell(0, 2), cell(2, 0)],
                    vec![start, cell(2, 1)],
                    vec![cell(1, 1)],
                    vec![cell(3, 0), cell(0, 3)],
                ] {
                    let expected = all
                        .nearest(goals.iter().copied())
                        .map(|goal| (all.cost(goal).unwrap(), all.path_to(goal)));
                    assert_eq!(
                        PathTree::route_to_nearest(3, 3, start, &goals, clear),
                        expected,
                        "walls={walls}, start={start:?}, goals={goals:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn nearby_goal_does_not_search_the_whole_map() {
        let calls = std::cell::Cell::new(0);
        let clear = |_| {
            calls.set(calls.get() + 1);
            true
        };
        let start = CellCoordinate::new(10, 10);
        let goal = CellCoordinate::new(11, 10);
        let all = PathTree::search(120, 80, start, clear);
        let full_calls = calls.replace(0);
        assert_eq!(
            PathTree::route_to_nearest(120, 80, start, &[goal], clear),
            Some((all.cost(goal).unwrap(), all.path_to(goal))),
        );
        assert!(calls.get() < full_calls / 100);
    }

    #[test]
    fn targeted_routes_match_complete_search_costs_and_respect_obstacles() {
        for seed in 0..12 {
            let clear = |c: CellCoordinate| !(c.column == 7 && c.row != seed);
            let start = CellCoordinate::new(2, 3);
            let all = PathTree::search(20, 16, start, clear);
            for goal in [
                CellCoordinate::new(18, 14),
                CellCoordinate::new(1, 1),
                CellCoordinate::new(7, 15),
            ] {
                let route = PathTree::route(20, 16, start, goal, clear);
                assert_eq!(route.cost(goal), all.cost(goal));
                assert!(route.path_to(goal).iter().all(|&cell| clear(cell)));
            }
        }
    }
}
