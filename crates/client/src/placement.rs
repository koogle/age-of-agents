//! Building placement on the same rectangular cell grid used by the simulation.
use crate::{App, terrain};
use aoa_game::{CellCoordinate, Footprint};
use glam::Vec2;

impl App {
    /// Terrain, art and grid use one flat foundation plane, also during preview.
    pub(super) fn level_building_plots(
        &mut self,
        ghost: Option<(aoa_game::BuildingKind, CellCoordinate, bool)>,
    ) -> bool {
        let Some(snapshot) = self.view.snapshot.as_ref() else {
            return false;
        };
        let mut bounds: Vec<_> = snapshot
            .buildings
            .iter()
            .map(|b| {
                [
                    b.building.origin.column as f32 * terrain::CELL,
                    b.building.origin.row as f32 * terrain::CELL,
                    b.columns as f32 * terrain::CELL,
                    b.rows as f32 * terrain::CELL,
                ]
            })
            .collect();
        bounds.extend(
            snapshot
                .resources
                .iter()
                .filter(|r| r.field.is_some())
                .map(|r| {
                    let footprint = r.footprint();
                    [
                        footprint.origin.column as f32 * terrain::CELL,
                        footprint.origin.row as f32 * terrain::CELL,
                        footprint.columns as f32 * terrain::CELL,
                        footprint.rows as f32 * terrain::CELL,
                    ]
                }),
        );
        if let Some((kind, origin, _)) = ghost {
            let (columns, rows) = kind.size();
            bounds.push([
                origin.column as f32 * terrain::CELL,
                origin.row as f32 * terrain::CELL,
                columns as f32 * terrain::CELL,
                rows as f32 * terrain::CELL,
            ]);
        }
        self.view.heights.set_plots(bounds, ghost.is_some())
    }

    /// The site for `kind` under the cursor (centred on it) and whether it is clear.
    pub(super) fn placement(
        &self,
        pixel: Vec2,
        kind: aoa_game::BuildingKind,
    ) -> Option<(CellCoordinate, bool)> {
        let snapshot = self.view.snapshot.as_ref()?;
        let point = self
            .rig
            .ground_at(pixel, |x, z| self.view.heights.placement_at(x, z))?;
        let (columns, rows) = kind.size();
        let column = (point.x / terrain::CELL - columns as f32 / 2.0).round();
        let row = (point.z / terrain::CELL - rows as f32 / 2.0).round();
        if column < 0.0 || row < 0.0 {
            return None;
        }
        let origin = CellCoordinate {
            column: column as u16,
            row: row as u16,
        };
        let footprint = Footprint {
            origin,
            columns,
            rows,
        };
        if !footprint.fits_in(snapshot.columns, snapshot.rows) {
            return None;
        }
        Some((
            origin,
            snapshot.footprint_is_free(footprint)
                && snapshot
                    .stockpile
                    .affords(if self.build == crate::hud::BuildUi::PlacingField {
                        aoa_game::FIELD_COST
                    } else {
                        kind.cost()
                    })
                && (!kind.needs_coast() || snapshot.touches_sea(footprint))
                && (self.build != crate::hud::BuildUi::PlacingField
                    || snapshot.buildings.iter().any(|b| {
                        b.building.kind == aoa_game::BuildingKind::Farm && b.building.is_complete()
                    })),
        ))
    }
}
