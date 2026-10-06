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
clear route, then route cost, then storage-site ID (building or stopped shore ship), and only computes a blocked-route
fallback when the clear route is unavailable. Per-site searches can cost more
with many far-away compatible storage sites; profile populated settlements before
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

## Dock orientation (2026-10-05)

Docks automatically face their pier toward adjacent water, as requested on 2026-10-05.
`game/coast.rs::dock_facing` chooses the edge with most adjacent water cells;
ties prefer south, east, north, west. The existing 4×4 land footprint and coastal
eligibility are unchanged, including single-cell water contact and exclusion of
diagonal-only water. Both domain validation and client preview use this rule.

The snapshot helper samples only disclosed terrain and falls back to south when
no water is known. Placed docks and ghosts use the same helper. Static coastlines
make facing derived data, so no new persisted field or save reset is needed.
All four views have foundation, wall, roof and complete frames; see the
[directional provenance](../../assets/sprites/building_sources/directions/provenance.json).

Reproduce the visual check with `docs/verification/check_dock_facings.py`; see
[verification and review](../verification/dock-facing/README.md) for scope and release limits.

## Basic roads (2026-10-05)

Jakob requested dirt roads costing labour only, stone roads costing stone,
straight-line placement and +50% movement speed excluding animals. Road surfaces
live separately from blocking buildings in `game/roads.rs`. Initial balance is
two seconds of labour per cell and one stone per stone cell. Both types grant
1.5× speed when complete; unfinished cells remain walkable at ordinary speed.
Orders use inclusive horizontal/vertical endpoints, require visible walkable
land on one island, reserve new-cell costs once, and can resume existing
work. Crossing segments retain existing material and charge only for new cells. Buildings may still cover road cells; their ordinary occupancy takes priority.

The client Roads group uses two taps/clicks and snaps the endpoint to the dominant
axis. Roads use dedicated painted earth and irregular limestone swatches; building plots retain their original cobblestone. See the [road material review](../verification/roads/style/README.md).
Weighted Dijkstra minimizes travel time for friendly movement and drop-site
selection. Each edge averages endpoint traversal time, matching the movement
speed on transitions; diagonal costs approximate Euclidean distance at milliscale.
Animals and ships keep their unweighted routing and ordinary speeds.

The ten focused road tests cover detours, travel-time storage selection, exact
cell-boundary speed changes, diagonals/corners, animals, atomic rejection,
Stop/resume, saved progress, unloading first, shared work and mixed crossings.
The full workspace passes 289 tests (one manual benchmark ignored). Browser
procedure and visual evidence live in [road verification](../verification/roads/README.md).
Clicking an unfinished cell with villagers selected resumes all unfinished pieces
of its edge-connected road network (Jakob, 2026-10-06), including bends, mixed
materials and completed connecting pieces. Corner-only contact and separate roads
do not join the assignment. The clicked cell comes first, then cell-coordinate
order; the clicked piece must be visible, and occupancy and reachability checks
validate the whole assignment atomically. Existing connected work can extend
beyond current sight; visibility still gates every newly placed cell. Reissuing a multi-cell line keeps its explicit scope.
Costs and progress are retained; idle villagers never adopt road jobs.

Road tasks now accept a nonempty list of distinct existing cells, rather than
requiring a straight line. Placement still requires straight endpoints. The save
shape and version are unchanged, and resumed network tasks survive reload.

## Shared dimensions and interpolation (2026-10-06)

`FIELD_SIZE` owns field dimensions in the domain and placement preview. Land
units, wildlife and ships share cell/step position interpolation; movement speed
and routing stay separate. The proposed expanded footprint geometry helpers were
not retained: they grew source without sufficient subtraction. Existing placement,
route-preservation and distance scans remain unchanged.
