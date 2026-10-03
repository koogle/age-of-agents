//! Regressions for rendered work frames after the first item enters the load.
use super::*;
use aoa_game::{CarriedResource, GameWorld};

fn sheets() -> Sheets {
    Sheets::parse(
        include_bytes!("../../../../assets/sprites/villager.json"),
        include_bytes!("../../../../assets/sprites/villager_idle_hd.json"),
        include_bytes!("../../../../assets/sprites/resources.json"),
        include_bytes!("../../../../assets/sprites/towncenter.json"),
        include_bytes!("../../../../assets/sprites/buildings_hd.json"),
        [
            include_bytes!("../../../../assets/sprites/buildings_economy.json"),
            include_bytes!("../../../../assets/sprites/buildings_crafts.json"),
            include_bytes!("../../../../assets/sprites/buildings_civic.json"),
            include_bytes!("../../../../assets/sprites/units.json"),
        ],
    )
}

fn unit_sprite(view: &mut WorldView, sheets: &Sheets, time: f32) -> (usize, Sprite) {
    let (sprites, _) = view.draw_list(sheets, &Rig::new(), time, 0.016, &Selection::default());
    let mut units = sprites
        .into_iter()
        .filter(|(sheet, _)| VILLAGER_SHEETS.contains(sheet));
    let sprite = units.next().expect("visible villager");
    assert!(units.next().is_none());
    sprite
}

fn assert_pose(sheets: &Sheets, sprite: Sprite, name: &str, frame: usize) {
    assert!(
        sheets.villager.animations[name].values().any(|frames| {
            [false, true]
                .into_iter()
                .any(|mirror| sprite.uv == uv(frames[frame], sheets.villager.size, mirror))
        }),
        "expected {name} frame {frame}, got {:?}",
        sprite.uv
    );
}

#[test]
fn all_gathering_activities_keep_animating_with_partial_cargo() {
    let sheets = sheets();
    for (kind, pose) in [
        (ResourceKind::Wood, "chop"),
        (ResourceKind::Stone, "mine"),
        (ResourceKind::Gold, "mine"),
        (ResourceKind::Iron, "mine"),
        (ResourceKind::Coal, "mine"),
        (ResourceKind::Clay, "dig"),
        (ResourceKind::Food, "forage"),
        (ResourceKind::Fiber, "forage"),
    ] {
        for amount in [0.0, 1.0, 19.0] {
            for variant in 0..PEOPLE.len() {
                let mut snapshot = GameWorld::default().snapshot();
                snapshot.units.truncate(1);
                snapshot.resources[0].kind = kind;
                snapshot.units[0].unit.action = UnitAction::Gather {
                    resource_id: snapshot.resources[0].id.clone(),
                    phase: GatherPhase::Gathering,
                };
                snapshot.units[0].unit.cargo =
                    (amount > 0.0).then_some(CarriedResource { kind, amount });
                let id = snapshot.units[0].unit.id.clone();
                let mut view = WorldView::new();
                view.sync(snapshot);
                view.units.get_mut(&id).unwrap().variant = variant;
                let (sheet, first) = unit_sprite(&mut view, &sheets, 0.0);
                let (_, second) = unit_sprite(&mut view, &sheets, 1.1 / sheets.villager.fps[pose]);
                assert_eq!(sheet, variant);
                assert_pose(&sheets, first, pose, 0);
                assert_pose(&sheets, second, pose, 1);
                assert_ne!(first.uv, second.uv);
            }
        }
    }
}

#[test]
fn cargo_keeps_the_carry_pose_outside_active_gathering() {
    let sheets = sheets();
    let mut snapshot = GameWorld::default().snapshot();
    snapshot.units.truncate(1);
    let resource_id = snapshot.resources[0].id.clone();
    let building_id = snapshot.buildings[0].building.id.clone();
    for action in [
        UnitAction::Idle,
        UnitAction::Gather {
            resource_id: resource_id.clone(),
            phase: GatherPhase::ToResource,
        },
        UnitAction::Gather {
            resource_id: resource_id.clone(),
            phase: GatherPhase::Returning,
        },
        UnitAction::Gather {
            resource_id: resource_id.clone(),
            phase: GatherPhase::Depositing,
        },
        UnitAction::Deposit {
            building_id: building_id.clone(),
        },
        UnitAction::Build {
            building_id: building_id.clone(),
        },
        UnitAction::Cultivate {
            resource_id: resource_id.clone(),
        },
    ] {
        snapshot.units[0].unit.action = action;
        snapshot.units[0].unit.cargo = Some(CarriedResource {
            kind: ResourceKind::Wood,
            amount: 7.0,
        });
        let id = snapshot.units[0].unit.id.clone();
        let mut view = WorldView::new();
        view.sync(snapshot.clone());
        let (_, stopped) = unit_sprite(&mut view, &sheets, 0.0);
        let (_, later) = unit_sprite(&mut view, &sheets, 0.5);
        assert_pose(&sheets, stopped, "carry", 0);
        assert_eq!(stopped.uv, later.uv);
        let entry = view.units.get_mut(&id).unwrap();
        entry.moving = true;
        entry.walked = STRIDE_DISTANCE * 1.1;
        let (_, walking) = unit_sprite(&mut view, &sheets, 0.5);
        assert_pose(
            &sheets,
            walking,
            "carry",
            sheets.villager.animations["carry"]["front"].len() / 2,
        );
    }
}

#[test]
fn building_and_field_preparation_animate_after_unloading() {
    let sheets = sheets();
    let mut snapshot = GameWorld::default().snapshot();
    snapshot.units.truncate(1);
    for action in [
        UnitAction::Build {
            building_id: snapshot.buildings[0].building.id.clone(),
        },
        UnitAction::Cultivate {
            resource_id: snapshot.resources[0].id.clone(),
        },
    ] {
        snapshot.units[0].unit.action = action;
        snapshot.units[0].unit.cargo = None;
        let mut view = WorldView::new();
        view.sync(snapshot.clone());
        let (_, first) = unit_sprite(&mut view, &sheets, 0.0);
        let (_, second) = unit_sprite(&mut view, &sheets, 1.1 / sheets.villager.fps["build"]);
        assert_pose(&sheets, first, "build", 0);
        assert_pose(&sheets, second, "build", 1);
    }
}
