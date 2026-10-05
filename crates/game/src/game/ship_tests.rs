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
    assert_eq!(w.islands.len(), 1);
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

fn ocean_harbor() -> GameWorld {
    let mut world = harbor();
    // Connect the fixture's southern sea to the ocean arrival corner.
    for cell in &mut world.terrain {
        if cell.column == 0 || cell.row == 0 {
            cell.biome = TerrainBiome::Water;
            cell.elevation = 0.0;
        }
    }
    world.validate().unwrap();
    world
}

fn visit(world: &mut GameWorld, island_id: u64) -> Result<(), CommandError> {
    world.apply_command(Command::Voyage {
        ship_id: world.ships[0].id.clone(),
        island_id,
    })
}

#[test]
fn return_voyage_remembers_birth_dock_across_reload_and_ignores_other_docks() {
    let mut world = ocean_harbor();
    assert_eq!(world.ships[0].home_dock_id.as_deref(), Some("dock"));
    let dock = world
        .buildings
        .iter()
        .find(|b| b.id == "dock")
        .unwrap()
        .footprint();
    // This dock is closer to the ocean entry, but it is not this ship's home.
    world
        .buildings
        .push(building(BuildingKind::Dock, "other-dock", c(5, 26), None));
    let stock = world.stockpile.clone();
    visit(&mut world, 1).unwrap();
    let mut restored: GameWorld =
        serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
    visit(&mut restored, 0).unwrap();
    assert!(restored.ships[0].beside(dock));
    assert!(restored.ships[0].stopped());
    assert_eq!(restored.stockpile, stock);
    // Home has no resource nodes: returning to a dock must not require trees.
    assert!(restored.resources.is_empty());
    restored.validate().unwrap();
}

#[test]
fn legacy_ships_learn_the_departure_dock_and_away_ships_can_find_a_dock() {
    let world = ocean_harbor();
    let mut json = serde_json::to_value(&world).unwrap();
    json["ships"][0]
        .as_object_mut()
        .unwrap()
        .remove("home_dock_id");
    let mut old: GameWorld = serde_json::from_value(json).unwrap();
    assert!(old.ships[0].home_dock_id.is_none());
    visit(&mut old, 1).unwrap();
    assert_eq!(old.ships[0].home_dock_id.as_deref(), Some("dock"));
    // Also support saves made while already away, without origin metadata.
    old.ships[0].home_dock_id = None;
    visit(&mut old, 0).unwrap();
    assert!(
        old.ships[0].beside(
            world
                .buildings
                .iter()
                .find(|b| b.id == "dock")
                .unwrap()
                .footprint()
        )
    );
    old.validate().unwrap();
}

#[test]
fn return_voyage_uses_another_home_berth_and_rejects_a_fully_blocked_dock_atomically() {
    let mut world = ocean_harbor();
    let template = world.ships[0].clone();
    visit(&mut world, 1).unwrap();
    let home = world.islands.iter_mut().find(|i| i.id == 0).unwrap();
    let dock = home
        .buildings
        .iter()
        .find(|b| b.id == "dock")
        .unwrap()
        .footprint();
    let mut blocker = template.clone();
    blocker.id = "blocker-first".into();
    home.ships.push(blocker);
    let blocked_cell = template.cell;
    let mut partly_blocked = world.clone();
    visit(&mut partly_blocked, 0).unwrap();
    let ship = partly_blocked
        .ships
        .iter()
        .find(|s| s.id == template.id)
        .unwrap();
    assert!(ship.beside(dock));
    assert_ne!(ship.cell, blocked_cell);
    partly_blocked.validate().unwrap();

    let home = world.islands.iter_mut().find(|i| i.id == 0).unwrap();
    home.ships.clear();
    for (index, cell) in movement::interaction_cells(dock).enumerate() {
        if home.terrain[usize::from(cell.row * WORLD_COLUMNS + cell.column)].biome
            == TerrainBiome::Water
            && dock
                .cells()
                .any(|land| land.column.abs_diff(cell.column) + land.row.abs_diff(cell.row) == 1)
        {
            let mut blocker = template.clone();
            blocker.id = format!("blocker-{index}");
            blocker.cell = cell;
            home.ships.push(blocker);
        }
    }
    world.validate().unwrap();
    let before = world.clone();
    assert_eq!(visit(&mut world, 0), Err(CommandError::ShoreBlocked));
    assert_eq!(world, before);
}
