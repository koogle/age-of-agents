use super::*;
use aoa_game::{BUILDABLE, GameWorld};
use glam::Vec2;

#[test]
fn reassignment_status_explains_unloading_and_then_the_new_task() {
    let mut world = GameWorld::default();
    world.economy_rules = aoa_game::EconomyRules::Unrestricted;
    let visible = world.snapshot().resources[0].id.clone();
    world
        .resources
        .iter_mut()
        .find(|r| r.id == visible)
        .unwrap()
        .kind = ResourceKind::Food;
    world.units[0].cargo = Some(aoa_game::CarriedResource {
        kind: ResourceKind::Wood,
        amount: 7.0,
    });
    world.units[0].action = UnitAction::Gather {
        resource_id: visible,
        phase: aoa_game::GatherPhase::Returning,
    };
    assert!(unit_detail(&world).starts_with("Unloading wood before gathering food"));
    world.units[0].cargo = None;
    world.units[0].action = UnitAction::Gather {
        resource_id: "outside-fog".into(),
        phase: aoa_game::GatherPhase::ToResource,
    };
    assert_eq!(unit_detail(&world), "Heading out to gather · HP 100/100");
    world.units[0].health = 76.0;
    assert!(unit_detail(&world).ends_with("HP 76/100"));
}

#[test]
fn queued_products_use_their_production_art() {
    for (product, expected) in [
        (ProductKind::TransportShip, "transport"),
        (ProductKind::Villager, "command_train"),
        (ProductKind::Guard, "unit_guard"),
        (ProductKind::Archer, "unit_archer"),
        (ProductKind::Healer, "unit_healer"),
        (ProductKind::SiegeCart, "unit_siege_cart"),
        (ProductKind::Timber, "resource_timber"),
        (ProductKind::Steel, "resource_steel"),
        (ProductKind::Bricks, "resource_bricks"),
        (ProductKind::Cloth, "resource_cloth"),
        (ProductKind::Rations, "resource_rations"),
    ] {
        let mut snapshot = aoa_game::GameWorld::default().snapshot();
        let building = &mut snapshot.buildings[0].building;
        building.produces = vec![product];
        building.queue = vec![aoa_game::QueuedBuildingJob {
            id: 1,
            job: aoa_game::BuildingJob::Produce {
                product,
                elapsed_seconds: 0.0,
            },
        }];
        let model = Model {
            resource_island: 0,
            snapshot: Some(&snapshot),
            units: &[],
            building: Some(&snapshot.buildings[0].building.id),
            ship: None,
            build: BuildUi::Off,
            show_grid: false,
            toast: None,
            camera: glam::Vec2::ZERO,
        };
        let offered = selection_model(&snapshot, &model).unwrap().4;
        assert_eq!(
            offered
                .iter()
                .find(|c| c.action == Action::Produce(product))
                .unwrap()
                .icon,
            expected
        );
        assert_eq!(queued_commands(&snapshot, &model)[0].icon, expected);
    }
}

fn unit_detail(world: &GameWorld) -> String {
    let snapshot = world.snapshot();
    let units = [snapshot.units[0].unit.id.clone()];
    let model = Model {
        resource_island: 0,
        snapshot: Some(&snapshot),
        units: &units,
        building: None,
        build: BuildUi::Off,
        ship: None,
        show_grid: false,
        toast: None,
        camera: Vec2::ZERO,
    };
    selection_model(&snapshot, &model).unwrap().2
}

#[test]
fn material_upgrade_control_explains_locked_paid_pending_and_complete_states() {
    let mut world = GameWorld::default();
    let command = |w: &GameWorld| {
        commands_for_town_center(w)
            .into_iter()
            .find(|c| c.action == Action::UpgradeBuilding)
            .unwrap()
    };
    let locked = command(&world);
    assert!(!locked.enabled);
    assert!(locked.detail.contains("Discover clay"));
    world.economy_rules = aoa_game::EconomyRules::Unrestricted;
    world.inventories[0].bricks = 30.0;
    world.inventories[0].timber = 15.0;
    assert!(command(&world).enabled);
    world
        .apply_command(aoa_game::Command::UpgradeBuilding {
            building_id: world.buildings[0].id.clone(),
        })
        .unwrap();
    assert!(!command(&world).enabled);
    assert!(command(&world).detail.contains("in progress"));
    world.tick(aoa_game::BUILDING_UPGRADE_SECONDS);
    assert!(!command(&world).enabled);
    assert_eq!(command(&world).label, "Masonry complete");
}

#[test]
fn field_placement_keeps_water_cost_visible_without_hover() {
    let snapshot = GameWorld::default().snapshot();
    let units = [snapshot.units[0].unit.id.clone()];
    let model = Model {
        resource_island: 0,
        snapshot: Some(&snapshot),
        units: &units,
        building: None,
        build: BuildUi::PlacingField,
        ship: None,
        show_grid: false,
        toast: None,
        camera: Vec2::ZERO,
    };
    let (_, title, detail, _, _) = selection_model(&snapshot, &model).unwrap();
    assert_eq!(title, "Place field");
    assert!(detail.contains("10 wood, 5 stone, 10 water"));
}

