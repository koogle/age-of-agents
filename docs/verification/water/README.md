# Water resource verification

Working-tree implementation based on `1bed03a`. Verification completed
2026-10-06 UTC. Not committed, merged or deployed.

## Reproduction

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy -p aoa-client --target wasm32-unknown-unknown --locked -- -D warnings
cargo build -p aoa-game --example water_fixture --profile test --locked
./scripts/build_web.sh
python3 docs/verification/water/check_browser.py
python3 scripts/check_sprite_resolution.py
python3 -m unittest discover -s scripts -p test_modal_manage.py
```

The browser driver serves current assets and WASM on loopback port 8001. Real
mouse/touch input emits typed commands applied to the shared `aoa-game`
simulation through the fixture example. It advances validated 0.1-second ticks
in batches between actions to make two complete harvests practical. Only milestone
snapshots are published during accelerated waiting; the driver waits for their
receipt and three render frames before issuing the next input. This tests
authoritative gameplay and browser input/rendering, not Axum or production
SQLite. The fixture uses a flat controlled riverbank; generation tests separately
cover first/later seeded islands. Phone coverage is Chromium DPR-2 emulation.

## Art review

References: existing wood, food and masonry icons at base `1bed03a`. Generation,
refinement and cutout sources are retained in `assets/ui/sources/water/` with
provenance; the FAL request IDs are in `assets/ui/tools/ledger.jsonl`.

The user rejected the initial icon on 2026-10-06. The earlier style pass is
superseded: orange shading and contour weight were too strong. That artwork and
comparison are preserved under `assets/ui/sources/water/rejected/`.

[Corrected comparison at 128px and 24/32px](icon-comparison.png):

- Ink: finer brown contours and sparse material marks now align with siblings.
- Color: pale buff clay and muted blue water replace the saturated orange body.
- Light/material: watercolor texture replaces the broad glossy highlight.
- Shape/detail: simplified pitcher silhouette and handle, no floating droplet.
- Camera/scale: elevated view and shared optical normalization retained.
- Integration: clean alpha on parchment, green and blue; water remains visible
  at 24/32px. World packing retains the visible ground-contact anchor.

The correction changes artwork only. `check_art.py` captures current desktop
and DPR-2 phone presentation without rerunning the gameplay sequence. The presentation snapshot sets water
to 20 so the HUD pill is visible; this is not collection evidence. Earlier
mechanics screenshots below retain the old artwork and are historical evidence.

Corrected runtime review: desktop and DPR-2 phone HUD, riverbank and maximum
zoom inspected; fine contours and pale clay remain legible, the blue mouth
identifies water, and alpha edges are clean. Both captures completed with no
browser errors ([presentation results](art-results.json)).

- [Desktop HUD/world](desktop-art-riverbank.png), [maximum zoom](desktop-art-maximum-zoom.png).
- [Phone HUD/world](phone-art-riverbank.png), [maximum zoom](phone-art-maximum-zoom.png).

All 299 world sprite frames pass the 512px minimum check. The new HUD icon passes
normalization. Every previously occupied resource-atlas frame is pixel-identical
to the base; the new jug uses a previously unused 512px cell.

## Code-quality review

The domain remains authoritative. The only renewable-resource branch is in the
existing gather mutation; carrying, deposits, resumption and spending reuse the
normal paths. Field preparation still reserves the shared typed cost once after
validation. Water source placement is isolated in a small worldgen module and
checks both terrain connectivity and resource approaches. No new autonomous
worker task, building, command, inventory system, or dependency was added.

Save version 15 reflects required water balances. Existing current-version
corruption handling remains unchanged. All modified client files remain below
1,000 lines. Final Rust verification: all 271 workspace tests pass (13 server, 89 client,
169 domain; one manual benchmark ignored), formatting and strict native/WASM
lint pass. Six production-verifier tests pass. The final browser replay passes on desktop mouse (1100x750, DPR 1) and emulated
phone touch (430x932, DPR 2), including touch panning to reach the riverbank.
Both send accepted Gather, Stop, PlantField, and Cultivate commands, collect
22.133333333333333 water (including a partial personal load), spend exactly 20
across two field cycles, and deliver 240 food. The source retains its 120-unit
renewable bookkeeping balance. No browser errors occurred. See [results](results.json).

- [Desktop field](desktop-field.png), [visible placement cost](desktop-field-cost.png),
  [maximum zoom](desktop-maximum-zoom.png).
- [Phone field](phone-field.png), [visible placement cost](phone-field-cost.png),
  [maximum zoom](phone-maximum-zoom.png).

The field-placement pill was changed during visual review so water costs stay
visible on touch; its new regression and the complete suite passed afterward.
The accelerated driver waits for the final snapshot and rendered frames before
clicking depleted fields, avoiding stale-view commands during batched ticks.
Neither these accelerated replays nor emulation establish physical-phone or
production performance. Existing accessibility gaps remain tracked separately.
