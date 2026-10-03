# Age of Agents

A deliberately small, mobile-first 3D RTS vertical slice built with a Rust authoritative server, a fullscreen WebGL (Three.js) frontend, WebSocket state streaming, and SQLite persistence.

The current vertical slice is intentionally bounded: command villagers through a resource gather/carry/deposit economy, construct town centers, train villagers, and research five gathering improvements. There are no LLM agents or autonomous NPC policies. Villagers remain idle until commanded.

## Building expansion in the current checkout

The Rust client exposes all 17 catalog buildings in four purpose-based groups: Town, Gathering, Production and Military. Every building has a cost, a real footprint, cost-scaled construction, and authoritative placement/occupancy checks. Placement also rejects foundations that would cut any unit off from previously reachable ground; temporary unit traffic does not block safe construction. Catalog artwork from `claude/fal-catalog-sprites` is integrated: all added buildings render their own construction stages, and guards, archers, healers and siege carts use their own idle/walking sprites. Their action frames are present for later combat/healing. Build-menu buttons and selected-building portraits use the matching completed HD building frame. Select a villager, choose Build, then choose a group; All types returns to the groups.

- Mining camps accept stone, gold, iron and coal; farms accept food and fiber. Completed camps/farms improve matching gathering by 25% within six cells of their footprint; bonuses do not stack. Existing direct gathering remains available.
- Lumber mill: 10 wood → 5 timber (8 s). Smelter: 5 iron + 5 coal → 5 steel (10 s). Kiln: 10 clay + 5 wood → 5 bricks (8 s). Weaver: 10 fiber → 5 cloth (8 s). Kitchen: 10 food → 5 rations (8 s).
- Barracks train guards; ranges train archers; workshops build siege carts; infirmaries train healers. These units can move and stop; combat and healing are deferred. Only villagers gather and build. All units and queued trainees consume housing; processing jobs do not.
- Farms, mining camps, lumber mills, kilns and weavers offer matching existing research. Town centers retain all five technologies. The same technology cannot be queued twice.
- Monuments consume timber, bricks, cloth, gold and steel and provide a sight radius of 24 cells. Docks retain coastal placement; ship production is a subsequent slice.

Select a completed production building and tap its production button. Inputs are reserved once, and the single job slot shows progress. Unit production waits for a free adjacent cell if blocked. Processed goods appear in the resource HUD with names and counts. Jobs, foundations and unit kinds survive saving/reloading.

Fresh islands include coal in highland/scrubland deposits. Existing saves retain their terrain and resources; a previous island without coal needs a new island to use the smelter. Starter-island resource restrictions, transport, multiple islands and local trading inventories are still subsequent work. The grouped menu and catalog assets are deployed in the Rust client at `/play`.

Sprite integration must preserve at least 512×512 authored pixels per frame across **every action, facing and construction stage**, using the original high-resolution renders. Screen sharpness depends on frame pixels and gameplay size, not DPI metadata; upscaling 256-pixel art does not restore detail. Run `python3 scripts/check_sprite_resolution.py` as the strict integration gate (`--report-only` for the current migration inventory), then inspect desktop and DPR-2 phone gameplay at maximum zoom. The audit of `claude/fal-catalog-sprites` at `c968f69` found 48 building frames at 512 pixels but 60 military-unit frames at 256 pixels. Current villager walking, carrying and work frames also remain 256 pixels. The original renders are absent from this checkout, so those integrated sheets still need an HD source repack. Resolution checks do not replace source-quality and visual review.

## Milestone 1

