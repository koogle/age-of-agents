use super::*;

fn funded_world() -> GameWorld {
    let mut world = fixture::fixture();
    world.inventories[0].food = 1000.0;
    world.inventories[0].wood = 1000.0;
    world
}
fn train(world: &mut GameWorld) -> Result<(), CommandError> {
    world.apply_command(Command::Produce {
        building_id: "base-1".into(),
        product: ProductKind::Villager,
    })
}
fn research(world: &mut GameWorld, technology: TechnologyKind) -> Result<(), CommandError> {
    world.apply_command(Command::Research {
        building_id: "base-1".into(),
        technology,
    })
}
fn cancel(world: &mut GameWorld, queue_id: u64) -> Result<(), CommandError> {
    world.apply_command(Command::CancelQueuedJob {
        building_id: "base-1".into(),
        queue_id,
    })
}

#[test]
fn mixed_queue_pays_once_refunds_middle_task_and_finishes_in_order_after_reload() {
    let mut world = funded_world();
    train(&mut world).unwrap();
    research(&mut world, TechnologyKind::Masonry).unwrap();
    train(&mut world).unwrap();
    train(&mut world).unwrap();
    assert_eq!(
        (world.inventories[0].food, world.inventories[0].wood),
        (810.0, 980.0)
    );
    let cancelled = world.buildings[0].queue[1].id;
    cancel(&mut world, cancelled).unwrap();
    assert_eq!(
        (world.inventories[0].food, world.inventories[0].wood),
        (860.0, 980.0)
    );
    assert_eq!(world.villagers_and_trainees(), 4);
    let before = world.clone();
    assert_eq!(
        cancel(&mut world, cancelled),
        Err(CommandError::QueuedJobNotFound)
    );
    assert_eq!(world, before);
    world.tick(2.0);
    let mut loaded: GameWorld =
        serde_json::from_str(&serde_json::to_string(&world).unwrap()).unwrap();
    loaded.validate().unwrap();
    for dt in [4.0, 8.0, 8.0, 8.0] {
        world.tick(dt);
        loaded.tick(dt);
        assert_eq!(world, loaded);
        world.validate().unwrap();
    }
    assert_eq!(world.units.len(), 4);
    assert_eq!(world.researched_technologies, vec![TechnologyKind::Masonry]);
    assert!(world.buildings[0].jobs().next().is_none());
    assert_eq!(
        (world.inventories[0].food, world.inventories[0].wood),
        (860.0, 980.0)
    );
}

#[test]
fn cancellation_cannot_target_promoted_job_or_reuse_its_id() {
    let mut world = funded_world();
    train(&mut world).unwrap();
    research(&mut world, TechnologyKind::Masonry).unwrap();
    let promoted = world.buildings[0].queue[0].id;
    world.tick(VILLAGER_PRODUCTION_SECONDS);
    train(&mut world).unwrap();
    assert_ne!(world.buildings[0].queue[0].id, promoted);
    let before = world.clone();
    assert_eq!(
        cancel(&mut world, promoted),
        Err(CommandError::QueuedJobNotFound)
    );
    assert_eq!(world, before);
    let waiting = world.buildings[0].queue[0].id;
    cancel(&mut world, waiting).unwrap();
    assert_eq!(
        (world.inventories[0].food, world.inventories[0].wood),
        (910.0, 980.0)
    );
    train(&mut world).unwrap();
    assert!(world.buildings[0].queue[0].id > waiting);
}

#[test]
fn waiting_trainees_reserve_housing_and_cancellation_releases_it() {
    let mut world = funded_world();
    for _ in 0..3 {
        train(&mut world).unwrap();
    }
    assert_eq!(world.villagers_and_trainees(), world.housing());
    let before = world.clone();
    assert_eq!(train(&mut world), Err(CommandError::PopulationCapReached));
    assert_eq!(world, before);
    let id = world.buildings[0].queue[0].id;
    cancel(&mut world, id).unwrap();
    train(&mut world).unwrap();
    assert_eq!(world.inventories[0].food, 850.0);
}

