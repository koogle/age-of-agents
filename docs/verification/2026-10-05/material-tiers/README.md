# Clay and building material tiers — 2026-10-05

[PR #103](https://github.com/koogle/age-of-agents/pull/103). Main integrated through `2bedc41` (runtime integration `d79a318`); not merged or deployed. [All 17 buildings](catalog.jpg), [short preview](preview.jpg), and [four dock directions in-game](dock-directions.jpg).

Clay joins iron/coal on island 2, preserving metal access. Island 3 retains clay; island 4 adds fiber. All 17 buildings keep their identities and functions. Base Workshop/Monument brick costs become stone, making Workshop the 15th starter building. Discovering clay enables one paid 20-second masonry job per completed building. Costs are 10/5, 20/10 or 30/15 bricks/timber by building size. Payments use the building’s island and connected ships; waiting cancellation refunds once to its island. Paused orders are rejected without payment until the player resumes. These are material/visual upgrades without stat bonuses.

Save version 15 adds the required material flag to upstream’s version-14 island storage, health and wildlife schema. Incompatible saves reset under the existing development policy; current corrupt saves remain errors.

## Executed verification

- The latest full Rust suite passes 273 tests (13 server, 89 client, 171 domain; one manual benchmark ignored); all 89 client tests were additionally rechecked after the final client-only time-control integration; strict native/WASM lint passes. Formatting passes and JS/WASM is rebuilt from combined source.
- Sprite audit: 383 frames, zero below the HD minimum. Generated sources are retained and only downsampled. Icon, field-preparation and transport audits pass. Six Python verifier tests pass.
- The complete HTTP verifier passes against the isolated localhost server. Its fixed output says “production”, but this does not claim production verification.
- Actual desktop mouse and emulated DPR2-phone touch commands reserve 30 bricks/15 timber once, finish on the original plot, and survive reload without page errors. `desktop/result.json` was refreshed on pause integration `f385b90`; `phone/result.json` and the completed phone screenshot were refreshed on the compact-time-control integration `d79a318`. Both resume at 1× before ordering, then finish at 2×. Earlier construction/zoom captures remain; `--functional-only` repeats commands/reload and captures the completed result.
- The isolated SQLite database contains schema 15 and the completed `masonry: true` building. Domain tests cover missing discovery/funds, duplicate commands, foundations/full queues, stable cancellation, corrupt state, all catalog kinds and island-specific spending/refunds.
- Final dock presentation replay on `ca9d97c` captures 24 combinations: four directions, wood/masonry/roof stage, desktop/phone. Actual pointer/touch picking passes; no browser errors. [Results](dock-results.json) and [contact sheet](dock-directions.jpg). This controlled WebSocket fixture establishes rendering/picking, not authoritative upgrade payment.
- Maximum pinch views of both town-center tiers and maximum wheel view of masonry were inspected. `desktop/starter-close-zoom.png` is one wheel step; the retained driver now uses three clamped steps to reach the limit.

Reproduce upgrades with `check_browser.py --output DIR [--phone]` after starting the current server on an isolated seed-123 SQLite world, paused, unrestricted, with 100 bricks/timber/food. Restore that disposable fixture before each run. Never reset or fund production. `--functional-only` repeats commands/reload and captures completion while retaining earlier construction/zoom images. For dock rendering: `python3 docs/verification/check_dock_facings.py --material-tiers-only --output DIR`; omit that flag to also capture unchanged foundation/wall stages.

## Thermonuclear review

A required `masonry` flag and existing typed queue avoid duplicate building kinds or an independent upgrade system. Validation precedes `spend_at`; duplicate upgrades are rejected, waiting cancellation uses stable IDs, and completion changes only the material flag. `credit_at` keeps refunds local. No new runtime dependency, footprint claim, autonomous friendly behavior or building stat modifier is introduced. Client files remain below 1,000 lines.

Upstream island storage, shore-facing docks, shore pickup, wildlife, mobile cargo, Stop artwork, concise README, gathering continuation and pause/reconnect behavior and compact time controls are preserved. Wildlife retains sprite slot 13, and masonry uses slots 14–18. Both HD building atlases use the same expanded 2048×3584 manifests and directional deck corners. Save version 15 avoids collisions with released upstream version 14. Presentation/menu fixtures include the new required material fields. The README stays concise; behavior and provenance live in the owning guides.

Early whole-sheet drafts lacked authored resolution and some pair edits changed identity; isolated reference crops corrected them. A stale selection portrait was found and fixed. The draft proposal to delay metals was corrected before release: clay is additive on island 2. Sources and rejected approaches are recorded in [provenance](../../../../assets/sprites/material_sources/README.md). A dock replay was interrupted by an upstream merge changing its fixture; it was rerun to completion after reconciliation.

## Limits and release

Software Chromium with emulated touch is not physical-phone/native-platform verification. Existing accessible-DOM and native/safe-area gaps remain. All buildings were inspected in the catalog; town center is the authoritative browser upgrade representative, with all dock directions separately rendered.

Installing `modal[api-proxy-support]` repaired the original CLI connection failure. API workspace lookup confirms that injected credentials select `radiantai`; no `koogle-frick` production profile is configured. No deployment was attempted to that different workspace. Release remains pending through the existing production workflow after review.
