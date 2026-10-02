# Open Work

**Last updated:** 2026-10-02T15:20:00Z
**Branch:** `claude/rts-frontend-webgl-canvas-gukaet` (seeded islands, on top of master fa6cd43)
**Overall status:** Rust/wgpu client working in the browser at `/play`; legacy Three.js client still at `/`.

## Current goal

Core loop: start on a seeded island that holds everything a fishing boat needs, gather and build up, then leave the island. Next come smoothness and speed, then the boat itself, then adversarial events (pirate raid, drought) that reward stockpiling.

## Done

- Workspace: `crates/game` (simulation + navigation, 54 tests), root server, `crates/client` (wgpu 30, winit 0.30).
- Client renders painted biome ground (texture array, ragged blends, fog of war, cloud shadows), sea, illustrated villager/resource/tree billboards, blob shadows and selection rings, depth-based ink lines, tilt-shift and ACES; HUD (resource coins, globe minimap, speed coins, info pill, command coins, toasts) painted on the GPU from the generated kit and Nunito.
- Sources: in-process simulation (native default, `/play?local`) or the hosted server over WebSocket.
- Verified in headless Chromium/WebGL2: select, gather, train via HUD, build placement via HUD, speed coins; no page errors.
- Playtest fixes (this branch): canvas follows device pixels (speed coins and all clicks were dead on devicePixelRatio 2); 60×40 half-unit grid with clustered resources and 4×4 town centers (schema 6); `Command::Stop` with a Stop medallion and X; hover target ring while villagers are selected; working villagers lean toward their work; HUD icons fitted by painted bounds, gap-free command bar, fixed-width info pill (no flicker); serde_json `float_roundtrip` so saves reload bit-identical. Verified at DPR 1 and 2, mouse and touch.

## Open work

0. Touch: two fingers pinch-zoom, twist-rotate and pan; taps pick the terrain surface (the seeded island's elevation had made taps miss).
0. `/play` shows a loading overlay with the generated title and buildings rising through their stages (`assets/loading/`) (game download, then art files fetched in parallel, then world setup) until the first world frame is drawn.
0. Seeded islands (merged in PR #22): `crates/game/src/game/worldgen.rs` generates the island from `AGE_OF_AGENTS_SEED`, `POST /reset?seed=N`, or `/play?local&seed=N`; schema 7 drops older saves. Next: a fishing boat (`FISHING_BOAT_COST`) as the island's goal.
1. Parity before `/` switches to the Rust client (review blockers: the HUD has no accessible DOM button mirror yet, so mouse and touch are not yet identical): box select, globe drag, DOM accessibility mirror, clouds/volcano/ships, float texts and work particles, persistence for native local play.
2. Town center: illustrated temple stages are wired (foundation, 33%, 66%, complete, working); add a client-side doorway glow or smoke for the working state.
3. Native window not exercised in CI or this container (no display); verify `cargo run -p aoa-client` on a desktop.
4. Liveness beyond pairwise standoffs is untested.
5. Small resource nodes (30 per tree or bush) deplete quickly and the gatherer then goes idle; consider continuing to the nearest node of the same patch (AoE behaviour) if playtesting confirms the micromanagement hurts. Not done: it is close to autonomous task selection, which the milestone forbids.
6. The legacy Three.js client at `/` reads the new 60×40 grid but draws it at one world unit per cell (twice the size) and draws water and beach flat; it is frozen until removal.
7. Anno is now a named second inspiration (calm economy, growing settlement) in `AGENTS.md`.

## Generated art

- On master: `assets/ui/` icon kit and coin buttons, `assets/terrain/` painted ground (revision 2), `assets/sprites/villager{,_woman,_elder}` billboards (drawn by `frontend/villager-sprite.js`), `assets/models/` opt-in GLBs (temple town center on by default via `?glb=`).
- Branch `claude/fal-sprites-resources` (PR #12): base villager walk fix (no hop, moving carry stride; layout unchanged), and `assets/sprites/resources.{png,json}` resource/tree billboards (9 nodes, 21 sprites, depletion stages, base anchor, per-node world scale), not wired in yet.
- `assets/sprites/towncenter.{png,json}`: town center billboard (foundation, build33, build66, complete, working; 512 px cells, footprint-centre anchor) for the Rust/wgpu client (branch `claude/fal-towncenter-sprites`).
- `assets/sprites/villager_idle_hd.{png,json}`: 2× idle frames for all three people, drawn by the Rust client for every standing villager (walk, carry and work still use the 1× sheets).
- `assets/loading/`: loading-screen title wordmark and four-stage building sheet for `/play` (branch `claude/fal-loading-art`), not wired in.
- FAL spend about $10.05 in total; per-call ledgers live in `assets/*/tools/ledger.jsonl`.

## Blockers

None.

## Exact next action

Merge the seeded-island PR, redeploy Modal (schema 7 replaces the persisted world with a generated island on first start), and verify production with `scripts/modal_manage.py verify`.
