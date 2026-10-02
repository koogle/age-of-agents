# Age of Agents — Repository Context

## Product

Age of Agents is a deliberately small, mobile-friendly 2D isometric real-time strategy game inspired by late-1990s and early-2000s RTS games. The near-term reference is the clarity and immediacy of an early Age of Empires vertical slice, not a full simulation.

Despite the project name, the initial game contains **no LLM-controlled or autonomous AI agents**. In code and product language, use **villager**, **unit**, or **NPC** for game entities. Idle units remain idle until the player commands them.

## Milestone 1

The current playable demo proves these loops:

1. Isometric terrain tiles render cleanly on desktop and mobile.
2. A player can command a villager through typed gathering, bounded carrying, town-center deposits, and deterministic resumption across seven biome-compatible resources.
3. A player can construct a town center and train villagers through its single authoritative job slot.
4. A player can research five bounded gathering improvements through that same job slot.

Keep the world deterministic and small. Do not add combat, pathfinding frameworks, autonomous task selection, LLM calls, multiplayer, or generalized engine abstractions before this milestone is excellent.

## Architecture

- **Backend:** Rust, Axum, Tokio.
- **Authority:** The Rust server owns all world state and advances a fixed-timestep simulation.
- **Transport:** WebSocket typed commands, command acknowledgements, and full world snapshots. `GET /state` exists for debugging.
- **Persistence:** SQLite stores the authoritative world snapshot. Respect `AGE_OF_AGENTS_DB` everywhere.
- **Frontend:** Fullscreen WebGL canvas rendered with Three.js (vendored as one ES module under `frontend/vendor/`), plain ES modules, no build step. Three.js is the only framework. World assets (villagers, buildings, trees, resources, ground) come from generated high-quality renders (FAL/Midjourney) used as painted sprites and textures in the 3D world; procedural code geometry is only a temporary placeholder until a generated asset replaces it.
- **Rendering:** Three-quarter perspective camera with pan, zoom, and rotation. Soft two-tone cel shading with warm ink linework, a tilt-shift post-process, and one shared fog-of-war shader field (unexplored land lies under cloud). The interface is painted inside the WebGL canvas (an orthographic overlay of canvas-textured panels with hit regions); a visually hidden DOM mirror of every button keeps keyboard and screen-reader access.
- **Spatial authority:** Every unit, step target, building footprint, and live resource exclusively claims its cells; move destinations are reservations. `GameWorld::validate` must hold after every command and tick.
- **Deployment:** Modal. Verify locally before deploying.

## Interaction Contract

- Tap/click a villager to select it.
- Tap/click a resource with a villager selected to issue a gather order.
- Tap/click a foundation with villagers selected to have them help build it.
- Villagers carry at most 20 typed units, deposit at a town center, and resume unfinished gathering.
- Gathering is phase-driven: villagers wait at the node until full or depleted, and combined activity sprites replace duplicate unit-plus-resource rendering.
- Tap/click the build button, then valid ground, to issue a build order.
- Tap/click a town center to train a villager or start available research through the medallion buttons that appear at the bottom center.
- Drag pans. Wheel/pinch zooms. Right-drag, two-finger twist, or Q/E rotates.
- Mouse and touch semantics must match.
- Simulation speed is authoritative and controlled through 0×, 1×, and 2× buttons.
- A busy villager rejects replacement orders in Milestone 1; this avoids cancellation/refund complexity.

## Art Direction

The target is a soft 3D tilt-shift diorama of a sunlit Greek island. `assets/reference/diorama_primary.webp` is the primary reference; the older `mediterranean_*.webp` images only support the palette.

- Studio Ghibli-leaning cel look: soft two-tone toon ramp (no hard multi-band posterization), a warm rim light, thin warm-brown ink outlines whose width stays constant on screen, and a painted brush-stroke ground. The look is driven by rendering, palette, painted textures and sprites more than by model detail.
- Tilt-shift depth of field, puffy cumulus clouds, a distant snow-capped volcano, ships on a deep blue-teal sea.
- White limestone and marble, terracotta roofs, dark cypresses, olive trees, yellow-green and ochre land; striking accents against soft pastel-leaning tones.
- The interface is mostly hidden: a round globe minimap, a small speed pill, a resource pill listing only what the player has, and glossy round medallion buttons that appear only when something is selected. Icons and ornament should come from generated art (FAL/Midjourney), not hand-drawn code shapes.
- Readability at actual gameplay size matters more than close-up detail. No photorealism, text in images, or inconsistent character identity between animation frames.

## Engineering Rules

- Prefer subtraction and direct code over generalized machinery.
- A feature must earn its complexity in the current milestone.
- Keep domain logic deterministic and testable without the server.
- Use typed enums at command boundaries; avoid stringly typed optional-field command bags.
- Reject invalid commands without partially mutating state.
- Build costs are reserved exactly once and completed buildings appear exactly once.
- Corrupt persisted state is an error, not permission to silently reset the world.
- Keep frontend files under 1,000 lines; aim much lower.
- Do not use emojis in the game UI.
- Run the thermonuclear review in `docs/THERMONUCLEAR_REVIEW.md` before shipping meaningful changes.

## Workflow

- At the start of every work session, read `OPEN_WORK.md` before acting.
- Keep `OPEN_WORK.md` current after meaningful milestones, blocker changes, and before commit/push/deploy or ending a session. It is a compact current-state handoff, not an append-only diary.
- Parent-session work may proceed directly on `master` for this solo project.
- Every delegated subagent must work on its own feature branch or git worktree, push that branch, and open a pull request for Jakob to review. Subagents must never commit directly to `master`.
- A subagent PR must describe its scope, verification performed, generated assets, and any known limitations. Do not merge it automatically.
- After changes: format, test, lint, perform the thermonuclear review, verify the browser demo, and redeploy Modal.
- Keep `README.md` and `ROADMAP.md` synchronized with actual behavior.
