# Runtime diagnosis

Read before: Before fixing stuck units, frozen worlds, delayed input, or mismatched visible state.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with the running game

First identify the world source and use the [server/saves guide](server-and-saves.md)
for protocol and isolated database setup. Capture the reported sequence of actions,
seed if known, current task/cargo, and the revision actually served.

On a local hosted test server, inspect without changing the world:

```bash
curl -fsS http://localhost:8000/state
```

This returns a filtered `WorldSnapshot`, not the full persisted `GameWorld`; do
not use it as a raw SQLite fixture. Compare samples of `tick` and
`simulation_speed`, then inspect command results and the WebSocket snapshot stream.
See [verification](verification-and-handoffs.md) for a controlled presentation replay
when the problem is visual. Store a minimal reproducer and the conditions that
make it fail, not a full command transcript.

## Pause contract

Jakob reported orders and NPC movement during pause on 2026-10-05. At 0×,
`GameWorld::apply_command` rejects gameplay orders atomically with `GamePaused`;
speed controls remain available. Existing tasks resume when speed returns to 1×
or 2×. Selection and camera controls remain available. Reset remains a separate,
explicit world-management action. `WorldView::frame` holds
interpolation, gait distance and work-animation time while paused, including
unplayed remote movement; playback resumes from that held position.

Regression entry points: `paused_orders_are_rejected_without_changing_tasks_or_spending_resources`
in domain tests, `starvation_rebuffers_and_pause_freezes_until_resume` and
`native_accumulator_and_view_agree_at_every_frame_and_across_speed_changes` in
client movement tests, and `paused_gathering_holds_the_rendered_frame_and_resumes`
in activity tests. Check both the authoritative snapshot and rendered frame:
previously ticks stopped correctly while commands mutated tasks and the client
continued buffered motion and wall-clock work animations.

Verified 2026-10-05 on an isolated seed-123 hosted world using rebuilt WASM:
Chromium desktop mouse (1280×800, DPR1) and emulated phone touch (390×844, DPR2)
can pause/resume through the speed controls. While paused, WebSocket move, stop
and production orders return the pause error and the entire `/state` snapshot
remains unchanged; after resuming, ticks advance and orders succeed. This does
not establish physical-phone or native-window appearance.

## Reconnect recovery

Jakob reported strange web catch-up after reconnecting on 2026-10-05. Inspection
found that reconnect resets only the message sequence, preserving old playback
history, and the remote inbox queues snapshots without a bound. Reconnect and
excessive queued backlog now establish the newest snapshot as a fresh playback
baseline, including paused worlds, without replaying missed movement or feedback.
Ordinary short network delays retain interpolation; camera and selection survive
a reconnect to the same world. `crates/client/src/source/inbox.rs` keeps at most eight pending
distinct-tick snapshots, coalesces same-tick broadcasts, and switches to only the
newest snapshot until the renderer consumes a fresh baseline. Reconnect accepts
a restarted sequence (including zero), clears old deliveries and signals
`WorldView::reset_playback`; it never resends gameplay commands. The app clears
old activity feedback and does not announce transitions that happened offline.
A long stream gap can also correct position while paused before freezing again.

The inbox and movement tests cover sequence restarts, short/long gaps, paused
recovery, bounded backlog and uninterrupted short-delay interpolation. Run
`python3 docs/verification/replay_reconnect.py --output /tmp/aoa-reconnect-check`
after rebuilding WASM for real browser socket closure and suspended-rendering
checks. Verified 2026-10-05 in Chromium at 960×640/DPR1 and emulated phone
390×844/DPR2: sequence-zero reconnect displays the fresh paused position and
remains still; a 40-snapshot burst displays only its final position. The controlled
fixture measures sprite uploads and is not a gameplay or production test.

## Learned constraints and evidence

**Evidence:** [#32](https://github.com/koogle/age-of-agents/pull/32) found a paused
world; [#35](https://github.com/koogle/age-of-agents/pull/35) found disconnected
clients and stale caches after deployments;
[#40](https://github.com/koogle/age-of-agents/pull/40) found healthy snapshots but
expensive terrain rebuilds. [#44](https://github.com/koogle/age-of-agents/pull/44)
and [#80](https://github.com/koogle/age-of-agents/pull/80) were actual routing faults.

**Diagnostic order:**

1. Establish mode (hosted/native/browser-local), revision/bundle and reported
   seed/layout. Inspect authoritative tick, speed, order, cargo and queue state;
   do not assume the production world matches a local fixture.
2. Check WebSocket connection, advancing snapshot sequence, command rejection
   or acknowledgement, and current assets. Preserve reconnect sequence reset
   and the server's no-cache policy.
3. If state advances but the picture lags, measure rendering/upload time and
   presentation interpolation. Reproduce with controlled snapshots when useful.
4. If the authoritative task stalls, distinguish temporary reservations from
   missing drop sites, disconnected terrain, blocked spawns or exhausted nodes.
5. Reproduce in an isolated save. Without the reported save, describe a reproduced
   cause rather than claiming it is certainly the user's exact incident.

Never reset or resume a shared production world merely to investigate. Native
and browser-local `Source::Local` are in-memory simulations; hosted SQLite saves
are a separate path. Sources: `src/main.rs`, `crates/client/src/source.rs`,
[field audit limits](../FIELD_GATHERING_REVIEW.md#limits).

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).
