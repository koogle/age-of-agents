# Age of Agents

A Greek strategy roguelike inspired by Age of Empires and Anno. Build an island settlement, grow its economy, and explore further islands while trying to survive an increasingly hostile world.

The current prototype is an island-exploration RTS economy with a shared Rust simulation and native/browser clients.

## Proposed gameplay loop

Start on an island, gather resources, and build a settlement. Build ships to explore and settle more islands, find new resources and treasures, and research better tools and buildings. The long-term world keeps expanding as you discover islands, without a fixed island limit.

Time keeps moving forward and bring with it dangers and challenges. Runs are expected to end in defeat. Treasures and monuments unlock permanent upgrades, including new research and ways to skip early setup on later runs. The balance still needs work: losses should make the next run interesting without making rebuilding tedious.

Dock-built transports carry units between persistent islands. Each island has its own inventory, supplemented by cargo aboard stopped ships at its shore. All discovered settlements keep gathering, building, producing and researching. Combat, calamities, permanent upgrades and automatic trade routes are not implemented yet.

Build a transport at a completed dock for 60 wood + 20 timber (20 seconds). Each holds four passengers; no metal, cloth or housing is required. Select a transport and tap sea, or choose **Explore beyond the coast**. Sailing toward the frontier generates the next island before the ship reaches the map edge. Islands share one expanding map, with 64 cells of open water between their 120×80 generation regions; crossing takes real sailing time. Sites follow an expanding square spiral, so successive discoveries stay adjacent without a fixed island-count cap.

## Implemented roadmap

- Seeded islands with hills, rivers, biomes, clustered resources, and fog of war.
- Villager and group orders for movement, gathering, carrying, deposits, and shared construction that preserves unit escape routes.
- Completed lumber mills accept wood for automatic gathering deliveries and click-to-unload; gatherers use the nearest reachable compatible drop-off and resume work.
- Building orders in fog send the villager to explore first. A foundation is created and paid for only when the whole footprint is currently visible and placement is valid; blocked or unaffordable sites cancel without spending.
- New games start with food, wood and stone; wood can be processed into timber and food into rations. All 14 buildings whose construction uses wood, stone or timber are initially available (including Watchtower, Barracks and Range), alongside farm fields, with Forestry, Agriculture and Masonry research.
- The 17-building/13-resource catalog remains implemented. Construction unlocks follow building costs: discovered clay unlocks Workshop through bricks, fiber unlocks Infirmary through cloth, and Monument requires clay, fiber, gold, iron and coal. Affordability still controls construction and production; research retains its resource-discovery prerequisites. Existing saves retain unrestricted access.
- Farms with custom field-preparation animations and soil/cultivation/seedling/wheat sprites. Workers use the normal harvest, deposit and resume loop after preparation; field placement preserves walking and delivery routes. Existing harvesters retain their assignment while another worker replenishes the field. Depleted fields require a new paid order. Mining camps and local gathering bonuses are also implemented.
- Building task queues: one active task plus five waiting tasks, paid upfront; cancelling a waiting task refunds its full cost and releases housing.
- Completed technologies stay visible as disabled coins with a bronze laurel seal with a green check; hover or tap explains the researched upgrade. Research cannot be ordered twice.
- Wood → timber and food → rations processing on starter islands; island 2 adds iron/coal, island 3 clay, and island 4 fiber; that resource pattern repeats on later discoveries.
- Guard, archer, healer, and siege-cart production; these units currently only move and stop.
- Dock transport production through the paid task queue, water-only sailing, four passenger seats and shore landings. Passengers retain IDs, carried goods and housing usage; saves retain manifests and in-progress movement.
- Native and WebGL2 clients, mouse/touch controls, pause/speed controls, seeded reset, and SQLite saves.
- Distant zoom eases into a curved world overview; the globe shows the discovered archipelago in the same isometric orientation as the world view, with matching camera-marker and click/touch navigation.

See [ROADMAP.md](ROADMAP.md) for upcoming work and acceptance criteria, and [decisions.md](decisions.md) for design decisions.

## Architecture

| Location | Responsibility |
| --- | --- |
| `crates/game` | Deterministic simulation, commands, economy, navigation, and validation; no I/O. |
| `crates/client` | Shared wgpu renderer, input, and HUD for native and WebGL2 builds. |
| `src` | Axum/Tokio server, fixed simulation ticks, WebSocket commands/snapshots, and SQLite persistence. |
| `assets` | Painted sprites, terrain textures, UI art, and generation provenance. |
| `web` | Browser bootstrap and generated WebAssembly bindings; served at `/` and `/play`. |

Hosted games use the server's authoritative world. Native games and browser `?local` mode run the same simulation in-process. Game rules live in `crates/game`; the client handles presentation. `GameWorld::validate` checks occupancy and state invariants.

SQLite persistence applies to hosted games. Native and browser-local worlds currently live in memory and are not saved across application/page restarts.

Local unit rendering follows the fixed-step simulation accumulator. Hosted unit rendering buffers two ticks and preserves unplayed movement through delayed updates; short gaps hold position, then recover at a bounded playback rate. When remote playback falls more than 800 ms behind, it resynchronizes to the newest snapshot once and rebuilds the two-tick buffer instead of replaying stale movement. This explicit outage correction does not advance the walking animation. Walking and carrying use all authored gait poses, and facing follows movement with angular hysteresis. Work animations begin only after the displayed villager reaches its authoritative neighboring work cell, without a cosmetic positional offset.

The visual target is a sunlit Greek island diorama with painted sprites, cel shading, and tilt-shift depth of field. Building sprites remain visible when their roofs overlap the viewport, even if their ground anchors pass the near clipping plane at close zoom. See [the primary reference](assets/reference/diorama_primary.webp).

## Run locally

