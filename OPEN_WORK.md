# Open Work

**Last updated:** 2026-10-03T11:09:59+02:00
**Branch:** `master` (fixed camera heading; PRs #20 and #21 merged)
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

0. Scale: buildings are sized from their art so buildings fit compact plots (town center 5×5, house and granary 3×3, watchtower 2×2, dock 4×4); resources keep 8 cells from the town center's middle; schema 9 drops older saves, so production starts a new island on deploy. The start site may now also sit on scrubland or heath.
0. Robustness: the web client reconnects after a server restart (a deploy) and accepts the new server's snapshot numbering; every response carries `Cache-Control: no-cache`, so phones revalidate the page, game code and art after a deploy. Deploys queue: the workflow cancels a pending run when a newer merge arrives.
0. Buildings: villagers build town center, house, granary, watchtower and dock from a menu (greyscale when unaffordable); in-world house/granary/watchtower/dock art uses `assets/sprites/buildings_hd.png` (lossless 512 px cells), repacked from original 1024 px construction cutouts. The dock has no function until the fishing boat. The HUD command bar moves above the globe on narrow screens, and the page uses 100dvh so phone toolbars no longer hide it.
0. Touch: two fingers pinch-zoom and pan; the Rust camera heading is fixed (Q/E, right-drag and twist no longer rotate); taps pick the terrain surface (the seeded island's elevation had made taps miss).
0. `/play` shows a loading overlay with the generated title and buildings rising through their stages (`assets/loading/`) (game download, then art files fetched in parallel, then world setup) until the first world frame is drawn.
0. Seeded islands (merged in PR #22): `crates/game/src/game/worldgen.rs` generates the island from `AGE_OF_AGENTS_SEED`, `POST /reset?seed=N`, or `/play?local&seed=N`; schema 7 drops older saves. Next: a fishing boat (`FISHING_BOAT_COST`) as the island's goal.
0. Relief and rivers (merged PR #38): worldgen adds rolling hills and a ridge of peaks (`TerrainBiome::Mountain`, impassable, top 7% of land by height) and up to two rivers (`TerrainBiome::River`, impassable) that follow a priority-flood drainage route from the highlands to the sea, with sandbar fords (beach cells) on straight reaches; river banks turn to wetland. The client sinks river beds, lays animated water over them, paints peaks as rock with snow on the summits, and peaks rise steeper. Also fixed a ground-paint bug: cells past column 29 or row 19 sampled the edge column/row's painted layer. Old saves still load (no schema bump); press New island to see the new terrain. A generated rock texture would beat the shader's rock tint.
1. Parity before `/` switches to the Rust client (review blockers: the HUD has no accessible DOM button mirror yet, so mouse and touch are not yet identical): box select, globe drag, DOM accessibility mirror, clouds/volcano/ships, float texts and work particles, persistence for native local play.
2. Town center: illustrated temple stages are wired (foundation, 33%, 66%, complete, working); add a client-side doorway glow or smoke for the working state.
3. Native window not exercised in CI or this container (no display); verify `cargo run -p aoa-client` on a desktop.
4. Liveness beyond pairwise standoffs is untested.
5. Done on Jakob's request: gatherers move on to the nearest same-kind node within 10 cells; new orders replace a busy villager's task; sprites take their depth from a point in front of the anchor so slopes no longer cut off building bases.
6. The legacy Three.js client at `/` reads the new 60×40 grid but draws it at one world unit per cell (twice the size) and draws water and beach flat; it is frozen until removal.
7. Anno is now a named second inspiration (calm economy, growing settlement) in `AGENTS.md`.

- PR #20 adds the Stop icon, resource variant sheet, and scenery/work-particle sheet; renderer wiring remains future work.

- PR #21 replaces completed/working town-center roofs while preserving construction frames and anchors; includes reviewed source renders, packer, and Midjourney setup docs.

## Generated art

- On master: `assets/ui/` icon kit and coin buttons, `assets/terrain/` painted ground (revision 2), `assets/sprites/villager{,_woman,_elder}` billboards (drawn by `frontend/villager-sprite.js`), `assets/models/` opt-in GLBs (temple town center on by default via `?glb=`).
- Branch `claude/fal-sprites-resources` (PR #12): base villager walk fix (no hop, moving carry stride; layout unchanged), and `assets/sprites/resources.{png,json}` resource/tree billboards (9 nodes, 21 sprites, depletion stages, base anchor, per-node world scale), not wired in yet.
- `assets/sprites/towncenter.{png,json}`: town center billboard (foundation, build33, build66, complete, working; 512 px cells, footprint-centre anchor) for the Rust/wgpu client (branch `claude/fal-towncenter-sprites`).
- `assets/sprites/villager_idle_hd.{png,json}`: 2× idle frames for all three people, drawn by the Rust client for every standing villager (walk, carry and work still use the 1× sheets).
- `assets/loading/`: loading-screen title wordmark and four-stage building sheet for `/play` (branch `claude/fal-loading-art`), not wired in.
- FAL spend about $10.05 in total; per-call ledgers live in `assets/*/tools/ledger.jsonl`.

## Blockers

None.

## Current building-placement work

- Rectangular claims remain town center 5×5, house/granary 3×3, watchtower 2×2, dock 4×4. The fixed orthographic camera, Grid pill/G toggle, translucent placement ghost and level foundation ground are implemented.
- Latest correction: preserve the original sprite proportions. Removed the four-corner warp and shear, which tilted houses. Each completed/construction/preview sprite is uniformly scaled and centered inside its rectangular plot using the painted base bounds. The cobblestone surface, rather than distorted art, marks the exact claim. Adjacent plots can touch, while the original image determines the wall outline.
- Cobblestone is drawn beneath every building and preview, including foundations, and stays visible when Grid is off. The terrain index now carries a separate occupancy channel, keeping biome paint and fog data intact. World-space texture coordinates join adjacent pads. Existing generated `sprites/tile_stone.png` art was converted from its actual JPEG encoding to a genuine RGBA PNG at `terrain/cobblestone.png`; no new generation or visual art edits.
- Verification: 87 workspace tests passed, plus native/WebAssembly lint and formatting. Tests cover uniform proportions and containment for every stage, stable navigation, exact occupied-cell paving and unchanged biome data across completion. Desktop WebGL checks passed for grid on/off, town-center picking and visual review; DPR-2 phone checks passed for ghost drag/release and snapped foundation placement, with no page errors. The first browser check caught the mislabeled source image format; the genuine PNG runtime copy resolves it.
- Thermonuclear review passed: removed projective rendering/picking machinery, reused the terrain texture array and cell map, added no dependencies or game rules; client files remain below 1,000 lines.
- Previous alignment deployment (0c91fcc) succeeded. This correction is verified for master push; confirm the resulting deployment workflow before claiming it is live.
