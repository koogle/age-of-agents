# Basic roads verification — 2026-10-05

Roads are walkable surfaces with explicit villager work. Dirt costs only labour;
stone adds one stone per new cell. Both take two worker-seconds per cell and grant
1.5× friendly movement after completion. Each order is an inclusive horizontal or
vertical segment; mixed-material crossings reuse existing cells. Animals retain
their original routes and speeds.

## Reproduce

```bash
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy -p aoa-client --target wasm32-unknown-unknown --locked -- -D warnings
cargo fmt --all --check
cargo build -p age-of-agents --locked
cargo build -p aoa-game --example roads_fixture --locked
./scripts/build_web.sh
python3 docs/verification/check_roads.py --output /tmp/road-check
```

The browser driver needs a Python environment with `aiohttp` and `playwright`
and an installed Chromium. In the managed workspace, use the default Python;
cloud activation for Cargo can select a different Python without these packages.

The driver creates an isolated temporary SQLite save and server on loopback
port 8000. It refuses to start when that port is occupied and never opens the
configured or production database. It uses a flat, visible settlement from
`crates/game/examples/roads_fixture.rs`, Chromium software WebGL, desktop mouse
(1280×800 DPR1) and emulated phone touch (390×844 DPR2). Phone endpoints are
shifted four cells east to keep endpoint taps inside its narrow viewport.

It selects a villager, opens Build → Roads, places seven dirt cells and seven
stone cells through real inputs, checks dominant-axis snapping, waits for
authoritative completion, verifies stone stays at 30 then falls to 23, and
checks exactly two build-road commands with no browser errors. It waits for
rendered frames after authoritative completion before clicking the now-idle
Build control: a short wall-clock sleep under software WebGL can still leave
the previous Build/Stop layout on screen and click Stop accidentally.

## Checks

Combined with master `5a28839`, retaining the approved productive-building
availability gate, upstream art and granary field-yield bonus.

- 282 Rust tests pass; one existing manual archipelago benchmark is ignored.
- Native/WASM strict lint, formatting, rebuilt WASM and native server pass.
- Sprite resolution, field/transport assets and icon normalization pass. Six
  release-verifier tests pass. Existing generated assets are reused unchanged.
- Desktop mouse and DPR-2 phone touch both pass against the `8bf00b4` integration,
  without browser errors. [Result hashes](results.json) identify the server and WASM.
  The screenshots below show that build. The later granary-only integration is
  covered by the full Rust suite and a final desktop construction replay; see
  [final integration results](granary-integration/results.json).

## Visual review

**Updated 2026-10-06:** the user requested closer style matching. Dedicated road
materials replace the original reuse below; see the [current refinement and
visual evidence](style/README.md). The following screenshots retain the original
construction verification and are historical material previews.

### Original material pass

The original pass used approved clayland paint for dirt and the existing
cobblestone swatch for stone. These retained the terrain lighting, fog and
perspective. At that stage there was no new generated art or asset editing. Ink/material match was inherited
from those swatches; straight boundaries are deliberate grid edges. The existing
Build and stone medallions retain the approved palette and scale, with explicit
Dirt road / Stone road labels. Retained captures: [desktop before](desktop-before.png),
[desktop line preview](desktop-dirt-preview.png), [phone menu](phone-dirt-menu.png),
and the completed roads below. The phone captures use the default camera scale; physical device rendering
and maximum-zoom appearance remain unverified.

![Completed roads on desktop](desktop-stone-complete.png)

![Completed roads on emulated phone](phone-stone-complete.png)

## Thermonuclear review

One road module owns authoritative cost, work and validation. Roads do not claim
blocking cells, so they cannot split walking routes; existing buildings/resources
keep their ordinary occupancy. Typed commands use the existing atomic clone/apply
transaction. Paid cells are reused, work survives Stop, and only assigned workers
continue the requested line. No dependency, pathfinding framework, autonomous
job selection or new building requirement was introduced.

The existing deterministic search accepts optional travel-time weights. Friendly
steps and all route-cost selection use the same road factor; movement changes
speed exactly at cell boundaries, and weighted diagonal length uses 1414/1000.
Animal and sea movement stay unweighted. No route cache can become stale when a
road completes. Weighted searches keep corner blocking and deterministic ties.

Store version 15 resets incompatible development saves and preserves matching
saves; malformed road progress/duplicate cells fail validation. Road validation
follows terrain validation to avoid indexing malformed grids. Client files remain
below 1,000 lines. Mouse/touch use one placement flow.

Not merged or deployed. Physical phones and native window appearance are not
verified. Road speeds/routing are proven by domain tests; the browser driver
verifies construction, charging, rendering and input.