- Seeded island generation on a 120×80-cell map: value-noise elevation with rolling hills, winding valleys and three or four separate mountain ranges sets a sea level, the largest landmass becomes the island, beaches ring the coast, and biomes follow height and moisture; the summits are impassable mountain, and up to four rivers rise in the highlands and run downhill to the sea (or into each other) with sandbar fords on their straight reaches, banks turning to wetland; resources grow in biome-appropriate woodlines, berry patches and mine clumps; only wood and berries grow near the start, while stone, gold, iron, coal, clay and fiber lie at least 24 cells out so they must be found by exploring; every accepted island holds at least one and a half times a fishing boat's cost within reach of the start. Water, mountains and rivers (except at fords) are impassable. Cells are finer than a villager is tall
- Server-authoritative fog with visible, explored-dim, and unseen-dark terrain
- Selectable villagers; Shift-click adds or removes villagers, and a resource/foundation order sends the selected group to shared work
- Biome-compatible wood, food, stone, gold, iron, clay, and fiber gathering
- Bounded villager cargo with explicit return and town-center deposit phases; ivory italic resource labels rise when loads are deposited
- Command-driven construction of all 17 catalog building types (town center 5×5, house and granary 3×3, watchtower 2×2, dock 4×4; compact plots permit shared building edges) through foundations that several villagers can raise together; houses and town centers each house 5 villagers, a granary takes food and fiber, a watchtower sees far, a dock must touch the sea; construction takes 0.3 villager-seconds per resource of cost (house 4.5 s, town center 6 s, granary 7.5 s, watchtower and dock 9 s)
- Starting town-center base with single-slot villager production
- Thirteen typed shared stockpiles and a five-technology gathering tree
- Rust-authoritative fixed-timestep simulation
- Exclusive cell claims for every unit, step, building footprint, and resource, checked by a world validator
- Deterministic eight-neighbor routing without corner cutting
- Typed WebSocket commands and snapshots
- SQLite save/load
- Unified mobile and desktop controls
- Modal deployment

See [ROADMAP.md](ROADMAP.md) for the exact acceptance criteria and later milestones.

## Architecture

```text
crates/game      deterministic simulation (shared, no I/O)
   ├─ crates/client  wgpu renderer: native window or WebGL2 (wasm)
   │     ├─ painted ground, illustrated sprite billboards, ink + tilt-shift
   │     ├─ camera, pointer/touch input, GPU-painted HUD
   │     └─ world source: in-process simulation, or WebSocket to the server
   └─ src/ (server)  Axum: fixed timestep, command validation, SQLite
```

Everything is Rust. The server owns the hosted world; the native client and the
browser's `?local` mode run the same `aoa-game` simulation in-process, so the
game also runs without any server. The older Three.js client under `frontend/`
is still served at `/` until the Rust client reaches full parity.

### World soundness

The world is a 120×80 grid of cells, each half a world unit across, so a villager stands about one and a half cells tall. One derived occupancy map (`crates/game/src/game/occupancy.rs`) is the single source of truth for who owns which cell:

- Every building footprint (complete or foundation), live resource node, unit cell, and in-progress step target is an **exclusive claim**.
- A unit claims the next cell **before** stepping into it and releases its old cell only when the step completes, so two bodies never overlap, even mid-stride. Diagonal steps never cut past an occupied corner.
- A move destination is a **reservation**: others may walk through it but never stop on it, and foundations, spawns, and gathering spots avoid it.
- A build order places its foundation immediately, so the footprint is owned from the moment wood is charged; the building cannot appear twice or on top of anyone.

`GameWorld::validate` rebuilds the map and checks every structural invariant (bounds, exclusive claims, unique ids, consistent actions, finite non-negative amounts). It runs on every load (a violating save is a corruption error, never a silent reset), after each accepted command and tick in debug builds, and throughout a deterministic randomized-play test that also checks atomic rejection, material conservation, and exactly-once buildings.

Snapshots carry each unit's authoritative cell, its step, and an interpolated `position` in cell units for presentation.

## Run locally

Requirements:

- Rust 1.85 or newer
- A modern browser

Native window, simulation in-process (no server, no browser):

```bash
cargo run -p aoa-client --release
```

