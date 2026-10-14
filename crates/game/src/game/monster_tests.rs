//! Skeleton squads, centaur archers, cyclopes, the minotaur, island levels and
//! refilling lairs.
use super::*;

/// The fixture with its town center moved away, two villagers and `animals`.
fn arena(animals: &[(AnimalKind, u16, u16)], level: u8) -> GameWorld {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.buildings.clear();
    world.units[0].cell = CellCoordinate::new(10, 10);
    world.units[1].cell = CellCoordinate::new(5, 5);
    world.animals = animals
        .iter()
        .enumerate()
        .map(|(i, &(kind, x, y))| {
            Animal::spawn(
                format!("animal-{i}"),
                kind,
                CellCoordinate::new(x, y),
                level,
            )
        })
        .collect();
    world.refresh_exploration();
    world.validate().unwrap();
    world
}

fn run(world: &mut GameWorld, seconds: f64) {
    for _ in 0..(seconds * 10.0).round() as usize {
        world.tick(0.1);
    }
}

#[test]
fn centaurs_shoot_from_range_without_closing_in() {
    let mut world = arena(&[(AnimalKind::Centaur, 13, 10)], 1);
    run(&mut world, 1.05);
    assert_eq!(world.animals[0].cell, CellCoordinate::new(13, 10));
    assert_eq!(
        world.units[0].health,
        UNIT_HEALTH - AnimalKind::Centaur.damage()
    );
    // Out of range, a melee animal would have to close in first.
    let mut world = arena(&[(AnimalKind::Wolf, 13, 10)], 1);
    run(&mut world, 1.05);
    assert_eq!(world.units[0].health, UNIT_HEALTH);
}

#[test]
fn a_cyclops_club_sweeps_every_unit_around_him() {
    let mut world = arena(&[(AnimalKind::Cyclops, 11, 10)], 1);
    world.units[1].cell = CellCoordinate::new(12, 11);
    run(&mut world, 1.05);
    let hit = UNIT_HEALTH - AnimalKind::Cyclops.damage();
    assert_eq!(world.units[0].health, hit);
    assert_eq!(world.units[1].health, hit);
}

#[test]
fn one_skeleton_sighting_alerts_its_whole_squad() {
    let mut world = arena(
        &[
            (AnimalKind::Skeleton, 14, 10),
            (AnimalKind::Skeleton, 17, 10),
        ],
        1,
    );
    world.tick(0.1);
    // Only the first sees the villager; both set out.
    assert!(world.animals.iter().all(|a| a.step.is_some()));
}

#[test]
fn the_minotaur_sleeps_until_the_artifact_leaves_or_he_is_struck() {
    let mut world = arena(&[(AnimalKind::Minotaur, 11, 10)], 1);
    run(&mut world, 3.0);
    assert_eq!(world.units[0].health, UNIT_HEALTH);
    assert_eq!(world.animals[0].attack_seconds, 0.0);
    // Roused by the artifact, he hunts its bearer first, across the island.
    world.buildings.push(building(
        BuildingKind::Temple,
        TEMPLE_ID,
        CellCoordinate::new(40, 40),
        None,
    ));
    world.artifact_bearer = Some(world.units[1].id.clone());
    world.tick(0.1);
    let bearer = world.units[1].cell.center();
    let closer = |s: Step| {
        s.to.center().distance(bearer) < CellCoordinate::new(11, 10).center().distance(bearer)
    };
    assert!(world.animals[0].step.is_some_and(closer));
    let mut world = arena(&[(AnimalKind::Minotaur, 11, 10)], 1);
    world.animals[0].health -= 1.0;
    run(&mut world, 1.05);
    assert_eq!(
        world.units[0].health,
        UNIT_HEALTH - AnimalKind::Minotaur.damage()
    );
}

fn slay(world: &mut GameWorld) {
    world.units[0].kind = UnitKind::SiegeCart;
    world.animals[0].health = 1.0;
    world
        .apply_command(Command::AttackAnimal {
            unit_ids: vec![world.units[0].id.clone()],
            animal_id: world.animals[0].id.clone(),
        })
        .unwrap();
    run(world, 1.5);
    assert!(world.animals.is_empty());
}

#[test]
fn lairs_on_later_islands_refill_once_nobody_is_near() {
    // A boar is weak enough not to kill the villager before it falls.
    let mut world = arena(&[(AnimalKind::Boar, 11, 10)], 2);
    slay(&mut world);
    assert_eq!(world.respawns.len(), 1);
    world.units[1].cell = CellCoordinate::new(30, 32);
    run(&mut world, 240.0);
    // The hunter still stands beside the lair.
    assert!(world.animals.is_empty());
    world.units[0].cell = CellCoordinate::new(30, 30);
    world.tick(0.1);
    assert_eq!(world.animals.len(), 1);
    let boar = &world.animals[0];
    assert_eq!(
        (boar.id.as_str(), boar.cell, boar.level),
        ("animal-0", CellCoordinate::new(11, 10), 2)
    );
    assert_eq!(boar.health, boar.max_health());
    assert!(world.respawns.is_empty());
    world.validate().unwrap();
}

#[test]
fn skeletons_rise_fast_and_buildings_claim_lairs_for_good() {
    let mut world = arena(&[(AnimalKind::Skeleton, 11, 10)], 3);
    world.units[1].cell = CellCoordinate::new(30, 32);
    slay(&mut world);
    world.units[0].cell = CellCoordinate::new(30, 30);
    run(&mut world, 60.1);
    assert_eq!(world.animals.len(), 1);
    slay_from(&mut world, CellCoordinate::new(10, 10));
    world
        .buildings
        .push(town_center("claim", CellCoordinate::new(14, 14), None));
    world.units[0].cell = CellCoordinate::new(40, 40);
    run(&mut world, 120.0);
    assert!(world.animals.is_empty());
    assert_eq!(world.respawns[0].seconds_left, 0.0);
    world.validate().unwrap();
}

fn slay_from(world: &mut GameWorld, cell: CellCoordinate) {
    world.units[0].cell = cell;
    world.refresh_exploration();
    slay(world);
}

#[test]
fn the_second_island_raiders_and_bosses_never_return() {
    for (kind, level) in [
        (AnimalKind::Boar, 1),
        (AnimalKind::Barbarian, 3),
        (AnimalKind::Minotaur, 5),
    ] {
        let mut world = arena(&[(kind, 11, 10)], level);
        slay(&mut world);
        assert!(world.respawns.is_empty(), "{kind:?}");
    }
}

#[test]
fn corrupt_respawns_are_rejected() {
    let mut world = arena(&[(AnimalKind::Boar, 11, 10)], 2);
    slay(&mut world);
    for corrupt in [
        |r: &mut Respawn| r.seconds_left = f64::NAN,
        |r: &mut Respawn| r.seconds_left = -1.0,
        |r: &mut Respawn| r.level = 1,
        |r: &mut Respawn| r.kind = AnimalKind::Minotaur,
    ] {
        let mut bad = world.clone();
        corrupt(&mut bad.respawns[0]);
        assert!(bad.validate().is_err());
    }
    let mut bad = world.clone();
    bad.animals.push(Animal::spawn(
        "animal-0".into(),
        AnimalKind::Wolf,
        CellCoordinate::new(20, 20),
        2,
    ));
    assert!(bad.validate().is_err());
}
