use super::*;

fn raider(kind: AnimalKind, cell: CellCoordinate) -> Animal {
    Animal {
        id: "raider-0".into(),
        kind,
        home: cell,
        cell,
        step: None,
        health: kind.max_health(),
        attack_seconds: 0.0,
        heading: [1, 0],
    }
}

/// A free land cell on the second island, away from its wildlife.
fn landing_cell(world: &GameWorld) -> CellCoordinate {
    let occupancy = world.occupancy();
    world
        .terrain
        .iter()
        .map(|t| t.coordinate())
        .find(|&c| {
            world.island_at(c) == Some(1)
                && world.terrain
                    [usize::from(c.row) * usize::from(world.columns()) + usize::from(c.column)]
                .biome
                .is_walkable()
                && occupancy.is_free_for(c, None)
                && world
                    .animals
                    .iter()
                    .all(|a| a.cell.center().distance(c.center()) > 15.0)
        })
        .expect("land on the second island")
}

#[test]
fn first_landing_arms_a_seeded_raid_that_lands_a_war_band_on_the_second_island() {
    for seed in [1, 123, DEFAULT_SEED] {
        let mut world = GameWorld::generate(seed);
        world.tick(0.1);
        assert_eq!(world.raid, Raid::Waiting, "the home island never arms it");
        world.discover_island();
        world.tick(0.1);
        assert_eq!(world.raid, Raid::Waiting, "discovery alone does not arm it");
        let cell = landing_cell(&world);
        world.units[0].cell = cell;
        world.units[0].action = UnitAction::Idle;
        world.tick(0.1);
        let Raid::Incoming { seconds_left } = world.raid else {
            panic!("a landing arms the raid");
        };
        assert!((299.0..=600.0).contains(&seconds_left));
        let animals = world.animals.len();
        world.raid = Raid::Incoming { seconds_left: 0.05 };
        let mut replay = world.clone();
        world.tick(0.1);
        replay.tick(0.1);
        assert_eq!(world.raid, Raid::Landed);
        assert_eq!(world.animals, replay.animals);
        let band = &world.animals[animals..];
        assert!((10..=20).contains(&band.len()), "{} raiders", band.len());
        assert_eq!(band[0].kind, AnimalKind::Chieftain);
        for (n, raider) in band.iter().enumerate().skip(1) {
            let torch = n % 3 == 0;
            assert_eq!(raider.kind == AnimalKind::Torchbearer, torch);
            assert_eq!(raider.kind == AnimalKind::Barbarian, !torch);
        }
        for raider in band {
            assert_eq!(world.island_at(raider.cell), Some(1));
        }
        assert!(band[0].cell.center().distance(cell.center()) >= 20.0);
        let landed = world.animals.len();
        world.validate().unwrap();
        // The raid lands once.
        world.tick(0.1);
        assert_eq!(world.animals.len(), landed);
    }
}

#[test]
fn raiders_cut_down_units_in_reach_before_buildings() {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.units[1].cell = CellCoordinate::new(5, 70);
    world.units[0].cell = CellCoordinate::new(40, 30);
    world.animals = vec![raider(AnimalKind::Barbarian, CellCoordinate::new(41, 30))];
    world.refresh_exploration();
    world.validate().unwrap();
    world.tick(0.5);
    assert_eq!(world.animals[0].heading, [-1, 0]);
    for _ in 0..14 {
        world.tick(0.5);
    }
    assert!(
        world.units.iter().all(|u| u.id != "villager-1"),
        "7 blows kill"
    );
    assert_eq!(world.buildings[0].damage, 0.0);
    world.validate().unwrap();
}

#[test]
fn raiders_march_on_buildings_and_tear_them_down() {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.units[0].cell = CellCoordinate::new(5, 70);
    world.units[1].cell = CellCoordinate::new(7, 70);
    world.animals = vec![raider(AnimalKind::Chieftain, CellCoordinate::new(45, 19))];
    world.refresh_exploration();
    let mut seconds = 0.0;
    while world.buildings[0].damage == 0.0 {
        world.tick(0.25);
        seconds += 0.25;
        assert!(
            seconds < 15.0,
            "never reached the town center: {:?}",
            world.animals[0]
        );
    }
    assert_eq!(world.buildings[0].damage, 25.0);
    let animals_never_damage = AnimalKind::Wolf.building_damage();
    assert_eq!(animals_never_damage, 0.0);
    // One blow short of falling: the next one razes it and ends orders on it.
    world.buildings[0].damage = BuildingKind::TownCenter.max_health() - 1.0;
    world.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Wood,
        amount: 5.0,
    });
    world.units[0].action = UnitAction::Deposit {
        storage_id: "base-1".into(),
    };
    world.validate().unwrap();
    world.tick(1.0);
    assert!(world.buildings.is_empty());
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.validate().unwrap();
    // With nothing left standing, the raiders hunt down the island's units.
    for _ in 0..40 {
        world.tick(0.5);
    }
    assert!(
        world.animals[0]
            .cell
            .center()
            .distance(CellCoordinate::new(5, 70).center())
            < world.animals[0]
                .home
                .center()
                .distance(CellCoordinate::new(5, 70).center())
    );
    world.validate().unwrap();
}

#[test]
fn friendly_units_can_fight_raiders() {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.units[1].cell = CellCoordinate::new(5, 70);
    world.units[0].cell = CellCoordinate::new(40, 30);
    world.units[0].kind = UnitKind::Guard;
    world.animals = vec![raider(AnimalKind::Barbarian, CellCoordinate::new(41, 30))];
    world.animals[0].health = 20.0;
    world.refresh_exploration();
    world
        .apply_command(Command::AttackAnimal {
            unit_ids: vec!["villager-1".into()],
            animal_id: "raider-0".into(),
        })
        .unwrap();
    world.tick(1.0);
    assert!(world.animals.is_empty());
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.validate().unwrap();
}

#[test]
fn torchbearers_burn_buildings_fast_but_fight_people_poorly() {
    let torch = AnimalKind::Torchbearer;
    assert!(torch.building_damage() > AnimalKind::Chieftain.building_damage());
    assert!(torch.damage() < AnimalKind::Barbarian.damage());
    let mut world = fixture::fixture();
    world.resources.clear();
    world.units[0].cell = CellCoordinate::new(5, 70);
    world.units[1].cell = CellCoordinate::new(7, 70);
    world.animals = vec![raider(torch, CellCoordinate::new(33, 19))];
    world.refresh_exploration();
    world.tick(1.0);
    assert_eq!(world.buildings[0].damage, 30.0);
    world.validate().unwrap();
}

#[test]
fn villagers_repair_damaged_buildings_for_free() {
    let mut world = fixture::fixture();
    world.resources.clear();
    world.buildings[0].damage = 50.0;
    let stock = world.inventories.clone();
    world
        .apply_command(Command::Construct {
            unit_id: "villager-1".into(),
            building_id: "base-1".into(),
        })
        .unwrap();
    world.tick(1.0);
    assert_eq!(world.buildings[0].damage, 30.0);
    for _ in 0..4 {
        world.tick(0.5);
    }
    assert_eq!(world.buildings[0].damage, 0.0);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    assert_eq!(world.inventories, stock);
    // An intact building needs no work.
    assert_eq!(
        world.apply_command(Command::Construct {
            unit_id: "villager-1".into(),
            building_id: "base-1".into(),
        }),
        Err(CommandError::BuildingAlreadyComplete)
    );
    world.validate().unwrap();
}
