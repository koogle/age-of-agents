use super::*;

fn vessel(cell: CellCoordinate) -> TransportShip {
    TransportShip {
        id: "transport-test".into(),
        cell,
        step: None,
        destination: None,
        heading: [1, 0],
        passengers: Vec::new(),
        home_dock_id: None,
    }
}

#[test]
fn discovery_keeps_home_and_places_deterministic_separated_islands() {
    let mut a = GameWorld::generate(17);
    let home = a.clone();
    for _ in 0..3 {
        a.discover_island();
    }
    let mut b = GameWorld::generate(17);
    for _ in 0..3 {
        b.discover_island();
    }
    assert_eq!(a, b);
    assert_eq!(
        a.island_origins,
        vec![
            CellCoordinate::new(0, 0),
            CellCoordinate::new(184, 0),
            CellCoordinate::new(184, 144),
            CellCoordinate::new(0, 144)
        ]
    );
    assert_eq!(a.units, home.units);
    assert_eq!(a.buildings, home.buildings);
    for cell in home.terrain {
        assert_eq!(
            a.terrain[usize::from(cell.row) * usize::from(a.columns()) + usize::from(cell.column)],
            cell
        );
    }
    for cell in &a.terrain {
        if (120..184).contains(&cell.column) || (80..144).contains(&cell.row) {
            assert_eq!(cell.biome, TerrainBiome::Water);
        }
    }
    for (i, kinds) in [
        vec![ResourceKind::Iron, ResourceKind::Coal],
        vec![ResourceKind::Clay],
        vec![ResourceKind::Fiber],
    ]
    .iter()
    .enumerate()
    {
        let origin = a.island_origins[i + 1];
        for kind in kinds {
            assert!(a.resources.iter().any(|r| r.kind == *kind
                && r.cell.column >= origin.column
                && r.cell.column < origin.column + WORLD_COLUMNS
                && r.cell.row >= origin.row
                && r.cell.row < origin.row + WORLD_ROWS));
        }
    }
    a.validate().unwrap();
    let saved: GameWorld = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
    assert_eq!(saved, a);
}

#[test]
fn frontier_discovery_preserves_ship_position_and_sailing_order() {
    let mut world = GameWorld::default();
    world.ships.push(vessel(CellCoordinate::new(106, 0)));
    world
        .sail("transport-test", CellCoordinate::new(119, 0))
        .unwrap();
    world.tick(0.1);
    assert_eq!(world.island_origins.len(), 1);
    for _ in 0..8 {
        world.tick(0.1);
    }
    assert_eq!(world.island_origins.len(), 2);
    assert_eq!(
        world.ships[0].destination,
        Some(CellCoordinate::new(119, 0))
    );
    assert!(world.ships[0].cell.column < 112);
    assert_eq!(world.ships[0].cell.row, 0);
    for _ in 0..50 {
        world.tick(0.1);
    }
    assert_eq!(world.island_origins.len(), 2);
    assert_eq!(world.ships[0].cell, CellCoordinate::new(119, 0));
    world
        .sail("transport-test", CellCoordinate::new(184, 0))
        .unwrap();
    for _ in 0..200 {
        world.tick(0.1);
    }
    assert_eq!(world.ships[0].cell, CellCoordinate::new(184, 0));
    world.validate().unwrap();
}

#[test]
fn destination_shortcut_sails_without_swapping_or_teleporting() {
    let mut world = GameWorld::default();
    world.discover_island();
    world.ships.push(vessel(CellCoordinate::new(0, 0)));
    let home = world.buildings.clone();
    world
        .apply_command(Command::Voyage {
            ship_id: "transport-test".into(),
            island_id: 1,
        })
        .unwrap();
    assert_eq!(world.ships[0].cell, CellCoordinate::new(0, 0));
    assert!(world.ships[0].destination.unwrap().column >= 184);
    assert_eq!(world.buildings, home);
    assert_eq!(world.island_id, 0);
    let before = world.clone();
    assert!(
        world
            .apply_command(Command::Voyage {
                ship_id: "transport-test".into(),
                island_id: 50
            })
            .is_err()
    );
    assert_eq!(world, before);
}

fn legacy_world() -> GameWorld {
    let mut world = GameWorld::default();
    let mut away = GameWorld::generate(42);
    away.units[0].id = "villager-away-1".into();
    away.units[1].id = "villager-away-2".into();
    away.buildings[0].id = "base-away".into();
    world.islands.push(IslandState {
        id: 1,
        terrain: away.terrain,
        explored_cells: away.explored_cells,
        units: away.units,
        ships: vec![vessel(CellCoordinate::new(0, 0))],
        resources: away.resources,
        buildings: away.buildings,
    });
    world
}

