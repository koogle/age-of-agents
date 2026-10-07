use super::*;
use aoa_game::TRANSPORT_PASSENGERS;
use selection::{Command, Selected};

pub(super) fn selection(snapshot: &WorldSnapshot, model: &Model) -> Option<Selected> {
    let ship = snapshot
        .ships
        .iter()
        .find(|s| Some(s.id.as_str()) == model.ship)?;
    let mut commands = vec![Command {
        icon: "command_disembark",
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
    // Players steer by tapping the sea; shortcuts only return to discovered
    // islands: home first, then the others, never the one the ship is at.
    let origins = &snapshot.island_origins;
    let here = aoa_game::island_at(origins, ship.cell);
    let home = ship
        .home_dock_id
        .as_ref()
        .and_then(|id| snapshot.buildings.iter().find(|b| &b.building.id == id))
        .and_then(|dock| aoa_game::island_at(origins, dock.building.origin))
        .unwrap_or(0);
    let others = (0..origins.len()).filter(|&id| id != home);
    for id in std::iter::once(home).chain(others) {
        if Some(id) == here {
            continue;
        }
        commands.push(Command {
            icon: if id == home {
                "portrait_towncenter"
            } else {
                "command_sail"
            },
            label: if id == home {
                "Return home".into()
            } else {
                format!("Sail to island {}", id + 1)
            },
            detail: "Cross the ocean · all settlements keep working".into(),
            enabled: ship.stopped(),
            action: Action::Voyage(id as u64),
        });
    }
    if !ship.stopped() {
        commands.push(Command {
            icon: "command_stop",
            label: "Stop ship".into(),
            detail: "Finish current step · X".into(),
            enabled: true,
            action: Action::Stop,
        });
    }
    Some((
        "transport",
        format!(
            "Transport · {:.0}/{} resources",
            ship.cargo.total(),
            aoa_game::SHIP_RESOURCE_CAPACITY
        ),
        format!(
            "{}/{TRANSPORT_PASSENGERS} passengers · {}",
            ship.passengers.len(),
            if !ship.stopped() {
                "Sailing"
            } else if connection.is_some() {
                "Storage available to nearby island"
            } else {
                "Tap the sea to steer · near shore shares cargo"
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

    fn snapshot_seed() -> u64 {
        GameWorld::default().seed
    }

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
        // On the only discovered island there is nowhere to return: steer by hand.
        let actions = commands(&snapshot);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, Action::Disembark);
        assert_eq!(actions[0].icon, "command_disembark");
        assert!(
            actions
                .iter()
                .all(|c| c.label != "Explore beyond the coast")
        );
        let plan = aoa_game::archipelago_plan(snapshot_seed());
        snapshot.island_count = 3;
        snapshot.island_origins.extend([plan[1], plan[2]]);
        let labels = |snapshot: &WorldSnapshot| {
            commands(snapshot)
                .into_iter()
                .skip(1)
                .map(|c| (c.label, c.action))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            labels(&snapshot),
            vec![
                ("Sail to island 2".into(), Action::Voyage(1)),
                ("Sail to island 3".into(), Action::Voyage(2)),
            ]
        );
        // Away from home, Return home comes first and the current island is omitted.
        snapshot.ships[0].cell = CellCoordinate::new(plan[1].column + 2, plan[1].row + 2);
        assert_eq!(
            labels(&snapshot),
            vec![
                ("Return home".into(), Action::Voyage(0)),
                ("Sail to island 3".into(), Action::Voyage(2)),
            ]
        );
        assert_eq!(commands(&snapshot)[1].icon, "portrait_towncenter");
        snapshot.ships[0].cell = cell;
        assert!(!actions[0].enabled);
        snapshot.ships[0]
            .passengers
            .push(snapshot.units[0].unit.clone());
        assert!(commands(&snapshot)[0].enabled);
        snapshot.ships[0].destination = Some(CellCoordinate::new(0, 0));
        assert!(!commands(&snapshot)[0].enabled);
        assert_eq!(commands(&snapshot).last().unwrap().action, Action::Stop);
        assert_eq!(commands(&snapshot).last().unwrap().icon, "command_stop");
    }
}
