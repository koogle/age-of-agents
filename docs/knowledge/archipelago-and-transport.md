# Archipelago and transport

Read before: Before changing discovery, voyages, passengers, island resources, or map scaling.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with this system

The domain owns discovery and sailing; the client presents destinations and sends
commands. Start in [islands.rs](../../crates/game/src/game/islands.rs),
[ships.rs](../../crates/game/src/game/ships.rs) and
[island tests](../../crates/game/src/game/islands_tests.rs). Read
[continuous map](../CONTINUOUS_MAP.md) for the coordinate model and measured limits.

1. Identify whether the change affects generation, movement, passenger ownership,
   snapshot dimensions, save versioning, or several together.
2. Preserve deterministic seed/discovery ordering, existing settlements and ship
   position. Discovery adds land; destination shortcuts are sailing orders.
3. For save changes, use the [server and saves guide](server-and-saves.md).
   For new extents, consult [rendering/input](rendering-and-input.md).
4. Run the affected island/ship regressions, then exercise the actual Explore,
   crossing, landing and return flow on an isolated save.

```bash
cargo test -p aoa-game --locked islands_tests
cargo test -p aoa-game --locked ship_tests
```

For a scaling investigation, the existing ignored measurement is
`cargo test -p aoa-game --release archipelago_budget -- --ignored --nocapture`.
It measures a starter-population world; document population and revealed terrain
when adding a new measurement. Do not extrapolate it to many busy settlements.

## Learned constraints and evidence

**Evidence:** [#55](https://github.com/koogle/age-of-agents/pull/55) removed a
starter boat budget requiring materials unavailable before departure.
[#71](https://github.com/koogle/age-of-agents/pull/71) added ship goods holds;
[#73](https://github.com/koogle/age-of-agents/pull/73) introduced map exchanges,
local inventories and paused away islands;
[#83](https://github.com/koogle/age-of-agents/pull/83) removed ship goods and pooled
stocks; [#90](https://github.com/koogle/age-of-agents/pull/90) replaced instant
voyages with continuous sailing and simultaneous simulation.

**Lesson:** Before changing exploration, state the resource scope, discovery
trigger, travel behavior, background simulation, passenger ownership and save
conversion together. Current choices are in [continuous map](../CONTINUOUS_MAP.md)
and the README. PR #94 restores local inventories and a 50-resource ship hold by explicit user request; do not restore the historical 200-good holds, paused settlements, or first-ship-generation design. First departure must remain affordable from starter
resources, including timber's raw wood cost.

**Check:** `crates/game/src/game/islands_tests.rs` covers non-teleporting shortcuts,
both settlements producing, runtime map dimensions and current-format round trips;
`ship_tests.rs` covers original-dock return, passengers and blocked commands.
Historical island translation and inventory pooling are removed per Jakob’s
2026-10-05 instruction. Discovery adds generated terrain/resources directly to
the continuous world; incompatible hosted saves reset by store version. Consult
[the save policy](server-and-saves.md#snapshot-and-save-version-policy) before changing persisted fields.

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).

## Island storage and movable ship holds (PR #94)

Each discovered island owns a stockpile. A stopped shore ship contributes its cargo to that island’s available resources and accepts villager deposits up to its remaining 50-resource capacity. Cargo stays aboard on arrival and leaves with the ship; passengers have four independent seats. Research and training still require buildings. Explicit load/unload commands transfer up to 10 per button tap at a completed dock, without NPC labor.

`storage.rs` owns compatible building/ship sites and local resource reservation. Costs consume shore stores first, then connected holds; production output and queue refunds remain on the job’s island. Partial deposits preserve the villager’s remainder when a ship fills or departs. Future automated trading ports are intended but not implemented. The shared map and all-settlement simulation remain intact.

## Shore pickup (2026-10-05)

User-requested behavior: clicking a transport with boarding units selected should
bring the ship to a reachable shore so both can meet. Implemented in `ships.rs`: the first boarding order chooses a berth using the
combined reachable land/water route cost with deterministic cell ties. A stopped,
reachable ship keeps its berth; additional passengers join the same pickup. Units
walk to the destination shore and wait until the vessel stops before boarding.
Dock bridging and seat reservations remain supported. No shared reachable shore
rejects the order without changing either actor. Explicit sailing or Stop ship
cancels pickup orders; a redirected ship finishes its current step.

The existing ship destination and Board action encode pickup, so no persisted
fields or store-version change are needed. Validation permits boarding a moving
ship; `tick_board` enforces arrival before passenger ownership changes. Searches
compare complete land/sea trees only when selecting a berth; walking uses the
bounded nearest-goal route. `ship_tests` covers offshore group pickup, replay after
a mid-approach save, unreachable shores, cancellation, and mid-sail redirection. A crowded shoreline keeps queued passengers waiting for
space instead of dropping their orders while the vessel approaches.
Clicking with carriers still orders a cargo deposit, as before.

For browser pickup fixtures, the initial view centers on the first ship via
`crates/client/src/islands.rs::frame_town_center`; `Rig::new`'s default target is
not the rendered camera target. Use an isolated save, resume before issuing orders, select the empty-handed
unit and click/tap the offshore ship, check the Board acknowledgement and destination,
and verify the passenger manifest only changes at the stopped berth. At 0×, Board
and Stop ship are rejected without mutating either actor; see the
[pause contract](runtime-debugging.md#pause-contract).

## Terrain-first generation (2026-10-05)

Jakob requested varied outlines followed by relief and terrain-derived rivers.
`worldgen/shape.rs` selects a seed-stable rounded, square, long, bay or lobed mask;
rerolls preserve the family. The bay's central basin remains connected to the sea.
`worldgen.rs` retains the main land component, applies coastal slopes and noisy
ridge relief, and derives mountain/highland bands from the resulting elevation.

`worldgen/drainage.rs` priority-floods depressions to spill height with a small
positive gradient. Steepest cardinal descent defines a single acyclic drainage
network; accumulated upstream cells select headwaters. Rivers follow that network
through confluences to water; they no longer use a separate meander height field
or independently flatten beds. Sandbar fords and reachable resource checks remain.
This is a procedural relief approximation, not a physical erosion simulation.
Snapshot height quantization can display tiny slopes as flat, but preserves order.

Generation changes affect new games and newly discovered islands. Existing saved
terrain remains authoritative; no persisted fields changed or save reset is needed.
Run `cargo test -p aoa-game --locked worldgen` for shape, runoff, downhill-route,
resource and settlement checks. Near-start wood and food must be within town-center
sight, not merely within the wider resource-placement radius: the latter can leave
a fresh settlement with no visible gather targets. The `island_preview` example exports fully explored
snapshots; `docs/verification/island-generation/preview.py` renders relief comparisons.
