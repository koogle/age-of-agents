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
fn stationary_attack_faces_target_and_clears_phase_after_retreat() {
    for kind in [
        AnimalKind::Wolf,
        AnimalKind::Bear,
        AnimalKind::Boar,
        AnimalKind::Lioness,
        AnimalKind::Lion,
    ] {
        let mut world = wildlife(kind);
        world.tick(0.3);
        assert_eq!(world.animals[0].heading, [-1, 0]);
        assert_eq!(world.animals[0].attack_seconds, 0.3);
        world.simulation_speed = 0.0;
        let paused = world.animals[0].clone();
        world.tick(10.0);
        assert_eq!(world.animals[0], paused);
        world.simulation_speed = 1.0;
        world.units[0].cell = CellCoordinate::new(80, 60);
        world.tick(0.1);
        assert_eq!(world.animals[0].attack_seconds, 0.0);
        world.validate().unwrap();
    }
}

#[test]
fn wolf_and_boar_spawn_deterministically_away_from_starter_units() {
    for seed in [1, 123, DEFAULT_SEED] {
        let world = GameWorld::generate(seed);
        assert_eq!(world.animals, GameWorld::generate(seed).animals);
        assert_eq!(world.animals.len(), 2);
        assert_eq!(world.animals[1].kind, AnimalKind::Boar);
        assert_eq!(world.animals[0].kind, AnimalKind::Wolf);
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
    let stock = world.inventories.clone();
    for _ in 0..7 {
        world.tick(1.0);
    }
    assert!(!world.units.iter().any(|u| u.id == id));
    assert_eq!(world.inventories, stock);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.validate().unwrap();
}

fn archer_encounter(kind: AnimalKind, count: usize) -> GameWorld {
    let mut world = wildlife(kind);
    let template = world.units[0].clone();
    let positions = [(10, 10), (11, 9), (12, 10), (11, 11), (10, 9)];
    world.units = positions[..count]
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| {
            let mut unit = template.clone();
            unit.id = format!("archer-{i}");
            unit.kind = UnitKind::Archer;
            unit.cell = CellCoordinate::new(x, y);
            unit
        })
        .collect();
    world.refresh_exploration();
    world
        .apply_command(Command::AttackAnimal {
            unit_ids: world.units.iter().map(|u| u.id.clone()).collect(),
            animal_id: "beast".into(),
        })
        .unwrap();
    for _ in 0..600 {
        world.tick(0.1);
        world.validate().unwrap();
        if world.animals.is_empty() || world.units.is_empty() {
            break;
        }
    }
    world
}

#[test]
fn wolf_requires_three_to_five_archers() {
    for count in 1..=5 {
        let world = archer_encounter(AnimalKind::Wolf, count);
        assert_eq!(world.units.len(), [0, 0, 1, 3, 4][count - 1]);
        assert_eq!(world.animals.is_empty(), count >= 3);
        if count >= 3 {
            assert!(!world.units.is_empty());
            assert!(world.units.iter().all(|u| u.action == UnitAction::Idle));
        } else {
            assert!(world.units.is_empty());
        }
    }
}

#[test]
fn bear_is_more_dangerous_than_a_wolf() {
    let world = archer_encounter(AnimalKind::Bear, 3);
    assert!(world.units.is_empty());
    assert!(!world.animals.is_empty());
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
    assert_eq!(
        world.apply_command(Command::Stop {
            unit_id: "villager-1".into(),
        }),
        Err(CommandError::GamePaused)
    );
    assert_eq!(world, before);
    world
        .apply_command(Command::SetSimulationSpeed { multiplier: 1.0 })
        .unwrap();
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
    let mut counts = std::collections::BTreeSet::new();
    for seed in [1, 2, 3, 123, DEFAULT_SEED] {
        let mut world = GameWorld::generate(seed);
        for _ in 0..3 {
            let old = world.animals.clone();
            let mut replay = world.clone();
            world.discover_island();
            replay.discover_island();
            assert_eq!(world.animals, replay.animals);
            assert_eq!(&world.animals[..old.len()], old.as_slice());
            let added = &world.animals[old.len()..];
            let boars = added.iter().filter(|a| a.kind == AnimalKind::Boar).count();
            assert!((2..=3).contains(&boars));
            let lions: Vec<_> = added.iter().filter(|a| a.kind.is_lion()).collect();
            assert!((3..=4).contains(&lions.len()));
            assert_eq!(lions[0].kind, AnimalKind::Lion);
            assert!(lions[1..].iter().all(|a| a.kind == AnimalKind::Lioness
                && a.home.center().distance(lions[0].home.center()) <= 3.0));
            let added: Vec<_> = added.iter().filter(|a| !a.kind.is_lion()).collect();
            assert!((4..=7).contains(&added.len()));
            assert_eq!(
                added.iter().filter(|a| a.kind == AnimalKind::Bear).count(),
                1
            );
            assert_eq!(
                added.iter().filter(|a| a.kind == AnimalKind::Wolf).count(),
                added.len() - boars - 1
            );
            counts.insert(added.len() - boars);
            world.validate().unwrap();
        }
    }
    assert_eq!(counts, [2, 3, 4].into_iter().collect());
}

