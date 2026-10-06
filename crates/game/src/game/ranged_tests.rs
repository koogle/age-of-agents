use super::*;

fn encounter(distance: u16) -> GameWorld {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.units.truncate(1);
    world.units[0].kind = UnitKind::Archer;
    world.units[0].cell = CellCoordinate::new(10, 10);
    let cell = CellCoordinate::new(10 + distance, 10);
    world.animals = vec![Animal {
        id: "target".into(),
        kind: AnimalKind::Wolf,
        cell,
        home: cell,
        step: None,
        health: AnimalKind::Wolf.max_health(),
        attack_seconds: 0.0,
        heading: [-1, 0],
    }];
    world.refresh_exploration();
    world.validate().unwrap();
    world
}

fn order(world: &mut GameWorld) -> Result<(), CommandError> {
    world.apply_command(Command::AttackAnimal {
        unit_ids: world.units.iter().map(|u| u.id.clone()).collect(),
        animal_id: "target".into(),
    })
}

fn hunter_tick(world: &mut GameWorld, dt: f64) {
    let UnitAction::AttackAnimal {
        elapsed_seconds, ..
    } = world.units[0].action
    else {
        panic!("lost hunting order")
    };
    world.tick_attack_animal(0, "target", elapsed_seconds, dt);
    world.validate().unwrap();
}

#[test]
fn archer_fires_at_four_cells_and_walks_only_when_out_of_range() {
    let mut world = encounter(4);
    order(&mut world).unwrap();
    hunter_tick(&mut world, 1.0);
    assert_eq!(world.units[0].cell, CellCoordinate::new(10, 10));
    assert!(world.units[0].step.is_none());
    assert_eq!(world.animals[0].health, 282.0);
    let mut world = encounter(6);
    order(&mut world).unwrap();
    for _ in 0..30 {
        hunter_tick(&mut world, 0.1);
    }
    assert_eq!(world.animals[0].health, 264.0);
    assert_eq!(
        world.units[0]
            .cell
            .center()
            .distance(world.animals[0].cell.center()),
        4.0
    );
    assert!(world.units[0].step.is_none());
}

#[test]
fn movement_and_lost_range_reset_the_shot_windup() {
    let mut world = encounter(4);
    order(&mut world).unwrap();
    hunter_tick(&mut world, 0.9);
    world.animals[0].cell.column += 1;
    hunter_tick(&mut world, 0.01);
    assert!(world.units[0].step.is_some());
    assert!(matches!(
        world.units[0].action,
        UnitAction::AttackAnimal {
            elapsed_seconds: 0.0,
            ..
        }
    ));
    assert_eq!(world.animals[0].health, 300.0);
    world.animals[0].cell.column -= 1;
    hunter_tick(&mut world, 0.01);
    assert_eq!(world.animals[0].health, 300.0);
}

#[test]
fn archers_can_hit_moving_targets_but_cannot_shoot_outside_range() {
    let mut world = encounter(3);
    order(&mut world).unwrap();
    world.animals[0].step = Some(Step {
        to: CellCoordinate::new(14, 10),
        progress: 0.5,
    });
    hunter_tick(&mut world, 1.0);
    assert_eq!(world.animals[0].health, 282.0);
    world.animals[0].cell = CellCoordinate::new(14, 10);
    world.animals[0].step = Some(Step {
        to: CellCoordinate::new(15, 10),
        progress: 0.5,
    });
    hunter_tick(&mut world, 1.0);
    assert_eq!(world.animals[0].health, 282.0);
}

#[test]
fn shots_cross_water_and_friendly_units_but_mixed_unreachable_orders_are_atomic() {
    let mut world = encounter(4);
    let columns = world.columns();
    for row in 0..world.rows() {
        world.terrain[usize::from(row) * usize::from(columns) + 12].biome = TerrainBiome::River;
    }
    assert!(!world.can_reach_beside(
        0,
        Footprint {
            origin: world.animals[0].cell,
            columns: 1,
            rows: 1
        }
    ));
    order(&mut world).unwrap();
    hunter_tick(&mut world, 1.0);
    assert_eq!(world.animals[0].health, 282.0);
    let mut guard = world.units[0].clone();
    guard.id = "guard".into();
    guard.kind = UnitKind::Guard;
    guard.cell = CellCoordinate::new(11, 10);
    guard.action = UnitAction::Idle;
    world.units.push(guard);
    let before = world.clone();
    assert_eq!(order(&mut world), Err(CommandError::TargetUnreachable));
    assert_eq!(world, before);
    hunter_tick(&mut world, 1.0);
    assert_eq!(world.animals[0].health, 264.0);
    assert_eq!(world.units[1].action, UnitAction::Idle);
}

