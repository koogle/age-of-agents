# Decisions

Each entry records a design choice in one or two sentences. The current gameplay direction lives in [README.md](README.md#proposed-gameplay-loop).

- **Game direction:** Build a Greek strategy roguelike around island expansion, escalating threats, eventual defeat, and permanent progression. Fixed win-condition scenarios were an earlier proposal; survival runs are now the direction.
- **Player control:** Villagers and friendly units wait for orders; there are no LLM-controlled game entities. A gather order may continue onto nearby resources of the same kind.
- **Shared simulation:** Keep deterministic game rules in `crates/game`, shared by hosted, native, and browser-local games. Rendering never decides gameplay outcomes.
- **Rust client:** One wgpu client serves native and WebGL2 builds. The retired Three.js client and its asset pipeline are removed; `/` and `/play` both serve the Rust client.
- **Scope:** Prefer small, playable slices and direct typed data over a generic engine, ECS, recipe language, or broad technology matrix.
- **Spatial rules:** Buildings, live resources, units, and step targets claim exclusive cells; move destinations are reservations. Eight-neighbor routing cannot cut occupied corners.
- **Orders and costs:** Reject invalid commands atomically and reserve costs once. Stop or replacement orders keep cargo and construction progress.
- **Production:** Each building runs one task and holds up to five paid waiting tasks in submission order; cancelling a waiting task refunds its full cost. Unit production waits for a free adjacent cell, and both active and waiting trainees reserve housing.
- **Persistence:** Save authoritative snapshots in SQLite and respect `AGE_OF_AGENTS_DB`. Corrupt state is an error; incompatible historical schemas are intentionally discarded rather than translated unsafely.
- **Current economy:** Direct gathering remains available alongside farm/mining-camp drop-offs and non-stacking local bonuses. Processing costs and quantities are initial balance values.
- **Fields:** Planting and replenishing fields require paid, explicit villager orders. Preparation workers continue into harvesting; exhausted fields never replenish automatically and idle villagers never replant.
- **Starter economy:** New islands supply food, wood and stone, with timber as the sole processed resource. Reserve metal, brick and cloth chains for complementary later islands; defer rations until provisioning is playable.
- **First departure:** The planned first transport consumes wood and timber only, so departure cannot depend on resources found elsewhere. Later trading posts need separate inventories and explicit shipping.
- **Island progression:** Plan transport and persistent destination islands before local inventories and inter-island trade. The long-term world keeps expanding through discovered islands without a fixed island limit; the prototype currently uses one island and shared stockpiles.
- **Globe overview:** Retain distant planet zoom during exploration. Add newly discovered islands cumulatively after ships and persistent island travel are implemented.
- **Camera:** Use a fixed orthographic isometric angle with pan and zoom. Building art keeps its proportions within level footprint plots.
- **Art:** Aim for a painted Greek island diorama with soft cel shading, fine ink lines, and tilt-shift depth of field. Generated sprites and textures replace procedural placeholders.
- **Asset workflow:** Generate with FAL, then upscale and refine with ChatGPT’s image tool to remove diffusion artifacts while preserving the approved style. Models, provenance, and checks are documented in [AGENTS.md](AGENTS.md#asset-workflow).
- **Sprite quality:** Require at least 512×512 authored pixels for every action, facing, and construction frame. Upscaling and DPI metadata do not recover missing source detail.
- **Deployment:** Host on Modal with quality checks and production verification after merges to `master`.

- **Local transport:** A dock trains a transport for 60 wood + 20 timber in 20 seconds; four passengers retain IDs, cargo and housing usage, while a separate 200-good hold transfers at completed docks. Ships land passengers at free connected shore cells without needing a dock; boarding orders reserve seats and departure cancels unfinished boarding.
- **Destination contract:** Future island IDs will derive from deterministic discovery order, with destination generation once on first transport completion and travel on departure. This slice keeps the existing island/stockpile; passengers are owned by either the land-unit list or one ship manifest, never both.

- Persistent islands: discovery-order IDs and mixed root seeds identify generated destinations; explicit voyages exchange local terrain, fog, entities and inventory while research and counters stay global. Away islands pause, and unload-at-shore permits founding an outpost from transported goods; loading still requires a dock.

- **Simplified transport and economy:** Transports carry up to four units only; all islands use one shared stockpile for deposits and spending, while away islands remain paused. Existing saved island inventories and ship holds are pooled on load; villagers retain personal carried loads until deposited.

- **Compact HUD:** Preserve painted medallions and parchment while aligning phone actions and globe/speed controls in one bottom band. Expand queues and submenus upward only when needed, and collapse construction choices during placement.

- 2026-10-05: Islands occupy one persistent coordinate space, with 64-cell ocean gaps and deterministic adjacent discovery along a growing square spiral; ships sail continuously and all discovered settlements simulate against the shared stockpile. Retain layouts in memory for now, bound ground geometry to the camera, compress repeated snapshot terrain and profile populated worlds before introducing streaming.
- **Minimap orientation:** Project the full map using the fixed world camera’s ground axes; terrain sampling, camera marker and click/touch navigation use the same invertible mapping.
