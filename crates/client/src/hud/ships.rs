use super::*;
use aoa_game::TRANSPORT_PASSENGERS;
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
                "Voyage with passengers · away settlements pause".into()
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
            "{}/{TRANSPORT_PASSENGERS} passengers · {}",
            ship.passengers.len(),
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
    use aoa_game::{CellCoordinate, GameWorld, TransportShip};
    #[test]
    fn passenger_and_voyage_controls_follow_ship_state() {
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
        });
        let commands = |snapshot: &WorldSnapshot| {
            selection(
                snapshot,
                &Model {
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
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].action, Action::Disembark);
        assert!(!actions[0].enabled);
        assert_eq!(actions[1].action, Action::Voyage(1));
        assert!(actions[1].enabled);
        snapshot.ships[0]
            .passengers
            .push(snapshot.units[0].unit.clone());
        assert!(commands(&snapshot)[0].enabled);
        snapshot.ships[0].destination = Some(CellCoordinate::new(0, 0));
        assert!(!commands(&snapshot)[0].enabled);
        assert!(!commands(&snapshot)[1].enabled);
        assert_eq!(commands(&snapshot).last().unwrap().action, Action::Stop);
    }
}
