# Open Work

**Last updated:** 2026-10-02T09:30:00Z
**Branch:** `claude/rts-frontend-webgl-canvas-gukaet`
**Overall status:** Rust/wgpu client working in the browser at `/play`; legacy Three.js client still at `/`.

## Current goal

Run as much as possible in Rust: one wgpu client (native window and WebGL2) over the shared `crates/game` simulation, so the game is easy to run locally (`cargo run -p aoa-client`).

## Done

- Workspace: `crates/game` (simulation + navigation, 54 tests), root server, `crates/client` (wgpu 30, winit 0.30).
- Client renders painted biome ground (texture array, ragged blends, fog of war, cloud shadows), sea, illustrated villager/resource/tree billboards, blob shadows and selection rings, depth-based ink lines, tilt-shift and ACES; HUD (resource coins, globe minimap, speed coins, info pill, command coins, toasts) painted on the GPU from the generated kit and Nunito.
- Sources: in-process simulation (native default, `/play?local`) or the hosted server over WebSocket.
- Verified in headless Chromium/WebGL2: select, gather, train via HUD, build placement via HUD, speed coins; no page errors.

## Open work

1. Parity before `/` switches to the Rust client: box select, pinch/twist touch gestures, globe drag, DOM accessibility mirror, clouds/volcano/ships, float texts and work particles, persistence for native local play.
2. Town center: placeholder sheet `assets/game/building_town_center.png` until the FAL temple billboard (foundation/33/66/complete/working) lands; then draw construction stages.
3. Native window not exercised in CI or this container (no display); verify `cargo run -p aoa-client` on a desktop.
4. Liveness beyond pairwise standoffs is untested.

## Generated art

- On master: `assets/ui/` icon kit and coin buttons, `assets/terrain/` painted ground (revision 2), `assets/sprites/villager{,_woman,_elder}` billboards (drawn by `frontend/villager-sprite.js`), `assets/models/` opt-in GLBs (temple town center on by default via `?glb=`).
- Branch `claude/fal-sprites-resources` (PR #12): base villager walk fix (no hop, moving carry stride; layout unchanged), and `assets/sprites/resources.{png,json}` resource/tree billboards (9 nodes, 21 sprites, depletion stages, base anchor, per-node world scale), not wired in yet.
- FAL spend about $7.64 in total; per-call ledgers live in `assets/*/tools/ledger.jsonl`.

## Blockers

None.

## Exact next action

Review and merge the branch, then verify production.
