use super::*;

fn upgrade(world: &mut GameWorld) -> Result<(), CommandError> {
    world.apply_command(Command::UpgradeBuilding {
        building_id: "base-1".into(),
    })
}

fn funded() -> GameWorld {
    let mut w = fixture::fixture();
    w.inventories[0].bricks = 100.0;
    w.inventories[0].timber = 100.0;
    w.inventories[0].food = 100.0;
    w
}

#[test]
fn upgrade_pays_once_keeps_plot_and_functions_and_survives_reload() {
    let mut w = funded();
    let original = w.buildings[0].clone();
    upgrade(&mut w).unwrap();
    assert_eq!(
        (w.inventories[0].bricks, w.inventories[0].timber),
        (70.0, 85.0)
    );
    let before = w.clone();
    assert_eq!(upgrade(&mut w), Err(CommandError::BuildingAlreadyUpgraded));
    assert_eq!(w, before);
    w.tick(3.0);
    let mut loaded: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    w.tick(BUILDING_UPGRADE_SECONDS);
    loaded.tick(BUILDING_UPGRADE_SECONDS);
    assert_eq!(w, loaded);
    assert!(w.buildings[0].masonry);
    assert_eq!(w.buildings[0].origin, original.origin);
    assert_eq!(w.buildings[0].kind, original.kind);
    assert_eq!(w.buildings[0].produces, original.produces);
    assert_eq!(w.buildings[0].researches, original.researches);
    assert!(w.buildings[0].job.is_none());
    let before = w.clone();
    assert_eq!(upgrade(&mut w), Err(CommandError::BuildingAlreadyUpgraded));
    assert_eq!(w, before);
    w.validate().unwrap();
}

#[test]
fn waiting_upgrade_refunds_once_and_can_be_reordered() {
    let mut w = funded();
    w.apply_command(Command::Produce {
        building_id: "base-1".into(),
        product: ProductKind::Villager,
    })
    .unwrap();
    upgrade(&mut w).unwrap();
    let id = w.buildings[0].queue[0].id;
    w.apply_command(Command::CancelQueuedJob {
        building_id: "base-1".into(),
        queue_id: id,
    })
    .unwrap();
    assert_eq!(
        (w.inventories[0].bricks, w.inventories[0].timber),
        (100.0, 100.0)
    );
    let before = w.clone();
    assert_eq!(
        w.apply_command(Command::CancelQueuedJob {
            building_id: "base-1".into(),
            queue_id: id
        }),
        Err(CommandError::QueuedJobNotFound)
    );
    assert_eq!(w, before);
    upgrade(&mut w).unwrap();
    w.tick(VILLAGER_PRODUCTION_SECONDS);
    assert!(matches!(
        w.buildings[0].job,
        Some(BuildingJob::Upgrade { .. })
    ));
    w.tick(BUILDING_UPGRADE_SECONDS);
    assert!(w.buildings[0].masonry);
    w.validate().unwrap();
}

#[test]
fn upgrade_requires_discovered_clay_even_with_stock_and_rejects_insufficient_inputs() {
    let mut w = funded();
    w.economy_rules = EconomyRules::IslandProgression;
    w.resources.retain(|r| r.kind != ResourceKind::Clay);
    let before = w.clone();
    assert_eq!(upgrade(&mut w), Err(CommandError::UpgradeUnavailable));
    assert_eq!(w, before);
    w.economy_rules = EconomyRules::Unrestricted;
    w.inventories[0].timber = 0.0;
    let before = w.clone();
    assert_eq!(
        upgrade(&mut w),
        Err(CommandError::InsufficientResources(ResourceKind::Timber))
    );
    assert_eq!(w, before);
}

#[test]
fn corrupt_upgrade_states_are_rejected() {
    let mut w = funded();
    upgrade(&mut w).unwrap();
    w.buildings[0].masonry = true;
    assert!(w.validate().is_err());
    w.buildings[0].masonry = false;
    w.buildings[0].job = Some(BuildingJob::Upgrade {
        elapsed_seconds: BUILDING_UPGRADE_SECONDS,
    });
    assert!(w.validate().is_err());
    w.buildings[0].job = Some(BuildingJob::Upgrade {
        elapsed_seconds: 0.0,
    });
    w.buildings[0].enqueue(BuildingJob::Upgrade {
        elapsed_seconds: 0.0,
    });
    assert!(w.validate().is_err());
}

#[test]
fn every_catalog_building_can_upgrade_without_replacing_its_identity() {
    for kind in BUILDABLE {
        let mut w = funded();
        // Use the existing valid footprint; ticking the job directly isolates
        // material conversion from each building's placement requirements.
        w.buildings[0].kind = kind;
        w.buildings[0].produces = kind.products().to_vec();
        w.upgrade_building("base-1").unwrap();
        w.tick_building_job(0, BUILDING_UPGRADE_SECONDS);
        assert!(w.buildings[0].masonry, "{kind:?}");
        assert_eq!(w.buildings[0].id, "base-1");
        assert_eq!(w.buildings[0].kind, kind);
        assert!(w.buildings[0].job.is_none());
    }
}

