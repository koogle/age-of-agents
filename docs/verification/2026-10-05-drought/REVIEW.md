# Drought implementation review

First environmental danger on base `58813c1`, 2026-10-05. The product spec calls
for escalating calamities but leaves timing/effects open; this slice selects
reversible economic pressure with an advance warning. Timings are provisional.

## Scope and quality review

- Authoritative rules are confined to `crates/game`: one saved clock, a derived
  typed forecast, and a multiplier at the existing gathering-rate calculation.
  No event engine, new dependency, autonomous work or additional command.
- All discovered islands share the same weather and food stockpile. No hidden
  terrain/entities are exposed by the forecast. Existing costs, cargo, capacities,
  field preparation, placement, production, research and unit reservations remain.
- Drought multiplies existing bonuses. It cannot consume stock, create goods,
  exceed capacity, kill units or destroy buildings. Recovery restores the normal
  rate. This slice alone cannot end a run: passive food consumption is absent.
- Saved clocks must be finite and nonnegative; snapshots derive phase/cycle so
  they cannot drift apart in stored state. Store version 12 resets incompatible
  saves, including version 11, under the current accepted compatibility policy.
- Shared native/WebGL HUD uses existing text/panel primitives and places weather
  above pause/toast copy. No new input mode or art; no frontend file exceeds the
  1,000-line limit. The existing accessibility DOM gap remains open.
- Thermonuclear review found no need for a generic event manager, extra state
  machine, worker interruption or migration. Existing idle-world test now checks
  the new clock explicitly before comparing unchanged gameplay state.

## Verification

232 Rust tests passed (13 server, 76 client, 143 domain); one existing manual
benchmark ignored. New coverage includes phase boundaries, capped escalation,
pause/speed/reset, bad dt/clocks, food-only changes, conservation/recovery,
field/research/farm bonus composition, unchanged idle orders and reload.

Native and WASM strict lint and formatting pass. Desktop, DPR2 phone touch and landscape
browser runs pass with no page errors. Asset resolution, field preparation,
transport and icon checks pass; three Python
production-verifier tests pass. WebAssembly was rebuilt with Rust 1.99.0 and
wasm-bindgen 0.2.129; generated JS syntax passes. No asset changes.

The reproducible driver is [verify_environment.py](../verify_environment.py).
Screenshots and `results.json` in this directory record seed 123 with a real
isolated SQLite server, Chromium software WebGL, desktop 1280×800, emulated
390×844 DPR2 touch and short landscape 844×390. It advances only a stopped
fixture's saved clock, then exercises actual mouse/touch 2× and pause controls,
checks warning-to-drought onset and verifies pause freezes the forecast.
Physical phone and native platform appearance are not verified by this fixture.

Direct release is blocked: Modal 1.5.3 status failed with "Could not connect to
the Modal server" despite ready runtime credential bindings. Production has not
been changed.
