use super::*;

fn vessel(cell: CellCoordinate) -> TransportShip {
    TransportShip {
        id: "transport-test".into(),
        cell,
        step: None,
        destination: None,
        heading: [1, 0],
        passengers: Vec::new(),
        cargo: Default::default(),
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
        vec![ResourceKind::Iron, ResourceKind::Coal, ResourceKind::Clay],
        vec![ResourceKind::Clay],
        vec![ResourceKind::Fiber],
    ]
    .iter()
    .enumerate()
    {
        let origin = a.island_origins[i + 1];
        assert!(
            a.resources
                .iter()
                .filter(|r| r.cell.column >= origin.column
                    && r.cell.column < origin.column + WORLD_COLUMNS
                    && r.cell.row >= origin.row
                    && r.cell.row < origin.row + WORLD_ROWS)
                .all(|r| STARTER_RESOURCES.contains(&r.kind) || kinds.contains(&r.kind))
        );
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

fn settled_islands() -> GameWorld {
    let mut world = GameWorld::default();
    world.discover_island();
    let mut base = world.buildings[0].clone();
    base.id = "base-away".into();
    base.origin.column += 184;
    // Clear a settlement footprint and its adjacent production cells.
    world.resources.retain(|r| {
        r.cell.column.abs_diff(base.origin.column) > 12 || r.cell.row.abs_diff(base.origin.row) > 12
    });
    for cell in &mut world.terrain {
        if cell.column.abs_diff(base.origin.column) <= 12
            && cell.row.abs_diff(base.origin.row) <= 12
        {
            cell.biome = TerrainBiome::Meadow;
            cell.elevation = 0.0;
        }
    }
    world.buildings.push(base);
    world
}

#[test]
fn both_settlements_keep_producing_from_local_resources() {
    let mut world = settled_islands();
    world.inventories[0].food = 1000.0;
    world.inventories[1].food = 1000.0;
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
    assert_eq!(world.units.len(), 4);
    assert!(world.units.iter().filter(|u| u.cell.column >= 184).count() >= 1);
    world.validate().unwrap();
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

#[test]
fn local_training_cannot_spend_other_islands_food_and_refunds_stay_local() {
    let mut world = settled_islands();
    world.inventories[0].food = 100.0;
    let order = Command::Produce {
        building_id: "base-away".into(),
        product: ProductKind::Villager,
    };
    let before = world.clone();
    assert!(world.apply_command(order.clone()).is_err());
    assert_eq!(world, before);
    world.inventories[1].food = 100.0;
    world.apply_command(order.clone()).unwrap();
    world.apply_command(order).unwrap();
    assert_eq!(world.inventories[0].food, 100.0);
    assert_eq!(world.inventories[1].food, 100.0 - 2.0 * VILLAGER_FOOD_COST);
    let building = world
        .buildings
        .iter()
        .find(|b| b.id == "base-away")
        .unwrap();
    let queue_id = building.queue[0].id;
    world
        .apply_command(Command::CancelQueuedJob {
            building_id: "base-away".into(),
            queue_id,
        })
        .unwrap();
    assert_eq!(world.inventories[1].food, 100.0 - VILLAGER_FOOD_COST);
    assert_eq!(world.inventories[0].food, 100.0);
}