Requires Rust 1.85+ and a modern browser for the web client.

Native:

```bash
./scripts/run_native.sh
```

The script rebuilds changed code and launches the app. To build without launching,
run `cargo build -p aoa-client --release --locked`; the executable is
`target/release/age-of-agents-client` (`.exe` on Windows).

On Linux, install `zenity` or `kdialog` for the native reset dialog.

Browser:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
./scripts/build_web.sh
cargo run --release
```

Open <http://localhost:8000>, or <http://localhost:8000/?local> for an in-page simulation. Existing `/play` links serve the same Rust client.

Backward compatibility is currently disabled: incompatible store versions reset to a fresh world on startup. Current-version saves keep progress across restarts; Reset game erases that world. Persisted model changes bump `STORE_VERSION` in `src/store.rs`; no save migrations are maintained until explicitly requested. Saves default to `age_of_agents.db`; override with `AGE_OF_AGENTS_DB=/tmp/age-of-agents.db cargo run`. Set `AGE_OF_AGENTS_SEED` for a new world's seed, or use the Reset game dialog.

On phones and short landscape windows, action medallions sit bottom-left beside an 80px globe with pause, 1× and 2× aligned in one row above it. Selection details, wrapped queues and building submenus grow upward within the left column; choosing a building collapses the menu for placement. The painted artwork is shared with desktop.

Selected units have a broad ivory ring with a blue border around their feet, drawn above terrain and hidden by buildings. Select a unit, then tap/click ground to move or a resource to gather. Use Build for construction and select completed buildings for production/research. Repeated orders join the building’s queue; tap a coin in the queued row to cancel it and refund its cost. Shift-drag selects only the units inside the selection box; Control-click or Control-drag adds units (Command on Mac), and Shift-click replaces the selection; drag to pan, hold the mouse near an edge or corner to pan at a zoom-scaled speed, wheel to zoom toward the pointer, pinch to zoom around the fingers’ midpoint, X to stop, G for the grid, and Escape to cancel placement. Reset game erases the current world's progress. Gatherers keep chopping, mining, digging, or foraging while filling their load. Villagers carrying goods unload first before a new gather, build, or field-preparation assignment, then resume that task automatically; brief floating italic messages announce new work assignments, drop-offs and transitions to idle (walking stays quiet), and the selection status shows what comes next.

Select empty-handed units and tap a stopped transport to board; carriers unload into its hold instead. Select the ship and tap sea to sail, or tap a completed dock to return to its nearest reachable berth; X or Stop ship finishes its current step and stops. Land passengers unloads the whole manifest onto clear nearby land. Each ship carries 50 resources total in addition to four passengers. Its cargo panel is one horizontal strip, with a column for each resource present on the connected island or aboard, with resource icons, onboard quantities and generated shield buttons at each icon’s bottom corners: up to Load, down to Unload. The buttons transfer up to 10 at a time directly between the hold and island storage while stopped beside a completed dock. Cargo stays aboard on arrival. A stopped ship touching shore accepts villager deposits and makes cargo available for that island’s construction, production and research, alongside onshore stores; remaining cargo leaves with it. Buildings still provide training and research. The top bar shows the island under the cursor or touch, including nearby ships; building commands always spend their own island’s resources. Sailing shortcuts navigate to neighboring islands (preferring a completed dock) or the frontier, without teleporting. All discovered settlements keep simulating, and research remains shared. Incompatible hosted saves reset according to the current development save policy.

See [continuous map budgets](docs/CONTINUOUS_MAP.md) for memory measurements and current scaling limits.

## Contributing

Read [AGENTS.md](AGENTS.md) and [OPEN_WORK.md](OPEN_WORK.md) before starting. Keep changes small, game rules deterministic, and commands typed and atomic. Update the roadmap and handoff when behavior changes; keep design history in `decisions.md`.

Consult [the knowledge folder](docs/knowledge/INDEX.md) before changing a system. Its individual guides cover system behavior, tool/API procedures, troubleshooting and reusable learnings; update the relevant file during work, especially after developer steering. The [project-documentation skill](.agents/skills/project-documentation/SKILL.md) guides this continuing loop and can be selected automatically or invoked with `$project-documentation`. [docs/INDEX.md](docs/INDEX.md) routes the wider documentation.

Run the Rust checks:

```bash
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy -p aoa-client --target wasm32-unknown-unknown --locked -- -D warnings
```

For simulation-only iteration, use `cargo test -p aoa-game --locked`; run the full workspace suite before shipping. The test profile optimizes only `aoa-game`, retaining debug invariant checks without optimizing the renderer dependency graph. Deterministic soundness seeds run on up to four workers, preserving all eight seeds and 9,600 ticks.

For client changes, rebuild the web client and test the affected flow on desktop and phone. Apply [the code-quality review](docs/THERMONUCLEAR_REVIEW.md) before shipping; [CI](.github/workflows/deploy.yml) also checks browser bindings and current UI assets.

Asset changes must pass the relevant checks in `scripts/`. Audited villager, military, base-resource and building frames use 512×512 cells repacked from recovered sources and reviewed refinements. Run `python3 scripts/check_sprite_resolution.py` to validate all 282 audited frames. See [the asset workflow](AGENTS.md#asset-workflow) for generation and refinement, and [Midjourney tooling](docs/MIDJOURNEY.md) for setup.

Open a PR against `master`. Merges run quality checks, deploy to Modal, and verify production. Manual deployment to the production account: `MODAL_PROFILE=koogle-frick python3 scripts/modal_manage.py deploy`.

Movement regression scenario: `cargo test -p aoa-client seeded_npc_routes -- --nocapture`. See [the frame-validation scenarios](docs/MOVEMENT_VERIFICATION.md) for coverage and recovery expectations.
