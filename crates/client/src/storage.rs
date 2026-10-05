//! Presentation and input for island-local inventories and ship holds.
use super::*;
use aoa_game::{CargoDirection, ResourceKind, island_at};

impl App {
    pub(crate) fn select_storage_ship(&mut self, id: String) {
        self.selection.units.clear();
        self.selection.building = None;
        self.selection.ship = Some(id);
    }

    pub(crate) fn update_resource_island(&mut self) {
        let Some(snapshot) = &self.view.snapshot else {
            return;
        };
        let point = if self.hud.covers(self.cursor) {
            self.hud.map_point(self.cursor)
        } else {
            self.rig
                .ground_at(self.cursor, |x, z| self.view.heights.placement_at(x, z))
                .map(|p| Vec2::new(p.x, p.z))
        };
        if let Some(point) = point {
            let cell = aoa_game::CellCoordinate::new(
                (point.x / terrain::CELL).max(0.0) as u16,
                (point.y / terrain::CELL).max(0.0) as u16,
            );
            if let Some(island) = island_at(&snapshot.island_origins, cell) {
                self.resource_island = island;
            }
        }
        self.resource_island = self.resource_island.min(snapshot.inventories.len() - 1);
    }

    pub(crate) fn cycle_cargo(&mut self) {
        let kinds = &ResourceKind::ALL;
        let current = kinds
            .iter()
            .position(|&k| k == self.cargo_kind)
            .unwrap_or(0);
        self.cargo_kind = kinds[(current + 1) % kinds.len()];
    }

    pub(crate) fn transfer_cargo(&mut self, direction: CargoDirection, amount: f64) {
        if let Some(ship_id) = self.selection.ship.clone() {
            self.send(Command::TransferShipCargo {
                ship_id,
                kind: self.cargo_kind,
                amount,
                direction,
            });
        }
    }

    pub(crate) fn order_ship_storage(&mut self, ship_id: &str) {
        for unit_id in self.selection.units.clone() {
            let carrying = self.view.snapshot.as_ref().is_some_and(|s| {
                s.units
                    .iter()
                    .any(|u| u.unit.id == unit_id && u.unit.cargo.is_some())
            });
            self.send(if carrying {
                Command::Deposit {
                    unit_id,
                    storage_id: ship_id.into(),
                }
            } else {
                Command::Board {
                    unit_id,
                    ship_id: ship_id.into(),
                }
            });
        }
    }
}
