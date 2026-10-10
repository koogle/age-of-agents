# Age of Agents — Repository Context

## Product

Read [README.md](README.md), especially **Proposed gameplay loop**, at the start of every session. It is the source of truth for the Greek strategy roguelike direction; distinguish proposed features from the implemented roadmap. Read [decisions.md](decisions.md) for design choices and keep new entries to one or two sentences.

Age of Empires informs the RTS controls; Anno informs the settlement economy.

Despite the project name, the initial game contains **no LLM-controlled or autonomous AI agents**. In code and product language, use **villager**, **unit**, or **NPC** for game entities. Idle friendly units remain idle until the player commands them, except for one short step aside when they stand on another unit's reserved destination or block a stalled unit's route; hostile wildlife can pursue nearby units within its territory.

## Milestone 1

The current playable demo proves these loops:

1. Isometric terrain tiles render cleanly on desktop and mobile.
2. A player can command a villager through typed gathering, bounded carrying, town-center deposits, and deterministic resumption across nine raw resources, including renewable riverbank water.
3. A player can construct a town center and train villagers through its authoritative task queue.
4. A player can research five bounded gathering improvements through that same task queue.

Keep the simulation deterministic. Each run plans 5–7 islands from its seed and generates each as the player discovers it; the farthest site is the temple island, the only site marked on the globe before discovery; everything else stays under fog. Islands share one map and all discovered settlements simulate; inventories are island-local, supplemented by cargo in stopped shore ships (50 total resources, plus four passengers). Research remains shared. Territorial wolves/bears/boars, lion prides and explicit wildlife contact attacks are implemented; see [wildlife](docs/knowledge/wildlife.md). Broader combat, calamities and permanent progression remain future roadmap work. Do not add pathfinding frameworks, autonomous task selection (beyond a gatherer moving on to the next node of the same kind), LLM calls, multiplayer, or generalized engine abstractions before this milestone is excellent.

## Architecture

- **Backend:** Rust, Axum, Tokio.
- **Authority:** The Rust server owns all world state and advances a fixed-timestep simulation.
- **Transport:** WebSocket typed commands, command acknowledgements, and full world snapshots. `GET /state` exists for debugging.
- **Persistence:** Hosted SQLite stores the authoritative world snapshot; native/browser-local worlds are currently in-memory. Respect `AGE_OF_AGENTS_DB` on every hosted persistence path. Backward compatibility is not required until Jakob explicitly requests it: remove obsolete save models/migrations, bump `STORE_VERSION` for incompatible changes, and reset the stored world on version mismatch. Preserve matching-version saves and report corrupt current snapshots as errors; see [server and saves](docs/knowledge/server-and-saves.md).
- **Client:** Rust. `crates/client` is one wgpu renderer that runs as a native window (`cargo run -p aoa-client`, simulation in-process) and as WebGL2 in the browser (`scripts/build_web.sh`, served at `/` and `/play`). `crates/game` is the shared deterministic simulation; keep game logic there, never in the client. World assets (villagers, buildings, trees, resources, ground) come from generated high-quality renders (FAL/Midjourney) used as painted sprites and textures in the 3D world; procedural code geometry is only a temporary placeholder until a generated asset replaces it.
- **Rendering:** Fixed-angle orthographic isometric camera with pan and zoom; building anchors stay fixed in world space. Building sprites retain their original proportions and fit within rectangular cobblestone plots; rendered foundation plots are level, with touching plots sharing one height. A building's depth follows its footprint's front edge per screen column, and a villager hidden behind a building shows as a flat team-blue silhouette, as in Age of Empires II. Soft two-tone cel shading, a fine one-pixel ink-line finish pass (depth creases and silhouettes), a tilt-shift post-process, and one shared fog-of-war shader field (unexplored land lies under cloud). The interface is painted inside the WebGL canvas with shared drawing/hit-region geometry. Accessible DOM button equivalents are required but remain unimplemented; do not claim canvas hit regions provide screen-reader access.
- **Grid scale:** simulation cells are finer than a villager is tall (each island region is 120×80 cells, with 64-cell ocean gaps and half a world unit per cell; the client converts with `terrain::CELL`). Resources cluster tightly, as in Age of Empires.
- **Spatial authority:** Every unit, step target, building footprint, and live resource exclusively claims its cells; move destinations are reservations. `GameWorld::validate` must hold after every command and tick.
- **Deployment:** Modal. Verify locally before deploying.

## Interaction Contract

