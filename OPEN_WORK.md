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

0. Art: ten painted ground textures, the FAL temple town center, and a depth-based fine ink-line pass are in. Villagers are now generated illustrated billboards (`frontend/villager-sprite.js`, `assets/sprites/villager.*`). Trees and resource nodes are illustrated billboards too (`frontend/resource-sprites.js`). Next: rim scenery and the sky props are still procedural; sprites are not tinted by the explored/remembered fog; meadow texture is still flatter than prairie.
0b. Smoothness: units are drawn ~1.6 ticks behind the newest snapshot on a self-correcting presentation clock (`frontend/world-view.js`); verify on a real phone over Modal.

1. Jakob reviews the branch; merge to `master` triggers CI deploy to Modal (old saves drop because of schema 5).
2. Production verification after deploy (`python3 scripts/modal_manage.py verify`) plus a real phone check of touch rotate/pinch.
3. Liveness beyond pairwise standoffs (e.g. three-way jams in dead ends) is untested.
4. Decide whether to delete the unused 2D sprites under `assets/game/` and their Python checks.

## Generated art

- On master: `assets/ui/` icon kit and coin buttons, `assets/terrain/` painted ground (revision 2), `assets/sprites/villager{,_woman,_elder}` billboards (drawn by `frontend/villager-sprite.js`), `assets/models/` opt-in GLBs (temple town center on by default via `?glb=`).
- Branch `claude/fal-sprites-resources` (PR #12): base villager walk fix (no hop, moving carry stride; layout unchanged), and `assets/sprites/resources.{png,json}` resource/tree billboards (9 nodes, 21 sprites, depletion stages, base anchor, per-node world scale), not wired in yet.
- `assets/sprites/towncenter.{png,json}`: town center billboard (foundation, build33, build66, complete, working; 512 px cells, footprint-centre anchor) for the Rust/wgpu client (branch `claude/fal-towncenter-sprites`).
- FAL spend about $7.94 in total; per-call ledgers live in `assets/*/tools/ledger.jsonl`.

## Blockers

None.

## Exact next action

Review and merge the branch, then verify production.
