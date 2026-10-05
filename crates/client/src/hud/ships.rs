use super::*;
use aoa_game::{SHIP_RESOURCE_CAPACITY, TRANSPORT_PASSENGERS};
use selection::{Command, Selected};

pub(super) fn selection(snapshot: &WorldSnapshot, model: &Model) -> Option<Selected> {
    let ship = snapshot
        .ships
        .iter()
        .find(|s| Some(s.id.as_str()) == model.ship)?;
    let mut commands = vec![Command {
        icon: "portrait_group",
        label: "Land passengers".into(),
        detail: if ship.passengers.is_empty() {
            "Select units, then tap this ship to board".into()
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
    }];
    let connection = snapshot
        .ship_connections
        .iter()
        .find(|c| c.ship_id == ship.id);
    let kind = model.cargo_kind;
    let aboard = ship.cargo.amount(kind);
    let ashore = connection.map_or(0.0, |c| {
        snapshot.stored_inventories[c.island_id].amount(kind)
    });
    let docked = connection.is_some_and(|c| c.docked);
    commands.push(Command {
        icon: resource_icon(kind),
        label: format!("{} {:.0}", kind.name(), aboard),
        detail: format!(
            "{:.0} {} aboard · tap to choose another resource",
            aboard,
            kind.name()
        ),
        enabled: true,
        action: Action::CargoKind,
    });
    for (direction, amount, label) in [
        (
            aoa_game::CargoDirection::Load,
            ashore
                .min(10.0)
                .min((SHIP_RESOURCE_CAPACITY - ship.cargo.total()).max(0.0)),
            "Load",
        ),
        (aoa_game::CargoDirection::Unload, aboard.min(10.0), "Unload"),
    ] {
        commands.push(Command {
            icon: resource_icon(kind),
            label: format!("{label} {:.0}", amount),
            detail: if docked {
                format!(
                    "{:.0} on shore · {:.0}/50 aboard",
                    ashore,
                    ship.cargo.total()
                )
            } else {
                "Stop beside a completed dock to transfer".into()
            },
            enabled: docked && amount > 0.0,
            action: Action::TransferCargo(direction, amount),
        });
    }
    let current = snapshot
        .island_origins
        .iter()
        .enumerate()
        .min_by_key(|(_, origin)| {
            let x = i64::from(ship.cell.column)
                - i64::from(origin.column)
                - i64::from(aoa_game::WORLD_COLUMNS / 2);
            let y = i64::from(ship.cell.row)
                - i64::from(origin.row)
                - i64::from(aoa_game::WORLD_ROWS / 2);
            x * x + y * y
        })
        .map_or(0, |(id, _)| id as u64);
    for id in [current.checked_sub(1), Some(current + 1)]
        .into_iter()
        .flatten()
    {
        let frontier = id >= snapshot.island_count as u64;
        commands.push(Command {
            icon: "transport",
            label: if frontier {
                "Explore beyond the coast".into()
            } else {
                format!("Sail to island {}", id + 1)
            },
            detail: if frontier {
                "Sail to open water to discover the next island".into()
            } else {
                "Cross the ocean · all settlements keep working".into()
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
        format!("Transport · {:.0}/50 resources", ship.cargo.total()),
        format!(
            "{}/{TRANSPORT_PASSENGERS} passengers · {}",
            ship.passengers.len(),
            if ship.stopped() {
                if connection.is_some() {
                    "Storage available to nearby island"
                } else {
                    "Stop near shore to share cargo"
                }
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
    use aoa_game::{CellCoordinate, GameWorld, TransportShip};
    #[test]
    fn passenger_and_voyage_controls_follow_ship_state() {
        let mut snapshot = GameWorld::default().snapshot();
        let dock = &mut snapshot.buildings[0].building;
        dock.kind = BuildingKind::Dock;
        let cell = CellCoordinate::new(dock.origin.column, dock.origin.row + 4);
        snapshot.inventories[0].wood = 100.0;
        snapshot.ships.push(TransportShip {
            id: "transport-3".into(),
            cell,
            step: None,
            destination: None,
            heading: [1, 0],
            passengers: vec![],
            cargo: Default::default(),
            home_dock_id: None,
        });
        let commands = |snapshot: &WorldSnapshot| {
            selection(
                snapshot,
                &Model {
                    resource_island: 0,
                    cargo_kind: ResourceKind::Wood,
                    snapshot: Some(snapshot),
                    units: &[],
                    building: None,
                    ship: Some("transport-3"),
                    build: BuildUi::Off,
                    show_grid: false,
                    toast: None,
                    camera: Vec2::ZERO,
                },
            )
            .unwrap()
            .4
        };
        let actions = commands(&snapshot);
        assert_eq!(actions.len(), 5);
        assert_eq!(actions[4].action, Action::Voyage(1));
        assert_eq!(actions[0].action, Action::Disembark);
        assert!(!actions[0].enabled);
        snapshot.ships[0]
            .passengers
            .push(snapshot.units[0].unit.clone());
        assert!(commands(&snapshot)[0].enabled);
        snapshot.ships[0].destination = Some(CellCoordinate::new(0, 0));
        assert!(!commands(&snapshot)[0].enabled);
        assert_eq!(commands(&snapshot).last().unwrap().action, Action::Stop);
    }
}
