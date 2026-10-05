# Environmental dangers

Read before changing calamity timing, gathering penalties, forecasts or saves.

## First slice (2026-10-05)

The user requested starting environmental dangers in line with the survival-run
spec. Drought is the first implementation; timing is initial balance, not a
previously specified requirement. Combat, health, destruction, starvation and
run-ending disasters remain future work.

`crates/game/src/game/environment.rs` derives a global forecast from the saved
`GameWorld::environment_seconds`. Five minutes of calm precede a one-minute
warning; droughts last 60, 90, then 120 seconds (capped). Every cycle includes
another five minutes of calm. During drought, food harvesting (wild nodes and
fields alike) runs at 50%, multiplied with existing research and local bonuses.
Stockpiles, carried goods, node capacity, field preparation and other work are
unchanged. Idle villagers still need player orders. Players can stockpile food
before onset or assign workers to other resources during drought.

The clock advances by scaled simulation dt, freezes at pause, and resets with a
new world. Gathering uses the phase at the start of each simulation step, like
the other fixed-step economy rules. No wall clock, random draws, per-island state,
commands or event-history collection is introduced. The derived snapshot exposes
phase, cycle, countdown, duration and multiplier without exposing hidden terrain.
The shared client draws a forecast below resource totals, above pause/toast copy.

## Persistence and verification

Store version 12 adds the required clock; version 11 and other incompatible saves
reset under the existing [save policy](server-and-saves.md). Current-version
invalid clocks fail validation. Tests in `environment_tests.rs` cover boundaries,
escalation, pause/speed/reset, invalid dt, food-only effects, conservation,
recovery, field/research/farm bonus composition and deterministic reload. HUD copy is tested in `hud/environment.rs`.

Run the workspace tests and both native/WASM lint, rebuild the web bundle, then
inspect calm/warning/active/recovery on desktop and emulated phone using an
isolated save. Never advance or reset production for testing. See
[verification](verification-and-handoffs.md) and [release](build-integration-and-release.md).

This is economic pressure only: food has no passive consumption yet, so this
slice alone cannot end a run. Balancing and additional dangers remain open.

Browser recipe: `python3 docs/verification/verify_environment.py --output /tmp/aoa-drought-check`. It starts the real server on loopback port 8000 with a temporary database, edits only that stopped fixture to reach event boundaries, and exercises mouse/touch speed controls, onset and pause.