#[test]
fn migration_preserves_every_settlement_and_orders_and_is_idempotent() {
    let mut world = legacy_world();
    let old = world.clone();
    world.unify_islands().unwrap();
    assert!(world.islands.is_empty());
    assert_eq!(world.buildings.len(), 2);
    assert_eq!(world.units.len(), 4);
    assert_eq!(world.ships[0].cell, CellCoordinate::new(184, 0));
    assert_eq!(world.stockpile, old.stockpile);
    let mut expected = old.islands[0].buildings[0].origin;
    expected.column += 184;
    assert_eq!(world.buildings[1].origin, expected);
    let migrated = world.clone();
    world.unify_islands().unwrap();
    assert_eq!(world, migrated);
    world.validate().unwrap();
}

#[test]
fn both_settlements_keep_producing_from_shared_resources() {
    let mut world = legacy_world();
    world.unify_islands().unwrap();
    world.stockpile.food = 1000.0;
    for id in ["base-1", "base-away"] {
        world
            .apply_command(Command::Produce {
                building_id: id.into(),
                product: ProductKind::Villager,
            })
            .unwrap();
    }
    for _ in 0..100 {
        world.tick(1.0);
    }
    assert_eq!(world.units.len(), 6);
    assert!(world.units.iter().filter(|u| u.cell.column >= 184).count() >= 3);
    world.validate().unwrap();
}

#[test]
fn migration_rejects_corrupt_archives_without_replacing_them() {
    let mut world = legacy_world();
    world.islands[0].terrain.pop();
    let before = world.clone();
    assert!(world.unify_islands().is_err());
    assert_eq!(world, before);
}

#[test]
fn expanded_snapshot_terrain_round_trips_runtime_dimensions() {
    let mut world = GameWorld::default();
    world.discover_island();
    world.discover_island();
    let snapshot = world.snapshot();
    let loaded: WorldSnapshot =
        serde_json::from_str(&serde_json::to_string(&snapshot).unwrap()).unwrap();
    assert_eq!(loaded, snapshot);
    assert_eq!((loaded.columns, loaded.rows), (304, 224));
}

#[test]
#[ignore = "manual release-mode memory and snapshot budget measurement"]
fn archipelago_budget() {
    let mut world = GameWorld::default();
    for count in [1, 4, 16, 64] {
        while world.island_origins.len() < count {
            world.discover_island();
        }
        let terrain_bytes = world.terrain.capacity() * std::mem::size_of::<TerrainCell>();
        let start = std::time::Instant::now();
        for _ in 0..10 {
            world.tick(0.1);
        }
        let tick_ms = start.elapsed().as_secs_f64() * 100.0;
        let start = std::time::Instant::now();
        let bytes = serde_json::to_vec(&world.snapshot()).unwrap().len();
        eprintln!(
            "islands={count} cells={} terrain_bytes={terrain_bytes} snapshot_bytes={bytes} tick_ms={tick_ms:.3} snapshot_ms={:.3}",
            world.terrain.len(),
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
}

#[test]
fn migration_translates_mid_voyage_steps_passengers_and_destinations() {
    let mut world = legacy_world();
    let island = &mut world.islands[0];
    let passenger = island.units.remove(0);
    let old_cell = passenger.cell;
    island.ships[0].passengers.push(passenger);
    island.ships[0].step = Some(Step {
        to: CellCoordinate::new(1, 0),
        progress: 0.3,
    });
    island.ships[0].destination = Some(CellCoordinate::new(5, 0));
    world.unify_islands().unwrap();
    let ship = &world.ships[0];
    assert_eq!(
        ship.step,
        Some(Step {
            to: CellCoordinate::new(185, 0),
            progress: 0.3
        })
    );
    assert_eq!(ship.destination, Some(CellCoordinate::new(189, 0)));
    assert_eq!(
        ship.passengers[0].cell,
        CellCoordinate::new(old_cell.column + 184, old_cell.row)
    );
    world.validate().unwrap();
}

#[test]
fn local_visibility_matches_the_reference_full_map_scan_after_expansion() {
    let mut world = GameWorld::default();
    world.discover_island();
    world.ships.push(vessel(CellCoordinate::new(184, 0)));
    let eyes: Vec<_> = world
        .units
        .iter()
        .map(|u| (u.position(), UNIT_SIGHT_RADIUS))
        .chain(
            world
                .ships
                .iter()
                .map(|s| (s.position(), UNIT_SIGHT_RADIUS)),
        )
        .chain(
            world
                .buildings
                .iter()
                .map(|b| (b.footprint().center(), b.kind.sight_radius())),
        )
        .collect();
    let expected: BTreeSet<_> = world
        .terrain
        .iter()
        .map(|c| c.coordinate())
        .filter(|c| {
            eyes.iter()
                .any(|(eye, radius)| eye.distance(c.center()) <= *radius)
        })
        .collect();
    assert_eq!(world.visible_cells(), expected);
}
