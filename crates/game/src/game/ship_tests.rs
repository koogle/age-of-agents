use super::*;
fn c(x: u16, y: u16) -> CellCoordinate {
    CellCoordinate::new(x, y)
}
fn harbor() -> GameWorld {
    let mut w = fixture::fixture();
    w.resources.clear();
    for cell in &mut w.terrain {
        if cell.row >= 30 {
            cell.biome = TerrainBiome::Water;
            cell.elevation = 0.0;
        }
    }
    w.buildings
        .push(building(BuildingKind::Dock, "dock", c(20, 26), None));
    w.stockpile.wood = 1000.0;
    w.stockpile.timber = 1000.0;
    w.apply_command(Command::Produce {
        building_id: "dock".into(),
        product: ProductKind::TransportShip,
    })
    .unwrap();
    w.tick(20.0);
    assert_eq!(w.ships.len(), 1);
    w
}
fn order_board(w: &mut GameWorld, unit: usize) {
    w.apply_command(Command::Board {
        unit_id: w.units[unit].id.clone(),
        ship_id: w.ships[0].id.clone(),
    })
    .unwrap();
}
#[test]
fn transport_builds_once_with_exact_cost_and_no_housing() {
    let mut w = harbor();
    assert_eq!(w.stockpile.wood, 940.0);
    assert_eq!(w.stockpile.timber, 980.0);
    assert_eq!(w.villagers_and_trainees(), 2);
    for _ in 0..10 {
        w.tick(1.0);
    }
    assert_eq!(w.ships.len(), 1);
    assert!(w.islands.is_empty());
    assert_eq!(w.island_origins.len(), 1);
    assert!(w.ships[0].stopped());
    w.validate().unwrap();
}
#[test]
fn passengers_keep_identity_cargo_housing_and_reload_then_land() {
    let mut w = harbor();
    w.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Wood,
        amount: 7.0,
    });
    let id = w.units[0].id.clone();
    order_board(&mut w, 0);
    for _ in 0..200 {
        w.tick(0.1);
    }
    assert_eq!(w.units.len(), 1);
    assert_eq!(w.ships[0].passengers[0].id, id);
    assert_eq!(w.villagers_and_trainees(), 2);
    let mut loaded: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    assert_eq!(loaded, w);
    loaded.validate().unwrap();
    loaded
        .apply_command(Command::Disembark {
            ship_id: loaded.ships[0].id.clone(),
        })
        .unwrap();
    let u = loaded.units.iter().find(|u| u.id == id).unwrap();
    assert_eq!(u.cargo.as_ref().unwrap().amount, 7.0);
    assert_eq!(u.action, UnitAction::Idle);
    assert_eq!(loaded.units.len(), 2);
    assert!(loaded.ships[0].passengers.is_empty());
}
#[test]
fn sailing_is_water_only_stoppable_and_blocked_landing_is_atomic() {
    let mut w = harbor();
    order_board(&mut w, 0);
    for _ in 0..200 {
        w.tick(0.1);
    }
    let id = w.ships[0].id.clone();
    let before = w.clone();
    assert!(
        w.apply_command(Command::Sail {
            ship_id: id.clone(),
            to: c(20, 20)
        })
        .is_err()
    );
    assert_eq!(w, before);
    w.apply_command(Command::Sail {
        ship_id: id.clone(),
        to: c(25, 40),
    })
    .unwrap();
    w.tick(0.1);
    w.apply_command(Command::StopShip {
        ship_id: id.clone(),
    })
    .unwrap();
    for _ in 0..20 {
        w.tick(0.1);
    }
    assert!(w.ships[0].stopped());
    w.apply_command(Command::Sail {
        ship_id: id.clone(),
        to: c(25, 40),
    })
    .unwrap();
    for _ in 0..100 {
        w.tick(0.1);
    }
    assert_eq!(w.ships[0].cell, c(25, 40));
    let before = w.clone();
    assert_eq!(
        w.apply_command(Command::Disembark {
            ship_id: id.clone()
        }),
        Err(CommandError::ShoreBlocked)
    );
    assert_eq!(w, before);
    w.apply_command(Command::Sail {
        ship_id: id.clone(),
        to: c(40, 30),
    })
    .unwrap();
    for _ in 0..100 {
        w.tick(0.1);
    }
    w.apply_command(Command::Disembark { ship_id: id }).unwrap();
    assert_eq!(w.units.len(), 2);
    w.validate().unwrap();
}
#[test]
fn old_saves_default_to_no_ships_and_corrupt_manifests_fail() {
    let w = GameWorld::default();
    let mut json = serde_json::to_value(&w).unwrap();
    json.as_object_mut().unwrap().remove("ships");
    let old: GameWorld = serde_json::from_value(json).unwrap();
    assert_eq!(old, w);
    let mut w = harbor();
    w.ships[0].passengers.push(w.units[0].clone());
    assert!(w.validate().is_err());
    w.ships[0].passengers.clear();
    w.ships[0].heading = [2, 0];
    assert!(w.validate().is_err());
}
#[test]
fn departure_cancels_boarding_and_preserves_the_current_step() {
    let mut w = harbor();
    order_board(&mut w, 0);
    w.tick(0.1);
    let cargo = w.units[0].cargo.clone();
    w.apply_command(Command::Sail {
        ship_id: w.ships[0].id.clone(),
        to: c(40, 40),
    })
    .unwrap();
    assert_eq!(w.units[0].action, UnitAction::Idle);
    assert_eq!(w.units[0].cargo, cargo);
    w.tick(1.0);
    w.validate().unwrap();
}