Hosted-style server plus the Rust web client:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked   # matches Cargo.lock
./scripts/build_web.sh
cargo run --release
```

Open <http://localhost:8000/play> (server world) or <http://localhost:8000/play?local>
(simulation in the page). The legacy Three.js client stays at <http://localhost:8000>.

The default SQLite file is `age_of_agents.db`. Override it with:

```bash
AGE_OF_AGENTS_DB=/tmp/age-of-agents.db cargo run
```

## Controls

- **Select:** tap/click a villager to replace the selection. On desktop, Shift-drag from empty ground adds visible villagers whose feet fall inside the box; Shift-click adds or removes one villager. Touch uses single-unit tap selection. Ordinary mouse and touch drags continue to pan. Tap/click a town center to select it instead.
- **Move:** with one or more villagers selected, tap/click empty ground. Groups receive one atomic authoritative order and spread across distinct reachable cells.
- **Gather:** with villagers selected, tap/click a resource. Villagers walk beside it, gather two units per second, wait for a full 20-unit load unless the node depletes, deposit at the closest reachable completed building that accepts the cargo (town centers accept all resources; granaries accept food and fiber), and resume until depletion. Drop-off routes account for villagers currently blocking access; temporary approach reservations preserve the gathering order until they clear.
- **Stop:** with busy villagers selected, press the **Stop** medallion (or X). They finish the step they are taking and go idle, keeping any carried goods; a foundation keeps its progress. Giving a busy villager a new order does the same and then starts the new task.
- **Keep gathering:** when a node runs out, the gatherer moves on to the nearest reachable node of the same kind within 10 cells, and goes idle only when none is left.
- **Hidden villagers:** a villager behind a building shows through it as a team-blue silhouette, as in Age of Empires II; villagers in front of a building always draw over it.
- **Unload:** with villagers selected, tap a town center (or a granary for food and fiber) to send those carrying goods to unload there; they then wait for orders. A villager holding goods shows its load even when stopped.
- **Finish the load:** a villager carrying goods drops them off before gathering a different kind or building.
- **Target marker:** while villagers are selected, a ring under the pointer shows what a tap would do: gold over a resource or foundation to work on, white over ground to walk to.
- **Build:** select a villager and press the build button to open the building menu. Buildings you cannot afford are shown in greyscale; pick one, then tap/click ground (the ghost and rectangular plot turn green on a clear site). Costs: town center 20 wood, house 15 wood, granary 25 wood, watchtower 15 wood and 15 stone, dock 30 wood. The foundation appears immediately and rises through its drawn stages as the villager works. Tap a foundation with other villagers selected to have them help.
- **Produce:** select a town center and press the **Train villager** medallion. It reserves 50 food and produces one villager over six seconds; each building has one active production slot.
- **Research:** select a town center and press an available technology medallion (hover for its name and cost). Research reserves 40 food and 20 wood, occupies the building for eight seconds, and improves matching gather rates by 20%.
- **Pan:** drag with one pointer, WASD/arrow keys, or tap the minimap.
- **Zoom:** pinch or use the mouse wheel.
- **View:** fixed-angle orthographic isometric view; pan and zoom keep buildings seated on their cells.
- **Grid:** toggle the **Grid** pill (or G) to show square cells as diamonds. Choosing a building shows a translucent ghost and a green/red rectangular footprint; neighboring building plots can share an edge. Cobblestone marks the exact occupied rectangle even with the grid hidden; building and construction sprites fit inside it without skewing. Finished houses use the original design at a smaller visual scale within the same 3×3 claim; initial foundations fill the plot, while later construction follows the smaller house size; the larger granary shows wheat bundles, grain sacks and a wheat emblem. The rectangular cobblestone plot indicates the exact grid claim. Rendered foundation ground is level, and touching plots share one height; steps and decorations can extend beyond the walls.
- **Recover view:** reload to center the camera on the currently visible villagers.
- **Reset world:** in the Rust client (`/play`), tap the **New island** pill top-left, then tap it again within four seconds; the legacy client has a **Reset world** button with a confirmation. Either erases progress and starts a new island. `POST /reset?seed=N` regenerates a specific island; without a seed the server picks one. The first world uses `AGE_OF_AGENTS_SEED` (default `0xA6E0F0A6E7`). The Rust client accepts the same variable natively and `/play?local&seed=N` in the browser.
- **Simulation speed:** use **0×**, **1×**, or **2×** in the top bar to pause or change authoritative simulation speed. While paused, a **Paused · tap to resume** pill shows at the top; tap it to continue at 1×. Tapping a greyed-out command shows why it is unavailable. The web client reconnects by itself when the server restarts (for example during a deploy).
- **Cancel build placement:** press the cancel button or Escape.

Mouse and touch use the same command semantics.

Schema version 10 stores the seed, per-cell elevation and water alongside units as exclusive cell claims on the 120×80 grid and buildings as footprints. Older persisted worlds (free-floating positions, the coarser 30×20 grid of version 5, the all-land map of version 6, the smaller building footprints of versions 7 and 8, or the smaller 60×40 map of version 9) are intentionally dropped because they cannot be translated safely.

Snapshots encode terrain compactly (one character per cell for biome and fog, one for quantized elevation of explored cells), so a full snapshot is about 8 KB rather than over 100 KB.

## Development checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
for module in frontend/*.js; do node --check "$module"; done
python3 -m py_compile modal_app.py scripts/modal_manage.py scripts/process_resource_activity_sprites.py scripts/check_resource_activity_assets.py
python3 scripts/check_depleted_asset.py
python3 scripts/check_resource_activity_assets.py
```

