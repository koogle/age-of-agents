# Placement and route preservation

Read before: Before changing foundations, fields, occupancy, fogged placement, or navigation.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with this system

Start in [construction.rs](../../crates/game/src/game/construction.rs),
[fields.rs](../../crates/game/src/game/fields.rs),
[occupancy.rs](../../crates/game/src/game/occupancy.rs) and
[navigation.rs](../../crates/game/src/navigation.rs). Client
[placement.rs](../../crates/client/src/placement.rs) is a preview, not authority.

1. Trace the typed command to footprint/visibility/affordability checks and the
   existing `placement_preserves_routes` guard before spending or inserting state.
2. Include all affected units and their reserved next cells, not just the builder.
3. For deferred fogged placement, revalidate when exploration finishes; the world
   may have changed since the player requested the site.
4. Exercise a blocked placement and an accepted construction followed by delivery
   or walking out. Do not infer route safety from a green preview alone.

```bash
cargo test -p aoa-game --locked placement_tests
cargo test -p aoa-game --locked construction_tests
cargo test -p aoa-game --locked fields_tests
```

See [the field audit limits](../FIELD_GATHERING_REVIEW.md#limits) before diagnosing
an old save. Prevention does not relocate already-blocking plots, and an idle
unit in a narrow passage is a separate traffic problem.

## Destination searches and deterministic routes

`PathTree::route_to_nearest` stops Dijkstra when the cheapest requested goal is
settled. It preserves complete-search path and cell-order ties; use it for a
known movement/interaction destination. Drop-site selection still prefers a
clear route, then route cost, then building ID, and only computes a blocked-route
fallback when the clear route is unavailable. Per-site searches can cost more
with many far-away compatible buildings; profile populated settlements before
assuming the small-world gains generalize.

Keep complete trees for connectivity/placement checks and callers comparing
arbitrary terrain cells. A partial tree cannot replace those contracts. The
navigation tests compare exact paths across every 3×3 obstacle layout and check
that an adjacent destination does not flood the whole 120×80 grid. Gathering
regressions cover busy-only, incompatible, unfinished and blocked-nearby drop sites.

## Learned constraints and evidence

**Evidence:** [#46](https://github.com/koogle/age-of-agents/pull/46) prevented
buildings from trapping villagers. The same class of problem recurred for fields
in [#80](https://github.com/koogle/age-of-agents/pull/80): workers could reach and
harvest a plot but could no longer reach the delivery building.
[#79](https://github.com/koogle/age-of-agents/pull/79) deferred fogged foundations
and payment until exploration made validation possible.

**Lesson:** Reachability to the new object is only half the contract. A new
permanent footprint must preserve existing routes for builders, bystanders and
mid-step units, including future delivery. Use `placement_preserves_routes` in
the domain; do not build a separate field routing system. A preview or an earlier
affordability check never authorizes a later mutation without revalidation.

**Check:** `placement_tests.rs`, `fields_tests.rs`, and `construction_tests.rs`:
last walking cell, diagonal false exits, a one-cell bypass, both sides of a
bottleneck, independent islands, fog visibility and competing orders. Rejection
preserves costs, cargo, assignments and world state. Existing trapped saves are
not silently repaired by placement prevention.

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).
