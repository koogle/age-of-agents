# Age of Agents — Repository Context

## Product

Read [README.md](README.md), especially **Proposed gameplay loop**, at the start of every session. It is the source of truth for the Greek strategy roguelike direction; distinguish proposed features from the implemented roadmap. Read [decisions.md](decisions.md) for design choices and keep new entries to one or two sentences.

Age of Empires informs the RTS controls; Anno informs the settlement economy.

Despite the project name, the initial game contains **no LLM-controlled or autonomous AI agents**. In code and product language, use **villager**, **unit**, or **NPC** for game entities. Idle units remain idle until the player commands them.

## Milestone 1

The current playable demo proves these loops:

1. Isometric terrain tiles render cleanly on desktop and mobile.
2. A player can command a villager through typed gathering, bounded carrying, town-center deposits, and deterministic resumption across eight biome-compatible raw resources.
3. A player can construct a town center and train villagers through its authoritative task queue.
4. A player can research five bounded gathering improvements through that same task queue.

Keep the simulation deterministic. The long-term world should keep expanding as the player discovers islands, without a fixed island limit; the current single-island map is a prototype constraint. Treat combat, island expansion, calamities, and permanent progression as future roadmap work, not existing behavior. Do not add pathfinding frameworks, autonomous task selection (beyond a gatherer moving on to the next node of the same kind), LLM calls, multiplayer, or generalized engine abstractions before this milestone is excellent.

## Architecture

- **Backend:** Rust, Axum, Tokio.
- **Authority:** The Rust server owns all world state and advances a fixed-timestep simulation.
- **Transport:** WebSocket typed commands, command acknowledgements, and full world snapshots. `GET /state` exists for debugging.
- **Persistence:** SQLite stores the authoritative world snapshot. Respect `AGE_OF_AGENTS_DB` everywhere.
- **Client:** Rust. `crates/client` is one wgpu renderer that runs as a native window (`cargo run -p aoa-client`, simulation in-process) and as WebGL2 in the browser (`scripts/build_web.sh`, served at `/` and `/play`). `crates/game` is the shared deterministic simulation; keep game logic there, never in the client. World assets (villagers, buildings, trees, resources, ground) come from generated high-quality renders (FAL/Midjourney) used as painted sprites and textures in the 3D world; procedural code geometry is only a temporary placeholder until a generated asset replaces it.
- **Rendering:** Fixed-angle orthographic isometric camera with pan and zoom; building anchors stay fixed in world space. Building sprites retain their original proportions and fit within rectangular cobblestone plots; rendered foundation plots are level, with touching plots sharing one height. A building's depth follows its footprint's front edge per screen column, and a villager hidden behind a building shows as a flat team-blue silhouette, as in Age of Empires II. Soft two-tone cel shading, a fine one-pixel ink-line finish pass (depth creases and silhouettes), a tilt-shift post-process, and one shared fog-of-war shader field (unexplored land lies under cloud). The interface is painted inside the WebGL canvas (an orthographic overlay of canvas-textured panels with hit regions); a visually hidden DOM mirror of every button keeps keyboard and screen-reader access.
- **Grid scale:** simulation cells are finer than a villager is tall (currently 120×80 cells, half a world unit each; the client converts with `terrain::CELL`). Resources cluster tightly, as in Age of Empires.
- **Spatial authority:** Every unit, step target, building footprint, and live resource exclusively claims its cells; move destinations are reservations. `GameWorld::validate` must hold after every command and tick.
- **Deployment:** Modal. Verify locally before deploying.

## Interaction Contract

- Tap/click a villager to select it.
- Tap/click a resource with a villager selected to issue a gather order.
- Tap/click a foundation with villagers selected to have them help build it.
- Tap/click a town center (or a granary for food and fiber, or a lumber mill for wood) with villagers selected to have those carrying goods unload there. A villager holding goods shows the carry pose, even when stopped, except while actively gathering; partial loads must keep the work animation.
- Villagers carry at most 20 typed units, deposit at the nearest reachable compatible completed building (including lumber mills for wood), and resume unfinished gathering. When a node runs out they move on to the nearest reachable node of the same kind within 10 cells of it, else go idle. A villager holding goods finishes that load or drops it off first: given a new gather, build, or field-preparation assignment, it delivers its cargo to a compatible completed drop site before starting, including partial loads of the same kind. It retains and resumes the new assignment automatically.
- Gathering is phase-driven: villagers wait at the node until full or depleted, and combined activity sprites replace duplicate unit-plus-resource rendering.
- With villagers selected, a ring under the pointer previews the tap: gold over a resource or foundation, white over ground.
- Tap/click the build button to open the grouped building menu (all 17 catalog buildings plus farm fields; unaffordable ones in greyscale), pick one, then tap valid ground to issue a build order.
- Tap/click a town center to train a villager or start available research through the medallion buttons that appear at the bottom center.
- Left-drag or touch drag pans. While placing, drag the ghost and release to build. Wheel/pinch zooms. Use the Grid pill (or G) to toggle square cells shown as diamonds; placement always previews its rectangular footprint. The camera heading is fixed; right-drag, two-finger twist, and Q/E do not rotate it.
- Mouse and touch semantics must match.
- Simulation speed is authoritative and controlled through 0×, 1×, and 2× buttons.
- A new order replaces a villager's current task, and the Stop medallion (or X) idles it. Either way it finishes its current step, keeps cargo, and leaves foundation progress, so there is nothing to refund. A rejected order leaves the old task untouched.