#[test]
fn clear_shots_reject_obstacles_and_touching_diagonal_corners() {
    let mut world = encounter(4);
    let from = world.units[0].cell;
    let to = world.animals[0].cell;
    let blocker = CellCoordinate::new(12, 10);
    let index =
        usize::from(blocker.row) * usize::from(world.columns()) + usize::from(blocker.column);
    world.terrain[index].biome = TerrainBiome::Mountain;
    assert!(!world.archer_shot_clear(from, to, &world.occupancy()));
    world.terrain[index].biome = TerrainBiome::Meadow;
    world.resources.push(ResourceNode {
        id: "rock".into(),
        kind: ResourceKind::Stone,
        cell: blocker,
        amount: 20.0,
        capacity: 20.0,
        field: None,
    });
    assert!(!world.archer_shot_clear(from, to, &world.occupancy()));
    order(&mut world).unwrap();
    hunter_tick(&mut world, 0.1);
    assert_eq!(world.animals[0].health, 300.0);
    assert!(world.units[0].step.is_some());
    for _ in 0..40 {
        hunter_tick(&mut world, 0.1);
    }
    assert!(world.animals[0].health < 300.0);
    // Both directions must reject the same blocker at an exact corner.
    world.resources[0].cell = CellCoordinate::new(1, 0);
    let a = CellCoordinate::new(0, 0);
    let b = CellCoordinate::new(2, 2);
    assert!(!world.archer_shot_clear(a, b, &world.occupancy()));
    assert!(!world.archer_shot_clear(b, a, &world.occupancy()));
    world.resources.clear();
    assert!(world.archer_shot_clear(a, b, &world.occupancy()));
    assert!(world.archer_shot_clear(b, a, &world.occupancy()));
    // Building footprints are also solid, including unfinished ones.
    world.buildings[0].origin = CellCoordinate::new(11, 9);
    world.units.clear();
    world.animals.clear();
    assert!(!world.archer_shot_clear(from, CellCoordinate::new(14, 10), &world.occupancy()));
}

#[test]
fn ranged_orders_pause_reload_stop_and_cancel_when_target_is_hidden() {
    let mut world = encounter(4);
    order(&mut world).unwrap();
    hunter_tick(&mut world, 0.4);
    let mut replay: GameWorld =
        serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
    world.simulation_speed = 0.0;
    let paused = world.clone();
    world.tick(1.0);
    assert_eq!(world, paused);
    world.simulation_speed = 1.0;
    for _ in 0..5 {
        world.tick(0.1);
        replay.tick(0.1);
    }
    assert_eq!(world, replay);
    world
        .apply_command(Command::Stop {
            unit_id: world.units[0].id.clone(),
        })
        .unwrap();
    assert_eq!(world.units[0].action, UnitAction::Idle);
    order(&mut world).unwrap();
    world.animals[0].cell = CellCoordinate::new(80, 60);
    world.animals[0].home = world.animals[0].cell;
    world.animals[0].step = None;
    hunter_tick(&mut world, 1.0);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn open_ground_ranged_wolf_encounters_still_need_a_squad() {
    for count in 1..=5 {
        let mut world = encounter(4);
        let template = world.units[0].clone();
        for i in 1..count {
            let mut archer = template.clone();
            archer.id = format!("archer-{i}");
            archer.cell = CellCoordinate::new(10, 10 + i as u16);
            world.units.push(archer);
        }
        order(&mut world).unwrap();
        for _ in 0..600 {
            world.tick(0.1);
            world.validate().unwrap();
            if world.units.is_empty() || world.animals.is_empty() {
                break;
            }
        }
        assert_eq!(world.animals.is_empty(), count >= 3);
        assert_eq!(world.units.is_empty(), count < 3);
    }
}
