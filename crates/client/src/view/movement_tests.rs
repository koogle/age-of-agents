//! Exercise actual source/view clocks at render rates, including delayed batches.
use super::*;
use aoa_game::{CellCoordinate, GameWorld, Step};

fn sample(view: &mut WorldView, tick: u64) {
    let mut snapshot = view
        .snapshot
        .clone()
        .unwrap_or_else(|| GameWorld::default().snapshot());
    snapshot.tick = tick;
    snapshot.units.truncate(1);
    snapshot.units[0].position.x = 20.0 + tick as f64 * 0.3;
    snapshot.units[0].position.y = 20.0;
    view.sync(snapshot);
}

fn warm(view: &mut WorldView) {
    sample(view, 0);
    for frame in 1..=600 {
        if frame % 6 == 0 {
            sample(view, frame / 6);
        }
        view.frame(1.0 / 60.0, None);
    }
}

#[test]
fn regular_remote_updates_do_not_modulate_walking_speed() {
    for fps in [30, 60, 120, 144] {
        let mut view = WorldView::new();
        sample(&mut view, 0);
        let mut tick = 0;
        for frame in 1..fps * 5 {
            let next_tick = frame * 10 / fps;
            while tick < next_tick {
                tick += 1;
                sample(&mut view, tick);
            }
            view.frame(1.0 / fps as f32, None);
            if frame > fps {
                let speed = view.units.values().next().unwrap().velocity.x;
                assert!((speed - 1.5).abs() < 0.002, "{fps} FPS: {speed}");
            }
        }
    }
}

#[test]
fn late_batches_keep_the_unplayed_path_without_position_or_clock_jumps() {
    for gap in [3, 6] {
        let mut view = WorldView::new();
        warm(&mut view);
        for _ in 0..gap * 6 {
            view.frame(1.0 / 60.0, None);
        }
        let before = view.units.values().next().unwrap().position;
        let clock = view.render_tick.unwrap();
        for tick in 101..=100 + gap {
            sample(&mut view, tick);
        }
        assert_eq!(
            view.render_tick.unwrap(),
            clock,
            "sync must not advance playback"
        );
        assert_eq!(view.units.values().next().unwrap().position, before);
        assert!(view.units.values().next().unwrap().samples.len() <= MAX_POSITION_SAMPLES);
        for _ in 0..60 {
            view.frame(1.0 / 60.0, None);
            let entry = view.units.values().next().unwrap();
            assert!(entry.velocity.x >= -0.002);
            assert!(entry.velocity.x <= 1.652, "gap {gap}: {}", entry.velocity.x);
        }
    }
}

#[test]
fn starvation_rebuffers_and_pause_freezes_until_resume() {
    let mut view = WorldView::new();
    warm(&mut view);
    for _ in 0..120 {
        view.frame(1.0 / 60.0, None);
    }
    let held = view.render_tick.unwrap();
    sample(&mut view, 101);
    view.frame(1.0 / 60.0, None);
    assert_eq!(view.render_tick.unwrap(), held);
    sample(&mut view, 102);
    view.frame(1.0 / 60.0, None);
    assert!(view.render_tick.unwrap() > held);
    let mut paused = view.snapshot.clone().unwrap();
    paused.simulation_speed = 0.0;
    view.sync(paused.clone());
    let clock = view.render_tick;
    let animation = view.animation_time;
    let entry = view.units.values().next().unwrap();
    let (position, walked) = (entry.position, entry.walked);
    for _ in 0..120 {
        view.frame(1.0 / 60.0, None);
    }
    assert_eq!(view.render_tick, clock);
    assert_eq!(view.animation_time, animation);
    let entry = view.units.values().next().unwrap();
    assert_eq!((entry.position, entry.walked), (position, walked));
    paused.simulation_speed = 1.0;
    view.sync(paused);
    view.frame(1.0 / 60.0, None);
    assert!(view.render_tick > clock);
    assert!(view.animation_time > animation);
}

#[test]
fn native_accumulator_and_view_agree_at_every_frame_and_across_speed_changes() {
    use crate::source::Source;
    let mut source = Source::local(aoa_game::DEFAULT_SEED);
    let mut view = WorldView::new();
    let mut out = VecDeque::new();
    let mut previous = 0.0;
    for frame in 0..240 {
        if frame == 60 || frame == 120 || frame == 180 {
            source.send(aoa_game::Command::SetSimulationSpeed {
                multiplier: if frame == 60 {
                    0.0
                } else if frame == 120 {
                    2.0
                } else {
                    1.0
                },
            });
        }
        source.poll(1.0 / 60.0, &mut out);
        for snapshot in out.drain(..) {
            view.sync(snapshot);
        }
        let target = source.presentation_tick().unwrap();
        view.frame(1.0 / 60.0, Some(target));
        let clock = view.render_tick.unwrap();
        assert!(clock >= previous);
        assert!(clock - previous <= 1.0 / 6.0 + 1e-6);
        assert!(clock <= view.latest_tick);
        if (12..60).contains(&frame) || frame > 132 {
            assert!((clock - target).abs() < 1e-5);
        }
        if (60..120).contains(&frame) {
            assert_eq!(clock, previous, "local pause must hold presentation");
        }
        previous = clock;
    }
}

