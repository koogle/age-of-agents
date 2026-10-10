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

Jakob's 2026-10-05 correction supersedes PR #92's construction-only rule: show a building only when discovered resources support both construction and a productive use. Production buildings must have at least one recipe whose inputs can be supplied, including processed-material prerequisites. Non-production buildings retain their housing, storage, gathering or vision utility.

This explicitly authorizes hiding Barracks, Smelter, Kiln and Weaver on island one; Workshop additionally needs steel inputs as well as clay. Range remains useful because its archer recipe uses only food and timber. Keep ten starter buildings, grey out temporary stock shortages, and unlock later choices through resource discovery rather than a hardcoded island number. Discovery remains global across explored islands and persists after depletion; hidden nodes and injected stock do not bypass it. Existing buildings, jobs and saves remain intact; no schema change is needed.

Transport must remain affordable before later-island materials are available. Verify the whole budget, including raw wood consumed to make timber, rather than only the final recipe price.

Read [current decisions](../../decisions.md) for rationale. Deposited stocks belong to individual islands, supplemented by stopped shore ships; costs reserve once from shore then ship cargo, while outputs/refunds remain on the job’s island. Research remains shared. Personal cargo is covered by [tasks and cargo](tasks-and-cargo.md).

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
observe authoritative resource totals. [#64](https://github.com/koogle/age-of-agents/pull/64)
and [#77](https://github.com/koogle/age-of-agents/pull/77) explain why queue identity
and researched-state clarity are separate concerns.

Update this file after economic steering, a changed queue contract, or a newly
discovered failure case. Mark balance proposals as proposed until implemented;
write the useful procedure or invariant here and only its short accepted rationale
in `decisions.md`.


## Shared rule ownership (2026-10-06)

`Building::check_production/check_research/check_queue_space` supply typed
eligibility to both authoritative submission and HUD disabled-state generation.
Progression-filtered snapshot offers remain in `BuildingView`; the world still
revalidates discovery before accepting work. Public `housing/population` helpers
share counting across world and disclosed snapshots. `Stockpile::missing_resource`
owns cost checks; `RESEARCH_COST` and `BuildingJob::cost` own charging/refunds and
cost descriptions. Keep HUD wording/precedence in the client. Buildings derive
products and research from kind; capability removal uses save version 17.
