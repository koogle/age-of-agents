# Current handoff: persistent islands (2026-10-04)

## Implemented on `feat/island-discovery`

- Based on merged transport PR #71 (`0864057`). Fresh island PR pending creation; do not merge without fresh authorization.
- First completed transport generates island 2 exactly once. Existing ships in old saves can discover it on departure. Explicit previous/next voyage controls carry a stopped ship, its passengers and goods; frontier voyages generate further deterministic islands without a fixed cap.
- Discovery-order IDs and root-seed mixing; persistent terrain, fog, resources, buildings, units, ships, orders and island-local inventories. Away islands pause; research, discovered unlocks and entity counters are global. Cross-island duplicate research is rejected.
- Complementary, biome-appropriate reachable deposits: island 2 iron/coal, island 3 clay, island 4 fiber; repeat thereafter. Initial arrival is beside open ocean and a shore connected to resources.
- Shore unloading supports founding from transported wood (town center 20, dock 30); loading still requires a completed dock. Voyages are atomic and reject moving ships, unreachable ocean, invalid destinations and blocked arrival.
- Shared Rust mouse/touch controls show current island/count, previous/next voyage buttons and discovery details. Camera and terrain interpolation reset on island changes. Existing sprites reused; no new asset generation needed.

## Verification / release

- Seven focused domain tests cover deterministic complementary generation/biomes, repeated discoveries, persistent round trips/reload, separate inventories, four settlers founding a town center, cross-island research, old saves, corrupt archives, atomic rejection, and occupied ocean corners.
- All 178 workspace tests passed (13 server, 47 client, 118 domain). Strict native/WASM clippy, formatting, 270-frame asset audit, transport checks and icon normalization passed.
- Desktop 1280×800 and touch 390×844/DPR2 actual browser flow: board → discover/voyage → land → unload → return → revisit → discover island 3. No page errors. Both flows passed again after final generation/research refinements. SQLite contains all three discovered maps and both prior inventories.
- QA uses isolated databases under `/workspace/scratch/islands`; screenshots and flow logs are there. Rebuilt tracked WASM bundle.
- Thermonuclear review: authority stays in Rust domain, commands use existing atomic clone boundary, archives are explicit local data (no recursive worlds), all source files remain below 1,000 lines. Persistence validates archived maps and global identities/research.
- Modal deployment attempted; rejected because this environment has no Modal token. No production deployment of this branch occurred. Transport #71's merge/deploy workflow completed successfully; this feature awaits its own PR merge/deployment.

## Remaining roadmap

- Cumulative archipelago globe / destination selection beyond adjacent island buttons. Current globe displays the active map.
- Away-island simulation (currently paused), separate inventories for multiple settlements on the same island, trading-post specialization and trade routes.
- Improve accessible DOM mirror and touch controls (A3); the current canvas HUD is not a DOM mirror.
- Combat, adversaries, calamities, upgrades and other later milestones remain in ROADMAP.md. No autonomous NPC behavior was added.
