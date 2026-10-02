# Open Work

**Last updated:** 2026-10-02T14:00:00Z
**Branch:** `claude/rts-frontend-webgl-canvas-gukaet`
**Base commit:** `477bb8a`
**Overall status:** Direction change implemented on a feature branch; not merged, not deployed.

## Current goal

Replace the Canvas 2D client with a fullscreen Three.js WebGL client and make placement, movement, and actions provably sound in the Rust domain.

## Completed on the branch

- Spatial core rebuilt on exclusive cell claims (`src/game/occupancy.rs`, `src/game/movement.rs`, `src/navigation.rs`); schema version 5.
- 2×2 town center footprints, immediate foundations, `Construct` command for helpers.
- `GameWorld::validate` on load, after accepted commands, and after ticks in debug builds; randomized deterministic play test.
- New frontend modules: `app.js`, `controls.js`, `effects.js`, `hud.js`, `materials.js`, `models.js`, `net.js`, `terrain.js`, `world-view.js`, plus vendored `vendor/three.min.js` (r186).
- CI, `scripts/modal_manage.py verify`, README, ROADMAP, AGENTS.md, and the thermonuclear review updated for the new client.

## Verification completed

- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test` (53 tests) pass.
- `node --check` passes for every frontend module.
- Local server + headless Chromium (SwiftShader): gather, deposit, build foundation, complete town center, train villager, all through real clicks; no console errors; desktop and phone screenshots reviewed.

## Open work

0. Art: ten painted ground textures, the FAL temple town center, and a depth-based fine ink-line pass are in. Next: villager and resource sprites from generated renders; meadow texture is still flatter than prairie.
0b. Smoothness: units are drawn ~1.6 ticks behind the newest snapshot on a self-correcting presentation clock (`frontend/world-view.js`); verify on a real phone over Modal.

1. Jakob reviews the branch; merge to `master` triggers CI deploy to Modal (old saves drop because of schema 5).
2. Production verification after deploy (`python3 scripts/modal_manage.py verify`) plus a real phone check of touch rotate/pinch.
3. Liveness beyond pairwise standoffs (e.g. three-way jams in dead ends) is untested.
4. Decide whether to delete the unused 2D sprites under `assets/game/` and their Python checks.

## Generated art (branch `claude/fal-generated-assets`, PR into this branch)

- `assets/ui/`: FAL-generated illustrated icon kit (18 icons at 128 px, paper tile, Greek-key strip and 9-slice frame, `manifest.json`); the canvas HUD draws the icons.
- `assets/ui/buttons/`: blank coin button frames (normal, hover, pressed, disabled).
- `assets/models/`: generated GLBs selected with `?glb=`; the temple town center (126 KB) is on by default, while the rigged villager (275 KB, 5 clips) and the cypress (22 KB) are opt-in because they don't read better than the procedural ones at gameplay zoom.
- `assets/terrain/`: ten seamless painted ground textures (512 webp plus 1024 masters) for `frontend/ground-paint.js`.
- `assets/sprites/villager{,_woman,_elder}.{png,json}`: three illustrated villager billboard sheets, same layout (34 frames, 256 px cells, feet anchor 128,240), not wired in.
- FAL spend about $6.63 in total; per-call ledgers live in `assets/*/tools/ledger.jsonl`.

## Blockers

None.

## Exact next action

Review and merge the branch, then verify production.
