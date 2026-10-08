use super::tests::cell;
use super::*;

const SITE: CellCoordinate = CellCoordinate::new(55, 23);

fn world() -> GameWorld {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.inventories[0].wood = 100.0;
    world
}

fn order(world: &mut GameWorld) {
    world
        .apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: SITE,
            kind: BuildingKind::House,
        })
        .unwrap();
    assert!(matches!(
        world.units[0].action,
        UnitAction::ExploreBuild { .. }
    ));
    assert_eq!(world.buildings.len(), 1);
    assert_eq!(world.inventories[0].wood, 100.0);
}

#[test]
fn unseen_and_remembered_sites_wait_for_full_visibility_then_build_once() {
    for explored in [false, true] {
        let mut world = world();
        if explored {
            world.explored_cells = world.terrain.iter().map(|cell| cell.coordinate()).collect();
        }
        world.explored_cells.sort();
        order(&mut world);
        let start = world.units[0].cell;
        for _ in 0..400 {
            let visible = world.visible_cells();
            world.tick(0.1);
            if world.buildings.len() == 2 {
                assert!(
                    world.buildings[1]
                        .footprint()
                        .cells()
                        .all(|c| visible.contains(&c))
                );
                break;
            }
            assert_eq!(world.inventories[0].wood, 100.0);
        }
        assert_ne!(world.units[0].cell, start);
        assert_eq!(world.buildings.len(), 2);
        assert_eq!(world.inventories[0].wood, 85.0);
        for _ in 0..400 {
            world.tick(0.1);
        }
        assert!(world.buildings[1].is_complete());
        assert_eq!(world.buildings.len(), 2);
        assert_eq!(world.inventories[0].wood, 85.0);
        world.validate().unwrap();
    }
}

#[test]
fn discovered_obstacles_or_spent_resources_cancel_without_a_foundation() {
    for obstacle in [true, false] {
        let mut world = world();
        order(&mut world);
        if obstacle {
            world
                .terrain
                .iter_mut()
                .find(|t| t.coordinate() == SITE)
                .unwrap()
                .biome = TerrainBiome::Water;
        } else {
            world.inventories[0].wood = 0.0;
        }
        for _ in 0..400 {
            world.tick(0.1);
        }
        assert_eq!(world.units[0].action, UnitAction::Idle);
        // The villager says why it gave up, for its status label.
        let notice = world.units[0].notice.clone().expect("a reason");
        assert_eq!(
            notice.message,
            if obstacle {
                "Cannot build here. That spot is not clear for building."
            } else {
                "Cannot build here. Not enough wood for that building."
            }
        );
        let saved: GameWorld =
            serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
        assert_eq!(saved, world);
        assert_eq!(world.buildings.len(), 1);
        assert_eq!(
            world.inventories[0].wood,
            if obstacle { 100.0 } else { 0.0 }
        );
        assert!(world.visible_cells().contains(&SITE));
    }
}

#[test]
fn exploration_survives_reload_and_stop_or_replacement_cancels_it() {
    let mut world = world();
    order(&mut world);
    world.tick(0.1);
    let saved = serde_json::to_string(&world).unwrap();
    let mut restored: GameWorld = serde_json::from_str(&saved).unwrap();
    restored.validate().unwrap();
    for _ in 0..400 {
        world.tick(0.1);
        restored.tick(0.1);
    }
    assert_eq!(world, restored);
    for replacement in [
        Command::Stop {
            unit_id: "villager-1".into(),
        },
        Command::Move {
            unit_id: "villager-1".into(),
            to: cell(25, 24),
        },
    ] {
        let mut world = self::world();
        order(&mut world);
        world.apply_command(replacement).unwrap();
        for _ in 0..400 {
            world.tick(0.1);
        }
        assert_eq!(world.buildings.len(), 1);
        assert_eq!(world.inventories[0].wood, 100.0);
    }
}

#[test]
fn exploring_builder_unloads_cargo_before_walking_to_site() {
    let mut world = world();
    world.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Wood,
        amount: 5.0,
    });
    order(&mut world);
    world.tick(0.1);
    assert!(world.units[0].cargo.is_none());
    assert_eq!(world.inventories[0].wood, 105.0);
    assert!(matches!(
        world.units[0].action,
        UnitAction::ExploreBuild { .. }
    ));
    for _ in 0..400 {
        world.tick(0.1);
    }
    assert_eq!(world.inventories[0].wood, 90.0);
    assert!(world.buildings[1].is_complete());
}

#[test]
fn partial_visibility_does_not_place_and_competing_orders_charge_only_once() {
    let mut world = world();
    world.units[0].cell = cell(48, 23);
    let visible = world.visible_cells();
    assert!(visible.contains(&SITE));
    assert!(!visible.contains(&cell(57, 25)));
    order(&mut world);
    world
        .apply_command(Command::Build {
            unit_id: "villager-2".into(),
            origin: SITE,
            kind: BuildingKind::House,
        })
        .unwrap();
    for _ in 0..400 {
        world.tick(0.1);
    }
    assert_eq!(world.buildings.len(), 2);
    assert_eq!(world.inventories[0].wood, 85.0);
    assert!(world.buildings[1].is_complete());
}

#[test]
fn unreachable_and_out_of_bounds_sites_do_not_spend_or_leave_stuck_orders() {
    let mut world = world();
    for terrain in &mut world.terrain {
        if terrain.column == 45 {
            terrain.biome = TerrainBiome::Water;
        }
    }
    order(&mut world);
    world.tick(0.1);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    assert_eq!(
        world.units[0].notice.as_ref().map(|n| n.message.as_str()),
        Some("Cannot build here. No path leads there.")
    );
    assert_eq!(world.inventories[0].wood, 100.0);
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            origin: cell(u16::MAX, u16::MAX),
            kind: BuildingKind::House,
        }),
        Err(CommandError::InvalidBuildSite)
    );
    assert_eq!(world, before);
    world.units[0].action = UnitAction::ExploreBuild {
        origin: cell(u16::MAX, u16::MAX),
        kind: BuildingKind::House,
    };
    assert!(world.validate().is_err());
}
