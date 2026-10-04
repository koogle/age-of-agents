use super::*;
use aoa_game::{CargoDirection, TRANSPORT_GOODS, TRANSPORT_PASSENGERS};
use selection::{Command, Selected};

pub(super) fn selection(snapshot: &WorldSnapshot, model: &Model) -> Option<Selected> {
    let ship = snapshot
        .ships
        .iter()
        .find(|s| Some(s.id.as_str()) == model.ship)?;
    let kind = snapshot.catalog.resources[model.cargo_index % snapshot.catalog.resources.len()];
    let docked = ship.stopped()
        && snapshot.buildings.iter().any(|b| {
            b.building.kind == BuildingKind::Dock
                && b.building.is_complete()
                && ship.beside(b.building.footprint())
        });
    let shore = ship.stopped()
        && snapshot.terrain.iter().any(|c| {
            c.biome.is_some_and(|b| b.is_walkable())
                && c.column.abs_diff(ship.cell.column) <= 1
                && c.row.abs_diff(ship.cell.row) <= 1
        });
    let load = snapshot
        .stockpile
        .amount(kind)
        .min(20.0)
        .min(TRANSPORT_GOODS - ship.goods_total());
    let unload = ship.goods.amount(kind).min(20.0);
    let mut commands = vec![
        Command {
            icon: "portrait_group",
            label: "Land passengers".into(),
            detail: if ship.passengers.is_empty() {
                "Select villagers, then tap this ship to board".into()
            } else {
                format!(
                    "Needs clear shore · {}",
                    ship.passengers
                        .iter()
                        .map(|u| u.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            enabled: ship.stopped() && !ship.passengers.is_empty(),
            action: Action::Disembark,
        },
        Command {
            icon: resource_icon(kind),
            label: format!("Cargo {}", kind.name()),
            detail: format!(
                "Tap to change resource · {} aboard · {} in settlement",
                ship.goods.amount(kind),
                snapshot.stockpile.amount(kind)
            ),
            enabled: true,
            action: Action::ShipCargoNext,
        },
    ];
    for (direction, amount, label) in [
        (CargoDirection::Load, load, "Load"),
        (CargoDirection::Unload, unload, "Unload"),
    ] {
        commands.push(Command {
            icon: resource_icon(kind),
            label: format!("{label} {}", kind.name()),
            detail: if docked || (direction == CargoDirection::Unload && shore) {
                format!("Transfer {amount} goods to/from this island")
            } else {
                "Stop beside a completed dock to transfer goods".into()
            },
            enabled: (docked || (direction == CargoDirection::Unload && shore)) && amount > 0.0,
            action: Action::ShipTransfer(direction),
        });
    }
    for id in [
        snapshot.island_id.checked_sub(1),
        snapshot.island_id.checked_add(1),
    ]
    .into_iter()
    .flatten()
    {
        commands.push(Command {
            icon: "transport",
            label: format!("Sail to island {}", id + 1),
            detail: if id >= snapshot.island_count as u64 {
                "Discover a new island".into()
            } else {
                "Voyage with passengers and goods · away settlements pause".into()
            },
            enabled: ship.stopped(),
            action: Action::Voyage(id),
        });
    }
    if !ship.stopped() {
        commands.push(Command {
            icon: "command_cancel",
            label: "Stop ship".into(),
            detail: "Finish current step · X".into(),
            enabled: true,
            action: Action::Stop,
        });
    }
    Some((
        "transport",
        format!(
            "Transport · Island {} / {}",
            snapshot.island_id + 1,
            snapshot.island_count
        ),
        format!(
            "{}/{TRANSPORT_PASSENGERS} aboard · {}/{TRANSPORT_GOODS} goods · {}",
            ship.passengers.len(),
            ship.goods_total(),
            if ship.stopped() {
                "Tap sea or dock"
            } else {
                "Sailing"
            }
        ),
        None,
        commands,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aoa_game::{CellCoordinate, GameWorld, Stockpile, TransportShip};
    #[test]
    fn cargo_controls_require_a_stopped_dock_and_respect_hold_capacity() {
        let mut snapshot = GameWorld::default().snapshot();
        let dock = &mut snapshot.buildings[0].building;
        dock.kind = BuildingKind::Dock;
        let cell = CellCoordinate::new(dock.origin.column, dock.origin.row + 4);
        snapshot.stockpile.wood = 100.0;
        snapshot.ships.push(TransportShip {
            id: "transport-3".into(),
            cell,
            step: None,
            destination: None,
            heading: [1, 0],
            passengers: vec![],
            goods: Stockpile::default(),
        });
        let commands = |snapshot: &WorldSnapshot| {
            selection(
                snapshot,
                &Model {
                    snapshot: Some(snapshot),
                    units: &[],
                    building: None,
                    ship: Some("transport-3"),
                    cargo_index: 0,
                    build: BuildUi::Off,
                    show_grid: false,
                    toast: None,
                    camera: Vec2::ZERO,
                },
            )
            .unwrap()
            .4
        };
        assert!(commands(&snapshot)[2].enabled);
        assert!(!commands(&snapshot)[3].enabled);
        snapshot.ships[0].goods.wood = TRANSPORT_GOODS;
        assert!(!commands(&snapshot)[2].enabled);
        assert!(commands(&snapshot)[3].enabled);
        snapshot.ships[0].destination = Some(CellCoordinate::new(0, 0));
        assert!(!commands(&snapshot)[2].enabled);
        assert!(!commands(&snapshot)[3].enabled);
        assert_eq!(commands(&snapshot).last().unwrap().action, Action::Stop);
    }
}
