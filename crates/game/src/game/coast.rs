//! Shore-facing art uses the same edge test as dock placement.
use super::{BuildingKind, CellCoordinate, Footprint, TerrainBiome, WorldSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DockFacing {
    North,
    East,
    #[default]
    South,
    West,
}

/// Prefer the edge with most water; ties use south, east, north, west.
/// The callback must return false outside the map. Diagonal water never counts.
pub fn dock_facing(footprint: Footprint, water: impl Fn(i32, i32) -> bool) -> Option<DockFacing> {
    let (x, y) = (
        i32::from(footprint.origin.column),
        i32::from(footprint.origin.row),
    );
    let (right, bottom) = (
        x + i32::from(footprint.columns),
        y + i32::from(footprint.rows),
    );
    let edges = [
        (
            DockFacing::South,
            (x..right).filter(|&c| water(c, bottom)).count(),
        ),
        (
            DockFacing::East,
            (y..bottom).filter(|&r| water(right, r)).count(),
        ),
        (
            DockFacing::North,
            (x..right).filter(|&c| water(c, y - 1)).count(),
        ),
        (
            DockFacing::West,
            (y..bottom).filter(|&r| water(x - 1, r)).count(),
        ),
    ];
    let mut best = None;
    let mut count = 0;
    for (facing, adjacent) in edges {
        if adjacent > count {
            best = Some(facing);
            count = adjacent;
        }
    }
    best
}

impl WorldSnapshot {
    pub fn dock_facing(&self, origin: CellCoordinate) -> Option<DockFacing> {
        let (columns, rows) = BuildingKind::Dock.size();
        dock_facing(
            Footprint {
                origin,
                columns,
                rows,
            },
            |column, row| {
                column >= 0
                    && row >= 0
                    && column < i32::from(self.columns)
                    && row < i32::from(self.rows)
                    && self.terrain[row as usize * usize::from(self.columns) + column as usize]
                        .biome
                        == Some(TerrainBiome::Water)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const PLOT: Footprint = Footprint {
        origin: CellCoordinate { column: 0, row: 0 },
        columns: 4,
        rows: 4,
    };

    #[test]
    fn faces_each_water_edge_and_ignores_diagonals() {
        for (cell, facing) in [
            ((1, -1), DockFacing::North),
            ((4, 1), DockFacing::East),
            ((1, 4), DockFacing::South),
            ((-1, 1), DockFacing::West),
        ] {
            assert_eq!(dock_facing(PLOT, |x, y| (x, y) == cell), Some(facing));
        }
        assert_eq!(dock_facing(PLOT, |x, y| (x, y) == (4, 4)), None);
        assert_eq!(dock_facing(PLOT, |_, _| false), None);
    }

    #[test]
    fn prefers_broad_water_edge_with_stable_corner_ties() {
        assert_eq!(
            dock_facing(PLOT, |x, y| x == 4 || (x == 1 && y == 4)),
            Some(DockFacing::East)
        );
        assert_eq!(dock_facing(PLOT, |_, _| true), Some(DockFacing::South));
    }

    #[test]
    fn snapshot_ignores_unknown_water_and_clamps_map_edges() {
        let mut snapshot = super::super::GameWorld::default().snapshot();
        for cell in &mut snapshot.terrain {
            cell.biome = None;
        }
        let origin = CellCoordinate::new(0, 0);
        assert_eq!(snapshot.dock_facing(origin), None);
        let east = usize::from(snapshot.columns) + 4;
        snapshot.terrain[east].biome = Some(TerrainBiome::Water);
        assert_eq!(snapshot.dock_facing(origin), Some(DockFacing::East));
        snapshot.terrain[east].biome = Some(TerrainBiome::River);
        assert_eq!(snapshot.dock_facing(origin), None);
        assert_eq!(
            snapshot.dock_facing(CellCoordinate::new(snapshot.columns - 4, snapshot.rows - 4)),
            None
        );
    }
}
