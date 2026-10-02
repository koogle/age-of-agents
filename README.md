# Age of Agents

A deliberately small, mobile-first 3D RTS vertical slice built with a Rust authoritative server, a fullscreen WebGL (Three.js) frontend, WebSocket state streaming, and SQLite persistence.

The current vertical slice is intentionally bounded: command villagers through a seven-resource gather/carry/deposit economy, construct town centers, train villagers, and research five gathering improvements. There are no LLM agents or autonomous NPC policies. Villagers remain idle until commanded.

## Milestone 1

- Persistent deterministic 30×20-cell Voronoi-style island with eight connected biomes
- Server-authoritative fog with visible, explored-dim, and unseen-dark terrain
- Selectable villagers
- Biome-compatible wood, food, stone, gold, iron, clay, and fiber gathering
- Bounded villager cargo with explicit return and town-center deposit phases
- Command-driven construction of one building type through 2×2 foundations that several villagers can raise together
- Starting town-center base with single-slot villager production
- Seven typed shared stockpiles and a five-technology gathering tree
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
WebGL client (Three.js, vendored)
  ├─ pointer/touch input and camera rig
  ├─ procedural cel-shaded 3D models and animation
  └─ WebSocket commands/snapshots
              │
              ▼
Rust + Axum server
  ├─ deterministic game domain
  ├─ fixed timestep
  ├─ command validation
  └─ SQLite snapshot persistence
```

The server owns the world. The browser renders snapshots and sends player intent; it does not simulate authoritative outcomes.

### World soundness

The world is a 30×20 grid of cells. One derived occupancy map (`src/game/occupancy.rs`) is the single source of truth for who owns which cell:

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

```bash
cargo run
```

Open <http://localhost:8000>.

The default SQLite file is `age_of_agents.db`. Override it with:

```bash
AGE_OF_AGENTS_DB=/tmp/age-of-agents.db cargo run
```

## Controls

- **Select:** tap/click a villager to replace the selection. On touch, long-press a villager to add or remove it. On desktop, Shift-drag from empty ground box-selects visible villagers, and Shift-click adds or removes. Ordinary mouse and touch drags continue to pan. Tap/click a town center to select it instead.
- **Move:** with one or more villagers selected, tap/click empty ground. Groups receive one atomic authoritative order and spread across distinct reachable cells.
- **Gather:** with villagers selected, tap/click a resource. Villagers walk beside it, gather two units per second, wait for a full 20-unit load unless the node depletes, deposit at the nearest town center, and resume until depletion.
- **Build:** select a villager, press the build button (or B), then tap/click ground. A 2×2 foundation appears immediately and rises as the villager works. Tap a foundation with other villagers selected to have them help.
- **Produce:** select a town center and press the **Train villager** medallion. It reserves 50 food and produces one villager over six seconds; each building has one active production slot.
- **Research:** select a town center and press an available technology medallion (hover for its name and cost). Research reserves 40 food and 20 wood, occupies the building for eight seconds, and improves matching gather rates by 20%.
- **Pan:** drag with one pointer, WASD/arrow keys, or tap the minimap.
- **Zoom:** pinch or use the mouse wheel.
- **Rotate:** two-finger twist, right-drag, or Q/E.
- **Recover view:** reload to center the camera on the currently visible villagers.
- **Reset world:** press **Reset world** and confirm to erase progress and restore the deterministic starting state.
- **Simulation speed:** use **0×**, **1×**, or **2×** in the top bar to pause or change authoritative simulation speed.
- **Cancel build placement:** press the cancel button or Escape.

Mouse and touch use the same command semantics.

Schema version 5 stores units as exclusive cell claims and buildings as footprints. Older persisted worlds (free-floating positions) are intentionally dropped because they cannot be translated into the claim model safely.

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

The visual target is a soft 3D tilt-shift diorama of a sunlit Greek island (`assets/reference/diorama_primary.webp`): soft two-tone cel shading (`frontend/materials.js`) with warm ink outlines and a painted brush-stroke ground, a tilt-shift depth-of-field pass (`frontend/tilt-shift.js`), puffy clouds, a distant volcano and sailing ships (`frontend/sky.js`), turquoise sea, limestone, cypresses, olive trees, and a marble town center with a terracotta roof. Models are procedural code in `frontend/models.js`; generated GLB models and icons from FAL are being evaluated on a separate branch.

The interface is painted inside the WebGL canvas (`frontend/hud.js`, `frontend/ui-layer.js`) and stays mostly out of the way: a round globe minimap, a small speed pill, a resource pill that lists only what you have, and glossy medallion buttons that appear at the bottom center only when something is selected. A visually hidden DOM mirror keeps every command reachable by keyboard and screen reader.

The fog of war is one shared shader field: unexplored land lies under a bank of soft cloud, and explored-but-unwatched land is muted. Terrain heights come from fixed noise rather than biome data, so the shape of the land never leaks unexplored information.

The 2D sprites under `assets/game/` are no longer used by the client.
