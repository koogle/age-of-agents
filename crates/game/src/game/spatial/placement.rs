//! Physical placement and connectivity checks; costs and prerequisites stay in commands.

use super::interaction_cells;
use crate::game::*;
use crate::navigation::PathTree;

impl GameWorld {
    /// A foundation must not split any unit's reachable ground. Foundations
    /// already block the full completed footprint, so check before placing one.
    /// Ignore temporary unit claims and reservations, just like static routing.
    pub(in crate::game) fn placement_preserves_routes(&self, footprint: Footprint) -> bool {
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

    /// Whether a new footprint fits: in bounds and over cells nobody claims or reserves.
    pub(in crate::game) fn footprint_is_free(&self, footprint: Footprint) -> bool {
        if !footprint.fits_in(WORLD_COLUMNS, WORLD_ROWS) {
            return false;
        }
        let occupancy = self.occupancy();
        footprint
            .cells()
            .all(|cell| occupancy.is_free_for(cell, None))
    }

    /// Whether any cell beside the footprint (sharing an edge) is water.
    pub(in crate::game) fn touches_sea(&self, footprint: Footprint) -> bool {
        footprint.edge_cells(WORLD_COLUMNS, WORLD_ROWS).any(|cell| {
            self.terrain
                [usize::from(cell.row) * usize::from(WORLD_COLUMNS) + usize::from(cell.column)]
            .biome
                == TerrainBiome::Water
        })
    }
}

impl WorldSnapshot {
    /// Placement geometry using only what the client knows. Commands still
    /// validate against the full world, including reachability and game rules.
    pub fn footprint_is_free(&self, footprint: Footprint) -> bool {
        footprint.fits_in(self.columns, self.rows)
            && !self.resources.iter().any(|resource| {
                (resource.amount > 0.0 || resource.field.is_some())
                    && footprint.overlaps(resource.footprint())
            })
            && !self.units.iter().any(|view| {
                view.unit
                    .claimed_cells()
                    .any(|cell| footprint.contains(cell))
                    || matches!(view.unit.action, UnitAction::Move { to } if footprint.contains(to))
            })
            && !self
                .buildings
                .iter()
                .any(|view| footprint.overlaps(view.building.footprint()))
            && footprint.cells().all(|cell| {
                let terrain = &self.terrain
                    [usize::from(cell.row) * usize::from(self.columns) + usize::from(cell.column)];
                terrain.visibility != CellVisibility::Unseen
                    && terrain.biome.is_none_or(TerrainBiome::is_walkable)
            })
    }

    /// Whether known water shares an edge with this footprint.
    pub fn touches_sea(&self, footprint: Footprint) -> bool {
        footprint.edge_cells(self.columns, self.rows).any(|cell| {
            self.terrain
                [usize::from(cell.row) * usize::from(self.columns) + usize::from(cell.column)]
            .biome
                == Some(TerrainBiome::Water)
        })
    }
}