#[test]
fn corrupt_health_cooldown_and_occupancy_are_rejected() {
    let world = wildlife(AnimalKind::Bear);
    for health in [f64::NAN, 0.0, 101.0] {
        let mut bad = world.clone();
        bad.units[0].health = health;
        assert!(bad.validate().is_err());
    }
    for health in [f64::NAN, 0.0, AnimalKind::Bear.max_health() + 1.0] {
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
    assert_eq!(world.animals[0].health, AnimalKind::Bear.max_health());
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

#[test]
fn boar_hunting_damage_and_save_roundtrip() {
    let mut world = wildlife(AnimalKind::Boar);
    attack(&mut world);
    world.tick(1.0);
    assert_eq!(world.animals[0].health, 50.0);
    assert_eq!(world.units[0].health, 90.0);
    let mut restored: GameWorld =
        serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
    for _ in 0..5 {
        world.tick(1.0);
        restored.tick(1.0);
    }
    assert_eq!(world, restored);
    assert!(world.animals.is_empty());
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.validate().unwrap();
}

fn pride() -> GameWorld {
    let mut world = wildlife(AnimalKind::Lion);
    world.units[0].cell = CellCoordinate::new(60, 60);
    world.units[1].cell = CellCoordinate::new(5, 5);
    world.units.truncate(2);
    let template = world.animals[0].clone();
    world.animals = [
        (20, 40, AnimalKind::Lion),
        (22, 41, AnimalKind::Lioness),
        (24, 40, AnimalKind::Lioness),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (x, y, kind))| Animal {
        id: format!("lion-{i}"),
        kind,
        home: CellCoordinate::new(x, y),
        cell: CellCoordinate::new(x, y),
        health: kind.max_health(),
        ..template.clone()
    })
    .collect();
    world.refresh_exploration();
    world.validate().unwrap();
    world
}

#[test]
fn one_lion_sighting_alerts_the_whole_pride() {
    let mut world = pride();
    let intruder = world.units[0].id.clone();
    world.tick(0.1);
    assert!(
        world.animals.iter().all(|a| a.step.is_none()),
        "nobody in reach yet"
    );
    // Within the far lioness's aggro but beyond the leader's.
    world.units[0].cell = CellCoordinate::new(29, 40);
    world.tick(0.1);
    assert!(
        world.animals.iter().all(|a| a.step.is_some()),
        "the pride hunts together"
    );
    for _ in 0..80 {
        world.tick(0.1);
        world.validate().unwrap();
    }
    assert!(
        !world.units.iter().any(|u| u.id == intruder),
        "the pride brought the intruder down"
    );
    // A lone wolf beside the pride stays home: pride alerts are lions only.
    let mut world = pride();
    world.animals[2].kind = AnimalKind::Wolf;
    world.animals[2].health = AnimalKind::Wolf.max_health();
    world.units[0].cell = CellCoordinate::new(17, 40);
    world.tick(0.1);
    assert!(world.animals[1].step.is_some());
    assert!(world.animals[2].step.is_none());
}

#[test]
fn prides_do_not_chase_beyond_their_territory() {
    let mut world = pride();
    world.units[0].cell = CellCoordinate::new(29, 40);
    for _ in 0..5 {
        world.tick(0.1);
    }
    world.units[0].cell = CellCoordinate::new(70, 40);
    for _ in 0..200 {
        world.tick(0.1);
        world.validate().unwrap();
    }
    assert!(
        world
            .animals
            .iter()
            .all(|a| a.cell == a.home && a.step.is_none())
    );
}
