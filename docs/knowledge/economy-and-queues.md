# Economy, research and building queues

Read before: changing production recipes, research, resource gates, housing,
cancellation or queue UI. Maintained guide; source-reviewed 2026-10-05 against
`b054655`. Start in [economy.rs](../../crates/game/src/game/economy.rs),
[progression.rs](../../crates/game/src/game/progression.rs),
[domain.rs](../../crates/game/src/game/domain.rs) and
[queue tests](../../crates/game/src/game/queue_tests.rs).

## How to make a change

1. Trace the existing recipe/technology and the typed `Produce`, `Research` or
   `CancelQueuedJob` command in [game.rs](../../crates/game/src/game.rs). Preserve the
   authoritative offer and validation path; hiding a HUD coin alone is not a gate.
2. Define the new behavior across available, waiting, active, blocked, cancelled
   and completed states. Payment happens on submission; waiting cancellation
   refunds once. Active work is not cancelled through waiting-task cancellation.
3. Account for both active and waiting trainees in housing. A blocked output
   keeps later queued work waiting; do not create units twice on repeated ticks.
4. Preserve stable cancellation IDs: an old click cannot target a promoted task
   or accidentally cancel its successor. Check save/reload and legacy empty queues.
5. Update [HUD state](hud-and-accessibility.md) and visible instructions/status
   together. Completed research stays visible as disabled with its effect explained.

## Starter progression and resources

Fresh games begin with starter inputs; old saves can remain unrestricted. Unlocks
derive from discovered deposits, persist after depletion, and cannot be bypassed
by hidden nodes or a large stockpile. Transport must be affordable before the
player can obtain later-island materials. Verify the whole budget including
processed material inputs rather than checking only the final recipe price.

Read [current decisions](../../decisions.md) and the
[starter economy review](../STARTER_ECONOMY_REVIEW.md) for rationale; the review's
old transport deferrals are explicitly historical. Deposited stocks are shared
across islands; personal cargo is covered by [tasks and cargo](tasks-and-cargo.md).

## Checks and reusable learning

```bash
cargo test -p aoa-game --locked queue_tests
cargo test -p aoa-game --locked economy_tests
cargo test -p aoa-game --locked progression_tests
```

Cover a full queue, exact costs/refunds, stale cancellation, blocked spawning,
duplicate research across buildings, insufficient inputs, housing release and
reload. On desktop and phone, use actual controls to fill and cancel a queue and
observe authoritative resource totals. [Queue review](../BUILDING_QUEUES_REVIEW.md)
records the implementation evidence; [#64](https://github.com/koogle/age-of-agents/pull/64)
and [#77](https://github.com/koogle/age-of-agents/pull/77) explain why queue identity
and researched-state clarity are separate concerns.

Update this file after economic steering, a changed queue contract, or a newly
discovered failure case. Mark balance proposals as proposed until implemented;
write the useful procedure or invariant here and only its short accepted rationale
in `decisions.md`.
