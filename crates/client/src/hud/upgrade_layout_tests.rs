use super::*;
use aoa_game::{BuildingJob, EconomyRules, GameWorld};

fn is_upgrade(action: &Action) -> bool {
    matches!(action, Action::UpgradeBuilding)
        || matches!(action, Action::Explain(text) if text.starts_with("Upgrade to masonry:") || text.starts_with("Masonry complete:"))
}

#[test]
fn every_building_has_one_stable_top_right_upgrade_target_in_every_state() {
    let assets = pollster::block_on(Assets::load());
    let atlas = build_atlas(&assets);
    assert!(atlas.content.contains_key("command_upgrade"));
    let mut world = GameWorld::default();
    world.economy_rules = EconomyRules::Unrestricted;
    for kind in BUILDABLE {
        for state in ["available", "unaffordable", "pending", "complete"] {
            world.buildings[0].kind = kind;
            world.buildings[0].masonry = state == "complete";
            world.buildings[0].job = (state == "pending").then_some(BuildingJob::Upgrade {
                elapsed_seconds: 2.0,
            });
            world.inventories[0].bricks = if state == "unaffordable" { 0.0 } else { 100.0 };
            world.inventories[0].timber = 100.0;
            let snapshot = world.snapshot();
            let model = Model {
                resource_island: 0,
                snapshot: Some(&snapshot),
                units: &[],
                building: Some(&snapshot.buildings[0].building.id),
                ship: None,
                build: BuildUi::Off,
                show_grid: false,
                toast: None,
                camera: Vec2::ZERO,
            };
            for (width, height, scale) in [
                (320.0, 844.0, 2.0),
                (390.0, 844.0, 2.0),
                (844.0, 390.0, 2.0),
                (1440.0, 900.0, 1.0),
            ] {
                let mut hud = Hud::new();
                hud.layout(&atlas, &model, width * scale, height * scale, scale);
                let regions: Vec<_> = hud
                    .regions
                    .iter()
                    .filter(|r| is_upgrade(&r.action))
                    .collect();
                assert_eq!(regions.len(), 1, "{kind:?} {state}");
                let rect = regions[0].rect;
                let action = regions[0].action.clone();
                assert_eq!(
                    matches!(action, Action::UpgradeBuilding),
                    state == "available"
                );
                assert_eq!([rect[2], rect[3]], [44.0 * scale; 2]);
                assert!(
                    rect[0] >= 0.0
                        && rect[1] >= 0.0
                        && rect[0] + rect[2] <= width * scale
                        && rect[1] + rect[3] <= height * scale
                );
                assert!(
                    hud.quads.iter().any(|q| q.color == GLASS
                        && (q.rect[0] + q.rect[2] - rect[0] - 52.0 * scale).abs() < 0.01
                        && (rect[1] - q.rect[1] - 4.0 * scale).abs() < 0.01),
                    "upgrade must occupy description top-right"
                );
                let header = hud
                    .quads
                    .iter()
                    .find(|q| {
                        q.color == GLASS
                            && (rect[1] - q.rect[1] - 4.0 * scale).abs() < 0.01
                            && (q.rect[0] + q.rect[2] - rect[0] - 52.0 * scale).abs() < 0.01
                    })
                    .unwrap();
                assert_eq!(
                    header.rect[3],
                    52.0 * scale,
                    "one compact row for every building/state"
                );
                let description = hud
                    .regions
                    .iter()
                    .find(|r| r.rect[0] == header.rect[0] && r.rect[1] == header.rect[1])
                    .unwrap();
                let description_action = description.action.clone();
                assert!(matches!(description_action, Action::Explain(_)));
                let name_point = Vec2::new(
                    description.rect[0] + 20.0 * scale,
                    description.rect[1] + 26.0 * scale,
                );
                assert!(hud.press(name_point));
                assert_eq!(hud.release(), Some(description_action));
                let center = Vec2::new(rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0);
                hud.hover = Some(center);
                hud.layout(&atlas, &model, width * scale, height * scale, scale);
                assert_eq!(
                    hud.regions
                        .iter()
                        .find(|r| is_upgrade(&r.action))
                        .unwrap()
                        .rect,
                    rect,
                    "hover must not move its own target"
                );
                assert!(hud.press(center));
                assert_eq!(hud.release(), Some(action));
            }
        }
    }
}
