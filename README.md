# Age of Agents

A Greek strategy roguelike inspired by Age of Empires and Anno. Build an island settlement, grow its economy, and explore further islands while trying to survive an increasingly hostile world.

The current prototype is an island-exploration RTS economy with a shared Rust simulation and native/browser clients.

## Proposed gameplay loop

Start on an island, gather resources, and build a settlement. Build ships to explore and settle more islands, find new resources and treasures, and research better tools and buildings. Each run plans 5–7 islands spread across the sea; the farthest holds a temple with the Artifact of the Gods. The rest of the world stays under fog: only the temple is marked on the globe from the start, and each island generates as you reach it.

Time keeps moving as you expand. Planned threats include pirates and mythical creatures, followed by increasingly destructive calamities. Runs are expected to end in defeat. Treasures and monuments unlock permanent upgrades, including new research and ways to skip early setup on later runs.

Wildlife combat is implemented; broader combat, calamities and permanent progression are not implemented yet.

## Implemented roadmap

- Seeded islands with varied outlines and open bays, relief-derived mountains and downhill rivers, biomes, clustered resources and fog of war.
- Renewable riverbank water collection; fields consume water when planted or replenished.
- Villager and group orders for movement, gathering, carrying and construction.
- Straight dirt/stone roads with labour costs and 50% faster friendly movement.
- 17 buildings, 14 resources/products, farming with granary yield bonuses, processing, research and production queues.
- Buildings unlocked when discovered resources support construction and productive use.
- Transport ships with four passenger seats and cargo holds; continuous sailing between persistent islands with local inventories.
- A seeded 5–7 island archipelago per run under fog, with only the temple island marked on the globe; the temple, artifact and run victory are not implemented yet.
- Territorial wolves, bears and boars, unit health and hunting orders.
- Guard, archer, healer and siege-cart production; ranged combat and healing remain future work.
- Native and WebGL2 clients, mouse/touch controls, pause/speed controls and seeded reset.
- SQLite saves for hosted games; native and browser-local games are in-memory.

See [ROADMAP.md](ROADMAP.md) for upcoming work and [decisions.md](decisions.md) for design choices.

## Architecture

| Location | Responsibility |
| --- | --- |
| `crates/game` | Shared deterministic simulation and game rules. |
| `crates/client` | wgpu renderer, input and HUD for native and WebGL2. |
| `src` | Axum/Tokio server, WebSocket commands and SQLite persistence. |
| `assets` | Painted sprites, terrain textures, UI art and provenance. |
| `web` | Browser bootstrap and generated WebAssembly bindings. |

Hosted games use the server's authoritative world. Native games and browser `?local` mode run the same simulation in-process.

## Run locally

Requires Rust 1.85+ and a modern browser for the web client.

Native:

```bash
./scripts/run_native.sh
```

On Linux, install `zenity` or `kdialog` for the native reset dialog.

Browser:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
./scripts/build_web.sh
cargo run --release
```

Open <http://localhost:8000>, or <http://localhost:8000/?local> for an in-page simulation.

Hosted saves default to `age_of_agents.db`; override with `AGE_OF_AGENTS_DB`. Incompatible save versions reset during development. Set `AGE_OF_AGENTS_SEED` for a new world's seed, or use Reset game.

Select a unit, then click/tap ground to move or a resource to gather. Select a ship and tap the sea, fog included, to steer it; discovered islands, home first, are one tap away. Use Build for construction and select completed buildings for production/research. Drag to pan, wheel/pinch to zoom, X to stop, G for the grid and Escape to cancel placement.

## Contributing

Read [AGENTS.md](AGENTS.md), [OPEN_WORK.md](OPEN_WORK.md) and the relevant [system guides](docs/knowledge/INDEX.md) before starting.

```bash
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy -p aoa-client --target wasm32-unknown-unknown --locked -- -D warnings
```

For client changes, rebuild the web client and test on desktop and phone. Apply [the code-quality review](docs/THERMONUCLEAR_REVIEW.md) before shipping. See [build and release](docs/knowledge/build-integration-and-release.md) for deployment and [the asset workflow](AGENTS.md#asset-workflow) for art changes.