#[test]
fn clear_turns_face_the_actual_motion_on_the_next_draw() {
    let sheets = activity_tests::sheets();
    let mut view = WorldView::new();
    sample(&mut view, 0);
    sample(&mut view, 1);
    view.frame(0.1, Some(1.0));
    activity_tests::unit_sprite(&mut view, &sheets, 0.0);
    let before = view.units.values().next().unwrap().view;
    let mut reverse = view.snapshot.clone().unwrap();
    reverse.tick = 2;
    reverse.units[0].position.x -= 0.3;
    view.sync(reverse);
    view.frame(0.1, Some(2.0));
    activity_tests::unit_sprite(&mut view, &sheets, 0.1);
    let entry = view.units.values().next().unwrap();
    assert!(entry.velocity.x < 0.0);
    assert_ne!(
        entry.view, before,
        "no half-second facing hold after a reversal"
    );
}

#[test]
fn work_waits_for_arrival_and_never_moves_the_sprite_off_its_claimed_cell() {
    let sheets = activity_tests::sheets();
    let mut snapshot = GameWorld::default().snapshot();
    snapshot.units.truncate(1);
    let resource = snapshot.resources[0].clone();
    let work_cell = CellCoordinate::new(resource.cell.column - 1, resource.cell.row);
    for action in [
        UnitAction::Gather {
            resource_id: resource.id.clone(),
            phase: GatherPhase::Gathering,
        },
        UnitAction::Cultivate {
            resource_id: resource.id.clone(),
        },
        UnitAction::Build {
            building_id: snapshot.buildings[0].building.id.clone(),
        },
    ] {
        let at = if matches!(action, UnitAction::Build { .. }) {
            let origin = snapshot.buildings[0].building.origin;
            CellCoordinate::new(origin.column - 1, origin.row)
        } else {
            work_cell
        };
        snapshot.units[0].unit.action = action;
        snapshot.units[0].unit.cargo = None;
        snapshot.units[0].unit.cell = at;
        snapshot.units[0].unit.step = None;
        snapshot.units[0].position = snapshot.units[0].unit.position();
        let position = terrain::cell_center(at);
        let position = Vec3::new(position.x, 0.0, position.y);
        let unit = &snapshot.units[0].unit;
        assert!(work_target(&snapshot, unit, position).is_some());
        assert!(work_target(&snapshot, unit, position - Vec3::X * 0.1).is_none());
        let mut approaching = unit.clone();
        approaching.cell.column -= 1;
        approaching.step = Some(Step {
            to: at,
            progress: 0.9,
        });
        assert!(work_target(&snapshot, &approaching, position).is_none());
        approaching.step = None; // Blocked while still en route.
        approaching.cell.column -= 3;
        assert!(work_target(&snapshot, &approaching, position).is_none());
        let mut view = WorldView::new();
        view.sync(snapshot.clone());
        for frame in 0..120 {
            let (_, sprite) = activity_tests::unit_sprite(&mut view, &sheets, frame as f32 / 60.0);
            assert_eq!(sprite.anchor[0], position.x);
            assert_eq!(sprite.anchor[2], position.z);
        }
    }
}

#[test]
fn real_gather_order_walks_to_the_work_cell_before_showing_a_work_pose() {
    use crate::source::Source;
    let mut source = Source::local(aoa_game::DEFAULT_SEED);
    let mut out = VecDeque::new();
    source.poll(0.0, &mut out);
    let initial = out.pop_front().unwrap();
    let id = initial.units[0].unit.id.clone();
    let resource = initial.resources[0].clone();
    source.send(aoa_game::Command::Gather {
        unit_id: id.clone(),
        resource_id: resource.id.clone(),
    });
    assert_eq!(source.take_results(), vec![Ok(())]);
    let mut view = WorldView::new();
    view.sync(initial);
    let mut walked = false;
    let mut worked = false;
    for _ in 0..1200 {
        source.poll(1.0 / 60.0, &mut out);
        for snapshot in out.drain(..) {
            view.sync(snapshot);
        }
        view.frame(1.0 / 60.0, source.presentation_tick());
        let entry = &view.units[&id];
        walked |= entry.velocity.length() > WALK_SPEED;
        let snapshot = view.snapshot.as_ref().unwrap();
        let unit = &snapshot
            .units
            .iter()
            .find(|u| u.unit.id == id)
            .unwrap()
            .unit;
        if work_target(snapshot, unit, entry.position).is_some() {
            assert!(walked);
            assert!(unit.is_at_work_site(resource.footprint()));
            assert!(unit.step.is_none());
            let at = terrain::cell_center(unit.cell);
            assert!(Vec2::new(entry.position.x, entry.position.z).distance(at) < 0.001);
            worked = true;
            break;
        }
    }
    assert!(worked, "the real gather route must reach its work site");
}

