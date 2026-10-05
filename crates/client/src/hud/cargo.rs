//! Resource rows keep both transfer directions beside the relevant inventory.
use super::*;
use aoa_game::{CargoDirection, SHIP_RESOURCE_CAPACITY};

struct CargoRow {
    kind: ResourceKind,
    ashore: f64,
    aboard: f64,
    load: f64,
    unload: f64,
    docked: bool,
}

fn rows(snapshot: &WorldSnapshot, ship_id: &str) -> Vec<CargoRow> {
    let Some(ship) = snapshot.ships.iter().find(|s| s.id == ship_id) else {
        return Vec::new();
    };
    let connection = snapshot
        .ship_connections
        .iter()
        .find(|c| c.ship_id == ship_id);
    ResourceKind::ALL
        .iter()
        .filter_map(|&kind| {
            let ashore = connection.map_or(0.0, |c| {
                snapshot.stored_inventories[c.island_id].amount(kind)
            });
            let aboard = ship.cargo.amount(kind);
            (ashore > 0.0 || aboard > 0.0).then_some(CargoRow {
                kind,
                ashore,
                aboard,
                load: ashore
                    .min(10.0)
                    .min((SHIP_RESOURCE_CAPACITY - ship.cargo.total()).max(0.0)),
                unload: aboard.min(10.0),
                docked: ship.stopped() && connection.is_some_and(|c| c.docked),
            })
        })
        .collect()
}