#[test]
fn passenger_reservations_enforce_capacity_and_stop_releases_a_seat() {
    let mut w = harbor();
    for n in 0..3 {
        let mut unit = w.units[0].clone();
        unit.id = format!("extra-{n}");
        unit.cell = c(10 + n, 24);
        w.units.push(unit);
    }
    for i in 0..4 {
        order_board(&mut w, i);
    }
    let before = w.clone();
    assert_eq!(
        w.apply_command(Command::Board {
            unit_id: w.units[4].id.clone(),
            ship_id: w.ships[0].id.clone()
        }),
        Err(CommandError::ShipFull)
    );
    assert_eq!(w, before);
    w.apply_command(Command::Stop {
        unit_id: w.units[0].id.clone(),
    })
    .unwrap();
    order_board(&mut w, 4);
    for _ in 0..400 {
        w.tick(0.1);
    }
    assert_eq!(w.ships[0].passengers.len(), 4, "land units: {:?}", w.units);
    w.apply_command(Command::Sail {
        ship_id: w.ships[0].id.clone(),
        to: c(40, 30),
    })
    .unwrap();
    for _ in 0..100 {
        w.tick(0.1);
    }
    w.apply_command(Command::Disembark {
        ship_id: w.ships[0].id.clone(),
    })
    .unwrap();
    assert_eq!(w.units.len(), 5);
    w.validate().unwrap();
}

#[test]
fn blocked_ship_production_waits_and_queue_cancellation_refunds() {
    let mut w = harbor();
    let dock = w.buildings.last().unwrap().footprint();
    for cell in movement::interaction_cells(dock) {
        if w.terrain[usize::from(cell.row * WORLD_COLUMNS + cell.column)].biome
            != TerrainBiome::Water
            || w.ships.iter().any(|s| s.cell == cell)
        {
            continue;
        }
        let mut ship = w.ships[0].clone();
        ship.id = format!("block-{}-{}", cell.column, cell.row);
        ship.cell = cell;
        w.ships.push(ship);
    }
    let count = w.ships.len();
    for _ in 0..2 {
        w.apply_command(Command::Produce {
            building_id: "dock".into(),
            product: ProductKind::TransportShip,
        })
        .unwrap();
    }
    let paid = w.stockpile.clone();
    let queue_id = w.buildings.last().unwrap().queue[0].id;
    w.apply_command(Command::CancelQueuedJob {
        building_id: "dock".into(),
        queue_id,
    })
    .unwrap();
    assert_eq!(w.stockpile.wood, paid.wood + 60.0);
    assert_eq!(w.stockpile.timber, paid.timber + 20.0);
    w.tick(30.0);
    assert_eq!(w.ships.len(), count);
    assert!(w.buildings.last().unwrap().job.is_some());
    w.ships[0].cell = c(50, 40);
    w.tick(0.1);
    assert_eq!(w.ships.len(), count + 1);
    assert!(w.buildings.last().unwrap().job.is_none());
    w.validate().unwrap();
}

