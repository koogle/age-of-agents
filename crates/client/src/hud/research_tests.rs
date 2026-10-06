use super::*;
use aoa_game::GameWorld;
use glam::Vec2;

#[test]
fn specialist_research_explains_upgrade_lock_and_unlocks_after_completion() {
    let mut world = GameWorld::default();
    world.economy_rules = aoa_game::EconomyRules::Unrestricted;
    world.inventories[0].food = 100.0;
    world.inventories[0].wood = 100.0;
    world.buildings[0].kind = BuildingKind::Farm;
    world.buildings[0].researches = vec![TechnologyKind::Agriculture];
    let command = |world: &GameWorld| {
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
        selection_model(&snapshot, &model)
            .unwrap()
            .4
            .into_iter()
            .find(|c| c.action == Action::Research(TechnologyKind::Agriculture))
            .unwrap()
    };
    let locked = command(&world);
    assert!(!locked.enabled);
    assert_eq!(
        locked.detail,
        "Requires brick upgrade · Food gathering +20%"
    );
    world.buildings[0].job = Some(aoa_game::BuildingJob::Upgrade {
        elapsed_seconds: 19.9,
    });
    assert!(!command(&world).enabled);
    world.buildings[0].job = None;
    world.buildings[0].masonry = true;
    assert!(command(&world).enabled);
    world
        .researched_technologies
        .push(TechnologyKind::Agriculture);
    let done = command(&world);
    assert!(!done.enabled);
    assert_eq!(done.detail, "Research complete · Food gathering +20%");
}
