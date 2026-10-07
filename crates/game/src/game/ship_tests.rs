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
    w.inventories[0].wood = 1000.0;
    w.inventories[0].timber = 1000.0;
    w.apply_command(Command::Produce {
        building_id: "dock".into(),
        product: ProductKind::TransportShip,
    })
    .unwrap();
    w.tick(20.0);
    assert_eq!(w.ships.len(), 1);
    w
}

#[test]
fn water_uses_ship_capacity_and_only_supplies_the_connected_island() {
    let mut w = harbor();
    w.inventories[0].water = 60.0;
    let ship_id = w.ships[0].id.clone();
    w.apply_command(Command::TransferShipCargo {
        ship_id: ship_id.clone(),
        kind: ResourceKind::Water,
        amount: 50.0,
        direction: CargoDirection::Load,
    })
    .unwrap();
    assert_eq!(w.ships[0].cargo.water, 50.0);
    assert_eq!(w.inventories[0].water, 10.0);
    let before = w.clone();
    assert!(
        w.apply_command(Command::TransferShipCargo {
            ship_id,
            kind: ResourceKind::Water,
            amount: 1.0,
            direction: CargoDirection::Load
        })
        .is_err()
    );
    assert_eq!(w, before);
    w.spend_at(c(10, 10), &[(ResourceKind::Water, 20.0)])
        .unwrap();
    assert_eq!(w.inventories[0].water, 0.0);
    assert_eq!(w.ships[0].cargo.water, 40.0);
    w.discover_island();
    let before = w.clone();
    assert!(
        w.spend_at(w.island_origins[1], &[(ResourceKind::Water, 1.0)])
            .is_err()
    );
    assert_eq!(w, before);
    w.ships[0].destination = Some(c(50, 40));
    assert_eq!(w.available_on(0).water, 0.0);
    let loaded: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    assert_eq!(loaded.ships[0].cargo.water, 40.0);
    loaded.validate().unwrap();
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
    assert_eq!(w.inventories[0].wood, 940.0);
    assert_eq!(w.inventories[0].timber, 980.0);
    assert_eq!(w.villagers_and_trainees(), 2);
    for _ in 0..10 {
        w.tick(1.0);
    }
    assert_eq!(w.ships.len(), 1);
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
fn missing_ships_and_corrupt_manifests_fail() {
    let w = GameWorld::default();
    let mut json = serde_json::to_value(&w).unwrap();
    json.as_object_mut().unwrap().remove("ships");
    assert!(serde_json::from_value::<GameWorld>(json).is_err());
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
        if w.terrain[usize::from(cell.row) * usize::from(w.columns()) + usize::from(cell.column)]
            .biome
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
    let paid = w.inventories[0].clone();
    let queue_id = w.buildings.last().unwrap().queue[0].id;
    w.apply_command(Command::CancelQueuedJob {
        building_id: "dock".into(),
        queue_id,
    })
    .unwrap();
    assert_eq!(w.inventories[0].wood, paid.wood + 60.0);
    assert_eq!(w.inventories[0].timber, paid.timber + 20.0);
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
    let at_sea = super::islands_tests::open_water_beside(world.island_origins[1], 20);
    world.ships[0].cell = at_sea;
    let id = world.ships[0].id.clone();
    world
        .apply_command(Command::Voyage {
            ship_id: id,
            island_id: 0,
        })
        .unwrap();
    assert_eq!(world.ships[0].home_dock_id.as_deref(), Some("dock"));
    assert_eq!(world.ships[0].cell, at_sea);
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
    world.ships[0].cell = super::islands_tests::open_water_beside(world.island_origins[1], 20);
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

#[test]
fn cargo_transfers_are_atomic_and_share_one_fifty_resource_hold() {
    let mut w = harbor();
    let transfer = |kind, amount, direction| Command::TransferShipCargo {
        ship_id: "transport-1".into(),
        kind,
        amount,
        direction,
    };
    let id = w.ships[0].id.clone();
    let transfer = |kind, amount, direction| {
        let mut command = transfer(kind, amount, direction);
        if let Command::TransferShipCargo { ship_id, .. } = &mut command {
            *ship_id = id.clone();
        }
        command
    };
    let before = w.available_on(0);
    w.apply_command(transfer(ResourceKind::Wood, 30.0, CargoDirection::Load))
        .unwrap();
    w.apply_command(transfer(ResourceKind::Timber, 20.0, CargoDirection::Load))
        .unwrap();
    assert_eq!(w.ships[0].cargo.total(), 50.0);
    assert_eq!(w.available_on(0), before);
    for amount in [1.0, -1.0, f64::NAN, f64::INFINITY] {
        let saved = w.clone();
        assert!(
            w.apply_command(transfer(ResourceKind::Wood, amount, CargoDirection::Load))
                .is_err()
        );
        assert_eq!(w, saved);
    }
    w.apply_command(transfer(ResourceKind::Wood, 10.0, CargoDirection::Unload))
        .unwrap();
    assert_eq!(w.ships[0].cargo.total(), 40.0);
    assert_eq!(w.available_on(0), before);
    let loaded: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    assert_eq!(loaded, w);
    w.ships[0].destination = Some(c(50, 40));
    assert_eq!(w.available_on(0), w.inventories[0]);
    assert!(
        w.apply_command(transfer(ResourceKind::Wood, 1.0, CargoDirection::Load))
            .is_err()
    );
}

#[test]
fn shore_ship_supplies_construction_and_accepts_partial_villager_deposits() {
    let mut w = harbor();
    w.buildings.retain(|b| b.kind != BuildingKind::Dock);
    w.ships[0].cell = c(20, 30);
    w.ships[0].home_dock_id = None;
    w.inventories[0] = Stockpile::default();
    w.ships[0].cargo.wood = 45.0;
    assert_eq!(w.connected_island(0), Some(0));
    let id = w.ships[0].id.clone();
    w.units[0].cell = c(20, 29);
    w.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Stone,
        amount: 12.0,
    });
    w.apply_command(Command::Deposit {
        unit_id: w.units[0].id.clone(),
        storage_id: id.clone(),
    })
    .unwrap();
    w.tick(0.1);
    assert_eq!(w.ships[0].cargo.total(), 50.0);
    assert_eq!(w.units[0].cargo.as_ref().unwrap().amount, 7.0);
    assert!(
        w.apply_command(Command::TransferShipCargo {
            ship_id: id,
            kind: ResourceKind::Wood,
            amount: 1.0,
            direction: CargoDirection::Unload
        })
        .is_err()
    );
    w.spend_at(c(10, 10), &[(ResourceKind::Wood, 20.0)])
        .unwrap();
    assert_eq!(w.ships[0].cargo.wood, 25.0);
    assert_eq!(w.inventories[0].wood, 0.0);
    w.discover_island();
    let before = w.clone();
    assert!(
        w.spend_at(w.island_origins[1], &[(ResourceKind::Wood, 1.0)])
            .is_err()
    );
    assert_eq!(w, before);
    w.ships[0].destination = Some(c(50, 40));
    assert_eq!(w.available_on(0).wood, 0.0);
    assert_eq!(w.ships[0].cargo.wood, 25.0);
}