- Tap/click a villager to select it.
- Tap/click a resource with a villager selected to issue a gather order.
- Tap/click a visible wild animal (wolf, bear, boar, lioness or lion) with friendly units selected to issue a group attack order; with no units selected, inspect its health. Healers cannot attack. Retreat and Stop remain explicit player orders.
- Tap/click a foundation with villagers selected to have them help build it.
- Tap/click the Sanctuary of the Gods with units selected to send them for the Artifact of the Gods; the first to arrive carries it. A bearer within three cells of a completed home town center wins the run.
- Tap/click a town center (or a granary for food and fiber, or a lumber mill for wood) with villagers selected to have those carrying goods outside a gathering loop unload there. Selecting buildings leaves existing gathering loops intact. A villager holding goods shows the carry pose, even when stopped, except while actively gathering; partial loads must keep the work animation.
- Villagers carry at most 20 typed units, deposit at the nearest reachable compatible completed storage site (including lumber mills for wood and stopped shore ships with room), and resume unfinished gathering. When a node runs out they move on to the nearest reachable node of the same kind within 10 cells of the connected exhausted wild-resource patch (or the individual field), else go idle. A villager holding goods finishes that load or drops it off first: given a new gather, build, or field-preparation assignment, it delivers its cargo to a compatible completed drop site before starting, including partial loads of the same kind. It retains and resumes the new assignment automatically.
- Gathering is phase-driven: villagers wait at the node until full or depleted, and combined activity sprites replace duplicate unit-plus-resource rendering.
- With villagers selected, a ring under the pointer previews the tap: gold over a resource or foundation, white over ground.
- Tap/click the build button to open the grouped building menu (buildings unlocked when discovered materials support construction and a productive use, plus farm fields; ten first-island buildings are initially available, and unaffordable ones stay visible in greyscale), pick one, then tap valid ground to issue a build order.
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
- Treat the [style acceptance review](docs/knowledge/asset-pipeline.md#style-acceptance-is-a-merge-gate) as a blocking merge gate for art. Attach approved family references to generation and refinement, and include reference/result comparisons at actual display size. Technical asset checks do not establish a style match.
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

## Consult Maintained Knowledge Before Changes

Read [docs/knowledge/INDEX.md](docs/knowledge/INDEX.md) and open the guides relevant to the task **before editing code, choosing an implementation approach, or interacting with that system**. Reading the index alone is not sufficient. Consult additional guides when work crosses into another subsystem, such as saves, assets, GitHub or Modal.

The folder holds individual system guides, procedures, research and reusable learnings. Detailed PR evidence lives with the relevant topic; [the audit record](docs/REWORK_LESSONS.md) preserves coverage and attribution. Do not recreate one large lessons file.

## Workflow

- Feature regressions require Jakob’s explicit approval before release. Preserve existing player-facing features. Before releasing a change that removes, hides, disables, or adds prerequisites to an existing feature, present Jakob with the affected features, before/after behavior, rationale, save impact, and alternatives for explicit review. General milestone simplification is not approval for individual feature losses; record the user's specific approval in the change description. Watchtower, Barracks, and the existing building catalog remain important game features.
- Follow the **consult → work → learn/update → continue** documentation loop below; project context belongs in reviewable repository files.
- For any visual change (sprites, textures, UI, layout, or rendering), show relevant images directly in the chat alongside status updates to the developer as soon as a preview is available, and include the final visual result when finishing. Use current asset previews or in-game screenshots, label drafts and before/after comparisons clearly, and do not wait for the developer to ask to see the changes.
- At the start of every work session, read `OPEN_WORK.md` before acting.
- Keep `OPEN_WORK.md` current after meaningful milestones, blocker changes, and before commit/push/deploy or ending a session. It is a compact current-state handoff, not an append-only diary.
- Parent-session work may proceed directly on `master` for this solo project.
- Every delegated subagent must work on its own feature branch or git worktree, push that branch, and open a pull request for Jakob to review. Subagents must never commit directly to `master`.
- A subagent PR must describe its scope, verification performed, generated assets, and any known limitations. Do not merge it automatically.
- After runtime or asset changes: format, test, lint, perform the thermonuclear review, verify the browser demo, and redeploy Modal through the existing release workflow. For documentation-only changes, check links, factual claims, skill metadata when applicable, and `git diff --check`; apply the documentation/scope parts of the review. A documentation edit alone does not require a game build, browser run, or manual deployment.
- Keep `README.md` and `ROADMAP.md` synchronized with actual behavior.

## Documentation as Project Context

- Automatically use [project-documentation](.agents/skills/project-documentation/SKILL.md) for implementation, debugging, planning, review, system/tool interaction and documentation upkeep. Invoke explicitly with `$project-documentation`, or read the file directly if the host has not discovered it.
- Follow **consult → work → learn/update → continue**. Read the required product/handoff context and [knowledge index](docs/knowledge/INDEX.md) at task start, then the relevant guides before acting. Check important claims against current source and environment; historical reviews are evidence, not live instructions.
- Capture how systems work, successful interaction procedures, useful API/tool findings, troubleshooting, research, rejected approaches and verification limits—not only decisions. Put each coherent topic in its own file under `docs/knowledge/`, linking existing detailed specs and asset provenance rather than copying them.
- When the developer steers or corrects the work, update the affected guide before the next affected implementation step. Record the durable clarification, its source and rationale when known; mark intended behavior as pending until implemented. Do not silently turn a task-specific exception into a universal rule.
- Reconcile knowledge after meaningful findings, failed approaches, milestones and subsystem changes, and again before handoff/commit/push/deploy or ending. Routine authorized documentation maintenance does not need a separate request. Skip transient chatter and do not manufacture notes when nothing reusable was learned.
- Keep the index short: links and reading triggers. Use [the document scaffold](.agents/skills/project-documentation/references/knowledge-document.md) for new topics; split unrelated sections into separate files and repair inbound links. No reusable document should depend on a prior chat to be discoverable.
- `README.md` owns the product summary, `ROADMAP.md` proposed scope, `decisions.md` concise accepted choices, and `OPEN_WORK.md` current tasks/blockers. Detailed system knowledge belongs in the guides. Preserve unresolved work when cleaning the handoff, and label superseded instructions with replacement links.
- Preserve source paths, reproducible commands and relevant date/revision/API version. Distinguish inspected source from executed checks, proposals from implemented behavior, and merged from deployed/verified. Never store secrets or transcript dumps, or add a memory database/background daemon for this workflow.
