//! Seeded real-domain routes replayed through the production presentation/draw path.
//! Failure messages identify seed, rate, transport, frame and NPC for reproduction.
use super::*;
use aoa_game::{CellCoordinate, Command, GameWorld, WorldSnapshot};

struct Random(u64);
impl Random {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as usize % n
    }
}

fn routes(seed: u64) -> Vec<WorldSnapshot> {
    let mut rng = Random(seed);
    let mut world = GameWorld::generate(seed);
    let mut accepted = 0;
    let mut result = vec![world.snapshot()];
    for tick in 0..180 {
        // Sample NPCs without replacement. Their current authoritative cell is
        // the source; rejected random destinations are retried, never teleported.
        if tick % 30 == 0 {
            let mut ids: Vec<_> = (0..world.units.len()).collect();
            let count = 1 + rng.below(ids.len());
            for _ in 0..count {
                let index = ids.swap_remove(rng.below(ids.len()));
                let id = world.units[index].id.clone();
                let from = world.units[index].cell;
                for _ in 0..80 {
                    let to = CellCoordinate::new(
                        (i32::from(from.column) + rng.below(17) as i32 - 8).clamp(0, 119) as u16,
                        (i32::from(from.row) + rng.below(17) as i32 - 8).clamp(0, 79) as u16,
                    );
                    if to != from
                        && world
                            .apply_command(Command::Move {
                                unit_id: id.clone(),
                                to,
                            })
                            .is_ok()
                    {
                        accepted += 1;
                        break;
                    }
                }
            }
        }
        world.tick(0.1);
        world
            .validate()
            .expect("random route must preserve domain invariants");
        result.push(world.snapshot());
    }
    assert!(
        accepted >= 6,
        "seed={seed}: insufficient accepted movement commands ({accepted})"
    );
    // Allow the last destinations to finish so stopping/arrival is exercised too.
    for _ in 0..300 {
        world.tick(0.1);
        result.push(world.snapshot());
    }
    assert!(
        world.units.iter().all(|u| u.step.is_none()),
        "seed={seed}: route did not finish"
    );
    result
}

#[test]
fn seeded_npc_routes_are_correct_on_every_rendered_frame() {
    let sheets = activity_tests::sheets();
    let rig = Rig::new();
    let selection = Selection::default();
    let mut checked = 0;
    let mut moving = 0;
    let mut stopped = 0;
    let mut gait_frames = [false; 4];
    for seed in [7, 19, 41] {
        let history = routes(seed);
        for fps in [30, 60, 144] {
            // Native accumulator, steady remote, jittered batches, long outage.
            for transport in 0..4 {
                let mut view = WorldView::new();
                view.sync(history[0].clone());
                let mut delivered = 0;
                let mut distance = HashMap::<String, f32>::new();
                let mut previous = HashMap::<String, Vec2>::new();
                for frame in 1..fps * 48 {
                    let latest = (frame * 10 / fps).min(history.len() - 1);
                    let release = match transport {
                        2 => frame % (fps / 5) == 0, // 200 ms batches
                        3 => !(fps * 4..fps * 7).contains(&frame),
                        _ => true,
                    };
                    if release {
                        // Outage reconnect has only the newest snapshot. Other
                        // schedules retain every sample, as queued WS messages do.
                        if transport == 3 && latest > delivered + 8 {
                            view.sync(history[latest].clone());
                        } else {
                            for snapshot in &history[delivered + 1..=latest] {
                                view.sync(snapshot.clone());
                            }
                        }
                        delivered = latest;
                    }
                    let before = view.render_tick.unwrap();
                    let resync = transport != 0 && delivered as f64 - before > 8.0;
                    let local = (transport == 0)
                        .then_some((frame as f64 * 10.0 / fps as f64 - 1.0).max(0.0));
                    view.frame(1.0 / fps as f32, local);
                    let time = view.render_tick.unwrap();
                    let context =
                        format!("seed={seed} fps={fps} transport={transport} frame={frame}");
                    assert!(
                        time >= before && time <= delivered as f64,
                        "{context}: invalid clock"
                    );
                    if resync {
                        assert_eq!(time, delivered as f64, "{context}: stale reconnection");
                    } else {
                        assert!(
                            time - before <= 11.0 / fps as f64 + 1e-5,
                            "{context}: clock jump"
                        );
                    }
                    // Independent oracle: interpolate recorded authoritative
                    // coordinates, not the view's sample buffer or sample_at().
                    let lo = time.floor() as usize;
                    let hi = time.ceil() as usize;
                    let alpha = (time - lo as f64) as f32;
                    let (sprites, _) =
                        view.draw_list(&sheets, &rig, frame as f32 / fps as f32, &selection);
                    let bodies: Vec<_> = sprites
                        .iter()
                        .filter(|(sheet, _)| VILLAGER_SHEETS.contains(sheet))
                        .collect();
                    assert_eq!(
                        bodies.len(),
                        history[0].units.len(),
                        "{context}: missing body"
                    );
                    for (i, u) in history[lo].units.iter().enumerate() {
                        let id = &u.unit.id;
                        let b = &history[hi].units[i].position;
                        let expected = Vec2::new(
                            (u.position.x as f32 * (1.0 - alpha) + b.x as f32 * alpha)
                                * terrain::CELL,
                            (u.position.y as f32 * (1.0 - alpha) + b.y as f32 * alpha)
                                * terrain::CELL,
                        );
                        let entry = &view.units[id];
                        let at = Vec2::new(entry.position.x, entry.position.z);
                        assert!(
                            at.distance(expected) < 0.0001,
                            "{context} npc={id}: off authoritative route {at:?} != {expected:?}"
                        );
                        let sprite = &bodies[i].1;
                        assert!(
                            Vec2::new(sprite.anchor[0], sprite.anchor[2]).distance(at) < 0.0001,
                            "{context} npc={id}: sprite offset"
                        );
                        let traveled = distance.entry(id.clone()).or_default();
                        if let Some(old) = previous.insert(id.clone(), at) {
                            if !resync {
                                *traveled += at.distance(old);
                            }
                        }
                        if entry.moving {
                            moving += 1;
                            let phase = ((*traveled / 0.15).floor() as usize) % 4;
                            gait_frames[phase] = true;
                            // Every authored quarter-cycle must actually reach the GPU sprite.
                            let animation = &sheets.villager.animations["walk"];
                            assert!(
                                animation.values().any(|frames| [false, true]
                                    .into_iter()
                                    .any(|mirror| sprite.uv
                                        == uv(frames[phase], sheets.villager.size, mirror))),
                                "{context} npc={id}: wrong gait frame {phase}"
                            );
                        } else {
                            stopped += 1;
                        }
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(moving > 1000 && stopped > 1000 && gait_frames.into_iter().all(|seen| seen));
    println!(
        "seeded routes: {checked} NPC frames checked; {moving} moving, {stopped} stopped; all 4 gait phases"
    );
}
