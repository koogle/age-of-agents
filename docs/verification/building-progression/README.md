# Productive building visibility (2026-10-05)

User-requested replacement for construction-only discovery: buildings need
obtainable construction inputs and a productive use. Production buildings must
have at least one recipe supported by discovered raw resources. Housing,
storage, gathering bonuses and vision remain useful without production recipes.

First-island Military shows Watchtower and Range; Production shows Lumber Mill
and Kitchen. Barracks/Smelter require explored iron and coal; Kiln requires clay;
Weaver requires fiber; Workshop needs clay plus steel inputs. Range deliberately
remains available because archers cost food and timber. An empty stockpile greys
out useful buildings instead of hiding them. Discovery remains global, survives
depletion and reload, and cannot be bypassed with injected inventory.

## Reproduction

From the repository root, rebuild `./scripts/build_web.sh`, then:

```bash
cargo run -p aoa-game --example island_preview -- 17 > /tmp/starter.json
python docs/verification/building-progression/browser.py /tmp/starter.json /tmp/building-menu
```

The driver uses the actual domain-exported availability, flattening the scene and
centering one worker for repeatable clicks. It serves only loopback :8012, uses
an empty stockpile, and captures desktop 1280×800 mouse and emulated 390×844 DPR2
phone touch menus. It records the browser bundle hash and page errors. Screenshots
require visual inspection; this fixture verifies presentation, not live command
acceptance or a physical phone. The Rust tests verify authoritative build
rejection without mutation, generated island-two discovery, hidden deposits,
processed recipe inputs, depletion/reload and starter departure budgets.

## Results

All 269 workspace tests pass (one existing manual benchmark ignored), including
seven progression tests. Formatting, strict native/WASM lint, the release browser
rebuild, generated JavaScript syntax and `git diff --check` pass. Final desktop
mouse and emulated DPR2-phone touch captures have no browser errors. Visually
inspected Military and Production menus show the expected two buildings each,
greyed out with empty inventory. Bundle identity is recorded in [result.json](result.json).

- [Desktop Military](desktop-military.png) · [Desktop Production](desktop-production.png)
- [Phone Military](phone-military.png) · [Phone Production](phone-production.png)

## Code review

One domain predicate governs snapshots, construction and production commands;
no client-side progression rule, dependency, stored state or schema change was
added. Existing buildings and paid jobs remain in saved worlds. Availability now
uses existing construction costs and product recipes. Unrestricted test worlds
retain the full catalog. No new autonomous behavior or payment path was added.

Production deployment uses the existing master release workflow; this workspace
change was explicitly approved for PR creation and merge by the user after review. Deployment is tracked separately from merge status.
