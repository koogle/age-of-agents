use super::*;

fn cell(column: u16, row: u16) -> CellCoordinate {
    CellCoordinate::new(column, row)
}

fn world_with(kind: BuildingKind) -> GameWorld {
    let mut world = fixture::fixture();
    world.resources.clear();
    world
        .buildings
        .push(building(kind, "test-building", cell(10, 10), None));
    world
}

fn produce(world: &mut GameWorld, product: ProductKind) -> Result<(), CommandError> {
    world.apply_command(Command::Produce {
        building_id: "test-building".into(),
        product,
    })
}

#[test]
fn all_catalog_buildings_construct_complete_and_charge_exactly_once() {
    for kind in BUILDABLE {
        let mut world = fixture::fixture();
        world.resources.clear();
        if kind == BuildingKind::Dock {
            let index = 10 * usize::from(WORLD_COLUMNS) + 9;
            world.terrain[index].biome = TerrainBiome::Water;
            world.terrain[index].elevation = -0.2;
        }
        for &(resource, amount) in kind.cost() {
            world.stockpile.add(resource, amount);
        }
        assert!(!kind.cost().is_empty());
        assert!(kind.size().0 >= 2 && kind.size().1 >= 2);
        world
            .apply_command(Command::Build {
                unit_id: "villager-1".into(),
                origin: cell(10, 10),
                kind,
            })
            .unwrap();
        assert!(
            kind.cost()
                .iter()
                .all(|&(resource, _)| world.stockpile.amount(resource) == 0.0)
        );
        for _ in 0..1500 {
            world.tick(0.1);
        }
        assert!(world.buildings[1].is_complete(), "{kind:?}");
        assert_eq!(world.buildings[1].produces, kind.products());
        assert_eq!(world.buildings.len(), 2);
        world.validate().unwrap();
    }
}

#[test]
fn processing_reserves_inputs_finishes_once_and_survives_mid_job_reload() {
    for kind in [
        BuildingKind::LumberMill,
        BuildingKind::Smelter,
        BuildingKind::Kiln,
        BuildingKind::Weaver,
        BuildingKind::Kitchen,
    ] {
        let mut world = world_with(kind);
        let product = kind.products()[0];
        let before = world.clone();
        assert_eq!(
            produce(&mut world, product),
            Err(CommandError::InsufficientProductionResources)
        );
        assert_eq!(world, before);
        for &(resource, amount) in product.cost() {
            world.stockpile.add(resource, amount);
        }
        produce(&mut world, product).unwrap();
        assert!(
            product
                .cost()
                .iter()
                .all(|&(resource, _)| world.stockpile.amount(resource) == 0.0)
        );
        let reserved = world.clone();
        assert_eq!(
            produce(&mut world, product),
            Err(CommandError::BuildingBusy)
        );
        assert_eq!(world, reserved);
        world.tick(1.0);
        let mut loaded: GameWorld =
            serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
        loaded.validate().unwrap();
        for _ in 0..200 {
            world.tick(0.1);
            loaded.tick(0.1);
        }
        assert_eq!(loaded, world);
        let (output, amount) = product.output().unwrap();
        assert_eq!(world.stockpile.amount(output), amount);
        assert!(world.buildings[1].job.is_none());
        assert_eq!(world.villagers_and_trainees(), 2);
    }
}

#[test]
fn each_training_building_produces_its_unit_and_reserves_housing() {
    for kind in [
        BuildingKind::Barracks,
        BuildingKind::Range,
        BuildingKind::Workshop,
        BuildingKind::Infirmary,
    ] {
        let mut world = world_with(kind);
        let product = kind.products()[0];
        for &(resource, amount) in product.cost() {
            world.stockpile.add(resource, amount);
        }
        produce(&mut world, product).unwrap();
        assert_eq!(world.villagers_and_trainees(), 3);
        for _ in 0..150 {
            world.tick(0.1);
        }
        assert_eq!(world.units.len(), 3);
        assert_eq!(world.units[2].kind, product.unit_kind().unwrap());
        assert_eq!(world.units[2].action, UnitAction::Idle);
        let before = world.clone();
        assert_eq!(
            world.apply_command(Command::Build {
                unit_id: world.units[2].id.clone(),
                origin: cell(20, 20),
                kind: BuildingKind::House
            }),
            Err(CommandError::VillagerRequired)
        );
        assert_eq!(world, before);
    }
}

#[test]
fn blocked_spawn_waits_without_recharging_then_completes_once() {
    let mut world = world_with(BuildingKind::Barracks);
    let product = ProductKind::Guard;
    for &(resource, amount) in product.cost() {
        world.stockpile.add(resource, amount);
    }
    produce(&mut world, product).unwrap();
    // Ring the barracks with resources so every adjacent spawn cell is occupied.
    let footprint = world.buildings[1].footprint();
    for row in 9..15 {
        for column in 9..15 {
            let cell = cell(column, row);
            if footprint.is_interaction_cell(cell) {
                world.resources.push(ResourceNode {
                    id: format!("block-{column}-{row}"),
                    kind: ResourceKind::Stone,
                    cell,
                    amount: 1.0,
                    capacity: 1.0,
                });
            }
        }
    }
    for _ in 0..150 {
        world.tick(0.1);
    }
    assert_eq!(world.units.len(), 2);
    assert!(world.buildings[1].job.is_some());
    world.resources.clear();
    world.tick(0.1);
    assert_eq!(world.units.len(), 3);
    assert!(world.buildings[1].job.is_none());
    world.tick(0.1);
    assert_eq!(world.units.len(), 3);
    assert_eq!(world.stockpile.food, 0.0);
}

