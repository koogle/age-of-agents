use super::*;

#[test]
fn first_and_later_islands_have_reachable_renewable_riverbank_water() {
    for seed in 0..24 {
        for id in 0..4 {
            let island = if id == 0 {
                generate(seed)
            } else {
                destination(seed, id)
            };
            let sources: Vec<_> = island
                .resources
                .iter()
                .filter(|r| r.kind == ResourceKind::Water)
                .collect();
            assert!(!sources.is_empty(), "seed {seed}, island {id}");
            for r in sources {
                let i = index(r.cell);
                assert!(island.terrain[i].biome.is_walkable());
                assert!(neighbours4(i).any(|n| island.terrain[n].biome == TerrainBiome::River));
                assert_eq!(r.amount, r.capacity);
            }
            assert_eq!(
                reachable_only(
                    &island.terrain,
                    island.resources.clone(),
                    island.town_center,
                    island.villagers[0]
                )
                .len(),
                island.resources.len()
            );
        }
    }
}

fn index(cell: CellCoordinate) -> usize {
    at(usize::from(cell.column), usize::from(cell.row))
}

#[test]
fn one_seed_always_grows_the_same_island_and_seeds_differ() {
    assert_eq!(GameWorld::generate(7), GameWorld::generate(7));
    assert_ne!(
        GameWorld::generate(7).terrain,
        GameWorld::generate(8).terrain
    );
    assert_eq!(GameWorld::default().seed, DEFAULT_SEED);
}

#[test]
fn every_island_is_one_landmass_ringed_by_sea_and_beach() {
    for seed in 0..24 {
        let world = GameWorld::generate(seed);
        world.validate().unwrap();
        let land: Vec<bool> = world
            .terrain
            .iter()
            .map(|c| c.biome != TerrainBiome::Water)
            .collect();
        for (i, cell) in world.terrain.iter().enumerate() {
            let edge = cell.column == 0
                || cell.row == 0
                || cell.column == WORLD_COLUMNS - 1
                || cell.row == WORLD_ROWS - 1;
            assert!(!(edge && land[i]), "seed {seed}: land touches the map edge");
            assert_eq!(
                land[i],
                cell.elevation > 0.0,
                "seed {seed}: elevation disagrees with water"
            );
        }
        let first = land.iter().position(|&l| l).unwrap();
        let reached = distance_from(std::iter::once(first), |i| land[i]);
        assert!(
            land.iter()
                .zip(&reached)
                .all(|(&l, &d)| !l || d != u32::MAX),
            "seed {seed}: more than one island"
        );
        let share = land.iter().filter(|&&l| l).count() as f64 / land.len() as f64;
        assert!(
            (0.35..0.6).contains(&share),
            "seed {seed}: land share {share}"
        );
        assert!(world.terrain.iter().any(|c| c.biome == TerrainBiome::Beach));
        assert!(
            world
                .terrain
                .iter()
                .any(|c| c.biome == TerrainBiome::Highland)
        );
    }
}

#[test]
fn every_island_holds_enough_reachable_resources_to_reach_a_boat() {
    for seed in 0..24 {
        let world = GameWorld::generate(seed);
        assert!(
            world
                .resources
                .iter()
                .all(|r| STARTER_RESOURCES.contains(&r.kind))
        );
        for &(kind, cost) in &STARTER_RESOURCE_BUDGET {
            let total: f64 = world
                .resources
                .iter()
                .filter(|r| r.kind == kind)
                .map(|r| r.amount)
                .sum();
            assert!(total >= cost * 1.5, "seed {seed}: only {total} {kind:?}");
        }
        for (number, resource) in world.resources.iter().enumerate() {
            assert!(
                resource.kind == ResourceKind::Water
                    || grows_in(resource.kind).contains(&world.terrain[index(resource.cell)].biome)
            );
            // Villager 1 can walk to every node (the real pathfinder, not the generator's check).
            assert!(
                world.can_reach_beside(0, resource.footprint()),
                "seed {seed}: {} unreachable",
                resource.id
            );
            assert!(
                world.resources[number + 1..]
                    .iter()
                    .all(|other| other.id != resource.id)
            );
        }
        let base = world.buildings[0].footprint();
        assert!(world.units.iter().all(|u| base.is_interaction_cell(u.cell)));
        for kind in [ResourceKind::Wood, ResourceKind::Food] {
            assert!(
                world.resources.iter().any(
                    |r| r.kind == kind && r.cell.center().distance(base.center()) <= NEAR_START
                )
            );
        }
        for kind in [ResourceKind::Wood, ResourceKind::Food] {
            assert!(
                world.snapshot().resources.iter().any(|r| r.kind == kind),
                "seed {seed}: starter {kind:?} hidden by fog"
            );
        }
        // Everything else has to be found by exploring.
        for resource in &world.resources {
            if !matches!(
                resource.kind,
                ResourceKind::Wood | ResourceKind::Food | ResourceKind::Water
            ) {
                assert!(
                    resource.cell.center().distance(base.center()) >= FAR_FROM_START - 2.0,
                    "seed {seed}: {} lies by the start",
                    resource.id
                );
            }
        }
    }
}

