use super::tests::{cell, stand_beside};
use super::*;

#[test]
fn houses_can_share_edges_on_both_axes_but_never_overlap() {
    let mut world = fixture::fixture();
    world.inventories[0].wood = 100.0;
    let first = cell(20, 24);
    let (columns, rows) = BuildingKind::House.size();
    for origin in [
        first,
        cell(first.column + columns, first.row),
        cell(first.column, first.row + rows),
    ] {
        let footprint = Footprint {
            origin,
            columns,
            rows,
        };
        stand_beside(&mut world, 0, footprint);
        world
            .apply_command(Command::Build {
                unit_id: "villager-1".into(),
                origin,
                kind: BuildingKind::House,
            })
            .unwrap();
        for _ in 0..600 {
            world.tick(0.1);
        }
        assert!(world.buildings.last().unwrap().is_complete());
        world.validate().unwrap();
    }
    assert_eq!(world.inventories[0].wood, 55.0);
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(first.column + columns - 1, first.row),
            kind: BuildingKind::House,
        }),
        Err(CommandError::InvalidBuildSite)
    );
    assert_eq!(world, before);
}

// A narrow strip of land: the house fills its width and seals the west end.
fn bottleneck_world() -> GameWorld {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.buildings.clear();
    let (_, rows) = BuildingKind::House.size();
    for terrain in &mut world.terrain {
        terrain.biome =
            if (10..30).contains(&terrain.column) && (10..10 + rows).contains(&terrain.row) {
                TerrainBiome::Meadow
            } else {
                TerrainBiome::Water
            };
    }
    world.units[0].cell = cell(11, 10);
    world.units[1].cell = cell(20, 10);
    world.inventories[0].wood = 100.0;
    world.validate().unwrap();
    world
}

#[test]
fn building_cannot_seal_in_builder_or_another_villager() {
    for builder in ["villager-1", "villager-2"] {
        let mut world = bottleneck_world();
        let before = world.clone();
        assert_eq!(
            world.apply_command(Command::Build {
                unit_id: builder.into(),
                origin: cell(12, 10),
                kind: BuildingKind::House,
            }),
            Err(CommandError::TargetUnreachable)
        );
        assert_eq!(world, before);
        world.validate().unwrap();
    }
}

#[test]
fn building_with_a_real_exit_completes_and_builder_can_leave() {
    let mut world = bottleneck_world();
    let (_, rows) = BuildingKind::House.size();
    // One extra row supplies a legal walking route around the house.
    for terrain in &mut world.terrain {
        if (10..30).contains(&terrain.column) && terrain.row == 10 + rows {
            terrain.biome = TerrainBiome::Meadow;
        }
    }
    world
        .apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(12, 10),
            kind: BuildingKind::House,
        })
        .unwrap();
    for _ in 0..100 {
        world.tick(0.1);
    }
    assert!(world.buildings[0].is_complete());
    world
        .apply_command(Command::Move {
            unit_id: "villager-1".into(),
            to: cell(25, 10),
        })
        .unwrap();
    for _ in 0..200 {
        world.tick(0.1);
    }
    assert_eq!(world.units[0].cell, cell(25, 10));
    world.validate().unwrap();
}

#[test]
fn diagonal_contact_is_not_an_escape_route() {
    let mut world = bottleneck_world();
    let (_, rows) = BuildingKind::House.size();
    // These cells only touch the pocket diagonally past the house corner.
    for terrain in &mut world.terrain {
        if (12..30).contains(&terrain.column) && terrain.row == 10 + rows {
            terrain.biome = TerrainBiome::Meadow;
        }
    }
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(12, 10),
            kind: BuildingKind::House,
        }),
        Err(CommandError::TargetUnreachable)
    );
    assert_eq!(world, before);
}

#[test]
fn moving_villager_is_protected_and_keeps_its_order() {
    let mut world = bottleneck_world();
    world.units[0].action = UnitAction::Move { to: cell(10, 11) };
    world.units[0].step = Some(Step {
        to: cell(10, 10),
        progress: 0.5,
    });
    world.validate().unwrap();
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-2".into(),
            origin: cell(12, 10),
            kind: BuildingKind::House,
        }),
        Err(CommandError::TargetUnreachable)
    );
    assert_eq!(world, before);
}

#[test]
fn existing_disconnected_land_does_not_prevent_safe_building() {
    let mut world = bottleneck_world();
    // A separate island with another unit must not become a global reachability requirement.
    for terrain in &mut world.terrain {
        if (40..50).contains(&terrain.column) && (40..50).contains(&terrain.row) {
            terrain.biome = TerrainBiome::Meadow;
        }
    }
    world.units[1].cell = cell(42, 42);
    world
        .apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(27, 10),
            kind: BuildingKind::House,
        })
        .unwrap();
    world.validate().unwrap();
}

#[test]
fn building_cannot_consume_all_of_a_villagers_remaining_walking_space() {
    let mut world = bottleneck_world();
    world.units.truncate(1);
    for terrain in &mut world.terrain {
        if terrain.coordinate() != cell(11, 10)
            && !((12..15).contains(&terrain.column) && (10..13).contains(&terrain.row))
        {
            terrain.biome = TerrainBiome::Water;
        }
    }
    world.validate().unwrap();
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(12, 10),
            kind: BuildingKind::House,
        }),
        Err(CommandError::TargetUnreachable)
    );
    assert_eq!(world, before);
}
