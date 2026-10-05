# Movement frame validation

Run `cargo test -p aoa-client --locked`. For the randomized scenario and its coverage count:

```
cargo test -p aoa-client seeded_npc_routes --locked -- --nocapture
```

The scenario uses fixed seeds 7, 19 and 41 for reproducible worlds and command choices. Every three simulated seconds it samples villagers without replacement, chooses destinations near their actual source cells, and sends real `Move` commands. Invalid destinations are retried; villagers are never teleported into a test route. Commands can replace an unfinished route. The final routes are allowed to settle.

Recorded authoritative trajectories are replayed through `WorldView::sync`, `frame` and `draw_list` at 30, 60 and 144 FPS under native accumulator timing, steady remote delivery, batched remote delivery and a three-second disconnect with only the newest snapshot available on reconnection. Every rendered NPC frame checks:

- Position against an independently interpolated authoritative trajectory, not the view's sample buffer.
- Monotonic time, bounded normal advancement, and immediate recovery to current time after a long outage.
- Actual sprite anchors against the expected body position.
- Actual walking UVs against independently accumulated distance and all four authored gait phases.
- Coverage of both moving and stopped villagers; failure messages identify seed, FPS, transport, frame and NPC.

## Regressions that must remain detectable

| Failure | Guard |
| --- | --- |
| Speed pulses against regularly arriving ticks | `regular_remote_updates_do_not_modulate_walking_speed` |
| Unplayed history discarded during delayed batches | `late_batches_keep_the_unplayed_path_without_position_or_clock_jumps` |
| Two authored gait poses skipped | `seeded_npc_routes_are_correct_on_every_rendered_frame` and gait unit tests |
| Old facing persists after reversal | `clear_turns_face_the_actual_motion_on_the_next_draw` |
| Sprite slides into a work pose or works before arrival | `work_waits_for_arrival_and_never_moves_the_sprite_off_its_claimed_cell` and the real gather-route test |
| Reconnection replays stale movement for minutes | `reconnect_restores_recency_in_one_frame_without_animating_the_correction` |

The recovery regression covers sparse and fully batched gaps of 0.9, 1.5, 30 and 300 seconds, then checks ten seconds of subsequent frames. Native accumulator pause/resume and 2× speed changes, remote starvation and pause draining, and carry pose registration have separate focused tests.

## Recovery contract and limits

Regular movement and short packet gaps interpolate without clock resets. If remote playback falls more than eight ticks (800 ms) behind the latest received snapshot, it resynchronizes once to that snapshot, discards obsolete history and rebuilds the two-tick buffer. This is a deliberate visible position correction after an outage; it does not create walking velocity or advance gait distance. Tests must not mistake this recovery for ordinary smooth movement, or permit arbitrary jumps outside it.

Work animation requires the existing authoritative adjacent interaction cell; no cosmetic offset or new sub-cell destination is introduced. These tests validate geometry and pose selection, not whether every authored tool pixel touches its target or whether a zigzag route looks artistically ideal. Desktop/native/phone visual inspection remains necessary. Actions, cargo and resources still use the newest snapshot; this suite does not claim full historical-state playback.

## Fault-injection check (2026-10-05)

All six guards above were verified by temporarily restoring a representative fault in production presentation code: the old lag-proportional playback rate, six-sample front eviction, two-pose gait selection, held facing, a work-position offset, and disabled long-gap resynchronization. Each targeted test failed with the fault present. The fixes were restored before final verification. This checks that the suite can detect the failures, rather than merely asserting that the current implementation runs. It is not a claim that the entire historical application was replayed.
