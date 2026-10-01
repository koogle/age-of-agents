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
        open.push(Reverse((0_u32, start)));
        while let Some(Reverse((cost, current))) = open.pop() {
            let current_index = tree.index(current);
            if cost != tree.cost[current_index] {
                continue;
            }
            for (next, step_cost) in steps(current, columns, rows, &passable) {
                let candidate = cost + step_cost;
                let index = tree.index(next);
                if candidate < tree.cost[index] {
                    tree.cost[index] = candidate;
                    tree.previous[index] = Some(current);
                    open.push(Reverse((candidate, next)));
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