#[test]
fn every_island_has_ranges_of_peaks_and_rivers_that_reach_the_water() {
    for seed in 0..24 {
        let world = GameWorld::generate(seed);
        let biome = |i: usize| world.terrain[i].biome;
        let count = |b: TerrainBiome| world.terrain.iter().filter(|c| c.biome == b).count();
        assert!(count(TerrainBiome::Mountain) > 0, "seed {seed}: no peaks");
        // Peaks stand in several separate ranges, not one massif.
        let peaks: Vec<usize> = (0..world.terrain.len())
            .filter(|&i| biome(i) == TerrainBiome::Mountain)
            .collect();
        let mut ranges = 0;
        let mut seen = vec![false; world.terrain.len()];
        for &peak in &peaks {
            if !seen[peak] {
                ranges += 1;
                let reach = distance_from(std::iter::once(peak), |i| {
                    biome(i) == TerrainBiome::Mountain
                });
                for (i, &d) in reach.iter().enumerate() {
                    seen[i] |= d != u32::MAX;
                }
            }
        }
        assert!(ranges >= 2, "seed {seed}: only {ranges} mountain range");
        assert!(
            count(TerrainBiome::River) >= MIN_RIVER_LENGTH,
            "seed {seed}: no river"
        );
        // Following river cells (and the fords across them) leads to the water.
        let wet = |i: usize| matches!(biome(i), TerrainBiome::River | TerrainBiome::Beach);
        let mouth = distance_from(
            (0..world.terrain.len()).filter(|&i| biome(i) == TerrainBiome::Water),
            wet,
        );
        for (i, cell) in world.terrain.iter().enumerate() {
            if cell.biome == TerrainBiome::River {
                assert!(mouth[i] != u32::MAX, "seed {seed}: river cut off at {i}");
                assert!(cell.elevation > 0.0);
            }
        }
        // Every peak towers over the land around it on average.
        let mean = |b: TerrainBiome| {
            let cells = world.terrain.iter().filter(|c| c.biome == b);
            cells.clone().map(|c| f64::from(c.elevation)).sum::<f64>() / cells.count() as f64
        };
        assert!(mean(TerrainBiome::Mountain) > mean(TerrainBiome::Highland));
        assert!(mean(TerrainBiome::Highland) > mean(TerrainBiome::Meadow));
    }
}

#[test]
fn nobody_walks_or_builds_on_peaks_or_rivers() {
    let world = GameWorld::generate(1);
    for biome in [TerrainBiome::Mountain, TerrainBiome::River] {
        let cell = world
            .terrain
            .iter()
            .find(|c| c.biome == biome)
            .unwrap()
            .coordinate();
        let mut moved = world.clone();
        assert!(
            moved
                .apply_command(Command::Move {
                    unit_id: "villager-1".into(),
                    to: cell
                })
                .is_err()
        );
        let mut built = world.clone();
        built.inventories[0].wood = 1000.0;
        built.inventories[0].stone = 1000.0;
        let origin = CellCoordinate::new(cell.column.saturating_sub(1), cell.row.saturating_sub(1));
        let before_buildings = built.buildings.clone();
        let before_stockpile = built.inventories[0].clone();
        let _ = built.apply_command(Command::Build {
            kind: BuildingKind::House,
            unit_id: "villager-1".into(),
            origin,
        });
        // Fogged sites may accept exploration, but never create an invalid foundation.
        for _ in 0..400 {
            built.tick(0.1);
        }
        assert_eq!(built.buildings, before_buildings);
        assert_eq!(built.inventories[0], before_stockpile);
        let mut stranded = world.clone();
        stranded.units[0].cell = cell;
        assert!(stranded.validate().is_err());
    }
}