#[test]
fn offshore_pickup_meets_at_shore_and_reloads_mid_approach() {
    let mut w = harbor();
    w.ships[0].cell = c(40, 45);
    let start = w.ships[0].cell;
    w.units[0].cell = c(40, 29);
    w.units[1].cell = c(41, 28);
    order_board(&mut w, 0);
    let berth = w.ships[0].destination.unwrap();
    assert_eq!(berth.row, 30);
    assert_eq!(w.ships[0].cell, start);
    w.tick(0.1);
    assert!(w.ships[0].passengers.is_empty());
    order_board(&mut w, 1);
    assert_eq!(w.ships[0].destination, Some(berth));
    order_board(&mut w, 0);
    let mut loaded: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    for _ in 0..200 {
        w.tick(0.1);
        loaded.tick(0.1);
        w.validate().unwrap();
        assert_eq!(w, loaded);
    }
    assert_eq!(w.ships[0].cell, berth);
    assert_eq!(w.ships[0].passengers.len(), 2);
    assert!(w.units.is_empty());
}

#[test]
fn unreachable_pickup_rejects_without_changing_unit_or_ship() {
    let mut w = harbor();
    w.ships[0].cell = c(40, 45);
    // An unbroken water strip separates the units from every shore. Seal the
    // run's open ocean beyond the start island so it cannot bridge the strip.
    for tile in &mut w.terrain {
        if tile.column >= WORLD_COLUMNS || tile.row >= WORLD_ROWS {
            tile.biome = TerrainBiome::Mountain;
        } else if tile.row == 25 {
            tile.biome = TerrainBiome::Water;
        }
    }
    let before = w.clone();
    assert_eq!(
        w.apply_command(Command::Board {
            unit_id: w.units[0].id.clone(),
            ship_id: w.ships[0].id.clone(),
        }),
        Err(CommandError::ShoreBlocked)
    );
    assert_eq!(w, before);
}

