//! A single strip of resource columns, each with compact transfer shields.
use super::*;
use aoa_game::{CargoDirection, SHIP_RESOURCE_CAPACITY};

struct CargoRow {
    kind: ResourceKind,
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
        bottom: f32,
        s: f32,
    ) {
        let (Some(snapshot), Some(ship_id)) = (model.snapshot, model.ship) else {
            self.cargo_page = 0;
            return;
        };
        let resources = rows(snapshot, ship_id);
        let column_width = 108.0 * s;
        let room = (((width - 24.0 * s).min(864.0 * s) / column_width).floor() as usize).max(1);
        let pages = resources.len().div_ceil(room).max(1);
        self.cargo_page = self.cargo_page.min(pages - 1);
        let shown: Vec<_> = resources
            .iter()
            .skip(self.cargo_page * room)
            .take(room)
            .collect();
        let h = (78.0 + if pages > 1 { 40.0 } else { 0.0 }) * s;
        let w = if shown.is_empty() {
            300.0 * s
        } else {
            (if pages > 1 { room } else { shown.len() }) as f32 * column_width
        };
        let x = (width - w) / 2.0;
        let y = bottom - h;
        self.shape([x, y, w, h], GLASS, 1.0, 16.0 * s);
        self.regions.push(Region {
            rect: [x, y, w, h],
            action: Action::Explain("Up: load · Down: unload · up to 10 per tap at a dock".into()),
            enabled: true,
        });
        if shown.is_empty() {
            self.text(
                atlas,
                "No resources available here",
                (x + w / 2.0, y + 62.0 * s),
                12.0 * s,
                MUTED,
                true,
            );
        }
        for (i, resource) in shown.iter().enumerate() {
            let left = x + i as f32 * column_width;
            let center = left + column_width / 2.0;
            self.coin(
                atlas,
                resource_icon(resource.kind),
                [center - 16.0 * s, y + 8.0 * s, 32.0 * s, 32.0 * s],
                true,
                false,
            );
            self.text(
                atlas,
                &format!("{} aboard", quantity(resource.aboard)),
                (center, y + 68.0 * s),
                10.0 * s,
                INK,
                true,
            );
            for (index, direction, amount, label) in [
                (0, CargoDirection::Load, resource.load, "Load"),
                (1, CargoDirection::Unload, resource.unload, "Unload"),
            ] {
                let enabled = resource.docked && amount > 0.0;
                // Keep generous hit areas while the painted shields are small.
                let rect = [
                    left + (10.0 + index as f32 * 44.0) * s,
                    y + 20.0 * s,
                    44.0 * s,
                    40.0 * s,
                ];
                let icon = if direction == CargoDirection::Load {
                    "shield_up"
                } else {
                    "shield_down"
                };
                let size = if self.hovered(rect) && enabled {
                    26.0
                } else {
                    24.0
                } * s;
                self.sprite(
                    atlas,
                    icon,
                    [
                        center + (index as f32 * 32.0 - 16.0) * s - size / 2.0,
                        rect[1] + (rect[3] - size) / 2.0,
                        size,
                        size,
                    ],
                    [1.0, 1.0, 1.0, if enabled { 1.0 } else { 0.3 }],
                );
                self.regions.push(Region {
                    rect,
                    action: if enabled {
                        Action::TransferCargo(resource.kind, direction, amount)
                    } else {
                        Action::Explain(if !resource.docked {
                            "Stop beside a completed dock to transfer".into()
                        } else {
                            format!("{label}: no resources or space available")
                        })
                    },
                    enabled: true,
                });
            }
        }
        if pages > 1 {
            let ry = y + 78.0 * s;
            self.text(
                atlas,
                &format!("{} / {pages}", self.cargo_page + 1),
                (x + w / 2.0, ry + 25.0 * s),
                11.0 * s,
                MUTED,
                true,
            );
            for (left, label, page, enabled) in [
                (
                    x + 8.0 * s,
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
                    [left, ry, 80.0 * s, 36.0 * s],
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
        assert_eq!(stone.unload, 10.0);
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
                hud.cargo_panel(&atlas, &model, width * scale, bottom * scale, scale);
                if hud.cargo_page != page {
                    break;
                }
                let buttons: Vec<_> = hud
                    .regions
                    .iter()
                    .filter(|r| matches!(r.action, Action::TransferCargo(..)))
                    .collect();
                for (i, button) in buttons.iter().enumerate() {
                    assert_eq!(button.rect[1], buttons[0].rect[1]);
                    assert!(
                        button.rect[0] >= 0.0 && button.rect[0] + button.rect[2] <= width * scale
                    );
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
