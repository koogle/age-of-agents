# Villager tasks and cargo

Read before: Before changing gathering, reassignment, drop-offs, field work, or activity animation.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with this system

Read [gathering.rs](../../crates/game/src/game/gathering.rs),
[fields.rs](../../crates/game/src/game/fields.rs), command application in
[game.rs](../../crates/game/src/game.rs), and cargo acceptance in
[domain.rs](../../crates/game/src/game/domain.rs). The client does not choose work.

1. Identify the existing order, phase, carried kind/amount and intended next task.
2. Implement transitions through the existing domain command/tick paths; retain
   atomic rejection and distinguish a new order from continued gathering.
3. Follow the full cycle across travel, work, delivery and resumption. Treat a
   temporary reservation as a different condition from permanently blocked land.
4. Check [activity presentation](../../crates/client/src/view/activity_tests.rs)
   and selection text if the visible meaning of a phase changes.

```bash
cargo test -p aoa-game --locked gathering_tests
cargo test -p aoa-game --locked fields_tests
cargo test -p aoa-client --locked activity_tests
```

PR #88 is now merged (integration checked at `0a863a4`). Existing harvesters retain
their assigned field while paid replenishment is underway; Stop and reassignment
still take priority. Idle workers are not recruited and exhausted fields are not
automatically replanted. The regression cases include reload, partial cargo and
both worker update orders.

## Learned constraints and evidence

**Evidence:** [#31](https://github.com/koogle/age-of-agents/pull/31) allowed
same-kind partial loads to continue and builders to carry goods without a drop
site. [#47](https://github.com/koogle/age-of-agents/pull/47) tightened that behavior
to unload before a new assignment, including partial same-kind loads.
[#44](https://github.com/koogle/age-of-agents/pull/44) fixed a blocked nearby
drop-off hiding a reachable alternative;
[#85](https://github.com/koogle/age-of-agents/pull/85) added wood delivery to mills.
[The activity regression](../ACTIVITY_SPRITES_REVIEW.md) shows why “has cargo” cannot
alone decide whether a villager is working or carrying.

**Lesson:** Distinguish continuing an existing gather order from issuing a new
one. New gather/build/cultivate work unloads first, retains its intended task,
and waits if no compatible completed drop-off is available. Temporary traffic
must not be confused with a permanently unreachable target. Animation and HUD
copy describe the authoritative phase: active gathering with partial cargo
still works; stopped or unloading cargo still uses the carry pose.

**Check:** Exercise empty/partial/full cargo, same/different resource kinds,
busy/incompatible/unfinished drop sites, Stop, replacement, depletion and reload.
Start with `gathering_tests.rs` and
`crates/client/src/view/activity_tests.rs`; use a complete delivery/resumption
cycle rather than treating one successful command as proof of the loop.

## Automatic continuation at a patch boundary

The developer reported regular-bush gatherers going Idle after automatic delivery,
with nearby food remaining and no further clicks, especially at 2× (2026-10-05).
The developer also suspects any resource with a distant drop-off is affected.
Coverage includes food, wood and stone with both nearby and distant
storage, including deliveries while the original node still has stock.
A visible-production-layout replay reproduced this when `berries-34` was the last
node harvested: its 10-cell circle excluded a neighboring patch, although other
bushes in the connected exhausted patch were close enough. Starting on another
bush could harvest both patches. Earlier field-exhaustion and direct-deposit
observations did not explain this automatic failure.

`next_resource` now measures distance from the connected exhausted wild-resource
patch: adjacent cells of the same kind, excluding fields. It keeps the 10-cell
limit, nearest-distance/ID ordering and reachable-target check. Disconnected
exhausted patches cannot extend the range, fields retain their individual search
origin, and Stop/manual replenishment remain unchanged. No persisted fields change.

`exhausted_patch_continuation_survives_distant_deliveries`
fails before this change and passes at 1×/2× afterward. It fills one load across
two bushes, resumes at the neighboring patch after delivery, credits all 60 food,
and leaves distant resources alone. It covers food, wood and stone with nearby
and distant storage. `live_resource_assignment_survives_repeated_distant_deliveries`
checks three full deliveries while the original node remains live at 1×/2×;
drop-off distance alone did not reproduce abandonment. Generated-world coverage uses two workers across
seven seeds at 2×. The production-layout replay reconstructs only known terrain
(unseen becomes water), so it is supporting evidence rather than an exact save.
The [isolated browser comparison](../verification/2026-10-05/bush-delivery-2x.json)
uses one gather command at 2× and no later input: old code delivers 20 then idles
with 40 nearby food left; fixed code delivers 60 and leaves distant food untouched.
Both runs have no page errors. Modal's API was unreachable during diagnosis despite configured credentials;
production was only read through `/state` and was never reset or commanded.

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).

## Building and ship storage

PR #94 generalizes delivery targets to `storage_id`: completed compatible buildings and stopped shore ships with remaining hold capacity. Preserve residual personal cargo on a partial deposit, and reselect a compatible site if a ship moves away. `storage.rs` and `ship_tests.rs` cover local spending, capacity, transfers, and deposit resumption. Bounded nearest-goal routing applies to both kinds of storage site.

## Renewable water collection

Water gathering uses the existing carry/deliver/resume task. Drawing water does
not reduce a riverbank source's finite bookkeeping amount; it does increase
finite personal cargo and storage normally. Stop still cancels collection.
See [water](water-resource.md) for placement and per-cycle field costs.