#[test]
fn grouped_menu_exposes_every_building_once() {
    let mut world = GameWorld::default();
    world.economy_rules = aoa_game::EconomyRules::Unrestricted;
    let snapshot = world.snapshot();
    let units = [snapshot.units[0].unit.id.clone()];
    let mut seen = Vec::new();
    for group in super::super::BuildingGroup::ALL {
        let model = Model {
            resource_island: 0,
            snapshot: Some(&snapshot),
            units: &units,
            building: None,
            build: BuildUi::Group(group),
            ship: None,
            show_grid: false,
            toast: None,
            camera: Vec2::ZERO,
        };
        let commands = selection_model(&snapshot, &model).unwrap().4;
        assert!(commands.len() <= 8);
        for command in commands {
            if let Action::Place(kind) = command.action {
                seen.push(kind);
            }
        }
    }
    assert_eq!(seen.len(), BUILDABLE.len());
    for kind in BUILDABLE {
        assert_eq!(seen.iter().filter(|&&item| item == kind).count(), 1);
    }
}

#[test]
fn processor_commands_use_recipe_costs_and_show_blocked_jobs() {
    let mut world = GameWorld::default();
    world.economy_rules = aoa_game::EconomyRules::Unrestricted;
    world.buildings[0].kind = BuildingKind::Smelter;
    world.buildings[0].produces = BuildingKind::Smelter.products().to_vec();
    world.buildings[0].researches.clear();
    let poor = commands_for_town_center(&world);
    assert_eq!(poor.len(), 2);
    assert_eq!(poor[1].action, Action::UpgradeBuilding);
    assert!(!poor[1].enabled);
    assert_eq!(poor[0].action, Action::Produce(ProductKind::Steel));
    assert!(!poor[0].enabled);
    world.inventories[0].iron = 5.0;
    world.inventories[0].coal = 5.0;
    assert!(commands_for_town_center(&world)[0].enabled);
    world.buildings[0].job = Some(aoa_game::BuildingJob::Produce {
        product: ProductKind::Steel,
        elapsed_seconds: 1.0,
    });
    assert!(commands_for_town_center(&world)[0].enabled);
    let building_id = world.buildings[0].id.clone();
    for _ in 0..aoa_game::MAX_QUEUED_JOBS {
        world.inventories[0].iron = 5.0;
        world.inventories[0].coal = 5.0;
        world
            .apply_command(aoa_game::Command::Produce {
                building_id: building_id.clone(),
                product: ProductKind::Steel,
            })
            .unwrap();
    }
    world.inventories[0].iron = 5.0;
    world.inventories[0].coal = 5.0;
    let full = commands_for_town_center(&world);
    assert!(!full[0].enabled);
    assert_eq!(full[0].detail, "Queue is full");
    let snapshot = world.snapshot();
    let model = Model {
        resource_island: 0,
        snapshot: Some(&snapshot),
        units: &[],
        building: Some(&building_id),
        build: BuildUi::Off,
        ship: None,
        show_grid: false,
        toast: None,
        camera: Vec2::ZERO,
    };
    let queued = queued_commands(&snapshot, &model);
    assert_eq!(queued.len(), aoa_game::MAX_QUEUED_JOBS);
    assert_eq!(
        queued[2].action,
        Action::CancelQueuedJob(world.buildings[0].queue[2].id)
    );
    assert!(queued[2].detail.contains("iron"));
    assert!(queued[2].detail.contains("coal"));
}

fn commands_for_town_center(world: &GameWorld) -> Vec<Command> {
    selected_town_center(world).4
}

fn selected_town_center(world: &GameWorld) -> Selected {
    let snapshot = world.snapshot();
    let model = Model {
        resource_island: 0,
        snapshot: Some(&snapshot),
        units: &[],
        building: Some(&snapshot.buildings[0].building.id),
        build: BuildUi::Off,
        ship: None,
        show_grid: false,
        toast: None,
        camera: Vec2::ZERO,
    };
    selection_model(&snapshot, &model).unwrap()
}