#[test]
fn explicit_ship_orders_cancel_pickup_and_keep_steps() {
    for stop in [false, true] {
        let mut w = harbor();
        w.ships[0].cell = c(40, 45);
        order_board(&mut w, 0);
        w.tick(0.1);
        let step = w.ships[0].step;
        let ship_id = w.ships[0].id.clone();
        w.apply_command(if stop {
            Command::StopShip { ship_id }
        } else {
            Command::Sail {
                ship_id,
                to: c(60, 50),
            }
        })
        .unwrap();
        assert_eq!(w.units[0].action, UnitAction::Idle);
        assert_eq!(w.ships[0].step, step);
        w.tick(1.0);
        w.validate().unwrap();
    }
}

#[test]
fn pickup_redirects_a_sailing_ship_without_interrupting_its_step() {
    let mut w = harbor();
    w.ships[0].cell = c(40, 40);
    w.apply_command(Command::Sail {
        ship_id: w.ships[0].id.clone(),
        to: c(60, 50),
    })
    .unwrap();
    w.tick(0.1);
    let step = w.ships[0].step;
    order_board(&mut w, 0);
    assert_eq!(w.ships[0].step, step);
    assert_eq!(w.ships[0].destination.unwrap().row, 30);
    for _ in 0..400 {
        w.tick(0.1);
    }
    assert_eq!(w.ships[0].passengers.len(), 1);
    w.validate().unwrap();
}

#[test]
fn four_passengers_wait_for_pickup_on_a_crowded_shore() {
    let mut w = harbor();
    w.ships[0].cell = c(40, 60);
    w.units[0].cell = c(40, 29);
    w.units[1].cell = c(39, 29);
    for n in 0..2 {
        let mut unit = w.units[0].clone();
        unit.id = format!("pickup-{n}");
        unit.cell = c(40 + n, 28);
        w.units.push(unit);
    }
    for i in 0..4 {
        order_board(&mut w, i);
    }
    let before = w.clone();
    order_board(&mut w, 0);
    assert_eq!(w, before, "repeated boarding must not consume another seat");
    for _ in 0..400 {
        w.tick(0.1);
    }
    assert_eq!(w.ships[0].passengers.len(), 4, "remaining: {:?}", w.units);
    w.validate().unwrap();
}

#[test]
fn a_newly_launched_ship_can_steer_into_the_fog_at_once() {
    let mut w = harbor();
    assert_eq!(
        (w.columns(), w.rows()),
        plan_extent(&archipelago_plan(w.seed)),
        "the ocean must span the run as soon as a dock launches a ship"
    );
    let id = w.ships[0].id.clone();
    let site = archipelago_plan(w.seed)[1];
    super::islands_tests::steer_toward(&mut w, &id, site);
    assert_eq!(w.island_origins[1], site);
    w.validate().unwrap();
}
