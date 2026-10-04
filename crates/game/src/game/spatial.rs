//! World-space geometry, occupancy and placement checks shared by simulation and previews.

use serde::{Deserialize, Serialize};

use super::{WORLD_COLUMNS, WORLD_ROWS};
use crate::navigation::offset;

pub(super) mod occupancy;
mod placement;
#[cfg(test)]
mod tests;

/// A continuous point in cell units: `(column + 0.5, row + 0.5)` is a cell center.
/// Positions are derived for presentation and sight; they are never authoritative.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub(super) fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CellCoordinate {
    pub column: u16,
    pub row: u16,
}

impl CellCoordinate {
    pub const fn new(column: u16, row: u16) -> Self {
        Self { column, row }
    }

    pub fn center(self) -> Position {
        Position {
            x: f64::from(self.column) + 0.5,
            y: f64::from(self.row) + 0.5,
        }
    }

    /// Chebyshev adjacency: the eight cells around `self`.
    pub(super) fn touches(self, other: Self) -> bool {
        self != other
            && self.column.abs_diff(other.column) <= 1
            && self.row.abs_diff(other.row) <= 1
    }
}

/// An axis-aligned block of cells. Every static thing in the world claims one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Footprint {
    pub origin: CellCoordinate,
    pub columns: u16,
    pub rows: u16,
}

impl Footprint {
    /// Check the full rectangle before iterating or indexing its cells.
    pub fn fits_in(self, columns: u16, rows: u16) -> bool {
        u32::from(self.origin.column) + u32::from(self.columns) <= u32::from(columns)
            && u32::from(self.origin.row) + u32::from(self.rows) <= u32::from(rows)
    }

    /// Sharing an edge is allowed; sharing any area is an overlap.
    pub fn overlaps(self, other: Self) -> bool {
        u32::from(self.origin.column) < u32::from(other.origin.column) + u32::from(other.columns)
            && u32::from(other.origin.column)
                < u32::from(self.origin.column) + u32::from(self.columns)
            && u32::from(self.origin.row) < u32::from(other.origin.row) + u32::from(other.rows)
            && u32::from(other.origin.row) < u32::from(self.origin.row) + u32::from(self.rows)
    }

    /// In-bounds cells along the four edges, excluding diagonal corners.
    pub fn edge_cells(self, columns: u16, rows: u16) -> impl Iterator<Item = CellCoordinate> {
        let width = i32::from(self.columns);
        let height = i32::from(self.rows);
        (0..width)
            .flat_map(move |dx| [(dx, -1), (dx, height)])
            .chain((0..height).flat_map(move |dy| [(-1, dy), (width, dy)]))
            .filter_map(move |(dx, dy)| offset(self.origin, dx, dy, columns, rows))
    }

    pub fn cells(self) -> impl Iterator<Item = CellCoordinate> {
        (self.origin.row..self.origin.row + self.rows).flat_map(move |row| {
            (self.origin.column..self.origin.column + self.columns)
                .map(move |column| CellCoordinate::new(column, row))
        })
    }

    pub fn contains(self, cell: CellCoordinate) -> bool {
        (self.origin.column..self.origin.column + self.columns).contains(&cell.column)
            && (self.origin.row..self.origin.row + self.rows).contains(&cell.row)
    }

    /// A cell from which a villager can work on this footprint: outside it and
    /// touching it, including diagonally.
    pub(super) fn is_interaction_cell(self, cell: CellCoordinate) -> bool {
        !self.contains(cell) && self.cells().any(|inner| inner.touches(cell))
    }

    pub fn center(self) -> Position {
        Position {
            x: f64::from(self.origin.column) + f64::from(self.columns) / 2.0,
            y: f64::from(self.origin.row) + f64::from(self.rows) / 2.0,
        }
    }
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

pub(super) fn in_bounds(cell: CellCoordinate) -> bool {
    cell.column < WORLD_COLUMNS && cell.row < WORLD_ROWS
}
