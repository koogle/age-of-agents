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

This explicitly authorizes hiding Barracks, Smelter, Kiln and Weaver on island one; Workshop needs steel inputs, while its material-tier base construction uses stone instead of bricks. Range remains useful because its archer recipe uses only food and timber. Keep ten starter buildings, grey out temporary stock shortages, and unlock later choices through resource discovery rather than a hardcoded island number. Discovery remains global across explored islands and persists after depletion; hidden nodes and injected stock do not bypass it. Existing buildings, jobs and saves remain intact; no schema change is needed.

Transport must remain affordable before later-island materials are available. Verify the whole budget, including raw wood consumed to make timber, rather than only the final recipe price.

Read [current decisions](../../decisions.md) and the
[starter economy review](../STARTER_ECONOMY_REVIEW.md) for rationale; the review's
old transport deferrals are explicitly historical. Deposited stocks belong to individual islands, supplemented by stopped shore ships; costs reserve once from shore then ship cargo, while outputs/refunds remain on the job’s island. Research remains shared. Personal cargo is covered by [tasks and cargo](tasks-and-cargo.md).

## Material tiers

Jakob requested clay as a second-island resource, wood or thatched roofs for the
base buildings, and clay/brick upgraded versions while retaining the existing
building catalog. Revised working default stated in chat: add clay alongside
iron/coal on island 2, retain clay on island 3 and fiber on island 4 (the destination
pattern repeats), and upgrade in place. Moving metals to island 3 was rejected
before release because it would delay existing steel recipes without explicit
approval. The unanswered clarification does not authorize a feature delay.

All base construction is clay/brick-free: Workshop replaces its 15 bricks with
15 stone, while steel recipe inputs still gate its productive availability; Monument replaces 30 bricks with 30 stone
and keeps its other prerequisites. No current building function is removed.
The starter kiln remains buildable from wood/stone and processes 10 clay + 5 wood
into 5 bricks, avoiding a circular requirement for the first upgrade.

`UpgradeBuilding` is a typed domain command using the existing paid task queue.
Like every gameplay order, upgrades require resuming a paused world first.
After clay discovery it charges the building's brick/timber upgrade recipe once;
20 seconds of active queue work sets its `masonry` flag. A queued upgrade refunds
fully on cancellation to the building’s island. Payment uses its shore stores and stopped connected ship holds, through the same `spend_at` boundary as other jobs. Duplicate pending/completed upgrades are rejected. The
building keeps its ID, plot, products, drop-off and housing behavior. Upgrades
unlock optional specialist research as described below; they add no automatic stat bonuses. Costs are initial
balance values (10/20/30 bricks and 5/10/15 timber by building size).

Snapshots expose authoritative upgrade availability; hidden clay or injected stock
cannot bypass discovery. The UI shows prerequisites, price, progress, waiting
cancellation and completed state. Save version 17 resets incompatible worlds per
the existing save policy; no migration or deserialization default is added.

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

Verification and reproducible menu captures: [productive building visibility](../verification/building-progression/README.md).

## Brick upgrades unlock optional research (2026-10-06)

Jakob requested research unlocks from brick upgrades, at least for noncritical
buildings such as the Farm. Basic food gathering, field preparation, production,
brick-making and first departure must remain possible without upgraded buildings.
Agriculture's +20% gathering bonus is optional; its brick requirement must be
validated by the domain and explained by a disabled research control in the HUD.
The Town Center offers only Masonry, also available at the base Kiln. Agriculture
belongs to the Farm, Forestry to the Lumber Mill, Mining to the Mining Camp and
Textiles to the Weaver. These four existing optional +20% bonuses require that
specific building's completed masonry upgrade; a queued or active upgrade is not
enough. Existing technology prerequisites and global uniqueness still apply.
This is the working scope stated in chat after the optional clarification; no
new research effects or additional production gates are introduced.

The disabled research control remains visible with “Requires brick upgrade” and
its bonus. Completed research remains visible as complete. Locked direct commands
reject before spending. Validation rejects locked saved research jobs; schema 17
resets older catalog/job saves under the existing no-compatibility policy.
`BuildingKind::technologies` owns the available research catalog; snapshots and
commands both derive from it, preventing old stored offers from bypassing the rule.
