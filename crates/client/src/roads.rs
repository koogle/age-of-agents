//! The same two-endpoint placement flow for mouse and touch.
use crate::{App, hud::BuildUi, terrain};
use aoa_game::{CellCoordinate, Command, RoadKind, road_line};
use glam::Vec2;

impl App {
    pub(super) fn resume_road(&mut self, target: &crate::Target, units: &[String]) -> bool {
        let crate::Target::Ground(cell) = target else {
            return false;
        };
        let Some(snapshot) = self.view.snapshot.as_ref() else {
            return false;
        };
        let Some(road) = snapshot
            .roads
            .iter()
            .find(|r| r.cell == *cell && r.work.is_some())
        else {
            return false;
        };
        let kind = road.kind;
        let workers: Vec<_> = snapshot
            .units
            .iter()
            .filter(|u| units.contains(&u.unit.id) && u.unit.kind == aoa_game::UnitKind::Villager)
            .map(|u| u.unit.id.clone())
            .collect();
        if workers.is_empty() {
            return false;
        }
        for unit_id in workers {
            self.send(Command::BuildRoad {
                unit_id,
                start: *cell,
                end: *cell,
                kind,
            });
        }
        true
    }

    fn road_endpoint(&self, pixel: Vec2, start: Option<CellCoordinate>) -> Option<CellCoordinate> {
        let point = self.ground_at(pixel)?;
        let snapshot = self.view.snapshot.as_ref()?;
        let mut cell = terrain::cell_at(point.x, point.z, snapshot.columns, snapshot.rows)?;
        if let Some(start) = start {
            if cell.column.abs_diff(start.column) >= cell.row.abs_diff(start.row) {
                cell.row = start.row;
            } else {
                cell.column = start.column;
            }
        }
        Some(cell)
    }

    fn road_clear(&self, start: CellCoordinate, end: CellCoordinate, kind: RoadKind) -> bool {
        let Some(snapshot) = &self.view.snapshot else {
            return false;
        };
        let Some(cells) = road_line(start, end) else {
            return false;
        };
        let Some(island) = aoa_game::island_at(&snapshot.island_origins, start) else {
            return false;
        };
        let mut cost = 0.0;
        for cell in cells {
            let terrain = &snapshot.terrain
                [cell.row as usize * snapshot.columns as usize + cell.column as usize];
            if aoa_game::island_at(&snapshot.island_origins, cell) != Some(island)
                || terrain.visibility != aoa_game::CellVisibility::Visible
                || terrain.biome.is_none_or(|b| !b.is_walkable())
                || snapshot
                    .buildings
                    .iter()
                    .any(|b| b.building.footprint().contains(cell))
                || snapshot
                    .resources
                    .iter()
                    .any(|r| (r.amount > 0.0 || r.field.is_some()) && r.footprint().contains(cell))
                || snapshot
                    .animals
                    .iter()
                    .any(|a| a.cell == cell || a.step.is_some_and(|s| s.to == cell))
            {
                return false;
            }
            if !snapshot.roads.iter().any(|r| r.cell == cell) {
                cost += kind.stone_per_cell();
            }
        }
        snapshot.inventories[island].stone >= cost
    }

    pub(super) fn road_tap(&mut self, pixel: Vec2) -> bool {
        let BuildUi::PlacingRoad { kind, start } = self.build else {
            return false;
        };
        if let Some(end) = self.road_endpoint(pixel, start) {
            if self.road_clear(start.unwrap_or(end), end, kind) {
                if let (Some(start), Some(unit_id)) = (start, self.selection.units.first().cloned())
                {
                    self.send(Command::BuildRoad {
                        unit_id,
                        start,
                        end,
                        kind,
                    });
                    self.build = BuildUi::Off;
                } else {
                    self.build = BuildUi::PlacingRoad {
                        kind,
                        start: Some(end),
                    };
                }
            } else {
                self.toast = Some((
                    "Road needs clear visible land and enough stone.".into(),
                    crate::now_seconds() + 3.0,
                ));
            }
        }
        true
    }

    pub(super) fn road_preview(&self, pixel: Vec2) -> Option<([f32; 4], bool)> {
        let BuildUi::PlacingRoad { kind, start } = self.build else {
            return None;
        };
        if self.hud.covers(pixel) {
            return None;
        }
        let end = self.road_endpoint(pixel, start)?;
        let start = start.unwrap_or(end);
        Some((
            [
                start.column.min(end.column) as f32 * terrain::CELL,
                start.row.min(end.row) as f32 * terrain::CELL,
                (start.column.abs_diff(end.column) + 1) as f32 * terrain::CELL,
                (start.row.abs_diff(end.row) + 1) as f32 * terrain::CELL,
            ],
            self.road_clear(start, end, kind),
        ))
    }
}
