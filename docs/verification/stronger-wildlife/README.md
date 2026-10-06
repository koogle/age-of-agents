# Stronger wildlife — 2026-10-06

User target: one wolf should take roughly 3–5 archers, with substantially higher
animal health and damage. Wolf: 40 → 300 HP, 8 → 35 damage per contact hit.
Bear: 100 → 600 HP, 16 → 50 damage per contact hit. Attack cadence is unchanged.

`cargo test -p aoa-game --locked wildlife_tests` exercises full-health encounters
at the server's 100ms timestep, validating occupancy and state after every tick.
Archers currently fight at contact range. In the open contact fixture:

| Archers | Wolf outcome | Surviving archers |
| --- | --- | --- |
| 1 | Survives | 0 |
| 2 | Survives | 0 |
| 3 | Defeated | 1 |
| 4 | Defeated | 3 |
| 5 | Defeated | 4 |

Three archers lose to a bear. These are deterministic fixture outcomes, not a
promise for every terrain layout or staggered arrival. Idle friendlies still
need an explicit attack order. Territory, movement, attack art and friendly
health/damage are unchanged. Existing saved current health persists; increased
damage applies on load and newly spawned animals receive the increased HP.

Browser procedure: rebuild with `./scripts/build_web.sh` and `cargo build --locked`,
then run `python3 docs/verification/verify_wildlife.py --output /tmp/stronger-wildlife`.
The isolated SQLite fixture puts four archers near a full-health wolf. Actual
mouse/touch selects an archer and attacks; an explicit WebSocket command orders
the remaining squad because additive touch selection is not implemented.
It checks mutual damage, wolf defeat, survivors, order cleanup and page errors.
Both desktop (1280×800, DPR1) and emulated phone (390×844, DPR2) pass;
[observed state](results.json) records survivors, injuries and zero page errors.
Physical phones and native-window appearance were not tested.

All 290 workspace tests pass (one existing benchmark ignored), along with native
and WASM strict Clippy, formatting, rebuilt WASM/server and whitespace checks.
WASM SHA256: `34aeaa9a0f426b31044f433f96df378d7a4415d635534edd022f38779cf7d047`.

Thermonuclear review: runtime changes are four domain balance constants; no new
engine, dependency, protocol, save schema or client-side rules. The old lone-villager
victory test is replaced with group outcome and order-cleanup coverage. Bounds
validation and blocked-corner tests follow the new maximum. No art changes.