#[test]
fn research_cannot_be_reserved_twice_at_different_buildings() {
    let mut world = world_with(BuildingKind::Farm);
    world.stockpile.food = 100.0;
    world.stockpile.wood = 100.0;
    world
        .apply_command(Command::Research {
            building_id: "base-1".into(),
            technology: TechnologyKind::Agriculture,
        })
        .unwrap();
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Research {
            building_id: "test-building".into(),
            technology: TechnologyKind::Agriculture
        }),
        Err(CommandError::TechnologyInProgress)
    );
    assert_eq!(world, before);
}

#[test]
fn extraction_bonus_is_local_non_stacking_and_requires_completion() {
    let mut world = world_with(BuildingKind::Farm);
    assert_eq!(
        world.extraction_multiplier(ResourceKind::Food, cell(14, 14)),
        1.25
    );
    assert_eq!(
        world.extraction_multiplier(ResourceKind::Fiber, cell(14, 14)),
        1.25
    );
    assert_eq!(
        world.extraction_multiplier(ResourceKind::Wood, cell(14, 14)),
        1.0
    );
    assert_eq!(
        world.extraction_multiplier(ResourceKind::Food, cell(40, 30)),
        1.0
    );
    world
        .buildings
        .push(building(BuildingKind::Farm, "farm-2", cell(16, 10), None));
    assert_eq!(
        world.extraction_multiplier(ResourceKind::Food, cell(14, 14)),
        1.25
    );
    world.buildings[1].construction = Some(0.0);
    world.buildings[2].construction = Some(0.0);
    assert_eq!(
        world.extraction_multiplier(ResourceKind::Food, cell(14, 14)),
        1.0
    );
    let camp = world_with(BuildingKind::MiningCamp);
    assert_eq!(
        camp.extraction_multiplier(ResourceKind::Coal, cell(14, 14)),
        1.25
    );
    assert!(BuildingKind::MiningCamp.accepts(ResourceKind::Iron));
    assert!(!BuildingKind::MiningCamp.accepts(ResourceKind::Food));
}

#[test]
fn unavailable_jobs_and_corrupt_job_capabilities_are_rejected() {
    let mut world = world_with(BuildingKind::LumberMill);
    let before = world.clone();
    assert_eq!(
        produce(&mut world, ProductKind::Steel),
        Err(CommandError::ProductUnavailable)
    );
    assert_eq!(world, before);
    world.buildings[1].job = Some(BuildingJob::Produce {
        product: ProductKind::Steel,
        elapsed_seconds: 1.0,
    });
    assert!(world.validate().is_err());
}

#[test]
fn farm_bonus_changes_gathering_and_camp_accepts_actual_deposits() {
    let mut farm = world_with(BuildingKind::Farm);
    farm.resources.push(ResourceNode {
        id: "crop".into(),
        kind: ResourceKind::Food,
        cell: cell(14, 14),
        amount: 20.0,
        capacity: 20.0,
    });
    farm.units[0].cell = cell(13, 14);
    farm.units[0].action = UnitAction::Gather {
        resource_id: "crop".into(),
        phase: GatherPhase::Gathering,
    };
    farm.tick(1.0);
    assert_eq!(farm.units[0].cargo.as_ref().unwrap().amount, 2.5);
    let mut camp = world_with(BuildingKind::MiningCamp);
    camp.units[0].cell = cell(13, 10);
    camp.units[0].cargo = Some(CarriedResource {
        kind: ResourceKind::Iron,
        amount: 10.0,
    });
    camp.apply_command(Command::Deposit {
        unit_id: "villager-1".into(),
        building_id: "test-building".into(),
    })
    .unwrap();
    camp.tick(0.1);
    assert_eq!(camp.stockpile.iron, 10.0);
    assert!(camp.units[0].cargo.is_none());
}

#[test]
fn processing_does_not_reserve_housing_but_parallel_training_does() {
    let mut world = world_with(BuildingKind::LumberMill);
    world.stockpile.wood = 10.0;
    produce(&mut world, ProductKind::Timber).unwrap();
    assert_eq!(world.villagers_and_trainees(), 2);
    for index in 0..4 {
        world.buildings.push(building(
            BuildingKind::Barracks,
            &format!("b-{index}"),
            cell(2 + index * 6, 2),
            None,
        ));
        world.stockpile.food = 40.0;
        world.stockpile.steel = 2.0;
        let before = world.clone();
        let result = world.apply_command(Command::Produce {
            building_id: format!("b-{index}"),
            product: ProductKind::Guard,
        });
        if index < 3 {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CommandError::PopulationCapReached));
            assert_eq!(world, before);
        }
    }
}
