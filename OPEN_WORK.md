# Open Work

**Last updated:** 2026-10-01T21:30:00Z
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

1. Jakob reviews the branch; merge to `master` triggers CI deploy to Modal (old saves drop because of schema 5).
2. Production verification after deploy (`python3 scripts/modal_manage.py verify`) plus a real phone check of touch rotate/pinch.
3. Liveness beyond pairwise standoffs (e.g. three-way jams in dead ends) is untested.
4. Decide whether to delete the unused 2D sprites under `assets/game/` and their Python checks.

## Generated art (branch `claude/fal-generated-assets`, PR into this branch)

- `assets/ui/`: FAL-generated paper-and-ink UI kit (18 icons at 128 px, seamless paper tile, Greek-key strip and 9-slice frame, `manifest.json`); not loaded by the client yet, intended for the canvas HUD.
- `assets/models/villager.glb`: rigged Tripo + Meshy villager (275 KB, 5 clips) behind `?villager=glb`; procedural villager stays default because it reads more clearly at gameplay zoom.
- FAL spend about $2.27 in total; per-call ledgers live in `assets/*/tools/ledger.jsonl`.

## Blockers

None.

## Exact next action

Review and merge the branch, then verify production.