fn quantity(amount: f64) -> String {
    if amount > 0.0 && amount < 0.01 {
        return "<0.01".into();
    }
    format!("{amount:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

impl Hud {
    pub(super) fn cargo_panel(
        &mut self,
        atlas: &Atlas,
        model: &Model,
        width: f32,
        top: f32,
        bottom: f32,
        s: f32,
    ) {
        let (Some(snapshot), Some(ship_id)) = (model.snapshot, model.ship) else {
            self.cargo_page = 0;
            return;
        };
        let rows = rows(snapshot, ship_id);
        let room = ((bottom - top - 44.0 * s) / (44.0 * s)).floor().max(1.0) as usize;
        let pages = rows.len().div_ceil(room).max(1);
        self.cargo_page = self.cargo_page.min(pages - 1);
        let shown: Vec<_> = rows
            .iter()
            .skip(self.cargo_page * room)
            .take(room)
            .collect();
        let h = (44.0 + shown.len().max(1) as f32 * 44.0) * s;
        let w = (width - 24.0 * s).min(380.0 * s);
        let x = (width - w) / 2.0;
        let y = bottom - h;
        self.shape([x, y, w, h], GLASS, 1.0, 16.0 * s);
        self.regions.push(Region {
            rect: [x, y, w, h],
            action: Action::Explain(
                "Load and unload at a completed dock · up to 10 per tap".into(),
            ),
            enabled: true,
        });
        self.text(
            atlas,
            if pages > 1 { "Cargo" } else { "Ship resources" },
            (x + 12.0 * s, y + 26.0 * s),
            13.0 * s,
            INK,
            false,
        );
        if shown.is_empty() {
            self.text(
                atlas,
                "No resources aboard or on this island",
                (x + 12.0 * s, y + 67.0 * s),
                12.0 * s,
                MUTED,
                false,
            );
        }
        for (i, row) in shown.iter().enumerate() {
            let ry = y + (44.0 + i as f32 * 44.0) * s;
            self.coin(
                atlas,
                resource_icon(row.kind),
                [x + 8.0 * s, ry + 4.0 * s, 30.0 * s, 30.0 * s],
                true,
                false,
            );
            self.text(
                atlas,
                row.kind.name(),
                (x + 44.0 * s, ry + 15.0 * s),
                12.0 * s,
                INK,
                false,
            );
            self.text(
                atlas,
                &format!(
                    "{} shore · {} aboard",
                    quantity(row.ashore),
                    quantity(row.aboard)
                ),
                (x + 44.0 * s, ry + 31.0 * s),
                10.0 * s,
                MUTED,
                false,
            );
            for (index, direction, amount, label) in [
                (0, CargoDirection::Load, row.load, "Load"),
                (1, CargoDirection::Unload, row.unload, "Unload"),
            ] {
                let enabled = row.docked && amount > 0.0;
                let rect = [
                    x + w - (164.0 - index as f32 * 80.0) * s,
                    ry,
                    76.0 * s,
                    40.0 * s,
                ];
                self.cargo_button(
                    atlas,
                    rect,
                    &format!("{label} {}", quantity(amount)),
                    if enabled {
                        Action::TransferCargo(row.kind, direction, amount)
                    } else {
                        Action::Explain(if !row.docked {
                            "Stop beside a completed dock to transfer".into()
                        } else {
                            "No resources or space available for this transfer".into()
                        })
                    },
                    enabled,
                    s,
                );
            }
        }
        if pages > 1 {
            let ry = y;
            self.text(
                atlas,
                &format!("{} / {pages}", self.cargo_page + 1),
                (x + w - 128.0 * s, ry + 26.0 * s),
                12.0 * s,
                INK,
                true,
            );
            for (left, label, page, enabled) in [
                (
                    x + w - 252.0 * s,
                    "Previous",
                    self.cargo_page.saturating_sub(1),
                    self.cargo_page > 0,
                ),
                (
                    x + w - 88.0 * s,
                    "Next",
                    self.cargo_page + 1,
                    self.cargo_page + 1 < pages,
                ),
            ] {
                self.cargo_button(
                    atlas,
                    [left, ry, 80.0 * s, 40.0 * s],
                    label,
                    Action::CargoPage(page.min(pages - 1)),
                    enabled,
                    s,
                );
            }
        }
    }

    fn cargo_button(
        &mut self,
        atlas: &Atlas,
        rect: [f32; 4],
        label: &str,
        action: Action,
        enabled: bool,
        s: f32,
    ) {
        self.shape(
            rect,
            if enabled {
                [1.0, 0.99, 0.96, 0.98]
            } else {
                [0.86, 0.85, 0.81, 0.7]
            },
            1.0,
            10.0 * s,
        );
        self.text(
            atlas,
            label,
            (rect[0] + rect[2] / 2.0, rect[1] + 25.0 * s),
            11.0 * s,
            if enabled { INK } else { MUTED },
            true,
        );
        self.regions.push(Region {
            rect,
            action,
            enabled: true,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{CellCoordinate, GameWorld, ShipConnection, TransportShip};

    #[test]
    fn available_rows_keep_aboard_goods_and_bound_each_transfer() {
        let mut s = GameWorld::default().snapshot();
        s.stored_inventories[0].wood = 35.0;
        s.stored_inventories[0].food = 2.5;
        s.ships.push(TransportShip {
            id: "ship".into(),
            cell: CellCoordinate::new(0, 0),
            step: None,
            destination: None,
            heading: [1, 0],
            passengers: vec![],
            cargo: Default::default(),
            home_dock_id: None,
        });
        s.ships[0].cargo.stone = 47.0;
        s.ship_connections.push(ShipConnection {
            ship_id: "ship".into(),
            island_id: 0,
            docked: true,
        });
        let visible = rows(&s, "ship");
        assert_eq!(visible.len(), 3);
        let wood = visible
            .iter()
            .find(|r| r.kind == ResourceKind::Wood)
            .unwrap();
        assert_eq!((wood.load, wood.unload), (3.0, 0.0));
        let food = visible
            .iter()
            .find(|r| r.kind == ResourceKind::Food)
            .unwrap();
        assert_eq!(food.load, 2.5);
        let stone = visible
            .iter()
            .find(|r| r.kind == ResourceKind::Stone)
            .unwrap();
        assert_eq!((stone.ashore, stone.unload), (0.0, 10.0));
        s.ship_connections.clear();
        let at_sea = rows(&s, "ship");
        assert_eq!(at_sea.len(), 1);
        assert_eq!(at_sea[0].kind, ResourceKind::Stone);
        assert!(!at_sea[0].docked);
        s.ship_connections.push(ShipConnection {
            ship_id: "ship".into(),
            island_id: 0,
            docked: true,
        });
        s.ships[0].cargo.stone = 50.0;
        assert!(rows(&s, "ship").iter().all(|r| r.load == 0.0));
    }
    #[test]
    fn long_lists_expose_every_resource_without_overlapping_buttons() {
        let assets = pollster::block_on(crate::assets::Assets::load());
        let atlas = build_atlas(&assets);
        let mut snapshot = GameWorld::default().snapshot();
        let balances: serde_json::Map<String, serde_json::Value> = ResourceKind::ALL
            .iter()
            .map(|kind| {
                (
                    serde_json::to_value(kind)
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned(),
                    serde_json::json!(100.0),
                )
            })
            .collect();
        snapshot.stored_inventories[0] =
            serde_json::from_value(serde_json::Value::Object(balances)).unwrap();
        snapshot.ships.push(TransportShip {
            id: "ship".into(),
            cell: CellCoordinate::new(0, 0),
            step: None,
            destination: None,
            heading: [1, 0],
            passengers: vec![],
            cargo: Default::default(),
            home_dock_id: None,
        });
        snapshot.ship_connections.push(ShipConnection {
            ship_id: "ship".into(),
            island_id: 0,
            docked: true,
        });
        let model = Model {
            snapshot: Some(&snapshot),
            ship: Some("ship"),
            units: &[],
            building: None,
            resource_island: 0,
            build: BuildUi::Off,
            show_grid: false,
            toast: None,
            camera: Vec2::ZERO,
        };
        for (width, top, bottom, scale) in [(390.0, 370.0, 650.0, 2.0), (1280.0, 110.0, 620.0, 1.0)]
        {
            let mut hud = Hud::new();
            let mut seen = Vec::new();
            for page in 0..13 {
                hud.regions.clear();
                hud.cargo_page = page;
                hud.cargo_panel(
                    &atlas,
                    &model,
                    width * scale,
                    top * scale,
                    bottom * scale,
                    scale,
                );
                if hud.cargo_page != page {
                    break;
                }
                let buttons: Vec<_> = hud
                    .regions
                    .iter()
                    .filter(|r| matches!(r.action, Action::TransferCargo(..)))
                    .collect();
                for (i, button) in buttons.iter().enumerate() {
                    assert!(
                        button.rect[1] >= top * scale
                            && button.rect[1] + button.rect[3] <= bottom * scale
                    );
                    for other in buttons.iter().skip(i + 1) {
                        let a = button.rect;
                        let b = other.rect;
                        assert!(
                            a[0] + a[2] <= b[0]
                                || b[0] + b[2] <= a[0]
                                || a[1] + a[3] <= b[1]
                                || b[1] + b[3] <= a[1]
                        );
                    }
                    if let Action::TransferCargo(kind, CargoDirection::Load, _) = button.action {
                        seen.push(kind);
                    }
                }
            }
            assert_eq!(seen, ResourceKind::ALL);
        }
    }
}
