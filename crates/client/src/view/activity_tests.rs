//! Regressions for rendered work frames after the first item enters the load.
use super::*;
use aoa_game::{CarriedResource, GameWorld};

pub(super) fn sheets() -> Sheets {
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
            include_bytes!("../../../../assets/sprites/buildings_sanctuary.json"),
            include_bytes!("../../../../assets/sprites/units.json"),
        ],
        include_bytes!("../../../../assets/sprites/villager_field_preparation.json"),
    )
}

pub(super) fn unit_sprite(view: &mut WorldView, sheets: &Sheets, time: f32) -> (usize, Sprite) {
    view.animation_time = time;
    let (sprites, _) = view.draw_list(sheets, &Rig::new(), &Selection::default());
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
                park_beside_work(&mut snapshot);
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
fn paused_gathering_holds_the_rendered_frame_and_resumes() {
    let sheets = sheets();
    let mut snapshot = GameWorld::default().snapshot();
    snapshot.units.truncate(1);
    snapshot.resources[0].kind = ResourceKind::Wood;
    snapshot.units[0].unit.action = UnitAction::Gather {
        resource_id: snapshot.resources[0].id.clone(),
        phase: GatherPhase::Gathering,
    };
    park_beside_work(&mut snapshot);
    let mut view = WorldView::new();
    view.sync(snapshot.clone());
    let draw = |view: &mut WorldView| {
        view.draw_list(&sheets, &Rig::new(), &Selection::default())
            .0
            .into_iter()
            .find(|(sheet, _)| VILLAGER_SHEETS.contains(sheet))
            .unwrap()
            .1
            .uv
    };
    let before = draw(&mut view);
    snapshot.simulation_speed = 0.0;
    view.sync(snapshot.clone());
    for local in [None, Some(0.0)] {
        for _ in 0..60 {
            view.frame(1.0 / 60.0, local);
            assert_eq!(draw(&mut view), before);
        }
    }
    snapshot.simulation_speed = 1.0;
    view.sync(snapshot);
    view.frame(1.1 / sheets.villager.fps["chop"], None);
    assert_ne!(draw(&mut view), before);
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
            storage_id: building_id.clone(),
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
        entry.walked = STRIDE_DISTANCE * 0.3;
        let (_, walking) = unit_sprite(&mut view, &sheets, 0.5);
        assert_pose(&sheets, walking, "carry", 1);
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
        let preparing = matches!(action, UnitAction::Cultivate { .. });
        snapshot.units[0].unit.action = action;
        snapshot.units[0].unit.cargo = None;
        let mut view = WorldView::new();
        park_beside_work(&mut snapshot);
        view.sync(snapshot.clone());
        let (sheet, first) = unit_sprite(&mut view, &sheets, 0.0);
        let fps = if preparing {
            4.0
        } else {
            sheets.villager.fps["build"]
        };
        let (_, second) = unit_sprite(&mut view, &sheets, 1.1 / fps);
        if preparing {
            assert_eq!(sheet, SHEET_FIELD_PREPARATION);
            let variant = view.units[&snapshot.units[0].unit.id].variant;
            let frames = &sheets.field_preparation.people[PEOPLE[variant]]["front"];
            for (sprite, frame) in [(first, 0), (second, 1)] {
                assert!(
                    [false, true].into_iter().any(|mirror| sprite.uv
                        == uv(frames[frame], sheets.field_preparation.size, mirror))
                );
            }
        } else {
            assert_pose(&sheets, first, "build", 0);
            assert_pose(&sheets, second, "build", 1);
        }
        assert_ne!(first.uv, second.uv);
    }
}

fn park_beside_work(snapshot: &mut WorldSnapshot) {
    let footprint = match &snapshot.units[0].unit.action {
        UnitAction::Build { building_id } => snapshot
            .buildings
            .iter()
            .find(|b| &b.building.id == building_id)
            .unwrap()
            .building
            .footprint(),
        UnitAction::Gather { resource_id, .. } | UnitAction::Cultivate { resource_id } => snapshot
            .resources
            .iter()
            .find(|r| &r.id == resource_id)
            .unwrap()
            .footprint(),
        _ => unreachable!(),
    };
    let cell = aoa_game::CellCoordinate::new(footprint.origin.column - 1, footprint.origin.row);
    snapshot.units[0].unit.cell = cell;
    snapshot.units[0].unit.step = None;
    snapshot.units[0].position = snapshot.units[0].unit.position();
}

#[test]
fn paused_wildlife_holds_its_rendered_stride_until_resumed() {
    let sheets = sheets();
    let mut snapshot = GameWorld::default().snapshot();
    let cell = snapshot.units[0].unit.cell;
    snapshot.animals = vec![aoa_game::Animal {
        id: "pause-wolf".into(),
        kind: aoa_game::AnimalKind::Wolf,
        home: cell,
        cell,
        step: Some(aoa_game::Step {
            to: aoa_game::CellCoordinate::new(cell.column + 1, cell.row),
            progress: 0.5,
        }),
        health: 40.0,
        attack_seconds: 0.0,
        heading: [1, 0],
    }];
    snapshot.simulation_speed = 0.0;
    let mut view = WorldView::new();
    view.sync(snapshot.clone());
    let sprite = |view: &mut WorldView| {
        view.draw_list(&sheets, &Rig::new(), &Selection::default())
            .0
            .into_iter()
            .find(|(sheet, _)| *sheet == 13)
            .unwrap()
            .1
    };
    let before = sprite(&mut view);
    for _ in 0..90 {
        view.frame(1.0 / 60.0, None);
        let held = sprite(&mut view);
        assert_eq!(held.uv, before.uv);
        assert_eq!(held.anchor, before.anchor);
    }
    snapshot.simulation_speed = 1.0;
    view.sync(snapshot);
    for _ in 0..15 {
        view.frame(1.0 / 60.0, None);
    }
    assert_ne!(sprite(&mut view).uv, before.uv);
}