## Art Direction

The target is a soft 3D tilt-shift diorama of a sunlit Greek island. `assets/reference/diorama_primary.webp` is the primary reference; the older `mediterranean_*.webp` images only support the palette.

- Studio Ghibli-leaning cel look: soft two-tone toon ramp (no hard multi-band posterization), a warm rim light, fine one-pixel pen lines on every silhouette and crease (as in `mediterranean_4.webp`), and painted generated ground textures. The look is driven by rendering, palette, painted textures and sprites more than by model detail.
- Tilt-shift depth of field, puffy cumulus clouds, a distant snow-capped volcano, ships on a deep blue-teal sea.
- White limestone and marble, terracotta roofs, dark cypresses, olive trees, yellow-green and ochre land; striking accents against soft pastel-leaning tones.
- The interface is mostly hidden: a round globe minimap, a small speed pill, a resource pill listing only what the player has, and glossy round medallion buttons that appear only when something is selected. Icons and ornament should come from generated art (FAL/Midjourney), not hand-drawn code shapes.
- Readability at actual gameplay size matters more than close-up detail. No photorealism, text in images, or inconsistent character identity between animation frames.

## Asset Workflow

- Generate sprites, textures, and UI art with FAL `fal-ai/nano-banana` and `fal-ai/nano-banana/edit`, using approved game art as the style reference. Use `fal-ai/birefnet/v2` for cutouts; `fal-ai/esrgan` is the upscaler used for the loading title, and `fal-ai/ideogram/v3` generated its lettering.
- Treat initial generations as drafts that need an upscaling and cleanup pass. Use ChatGPT's image tool (`OpenAI image_gen`) for refinement, as with the building walls, roof tiles, and cobblestone; remove diffusion artifacts, broken geometry, stray details, and cutout fringes before integration.
- Preserve the same palette, linework, lighting, camera, character identity, scale, and ground anchor across assets and animation/construction frames. Refine against approved sources rather than changing the style for each asset.
- Keep original renders and record the model, references, request IDs, and edits alongside the asset. Pack from the highest-quality source into at least 512×512 sprite frames; prefer original high-resolution detail when available, and do not treat a larger canvas or DPI change as recovered detail.
- Run `python3 scripts/check_sprite_resolution.py` and the relevant asset checks, then inspect maximum-zoom desktop and DPR-2 phone gameplay. `--report-only` inventories existing resolution gaps; it is not a passing integration check.

Provenance: [sprites](assets/sprites/README.md), [building refinements](assets/sprites/building_sources/provenance.json), [terrain](assets/terrain/README.md), [UI](assets/ui/README.md), and [loading art](assets/loading/README.md).

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

- For any visual change (sprites, textures, UI, layout, or rendering), show relevant images directly in the chat alongside status updates to the developer as soon as a preview is available, and include the final visual result when finishing. Use current asset previews or in-game screenshots, label drafts and before/after comparisons clearly, and do not wait for the developer to ask to see the changes.
- At the start of every work session, read `OPEN_WORK.md` before acting.
- Keep `OPEN_WORK.md` current after meaningful milestones, blocker changes, and before commit/push/deploy or ending a session. It is a compact current-state handoff, not an append-only diary.
- Parent-session work may proceed directly on `master` for this solo project.
- Every delegated subagent must work on its own feature branch or git worktree, push that branch, and open a pull request for Jakob to review. Subagents must never commit directly to `master`.
- A subagent PR must describe its scope, verification performed, generated assets, and any known limitations. Do not merge it automatically.
- After changes: format, test, lint, perform the thermonuclear review, verify the browser demo, and redeploy Modal.
- Keep `README.md` and `ROADMAP.md` synchronized with actual behavior.
