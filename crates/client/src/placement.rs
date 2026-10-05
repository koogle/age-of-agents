//! Building placement on the same rectangular cell grid used by the simulation.
use crate::{App, terrain};
use aoa_game::{CellCoordinate, UnitAction};
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
                    [
                        r.cell.column as f32 * terrain::CELL,
                        r.cell.row as f32 * terrain::CELL,
                        3.0 * terrain::CELL,
                        3.0 * terrain::CELL,
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
        if column < 0.0
            || row < 0.0
            || column as u16 + columns > snapshot.columns
            || row as u16 + rows > snapshot.rows
        {
            return None;
        }
        let origin = CellCoordinate {
            column: column as u16,
            row: row as u16,
        };
        let covers = |c: CellCoordinate| {
            (origin.column..origin.column + columns).contains(&c.column)
                && (origin.row..origin.row + rows).contains(&c.row)
        };
        // Unknown sites accept an exploration order. Occupancy and coast are
        // checked by the simulation after the entire footprint becomes visible.
        if self.build != crate::hud::BuildUi::PlacingField
            && snapshot.terrain.iter().any(|cell| {
                covers(CellCoordinate::new(cell.column, cell.row))
                    && cell.visibility != aoa_game::CellVisibility::Visible
            })
        {
            return Some((origin, snapshot.stockpile.affords(kind.cost())));
        }
        let blocked =
            snapshot.resources.iter().any(|r| {
                (r.amount > 0.0 || r.field.is_some()) && r.footprint().cells().any(covers)
            }) || snapshot.units.iter().any(|u| {
                covers(u.unit.cell)
                    || u.unit.step.is_some_and(|step| covers(step.to))
                    || matches!(u.unit.action, UnitAction::Move { to } if covers(to))
            }) || snapshot.buildings.iter().any(|b| {
                let o = b.building.origin;
                o.column < origin.column + columns
                    && origin.column < o.column + b.columns
                    && o.row < origin.row + rows
                    && origin.row < o.row + b.rows
            }) || (0..rows).any(|dy| {
                (0..columns).any(|dx| {
                    let cell = &snapshot.terrain[(origin.row + dy) as usize
                        * snapshot.columns as usize
                        + (origin.column + dx) as usize];
                    cell.visibility == aoa_game::CellVisibility::Unseen
                        || cell.biome.is_some_and(|biome| !biome.is_walkable())
                })
            });
        let water = |column: i32, row: i32| {
            column >= 0
                && row >= 0
                && column < i32::from(snapshot.columns)
                && row < i32::from(snapshot.rows)
                && snapshot.terrain[row as usize * snapshot.columns as usize + column as usize]
                    .biome
                    == Some(aoa_game::TerrainBiome::Water)
        };
        let (c0, r0) = (i32::from(origin.column), i32::from(origin.row));
        let (c1, r1) = (c0 + i32::from(columns), r0 + i32::from(rows));
        let coast = (c0..c1).any(|c| water(c, r0 - 1) || water(c, r1))
            || (r0..r1).any(|r| water(c0 - 1, r) || water(c1, r));
        Some((
            origin,
            !blocked
                && snapshot
                    .stockpile
                    .affords(if self.build == crate::hud::BuildUi::PlacingField {
                        aoa_game::FIELD_COST
                    } else {
                        kind.cost()
                    })
                && (coast || !kind.needs_coast())
                && (self.build != crate::hud::BuildUi::PlacingField
                    || snapshot.buildings.iter().any(|b| {
                        b.building.kind == aoa_game::BuildingKind::Farm && b.building.is_complete()
                    })),
        ))
    }
}