#[test]
fn nobody_walks_or_builds_on_water() {
    let mut world = GameWorld::generate(3);
    let water = world
        .terrain
        .iter()
        .find(|c| c.biome == TerrainBiome::Water && c.column > 2)
        .unwrap()
        .coordinate();
    let before = world.clone();
    let moved = world.apply_command(Command::Move {
        unit_id: "villager-1".into(),
        to: water,
    });
    assert!(moved.is_err());
    world.inventories[0].wood = 100.0;
    let _ = world.apply_command(Command::Build {
        kind: BuildingKind::TownCenter,
        unit_id: "villager-1".into(),
        origin: water,
    });
    for _ in 0..400 {
        world.tick(0.1);
    }
    assert_eq!(world.buildings, before.buildings);
    assert_eq!(world.inventories[0].wood, 100.0);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    let mut drowned = before.clone();
    drowned.units[0].cell = water;
    assert!(drowned.validate().is_err());
}

#[test]
fn relief_drains_without_uphill_steps_and_conserves_runoff() {
    let mut land = vec![false; COLUMNS * ROWS];
    let mut height = vec![-0.5; land.len()];
    // A closed bowl and flat plateau exercise spill filling, not just slopes.
    for y in 12..68 {
        for x in 12..108 {
            let i = at(x, y);
            land[i] = true;
            height[i] = if (35..85).contains(&x) && (25..55).contains(&y) {
                0.1
            } else {
                0.8
            };
        }
    }
    let flow = drainage::drain(&land, &mut height);
    let total: u32 = flow
        .accumulation
        .iter()
        .enumerate()
        .filter(|(i, _)| !land[*i])
        .map(|(_, &a)| a)
        .sum();
    assert_eq!(total as usize, land.iter().filter(|&&l| l).count());
    for i in (0..land.len()).filter(|&i| land[i]) {
        let n = flow.downstream[i];
        assert!(neighbours4(i).any(|adjacent| adjacent == n));
        assert!(height[n] < height[i]);
        assert!(flow.accumulation[n] >= flow.accumulation[i]);
    }
}

#[test]
fn generated_rivers_have_a_downhill_route_to_the_sea_including_fords() {
    for seed in 0..48 {
        let world = GameWorld::generate(seed);
        let mut reached = vec![false; world.terrain.len()];
        let mut queue = VecDeque::new();
        for (i, cell) in world.terrain.iter().enumerate() {
            if cell.biome == TerrainBiome::Water {
                reached[i] = true;
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            for n in neighbours4(i) {
                if !reached[n]
                    && matches!(
                        world.terrain[n].biome,
                        TerrainBiome::River | TerrainBiome::Beach
                    )
                    && world.terrain[n].elevation > world.terrain[i].elevation
                {
                    reached[n] = true;
                    queue.push_back(n);
                }
            }
        }
        for (i, cell) in world.terrain.iter().enumerate() {
            assert!(
                cell.biome != TerrainBiome::River || reached[i],
                "seed {seed}: river cannot drain at {i}"
            );
        }
    }
}

#[test]
fn shape_families_survive_generation_and_bays_stay_open() {
    let mut families = std::collections::BTreeSet::new();
    for seed in 0..48 {
        let world = GameWorld::generate(seed);
        let kind = shape::kind(seed);
        families.insert(format!("{kind:?}"));
        let land: Vec<_> = world
            .terrain
            .iter()
            .filter(|c| c.biome != TerrainBiome::Water)
            .collect();
        let width = land.iter().map(|c| c.column).max().unwrap()
            - land.iter().map(|c| c.column).min().unwrap();
        let height =
            land.iter().map(|c| c.row).max().unwrap() - land.iter().map(|c| c.row).min().unwrap();
        if kind == shape::Shape::Long {
            assert!(f64::from(width) / f64::from(height) > 1.8);
        }
        if kind == shape::Shape::Square {
            assert!((0.8..1.3).contains(&(f64::from(width) / f64::from(height))));
        }
        if kind == shape::Shape::Bay {
            let sea = distance_from(0..COLUMNS, |i| {
                world.terrain[i].biome == TerrainBiome::Water
            });
            assert_ne!(
                sea[at(COLUMNS / 2, ROWS / 2)],
                u32::MAX,
                "seed {seed}: closed bay"
            );
        }
    }
    assert_eq!(families.len(), 5);
}
