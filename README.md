# Age of Agents

A Greek strategy roguelike inspired by Age of Empires and Anno. Build an island settlement, grow its economy, and explore further islands while trying to survive an increasingly hostile world.

The current prototype is a single-island RTS economy with a shared Rust simulation and native/browser clients.

## Proposed gameplay loop

Start on an island, gather resources, and build a settlement. Build ships to explore and settle more islands, find new resources and treasures, and research better tools and buildings. The long-term world keeps expanding as you discover islands, without a fixed island limit.

Time keeps moving forward and bring with it dangers and challenges. Runs are expected to end in defeat. Treasures and monuments unlock permanent upgrades, including new research and ways to skip early setup on later runs. The balance still needs work: losses should make the next run interesting without making rebuilding tedious.

Ships, multiple islands, local inventories, combat, calamities, and permanent upgrades are not implemented yet. The planned first transport costs wood and timber, with no metal or cloth prerequisite.

Fresh games currently end at the settlement/timber economy; there is no departure option yet.

## Implemented roadmap

- Seeded islands with hills, rivers, biomes, clustered resources, and fog of war.
- Villager and group orders for movement, gathering, carrying, deposits, and shared construction that preserves unit escape routes.
- New games start with food, wood and stone; timber is their only processed resource. Town center, house, granary, farm/fields, lumber mill and dock are initially available, with Forestry, Agriculture and Masonry research.
- The 17-building/13-resource catalog remains implemented. Advanced industries and research unlock through discovery of their raw inputs; existing saves retain unrestricted access. New-game rations and healing production are deferred.
- Farms with harvestable, manually replenished fields and dedicated soil/cultivation/seedling/wheat sprites; mining camps and local gathering bonuses.
- Building task queues: one active task plus five waiting tasks, paid upfront; cancelling a waiting task refunds its full cost and releases housing.
- Wood → timber processing on starter islands; steel, bricks and cloth await future destination resources. Existing unrestricted saves retain all five processing chains, including rations.
- Guard, archer, healer, and siege-cart production; these units currently only move and stop.
- Native and WebGL2 clients, mouse/touch controls, pause/speed controls, seeded reset, and SQLite saves.
- Distant zoom eases into a curved world overview; explored islands will be added after ships and persistent island travel are implemented.

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

Existing saves keep their terrain, resources, buildings and economy rules; choose Reset game only when you want a fresh starter island (this erases that world). Saves default to `age_of_agents.db`; override with `AGE_OF_AGENTS_DB=/tmp/age-of-agents.db cargo run`. Set `AGE_OF_AGENTS_SEED` for a new world's seed, or use the Reset game dialog.

Selected units have a broad ivory ring with a blue border around their feet, drawn above terrain and hidden by buildings. Select a unit, then tap/click ground to move or a resource to gather. Use Build for construction and select completed buildings for production/research. Repeated orders join the building’s queue; tap a coin in the queued row to cancel it and refund its cost. Shift-click or Shift-drag adds units on desktop; drag to pan, hold the mouse near an edge or corner to pan gently, wheel to zoom toward the pointer, pinch to zoom around the fingers’ midpoint, X to stop, G for the grid, and Escape to cancel placement. Reset game erases the current world's progress. Gatherers keep chopping, mining, digging, or foraging while filling their load. Villagers carrying goods unload first before a new gather, build, or field-preparation assignment, then resume that task automatically; a brief floating italic message announces the resource being dropped off, and the selection status shows what comes next.

## Contributing

Read [AGENTS.md](AGENTS.md) and [OPEN_WORK.md](OPEN_WORK.md) before starting. Keep changes small, game rules deterministic, and commands typed and atomic. Update the roadmap and handoff when behavior changes; keep design history in `decisions.md`.

Run the Rust checks:

```bash
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy -p aoa-client --target wasm32-unknown-unknown --locked -- -D warnings
```

For client changes, rebuild the web client and test the affected flow on desktop and phone. Apply [the code-quality review](docs/THERMONUCLEAR_REVIEW.md) before shipping; [CI](.github/workflows/deploy.yml) also checks browser bindings and current UI assets.

Asset changes must pass the relevant checks in `scripts/`. Audited villager, military, base-resource and building frames use 512×512 cells repacked from recovered sources and reviewed refinements. Run `python3 scripts/check_sprite_resolution.py` to validate all 268 audited frames. See [the asset workflow](AGENTS.md#asset-workflow) for generation and refinement, and [Midjourney tooling](docs/MIDJOURNEY.md) for setup.

Open a PR against `master`. Merges run quality checks, deploy to Modal, and verify production. Manual deployment: `python3 scripts/modal_manage.py deploy`.
