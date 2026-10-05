use super::*;

fn wildlife(kind: AnimalKind) -> GameWorld {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.units[0].cell = CellCoordinate::new(10, 10);
    world.units[1].cell = CellCoordinate::new(5, 5);
    let cell = CellCoordinate::new(11, 10);
    world.animals = vec![Animal {
        id: "beast".into(),
        kind,
        home: cell,
        cell,
        step: None,
        health: kind.max_health(),
        attack_seconds: 0.0,
        heading: [1, 0],
    }];
    world.refresh_exploration();
    world.validate().unwrap();
    world
}
fn attack(world: &mut GameWorld) {
    world
        .apply_command(Command::AttackAnimal {
            unit_ids: vec![world.units[0].id.clone()],
            animal_id: "beast".into(),
        })
        .unwrap();
}

#[test]
fn wolves_and_bears_spawn_deterministically_away_from_starter_units() {
    for seed in [1, 123, DEFAULT_SEED] {
        let world = GameWorld::generate(seed);
        assert_eq!(world.animals, GameWorld::generate(seed).animals);
        assert_eq!(world.animals.len(), 3);
        assert!(world.animals.iter().any(|a| a.kind == AnimalKind::Bear));
        for a in &world.animals {
            assert!(
                world
                    .units
                    .iter()
                    .all(|u| u.cell.center().distance(a.cell.center()) >= 26.0)
            );
        }
        assert!(world.snapshot().animals.is_empty());
        world.validate().unwrap();
    }
}

#[test]
fn animal_attacks_hurt_and_kill_once_releasing_cells_and_cargo() {
    let mut world = wildlife(AnimalKind::Bear);
    let id = world.units[0].id.clone();
    world.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Wood,
        amount: 10.0,
    });
    let stock = world.stockpile.clone();
    for _ in 0..7 {
        world.tick(1.0);
    }
    assert!(!world.units.iter().any(|u| u.id == id));
    assert_eq!(world.stockpile, stock);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.validate().unwrap();
}

#[test]
fn explicit_hunting_kills_animal_and_clears_all_orders() {
    let mut world = wildlife(AnimalKind::Wolf);
    attack(&mut world);
    for _ in 0..4 {
        world.tick(1.0);
    }
    assert!(world.animals.is_empty());
    assert_eq!(world.units[0].action, UnitAction::Idle);
    assert_eq!(world.units[0].health, 76.0);
    assert_eq!(world.units[1].health, 100.0);
    world.validate().unwrap();
}

#[test]
fn group_attack_rejection_is_atomic_and_hidden_animals_never_leak() {
    let mut world = wildlife(AnimalKind::Bear);
    let before = world.clone();
    assert!(
        world
            .apply_command(Command::AttackAnimal {
                unit_ids: vec!["villager-1".into(), "missing".into()],
                animal_id: "beast".into()
            })
            .is_err()
    );
    assert_eq!(world, before);
    world.units[0].cell = CellCoordinate::new(80, 60);
    world.units[1].cell = CellCoordinate::new(82, 60);
    assert!(world.snapshot().animals.is_empty());
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::AttackAnimal {
            unit_ids: vec!["villager-1".into()],
            animal_id: "beast".into()
        }),
        Err(CommandError::AnimalNotVisible)
    );
    assert_eq!(world, before);
}

#[test]
fn pause_reload_and_stop_preserve_health_cargo_and_determinism() {
    let mut world = wildlife(AnimalKind::Bear);
    attack(&mut world);
    world.tick(0.4);
    let mut restored: GameWorld =
        serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
    for _ in 0..3 {
        world.tick(0.1);
        restored.tick(0.1);
    }
    assert_eq!(world, restored);
    world.simulation_speed = 0.0;
    let before = world.clone();
    world.tick(10.0);
    assert_eq!(world, before);
    world
        .apply_command(Command::Stop {
            unit_id: "villager-1".into(),
        })
        .unwrap();
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn animals_pursue_locally_and_retreat_to_their_territory() {
    let mut world = wildlife(AnimalKind::Wolf);
    world.units[0].cell = CellCoordinate::new(15, 10);
    let home = world.animals[0].home;
    for _ in 0..10 {
        world.tick(0.1);
    }
    assert!(world.animals[0].cell != home || world.animals[0].step.is_some());
    world.units[0].cell = CellCoordinate::new(80, 60);
    for _ in 0..100 {
        world.tick(0.1);
    }
    assert_eq!(world.animals[0].cell, home);
    assert!(world.animals[0].step.is_none());
    world.validate().unwrap();
}

#[test]
fn discovery_adds_wildlife_without_replacing_existing_animals() {
    let mut world = GameWorld::generate(123);
    let old = world.animals.clone();
    world.discover_island();
    assert_eq!(&world.animals[..old.len()], old.as_slice());
    assert_eq!(world.animals.len(), 6);
    world.validate().unwrap();
}

#[test]
fn corrupt_health_cooldown_and_occupancy_are_rejected() {
    let world = wildlife(AnimalKind::Bear);
    for health in [f64::NAN, 0.0, 101.0] {
        let mut bad = world.clone();
        bad.units[0].health = health;
        assert!(bad.validate().is_err());
        let mut bad = world.clone();
        bad.animals[0].health = health;
        assert!(bad.validate().is_err());
    }
    let mut bad = world.clone();
    bad.animals[0].attack_seconds = f64::INFINITY;
    assert!(bad.validate().is_err());
    let mut bad = world.clone();
    bad.animals[0].cell = bad.units[0].cell;
    assert!(bad.validate().is_err());
}

#[test]
fn melee_cannot_damage_through_blocked_diagonal_corners() {
    let mut world = wildlife(AnimalKind::Bear);
    world.animals[0].cell = CellCoordinate::new(11, 11);
    world.animals[0].home = world.animals[0].cell;
    for cell in [CellCoordinate::new(10, 11), CellCoordinate::new(11, 10)] {
        world.resources.push(ResourceNode {
            id: format!("block-{}", cell.column),
            kind: ResourceKind::Stone,
            cell,
            amount: 20.0,
            capacity: 20.0,
            field: None,
        });
    }
    world.units[0].action = UnitAction::AttackAnimal {
        animal_id: "beast".into(),
        elapsed_seconds: 0.0,
    };
    world.tick(0.1);
    assert_eq!(world.units[0].health, 100.0);
    assert_eq!(world.animals[0].health, 100.0);
    assert!(
        world.units[0].step.is_some(),
        "hunter should seek a clear attack position"
    );
    world.validate().unwrap();
}

#[test]
fn missing_terrain_is_a_validation_error_not_a_panic() {
    let mut world = wildlife(AnimalKind::Bear);
    world.terrain.clear();
    assert!(world.validate().is_err());
}