Before shipping structural changes, apply [docs/THERMONUCLEAR_REVIEW.md](docs/THERMONUCLEAR_REVIEW.md).

## Deploy to Modal

```bash
python3 scripts/modal_manage.py deploy
```

Useful management commands are `status`, `history`, `logs`, `rollover`, and `verify`. Stopping production requires the explicit `stop --confirm age-of-agents` safeguard.

Every push to `master` runs the same checks in GitHub Actions, deploys through Modal, and verifies that the production HTML, every frontend module, the vendored Three.js build, the world shape, exclusive unit cells, and unseen-terrain privacy match the committed checkout.

## Art direction

The visual target is a soft 3D tilt-shift diorama of a sunlit Greek island (`assets/reference/diorama_primary.webp`): soft two-tone cel shading (`frontend/materials.js`) with fine one-pixel ink lines found from the depth buffer, painted generated ground textures (`assets/terrain/`, `frontend/ground-paint.js`), a tilt-shift depth-of-field pass (`frontend/tilt-shift.js`), puffy clouds, a distant volcano and sailing ships (`frontend/sky.js`), turquoise sea, limestone, cypresses, olive trees, and a marble town center with a terracotta roof. Villagers, trees and resource nodes are generated illustrated billboard sprites (`assets/sprites/`, `frontend/villager-sprite.js`, `frontend/resource-sprites.js`, `frontend/billboard.js`); resource nodes step through drawn depletion stages and cut trees become stumps. The remaining procedural models in `frontend/models.js` are placeholders being replaced by sprites and textures generated from high-quality renders; generated GLB models from FAL (`assets/models/README.md`) already replace some of them: the marble temple town center is on by default, and `?glb=towncenter`, `?glb=all` or `?glb=none` choose explicitly, with procedural models as the fallback. `assets/ui/` holds the FAL-generated illustrated icon kit (`manifest.json`, provenance in `assets/ui/README.md`).

The interface is painted inside the WebGL canvas (`frontend/hud.js`, `frontend/ui-layer.js`) and stays mostly out of the way: a round globe minimap, a small speed pill, a resource pill that lists only what you have, and glossy medallion buttons that appear at the bottom center only when something is selected. A visually hidden DOM mirror keeps every command reachable by keyboard and screen reader.

The fog of war is one shared shader field: unexplored land lies under a bank of soft cloud, and explored-but-unwatched land is muted. Terrain heights come from fixed noise rather than biome data, so the shape of the land never leaks unexplored information.

The 2D sprites under `assets/game/` are no longer used by the client.

For Midjourney CLI/MCP installation and account setup, see
[Midjourney asset tools](docs/MIDJOURNEY.md).

Rust client presentation uses two walking/carrying poses paced by distance traveled and a neutral pose at rest. Painted sprites preserve their authored colors; terrain retains its grading and placement previews remain translucent. Pause and speed labels are centered by visible glyph bounds.

The loading animation reuses the lossless 512 px gameplay building sheets. Non-HQ completed and partial roofs have cleaned curved tile linework, retaining the illustrated designs.
