# Field gathering audit — 2026-10-05

## Finding and fix

A field already is a `ResourceNode`: after paid preparation, workers enter the
same `Gather` phases used by wild resources. Mouse and touch use the same target
handler; ripe fields issue `Gather`, while unfinished/depleted fields issue
`Cultivate`. Carry limits, compatible drop sites, resumption, and nearby food
continuation are shared. Replanting remains an explicit paid order.

The placement path was different: `PlantField` checked that the worker could
reach the plot, but did not preserve routes through its new permanent 3×3
footprint. A plot could therefore split a narrow land strip while remaining
reachable from both sides. Workers on one side could prepare and harvest it,
but could not reach their farm, granary, or town center to unload.

Field placement now calls the same `placement_preserves_routes` guard as building
placement, before inserting a node or spending materials. Rejection preserves
cargo, current assignments, stockpiles, and the world. No harvesting-specific
branch, schema migration, client rule, dependency, or automatic replanting was
added.

## Regression coverage

- A field across the only delivery route is rejected for a planter on either
  side, including when a different worker has cargo. This test fails on the
  previous implementation because placement incorrectly succeeds.
- A one-cell bypass permits planting, preparation, repeated farm deliveries,
  resumption after serialization/reload, complete depletion, and final idling.
- Field → wild food and wild food → field both exhaust and deliver the full
  combined amount, including partial baskets, without further orders or costs.
- Existing field tests cover shared preparation, carrying helpers, Stop,
  explicit replenishment, and complete repeated harvests.

## Verification

- All 186 workspace tests pass: 13 server, 51 client, 122 simulation. The ten
  focused field tests also pass after the final test-fixture cleanup.
- Formatting, whitespace validation, and strict native/all-feature and WASM
  Clippy checks pass. Rebuilt the tracked WebAssembly bundle and loaded that exact
  bundle in a fresh browser with no page errors.
- Hosted browser test against the current server and an isolated SQLite fixture:
  desktop mouse click prepares a field, harvests and delivers all 120 food in
  six loads, then idles. Phone touch (390×844, DPR 2) replants the exhausted
  field for exactly 10 wood + 5 stone and delivers the next 120 food. Both
  observe preparation, gathering, returning, depositing, resumption and final
  idle without additional work orders; no browser page errors.
- Browser scripts, screenshots and isolated data are in
  `/workspace/scratch/field-audit`; existing saved games were not touched.
- No production deployment: this is a dedicated review PR. Merge deployment
  remains the repository's normal release path.

## Thermonuclear review

The fix reuses an existing domain guard instead of adding a field-specific
routing system. Commands retain the existing atomic application boundary. Costs
are checked before route validation and charged once only on accepted placement.
All source files changed remain below 1,000 lines. Mouse/touch handlers and art
are unchanged. Existing field footprints, including exhausted ones, stay blocked
as required by occupancy; there is no implicit save repair.

## Limits

The user's saved layout was not supplied, so this is a reproduced field-specific
cause, not confirmation of the precise reported incident. Previously placed
fields that already sever routes are not relocated or removed. A separate,
existing traffic limitation also remains: an idle villager standing in a
one-cell passage can block another worker's delivery. Neither limitation is
specific to the harvesting phase machine.