#[test]
fn carrying_preserves_facing_and_anchor_when_the_authored_strip_faces_the_other_way() {
    let sheets = activity_tests::sheets();
    for variant in 0..3 {
        let mut view = WorldView::new();
        sample(&mut view, 0);
        let entry = view.units.values_mut().next().unwrap();
        entry.variant = variant;
        entry.moving = true;
        entry.velocity = Vec3::X;
        let (_, walking) = activity_tests::unit_sprite(&mut view, &sheets, 0.0);
        view.snapshot.as_mut().unwrap().units[0].unit.cargo = Some(aoa_game::CarriedResource {
            kind: ResourceKind::Wood,
            amount: 1.0,
        });
        let (_, carrying) = activity_tests::unit_sprite(&mut view, &sheets, 0.0);
        // The base/woman's front sack strips face right in the source art;
        // their walking strips face left. The elder's strips both face left.
        let walk_mirrored = walking.uv[0] > walking.uv[2];
        let carry_mirrored = carrying.uv[0] > carrying.uv[2];
        assert_eq!(walk_mirrored != carry_mirrored, variant < 2);
        assert_eq!(walking.anchor, carrying.anchor);
    }
}

#[test]
fn reconnect_restores_recency_in_one_frame_without_animating_the_correction() {
    for gap in [9, 15, 300, 3000] {
        for batch in [false, true] {
            let mut view = WorldView::new();
            warm(&mut view);
            for _ in 0..60 {
                view.frame(1.0 / 60.0, None);
            }
            let walked = view.units.values().next().unwrap().walked;
            if batch {
                for tick in 101..100 + gap {
                    sample(&mut view, tick);
                }
            }
            sample(&mut view, 100 + gap);
            view.frame(1.0 / 60.0, None);
            let entry = view.units.values().next().unwrap();
            assert_eq!(
                view.render_tick,
                Some(view.latest_tick),
                "gap={gap}, batch={batch}: stale replay"
            );
            assert_eq!(entry.velocity, Vec3::ZERO, "resync is not walking");
            assert_eq!(entry.walked, walked, "resync must not advance gait");
            assert_eq!(entry.samples.len(), 1);
            for frame in 1..=600 {
                if frame % 6 == 0 {
                    sample(&mut view, 100 + gap + frame / 6);
                }
                view.frame(1.0 / 60.0, None);
                assert!(view.latest_tick - view.render_tick.unwrap() <= 2.01);
            }
        }
    }
}

#[test]
fn reconnect_rebases_even_short_gaps_and_paused_worlds_without_animating_history() {
    for speed in [0.0, 1.0] {
        for gap in [2, 50] {
            let mut view = WorldView::new();
            warm(&mut view);
            let walked = view.units.values().next().unwrap().walked;
            let mut next = view.snapshot.clone().unwrap();
            next.tick += gap;
            next.simulation_speed = speed;
            next.units[0].position.x += 4.0;
            assert!(!view.sync(next), "reconnect does not reset the world");
            let newest = view
                .units
                .values()
                .next()
                .unwrap()
                .samples
                .back()
                .unwrap()
                .1;
            view.reset_playback();
            assert_eq!(view.render_tick, Some(view.latest_tick));
            let entry = view.units.values().next().unwrap();
            assert_eq!(entry.position, newest);
            assert_eq!(entry.velocity, Vec3::ZERO);
            assert_eq!(entry.walked, walked);
            assert!(!entry.moving);
            assert_eq!(entry.samples.len(), 1);
            for _ in 0..60 {
                view.frame(1.0 / 60.0, None);
                let entry = view.units.values().next().unwrap();
                assert_eq!(entry.position, newest);
                assert_eq!(entry.walked, walked);
            }
        }
    }
}

#[test]
fn a_paused_snapshot_after_a_long_stream_gap_establishes_current_position() {
    let mut view = WorldView::new();
    warm(&mut view);
    let animation = view.animation_time;
    let walked = view.units.values().next().unwrap().walked;
    sample(&mut view, 150);
    let mut paused = view.snapshot.clone().unwrap();
    paused.simulation_speed = 0.0;
    view.sync(paused);
    let expected = view
        .units
        .values()
        .next()
        .unwrap()
        .samples
        .back()
        .unwrap()
        .1;
    for _ in 0..60 {
        view.frame(1.0 / 60.0, None);
        let entry = view.units.values().next().unwrap();
        assert_eq!(entry.position, expected);
        assert_eq!(entry.walked, walked);
        assert_eq!(view.animation_time, animation);
    }
}
