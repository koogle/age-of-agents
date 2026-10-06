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
   or accidentally cancel its successor. Check current-format save/reload and required queue fields.
5. Update [HUD state](hud-and-accessibility.md) and visible instructions/status
   together. Completed research stays visible as disabled with its effect explained.

## Starter progression and resources

Implemented in [PR #92](https://github.com/koogle/age-of-agents/pull/92), with merge authorized on 2026-10-05: construction availability follows each building's costs, independently of current stock or the production recipes it can run. The user clarified that every building constructible from first-island materials must be available initially; discovery then expands the menu. This replaces the earlier six-building starter restriction.

Fourteen wood/stone/timber buildings are initially available: Town Center, House, Granary, Farm, Lumber Mill, Dock, Watchtower, Mining Camp, Smelter, Kiln, Weaver, Kitchen, Barracks and Range. Unaffordable buildings remain visible in grey. Kitchen can make rations from food; other production still requires actual recipe inputs.

Workshop unlocks with discovered clay for bricks; Infirmary with fiber for cloth; Monument with iron, coal, clay, fiber and gold for all its construction materials. Discovery persists after depletion and cannot be bypassed by hidden nodes or stock quantities. The unlock rules add no state. Save compatibility is separately deferred by Jakob’s 2026-10-05 instruction: incompatible store versions reset, and current saves require explicit economy/queue fields; see [save policy](server-and-saves.md#snapshot-and-save-version-policy).

Existing player-facing features remain important. Removing, hiding, disabling or adding prerequisites to them requires Jakob's explicit approval after a before/after impact review and before release; see [AGENTS.md](../../AGENTS.md#workflow). Starter simplification is not blanket approval for feature losses.

Transport must remain affordable before later-island materials are available. Verify the whole budget, including raw wood consumed to make timber, rather than only the final recipe price.

Read [current decisions](../../decisions.md) and the
[starter economy review](../STARTER_ECONOMY_REVIEW.md) for rationale; the review's
old transport deferrals are explicitly historical. Deposited stocks belong to individual islands, supplemented by stopped shore ships; costs reserve once from shore then ship cargo, while outputs/refunds remain on the job’s island. Research remains shared. Personal cargo is covered by [tasks and cargo](tasks-and-cargo.md).

## Water and fields (2026-10-05)

Water is a basic island-local resource collected at renewable riverbank sources.
Fields reserve 10 water alongside 10 wood and 5 stone once per planting or
replenishment cycle. A preparation helper/resumed order does not pay again.
The source placement, storage and save contract is in the [water guide](water-resource.md).

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