#[test]
fn ships_reserve_destinations_and_reload_mid_sail_deterministically() {
    let mut w = harbor();
    w.apply_command(Command::Produce {
        building_id: "dock".into(),
        product: ProductKind::TransportShip,
    })
    .unwrap();
    w.tick(20.0);
    let to = c(30, 40);
    w.apply_command(Command::Sail {
        ship_id: w.ships[0].id.clone(),
        to,
    })
    .unwrap();
    let before = w.clone();
    assert!(
        w.apply_command(Command::Sail {
            ship_id: w.ships[1].id.clone(),
            to
        })
        .is_err()
    );
    assert_eq!(w, before);
    w.tick(0.1);
    let mut loaded: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    for _ in 0..80 {
        w.tick(0.1);
        loaded.tick(0.1);
        assert_eq!(w, loaded);
    }
    assert_eq!(w.ships[0].cell, to);
}

#[test]
fn docking_selects_a_reachable_water_berth_and_rejects_other_buildings() {
    let mut w = harbor();
    let id = w.ships[0].id.clone();
    w.apply_command(Command::Sail {
        ship_id: id.clone(),
        to: c(40, 40),
    })
    .unwrap();
    w.tick(20.0);
    let before = w.clone();
    assert_eq!(
        w.apply_command(Command::DockShip {
            ship_id: id.clone(),
            building_id: "base-1".into()
        }),
        Err(CommandError::DockRequired)
    );
    assert_eq!(w, before);
    w.apply_command(Command::DockShip {
        ship_id: id,
        building_id: "dock".into(),
    })
    .unwrap();
    w.tick(20.0);
    assert!(w.ships[0].stopped());
    assert!(w.ships[0].beside(w.buildings.last().unwrap().footprint()));
    w.validate().unwrap();
}

#[test]
fn continuous_return_prefers_the_original_dock_and_keeps_position_until_sailing() {
    let mut world = harbor();
    world.buildings.insert(
        0,
        building(BuildingKind::Dock, "other-dock", c(50, 26), None),
    );
    world.discover_island();
    world.ships[0].cell = c(184, 0);
    let id = world.ships[0].id.clone();
    world
        .apply_command(Command::Voyage {
            ship_id: id,
            island_id: 0,
        })
        .unwrap();
    assert_eq!(world.ships[0].home_dock_id.as_deref(), Some("dock"));
    assert_eq!(world.ships[0].cell, c(184, 0));
    let target = world.ships[0].destination.unwrap();
    assert!((20..24).contains(&target.column));
    assert_eq!(target.row, 30);
    for _ in 0..200 {
        world.tick(0.5);
    }
    assert_eq!(world.ships[0].cell, target);
    assert!(world.ships[0].stopped());
}

#[test]
fn blocked_home_dock_rejects_the_continuous_return_atomically() {
    let mut world = harbor();
    world.discover_island();
    world.ships[0].cell = c(184, 0);
    for column in 20..24 {
        let mut blocker = world.ships[0].clone();
        blocker.id = format!("berth-{column}");
        blocker.cell = c(column, 30);
        world.ships.push(blocker);
    }
    let before = world.clone();
    assert!(
        world
            .apply_command(Command::Voyage {
                ship_id: world.ships[0].id.clone(),
                island_id: 0
            })
            .is_err()
    );
    assert_eq!(world, before);
}
