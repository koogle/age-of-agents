use super::*;

fn charted(seed: u64) -> GameWorld {
    let mut world = GameWorld::generate(seed);
    while !world.undiscovered_sites().is_empty() {
        world.discover_island();
    }
    world
}

fn stronghold(world: &GameWorld) -> &Building {
    world
        .buildings
        .iter()
        .find(|b| b.kind == BuildingKind::Stronghold)
        .expect("a charted run holds the stronghold")
}

#[test]
fn one_stronghold_and_its_garrison_stand_on_a_later_island() {
    for seed in [1, 7, 123, 4242, DEFAULT_SEED] {
        let mut world = GameWorld::generate(seed);
        world.discover_island();
        assert!(
            world
                .buildings
                .iter()
                .all(|b| !b.kind.is_generated() || world.island_at(b.origin) != Some(1)),
            "never on the raided island"
        );
        let world = charted(seed);
        let fort = stronghold(&world);
        let island = world.island_at(fort.origin).unwrap();
        assert!(island >= 2, "seed {seed}: island {island}");
        assert_ne!(
            world.islands[island],
            *archipelago_plan(seed).last().unwrap(),
            "never on the temple island"
        );
        assert_eq!(
            world
                .buildings
                .iter()
                .filter(|b| b.kind == BuildingKind::Stronghold)
                .count(),
            1
        );
        let garrison: Vec<_> = world
            .animals
            .iter()
            .filter(|a| world.garrisoned(world.stronghold_island(), a))
            .collect();
        assert!((10..=14).contains(&garrison.len()), "{}", garrison.len());
        assert_eq!(garrison[0].kind, AnimalKind::Chieftain);
        for guard in &garrison {
            assert!(guard.id.starts_with("garrison-"));
            assert_eq!(world.island_at(guard.home), Some(island));
            assert!(guard.home.center().distance(fort.footprint().center()) < 13.0);
        }
        world.validate().unwrap();
        assert_eq!(charted(seed), world, "deterministic");
    }
}

#[test]
fn the_stronghold_is_not_the_players_to_work_or_see_through() {
    let mut world = charted(7);
    let fort = stronghold(&world).clone();
    let visible = world.visible_cells();
    assert!(
        fort.footprint().cells().all(|c| !visible.contains(&c)),
        "lifts no fog"
    );
    world
        .buildings
        .iter_mut()
        .find(|b| b.id == fort.id)
        .unwrap()
        .damage = 10.0;
    assert_eq!(
        world.apply_command(Command::Construct {
            unit_id: "villager-1".into(),
            building_id: fort.id.clone(),
        }),
        Err(CommandError::NotBuildable)
    );
}

/// A world with the stronghold beside two guards and no garrison left.
fn assault(world: &mut GameWorld) -> Footprint {
    let walls = stronghold(world).footprint();
    world.animals.retain(|a| !a.kind.raids());
    let occupancy = world.occupancy();
    let posts: Vec<_> = movement::interaction_cells(walls)
        .filter(|&c| occupancy.is_free_for(c, None))
        .take(2)
        .collect();
    for (unit, cell) in world.units.iter_mut().zip(posts) {
        unit.cell = cell;
        unit.kind = UnitKind::Guard;
        unit.action = UnitAction::Idle;
    }
    world.refresh_exploration();
    walls
}