#[test]
fn waiting_research_is_unique_across_buildings_and_prerequisites_still_apply() {
    let mut world = funded_world();
    world.resources.clear();
    world.buildings.push(building(
        BuildingKind::Kiln,
        "mill",
        CellCoordinate::new(10, 10),
        None,
    ));
    train(&mut world).unwrap();
    research(&mut world, TechnologyKind::Masonry).unwrap();
    let mut camp = building(
        BuildingKind::MiningCamp,
        "camp",
        CellCoordinate::new(20, 10),
        None,
    );
    camp.masonry = true;
    world.buildings.push(camp);
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::Research {
            building_id: "mill".into(),
            technology: TechnologyKind::Masonry
        }),
        Err(CommandError::TechnologyInProgress)
    );
    assert_eq!(
        world.apply_command(Command::Research {
            building_id: "camp".into(),
            technology: TechnologyKind::Mining
        }),
        Err(CommandError::MissingTechnologyPrerequisite)
    );
    assert_eq!(world, before);
    let id = world.buildings[0].queue[0].id;
    cancel(&mut world, id).unwrap();
    world
        .apply_command(Command::Research {
            building_id: "mill".into(),
            technology: TechnologyKind::Masonry,
        })
        .unwrap();
}

#[test]
fn processing_queue_limit_and_insufficient_inputs_reject_atomically() {
    let mut world = funded_world();
    world.buildings.push(building(
        BuildingKind::Smelter,
        "smelter",
        CellCoordinate::new(10, 10),
        None,
    ));
    world.inventories[0].iron = 100.0;
    world.inventories[0].coal = 100.0;
    let produce = Command::Produce {
        building_id: "smelter".into(),
        product: ProductKind::Steel,
    };
    for _ in 0..=MAX_QUEUED_JOBS {
        world.apply_command(produce.clone()).unwrap();
    }
    let before = world.clone();
    assert_eq!(
        world.apply_command(produce.clone()),
        Err(CommandError::BuildingQueueFull)
    );
    assert_eq!(world, before);
    let id = world.buildings[1].queue[2].id;
    world
        .apply_command(Command::CancelQueuedJob {
            building_id: "smelter".into(),
            queue_id: id,
        })
        .unwrap();
    assert_eq!(
        (world.inventories[0].iron, world.inventories[0].coal),
        (75.0, 75.0)
    );
    world.inventories[0].coal = 0.0;
    let before = world.clone();
    assert_eq!(
        world.apply_command(produce),
        Err(CommandError::InsufficientProductionResources)
    );
    assert_eq!(world, before);
    for _ in 0..5 {
        world.tick(10.0);
    }
    assert_eq!(world.inventories[0].steel, 25.0);
    assert!(world.buildings[1].jobs().next().is_none());
}

#[test]
fn current_save_round_trips_queue_and_invalid_queues_are_rejected() {
    let mut world = funded_world();
    train(&mut world).unwrap();
    let json = serde_json::to_value(&world).unwrap();
    let loaded: GameWorld = serde_json::from_value(json).unwrap();
    assert_eq!(loaded, world);
    research(&mut world, TechnologyKind::Masonry).unwrap();
    let valid = world.clone();
    world.buildings[0].queue[0].job = BuildingJob::Produce {
        product: ProductKind::Steel,
        elapsed_seconds: 0.0,
    };
    assert!(world.validate().is_err());
    world = valid.clone();
    world.buildings[0].queue[0].id = world.buildings[0].next_queue_id;
    assert!(world.validate().is_err());
    world = valid.clone();
    world.buildings[0].queue[0].job = BuildingJob::Research {
        technology: TechnologyKind::Masonry,
        elapsed_seconds: 0.1,
    };
    assert!(world.validate().is_err());
    world = valid;
    world.buildings[0].job = None;
    assert!(world.validate().is_err());
}
