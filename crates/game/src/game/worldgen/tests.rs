use super::*;

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
            assert!(grows_in(resource.kind).contains(&world.terrain[index(resource.cell)].biome));
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
        // Everything else has to be found by exploring.
        for resource in &world.resources {
            if !matches!(resource.kind, ResourceKind::Wood | ResourceKind::Food) {
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
        built.stockpile.wood = 1000.0;
        built.stockpile.stone = 1000.0;
        let origin = CellCoordinate::new(cell.column.saturating_sub(1), cell.row.saturating_sub(1));
        assert!(
            built
                .apply_command(Command::Build {
                    kind: BuildingKind::House,
                    unit_id: "villager-1".into(),
                    origin
                })
                .is_err()
        );
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
    world.stockpile.wood = 100.0;
    let built = world.apply_command(Command::Build {
        kind: BuildingKind::TownCenter,
        unit_id: "villager-1".into(),
        origin: water,
    });
    assert!(built.is_err());
    world.stockpile.wood = before.stockpile.wood;
    assert_eq!(world, before);
    let mut drowned = before.clone();
    drowned.units[0].cell = water;
    assert!(drowned.validate().is_err());
}
