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

/// Open sea `distance` cells outside a site, in the gap toward the start island.
pub(super) fn open_water_beside(site: CellCoordinate, distance: u16) -> CellCoordinate {
    if site.column >= WORLD_COLUMNS + distance {
        CellCoordinate::new(site.column - distance, site.row + 10)
    } else {
        CellCoordinate::new(site.column + 10, site.row - distance)
    }
}

#[test]
fn every_run_plans_five_to_seven_separated_sites_with_the_temple_farthest() {
    let mut sizes = BTreeSet::new();
    for seed in 0..200 {
        let plan = archipelago_plan(seed);
        assert_eq!(plan, archipelago_plan(seed));
        assert!((5..=7).contains(&plan.len()), "seed {seed}: {plan:?}");
        sizes.insert(plan.len());
        assert_eq!(plan[0], CellCoordinate::new(0, 0));
        for (i, a) in plan.iter().enumerate() {
            for b in &plan[i + 1..] {
                let apart_x = a.column.abs_diff(b.column) >= WORLD_COLUMNS + 64;
                let apart_y = a.row.abs_diff(b.row) >= WORLD_ROWS + 64;
                assert!(apart_x || apart_y, "seed {seed}: {a:?} {b:?}");
            }
        }
        // The temple site is not a neighbor of the start: at least two crossings.
        let temple = plan[plan.len() - 1];
        assert!(temple.column >= 2 * WORLD_COLUMNS || temple.row >= 2 * WORLD_ROWS);
        // Not a line: the sites span both directions.
        assert!(plan.iter().any(|s| s.column > 0) && plan.iter().any(|s| s.row > 0));
    }
    assert_eq!(sizes, BTreeSet::from([5, 6, 7]));
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
    assert_eq!(a.island_origins, archipelago_plan(17)[..4]);
    assert_eq!(a.units, home.units);
    assert_eq!(a.buildings, home.buildings);
    for cell in home.terrain {
        assert_eq!(
            a.terrain[usize::from(cell.row) * usize::from(a.columns()) + usize::from(cell.column)],
            cell
        );
    }
    for cell in &a.terrain {
        if island_at(&a.island_origins, cell.coordinate()).is_none() {
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
fn approaching_a_planned_site_discovers_it_without_moving_the_ship() {
    let mut world = GameWorld::default();
    world.ships.push(vessel(CellCoordinate::new(0, 0)));
    world.expand_archipelago();
    let plan = archipelago_plan(world.seed);
    assert_eq!(
        (world.columns(), world.rows()),
        plan_extent(&plan),
        "the ocean spans the whole run once a ship exists"
    );
    assert_eq!(world.island_origins.len(), 1);
    let site = plan[1];
    world.ships[0].cell = open_water_beside(site, 30);
    let target = open_water_beside(site, 2);
    world.sail("transport-test", target).unwrap();
    let mut previous = world.ships[0].cell;
    while world.island_origins.len() == 1 {
        world.tick(0.1);
        let cell = world.ships[0].cell;
        assert!(cell.column.abs_diff(previous.column) <= 1 && cell.row.abs_diff(previous.row) <= 1);
        previous = cell;
        assert!(
            islands::site_distance(site, cell) >= 12,
            "discovered too late"
        );
    }
    assert_eq!(world.island_origins[1], site);
    assert_eq!(world.ships[0].destination, Some(target));
    for _ in 0..300 {
        world.tick(0.1);
    }
    assert_eq!(world.ships[0].cell, target);
    world.validate().unwrap();
}

/// Steer toward the heart of a fogged site, as a player tapping the sea would.
pub(super) fn steer_toward(world: &mut GameWorld, ship_id: &str, site: CellCoordinate) {
    let heart = CellCoordinate::new(site.column + WORLD_COLUMNS / 2, site.row + WORLD_ROWS / 2);
    world
        .apply_command(Command::Sail {
            ship_id: ship_id.into(),
            to: heart,
        })
        .unwrap();
    for _ in 0..20_000 {
        if world.ships[0].stopped() {
            break;
        }
        world.tick(0.1);
    }
}

#[test]
fn only_steering_reaches_uncharted_islands_and_shortcuts_return_to_charted_ones() {
    let mut world = GameWorld::default();
    world.ships.push(vessel(CellCoordinate::new(0, 0)));
    world.expand_archipelago();
    let plan = archipelago_plan(world.seed);
    // No shortcut targets an undiscovered island.
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Voyage {
            ship_id: "transport-test".into(),
            island_id: 1,
        }),
        Err(CommandError::InvalidDestination)
    );
    assert_eq!(world, before);
    for &site in &plan[1..] {
        if world.island_origins.contains(&site) {
            continue; // discovered on the way to an earlier site
        }
        steer_toward(&mut world, "transport-test", site);
        assert!(world.island_origins.contains(&site), "{site:?}");
        world.validate().unwrap();
    }
    assert_eq!(world.island_origins.len(), plan.len());
    // Once discovered, one order returns to any island, home included.
    world
        .apply_command(Command::Voyage {
            ship_id: "transport-test".into(),
            island_id: 0,
        })
        .unwrap();
    for _ in 0..20_000 {
        if world.ships[0].stopped() {
            break;
        }
        world.tick(0.1);
    }
    assert_eq!(world.island_at(world.ships[0].cell), Some(0));
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
    let destination = world.ships[0].destination.unwrap();
    assert!(islands::site_distance(world.island_origins[1], destination) <= 1);
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
    base.origin.column += world.island_origins[1].column;
    base.origin.row += world.island_origins[1].row;
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
    assert!(
        world
            .units
            .iter()
            .any(|u| world.island_at(u.cell) == Some(1))
    );
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
    assert_eq!(
        (loaded.columns, loaded.rows),
        plan_extent(&world.island_origins)
    );
    // Fog: only the temple site is revealed, not the other planned islands.
    assert_eq!(
        Some(&loaded.temple_site),
        archipelago_plan(world.seed).last()
    );
    let json = serde_json::to_string(&snapshot).unwrap();
    for site in &archipelago_plan(world.seed)[3..] {
        let hidden = serde_json::to_string(site).unwrap();
        assert!(*site == loaded.temple_site || !json.contains(&hidden));
    }
}

#[test]
#[ignore = "manual release-mode memory and snapshot budget measurement"]
fn archipelago_budget() {
    let mut world = GameWorld::default();
    for count in [1, 4, archipelago_plan(world.seed).len()] {
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
    world
        .ships
        .push(vessel(open_water_beside(world.island_origins[1], 20)));
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

#[test]
fn steering_into_an_uncharted_site_discovers_it_and_sails_on_to_its_coast() {
    let mut world = GameWorld::default();
    world.ships.push(vessel(CellCoordinate::new(0, 0)));
    world.expand_archipelago();
    let site = archipelago_plan(world.seed)[1];
    world.ships[0].cell = open_water_beside(site, 30);
    // Aim at the middle of the fogged site: open sea now, land once discovered.
    let heart = CellCoordinate::new(site.column + WORLD_COLUMNS / 2, site.row + WORLD_ROWS / 2);
    world.sail("transport-test", heart).unwrap();
    for _ in 0..600 {
        world.tick(0.1);
        world.validate().unwrap();
    }
    assert_eq!(world.island_origins.last(), Some(&site));
    assert!(
        world.ships[0].stopped(),
        "the ship must not chase a destination that became land"
    );
    // It sailed on into the island's waters, close enough to see the new coast.
    assert_eq!(islands::site_distance(site, world.ships[0].cell), 0);
    let shore = world
        .terrain
        .iter()
        .filter(|c| c.biome != TerrainBiome::Water)
        .map(|c| c.coordinate().center().distance(world.ships[0].position()))
        .fold(f64::MAX, f64::min);
    assert!(
        shore <= UNIT_SIGHT_RADIUS,
        "nearest land {shore} cells away"
    );
    let at = world.ships[0].cell;
    let stride = usize::from(world.columns());
    assert_eq!(
        world.terrain[usize::from(at.row) * stride + usize::from(at.column)].biome,
        TerrainBiome::Water
    );
}