#[test]
fn razing_the_stronghold_ends_the_raids() {
    let mut world = charted(7);
    assault(&mut world);
    world.raid = Raid::Incoming {
        wave: 3,
        seconds_left: 400.0,
    };
    let ids = vec!["villager-1".to_string(), "villager-2".to_string()];
    assert_eq!(
        world.apply_command(Command::AttackBuilding {
            unit_ids: ids.clone(),
            building_id: "base-1".into(),
        }),
        Err(CommandError::NotAttackable)
    );
    world
        .apply_command(Command::AttackBuilding {
            unit_ids: ids,
            building_id: STRONGHOLD_ID.into(),
        })
        .unwrap();
    world.tick(1.0);
    assert_eq!(stronghold(&world).damage, 50.0, "two guards, 25 a blow");
    world.validate().unwrap();
    world
        .buildings
        .iter_mut()
        .find(|b| b.id == STRONGHOLD_ID)
        .unwrap()
        .damage = BuildingKind::Stronghold.max_health() - 1.0;
    world.tick(1.0);
    assert!(world.buildings.iter().all(|b| b.id != STRONGHOLD_ID));
    assert_eq!(world.raid, Raid::Ended);
    assert!(world.units.iter().all(|u| u.action == UnitAction::Idle));
    assert!(world.snapshot().raids_ended);
    world.validate().unwrap();
    for _ in 0..100 {
        world.tick(10.0);
    }
    assert_eq!(world.raid, Raid::Ended);
    assert!(world.animals.iter().all(|a| !a.kind.raids()));
}

#[test]
fn healers_cannot_storm_the_stronghold() {
    let mut world = charted(7);
    assault(&mut world);
    world.units[0].kind = UnitKind::Healer;
    assert_eq!(
        world.apply_command(Command::AttackBuilding {
            unit_ids: vec!["villager-1".into()],
            building_id: STRONGHOLD_ID.into(),
        }),
        Err(CommandError::CannotAttack)
    );
}

#[test]
fn the_garrison_holds_its_ground_and_turns_out_together() {
    let mut world = charted(7);
    let garrison: Vec<_> = world
        .animals
        .iter()
        .filter(|a| world.garrisoned(world.stronghold_island(), a))
        .map(|a| a.id.clone())
        .collect();
    let before = world.animals.clone();
    for _ in 0..50 {
        world.tick(0.2);
    }
    assert_eq!(world.animals, before, "no intruder, no movement");
    // One intruder within sight of a single sentry rouses the whole garrison.
    let sentry = before.iter().find(|a| a.id == garrison[0]).unwrap();
    let occupancy = world.occupancy();
    let cell = movement::interaction_cells(sentry.footprint())
        .find(|&c| occupancy.is_free_for(c, None))
        .unwrap();
    world.units[0].cell = cell;
    world.units[0].kind = UnitKind::Guard;
    world.units[0].health = UNIT_HEALTH;
    world.tick(0.2);
    let moving = world
        .animals
        .iter()
        .filter(|a| garrison.contains(&a.id) && (a.step.is_some() || a.attack_seconds > 0.0))
        .count();
    assert!(
        moving >= garrison.len() / 2,
        "{moving} of {}",
        garrison.len()
    );
    for _ in 0..200 {
        world.tick(0.2);
        world.validate().unwrap();
    }
    for guard in world.animals.iter().filter(|a| garrison.contains(&a.id)) {
        assert!(guard.cell.center().distance(guard.home.center()) <= 14.0);
    }
}

#[test]
fn a_new_war_band_lands_after_the_last_one_falls() {
    let mut world = charted(7);
    world.raid = Raid::Landed { wave: 0 };
    world.tick(0.1);
    let Raid::Incoming {
        wave: 1,
        seconds_left,
    } = world.raid
    else {
        panic!("the next wave is due: {:?}", world.raid);
    };
    assert!((299.0..=600.0).contains(&seconds_left));
    world.raid = Raid::Incoming {
        wave: 1,
        seconds_left: 0.05,
    };
    world.tick(0.1);
    assert_eq!(world.raid, Raid::Landed { wave: 1 });
    let band: Vec<_> = world
        .animals
        .iter()
        .filter(|a| a.id.starts_with("raider-1-"))
        .collect();
    assert!((10..=20).contains(&band.len()));
    for raider in &band {
        assert_eq!(world.island_at(raider.cell), Some(1));
        assert!(!world.garrisoned(world.stronghold_island(), raider));
    }
    world.validate().unwrap();
}