#[test]
fn foundation_and_full_queue_reject_upgrade_without_payment() {
    let mut w = funded();
    w.buildings[0].construction = Some(0.0);
    let before = w.clone();
    assert_eq!(
        upgrade(&mut w),
        Err(CommandError::BuildingUnderConstruction)
    );
    assert_eq!(w, before);
    w.buildings[0].construction = None;
    for _ in 0..=MAX_QUEUED_JOBS {
        w.buildings[0].enqueue(BuildingJob::Produce {
            product: ProductKind::Villager,
            elapsed_seconds: 0.0,
        });
    }
    let before = w.clone();
    assert_eq!(upgrade(&mut w), Err(CommandError::BuildingQueueFull));
    assert_eq!(w, before);
}

#[test]
fn upgrades_spend_and_refund_only_the_buildings_island() {
    let mut w = funded();
    w.discover_island();
    w.inventories[1] = w.inventories[0].clone();
    w.inventories[0].bricks = 0.0;
    let before = w.clone();
    assert_eq!(
        upgrade(&mut w),
        Err(CommandError::InsufficientResources(ResourceKind::Bricks))
    );
    assert_eq!(w, before);
    w.inventories[0].bricks = 100.0;
    w.apply_command(Command::Produce {
        building_id: "base-1".into(),
        product: ProductKind::Villager,
    })
    .unwrap();
    let away = w.inventories[1].clone();
    upgrade(&mut w).unwrap();
    let queue_id = w.buildings[0].queue[0].id;
    w.apply_command(Command::CancelQueuedJob {
        building_id: "base-1".into(),
        queue_id,
    })
    .unwrap();
    assert_eq!(w.inventories[1], away);
    assert_eq!(
        (w.inventories[0].bricks, w.inventories[0].timber),
        (100.0, 100.0)
    );
    w.validate().unwrap();
}

#[test]
fn paused_upgrade_is_rejected_without_spending_until_resumed() {
    let mut w = funded();
    w.apply_command(Command::SetSimulationSpeed { multiplier: 0.0 })
        .unwrap();
    let before = w.clone();
    assert_eq!(upgrade(&mut w), Err(CommandError::GamePaused));
    w.tick(10.0);
    assert_eq!(w, before);
    w.apply_command(Command::SetSimulationSpeed { multiplier: 1.0 })
        .unwrap();
    upgrade(&mut w).unwrap();
    assert_eq!(
        (w.inventories[0].bricks, w.inventories[0].timber),
        (70.0, 85.0)
    );
}

#[test]
fn specialist_research_unlocks_only_after_upgrade_completion_and_survives_reload() {
    for (kind, tech) in [
        (BuildingKind::Farm, TechnologyKind::Agriculture),
        (BuildingKind::LumberMill, TechnologyKind::Forestry),
        (BuildingKind::MiningCamp, TechnologyKind::Mining),
        (BuildingKind::Weaver, TechnologyKind::Textiles),
    ] {
        let mut w = funded();
        w.resources.clear();
        w.inventories[0].wood = 1000.0;
        if let Some(prerequisite) = tech.prerequisite() {
            w.researched_technologies.push(prerequisite);
        }
        w.buildings.push(building(
            kind,
            "specialist",
            CellCoordinate::new(10, 10),
            None,
        ));
        let research = Command::Research {
            building_id: "specialist".into(),
            technology: tech,
        };
        let before = w.clone();
        assert_eq!(
            w.apply_command(research.clone()),
            Err(CommandError::MasonryUpgradeRequired)
        );
        assert_eq!(w, before, "locked research must not reserve inputs");
        assert_eq!(
            w.apply_command(Command::Research {
                building_id: "base-1".into(),
                technology: tech
            }),
            Err(CommandError::TechnologyUnavailable)
        );
        w.apply_command(Command::UpgradeBuilding {
            building_id: "specialist".into(),
        })
        .unwrap();
        w.tick(BUILDING_UPGRADE_SECONDS - 0.1);
        let before = w.clone();
        assert_eq!(
            w.apply_command(research.clone()),
            Err(CommandError::MasonryUpgradeRequired)
        );
        assert_eq!(w, before, "a pending upgrade does not unlock research");
        w.tick(0.1);
        let mut w: GameWorld = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
        w.validate().unwrap();
        let food = w.inventories[0].food;
        w.apply_command(research).unwrap();
        assert_eq!(w.inventories[0].food, food - RESEARCH_FOOD_COST);
        w.tick(RESEARCH_SECONDS);
        assert!(w.researched_technologies.contains(&tech));
        w.validate().unwrap();
    }
}

#[test]
fn cancelled_upgrade_does_not_unlock_research_or_allow_invalid_saved_jobs() {
    let mut w = funded();
    w.resources.clear();
    w.inventories[0].wood = 100.0;
    w.buildings.push(building(
        BuildingKind::LumberMill,
        "mill",
        CellCoordinate::new(10, 10),
        None,
    ));
    w.apply_command(Command::Produce {
        building_id: "mill".into(),
        product: ProductKind::Timber,
    })
    .unwrap();
    w.apply_command(Command::UpgradeBuilding {
        building_id: "mill".into(),
    })
    .unwrap();
    let queue_id = w.buildings[1].queue[0].id;
    w.apply_command(Command::CancelQueuedJob {
        building_id: "mill".into(),
        queue_id,
    })
    .unwrap();
    assert_eq!(
        w.apply_command(Command::Research {
            building_id: "mill".into(),
            technology: TechnologyKind::Forestry
        }),
        Err(CommandError::MasonryUpgradeRequired)
    );
    w.buildings[1].enqueue(BuildingJob::Research {
        technology: TechnologyKind::Forestry,
        elapsed_seconds: 0.0,
    });
    assert!(w.validate().is_err());
}