#[test]
fn building_status_describes_work_instead_of_reusing_button_instructions() {
    let mut world = GameWorld::default();
    world.economy_rules = aoa_game::EconomyRules::Unrestricted;
    for (product, command, activity) in [
        (
            ProductKind::Villager,
            "Train a villager",
            "Training a villager",
        ),
        (ProductKind::Guard, "Train a guard", "Training a guard"),
        (ProductKind::Archer, "Train an archer", "Training an archer"),
        (ProductKind::Healer, "Train a healer", "Training a healer"),
        (
            ProductKind::SiegeCart,
            "Build a siege cart",
            "Building a siege cart",
        ),
        (
            ProductKind::TransportShip,
            "Build a transport",
            "Building a transport",
        ),
        (ProductKind::Timber, "Make 5 timber", "Making 5 timber"),
        (ProductKind::Steel, "Make 5 steel", "Making 5 steel"),
        (ProductKind::Bricks, "Make 5 bricks", "Making 5 bricks"),
        (ProductKind::Cloth, "Make 5 cloth", "Making 5 cloth"),
        (ProductKind::Rations, "Make 5 rations", "Making 5 rations"),
    ] {
        world.buildings[0].kind = *BUILDABLE
            .iter()
            .find(|kind| kind.products().contains(&product))
            .unwrap();
        world.buildings[0].job = Some(aoa_game::BuildingJob::Produce {
            product,
            elapsed_seconds: 1.0,
        });
        let selected = selected_town_center(&world);
        assert_eq!(selected.2, activity);
        assert_eq!(selected.4[0].label, command);
        world.buildings[0].job = Some(aoa_game::BuildingJob::Produce {
            product,
            elapsed_seconds: product.seconds(),
        });
        assert_eq!(
            selected_town_center(&world).2,
            if product.unit_kind().is_some() || product == ProductKind::TransportShip {
                "Waiting for space outside the building"
            } else {
                activity
            }
        );
    }
    world.buildings[0].kind = BuildingKind::TownCenter;
    world.buildings[0].job = Some(aoa_game::BuildingJob::Research {
        technology: TechnologyKind::Forestry,
        elapsed_seconds: 1.0,
    });
    assert_eq!(selected_town_center(&world).2, "Researching Forestry");
}

#[test]
fn town_center_coins_follow_costs_and_prerequisites() {
    let mut world = GameWorld::default();
    world.economy_rules = aoa_game::EconomyRules::Unrestricted;
    world.inventories[0].food = 0.0;
    world.inventories[0].wood = 0.0;
    let poor = commands_for_town_center(&world);
    assert!(poor.iter().all(|c| !c.enabled), "nothing is affordable");
    world.inventories[0].food = 100.0;
    world.inventories[0].wood = 100.0;
    let rich = commands_for_town_center(&world);
    let enabled = |action: Action| rich.iter().find(|c| c.action == action).unwrap().enabled;
    assert!(enabled(Action::Produce(ProductKind::Villager)));
    assert!(enabled(Action::Research(TechnologyKind::Masonry)));
    assert!(
        !rich
            .iter()
            .any(|c| c.action == Action::Research(TechnologyKind::Mining)),
        "mining belongs to an upgraded mining camp"
    );
}

#[test]
fn completed_research_stays_visible_and_disabled_after_reload() {
    let mut world = GameWorld::default();
    world.inventories[0].food = 100.0;
    world.inventories[0].wood = 100.0;
    world
        .apply_command(aoa_game::Command::Research {
            building_id: world.buildings[0].id.clone(),
            technology: TechnologyKind::Masonry,
        })
        .unwrap();
    let command = |world: &GameWorld| {
        commands_for_town_center(world)
            .into_iter()
            .find(|c| c.action == Action::Research(TechnologyKind::Masonry))
            .unwrap()
    };
    assert!(!command(&world).enabled);
    assert_eq!(command(&world).detail, "Research queued or in progress");
    world.tick(aoa_game::RESEARCH_SECONDS + 0.1);
    assert!(
        world
            .researched_technologies
            .contains(&TechnologyKind::Masonry)
    );
    let saved = serde_json::to_string(&world).unwrap();
    let restored: GameWorld = serde_json::from_str(&saved).unwrap();
    let completed = command(&restored);
    assert!(!completed.enabled);
    assert_eq!(completed.label, "Masonry");
    assert!(completed.detail.starts_with("Research complete ·"));
}

#[test]
fn build_menu_greys_out_what_the_stockpile_cannot_cover() {
    let mut world = GameWorld::default();
    world.economy_rules = aoa_game::EconomyRules::Unrestricted;
    world.inventories[0] = Default::default();
    world.inventories[0].wood = 15.0;
    let snapshot = world.snapshot();
    let units = [snapshot.units[0].unit.id.clone()];
    let model = Model {
        resource_island: 0,
        snapshot: Some(&snapshot),
        units: &units,
        building: None,
        build: BuildUi::Group(super::super::BuildingGroup::Town),
        ship: None,
        show_grid: false,
        toast: None,
        camera: Vec2::ZERO,
    };
    let commands = selection_model(&snapshot, &model).unwrap().4;
    let enabled = |kind| {
        commands
            .iter()
            .find(|c| c.action == Action::Place(kind))
            .unwrap()
            .enabled
    };
    assert!(enabled(BuildingKind::House));
    assert!(!enabled(BuildingKind::TownCenter));
    assert!(!enabled(BuildingKind::Granary));
    assert!(!enabled(BuildingKind::Monument));
    assert!(!enabled(BuildingKind::Dock));
    assert!(commands.iter().any(|c| c.action == Action::Build));
}
