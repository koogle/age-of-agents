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
