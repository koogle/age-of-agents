# Roadmap

## Product principle

Grow the gather/build demo into the Greek strategy roguelike described in [README.md](README.md#proposed-gameplay-loop). Every slice must add an end-to-end player decision, remain deterministic and authoritative in Rust, preserve fog/collision/persistence rules, and be playable through the real WebGL UI on desktop and phone. Do not build a generic engine, ECS, recipe language, or broad technology matrix.

- Reset game opens a native seed input and progress-loss confirmation on desktop and WebGL; blank chooses a random island, and explicit seeds reproduce an island.

- Camera navigation uses pan and zoom with a fixed orthographic angle, responsive, zoom-scaled mouse edge/corner panning, pointer-anchored wheel zoom, midpoint-anchored pinch zoom, stable building anchors, and an optional diamond-shaped square grid. Distant zoom transitions into the curved planet overview.

- Villager presentation uses accumulator interpolation locally and buffered playback remotely, with full gait cycles, prompt facing changes, and work poses only after reaching the authoritative work cell.

## Current direction

Dock presentation now includes all four shoreline-facing orientations and matching
construction stages; placement previews use the same deterministic water-edge choice.

The implemented feature summary is in [README.md](README.md#implemented-roadmap). Starter-island resource generation and discovery-based unlocks are implemented. Dock-built local transport and passengers are implemented. Continuous sailing between persistent islands, progressive map expansion, a cumulative globe overview, simultaneous settlement simulation and island inventories supplemented by movable ship storage are implemented. Basic dirt/stone roads and travel-time routing are implemented locally, pending PR/release verification. Next: profile populated archipelagos and refine the economy. Each run plans 5–7 island sites from its seed, reveals the temple island on the globe, and generates islands as ships approach them. Carrying the Artifact of the Gods from the temple back to a home town center wins the run. Next (proposed, Jakob 2026-10-07): once the artifact is claimed, difficulty rises on the return and new monsters spawn; then escalating monster classes per island. Build on that loop with wolves, pirates, mythical creatures, escalating calamities, and permanent upgrades across runs.

Proposed next direction (2026-10-06, not implemented): a bounded 5–7 island run with a pre-planned archipelago revealed at the start, a temple holding the Artifact of the Gods on the final island, win/loss through the existing scenario state, and escalating monster classes per island. Phase order and open decisions are in [the run plan](docs/RUN_PLAN.md).

Territorial wolves, bears and boars are the first implemented danger, with explicit contact attack orders, health, pursuit and death cleanup. New games have one wolf and one boar on the first island; later islands have one bear, 1–3 wolves and 2–3 boars where safe spawn cells exist. Broad combat, calamities, treasures, and permanent progression remain proposals. Balance should make eventual defeat and the next run interesting without tedious rebuilding; event timing and upgrade rules remain open.

The slices below retain earlier acceptance criteria as implementation references. Their fixed-scenario goals and steel-first ordering are superseded by this direction; see [decisions.md](decisions.md).

## Fresh water

Implemented and locally verified in the working tree: water is the
fourteenth resource/product. Villagers collect renewable water at riverbank jug
markers and deposit it at town centers, docks, or stopped shore ships. Sources
use dry land, preserve walking routes and ford approaches, and are reachable on
first and later islands.

Planting and replenishing fields costs 10 wood, 5 stone, and 10 water, paid once
when preparation starts. Joining or resuming paid work costs nothing extra;
fields have no ongoing water drain, and exhausted fields still need explicit
replenishment orders. Kitchen recipes, clay processing, and irrigation upgrades
remain possible later uses, with no recipe changes in this slice.

Jakob approved this first slice on 2026-10-05. Save version 16 adds required water
inventories and resets incompatible hosted worlds under the existing policy.
See [water integration notes](docs/knowledge/water-resource.md) for behavior,
implementation, and verification.

## Resource-based island progression

Implemented for fresh games: food/wood/stone nodes and renewable water, timber and rations processing, ten starter buildings plus fields, and Forestry/Agriculture/Masonry research. Building availability requires discoverable construction inputs and, for production buildings, at least one usable recipe. Barracks and Smelter need iron and coal, Kiln needs clay, Weaver needs fiber, Workshop needs clay plus iron and coal, Infirmary needs fiber, and Monument needs clay/fiber/gold/iron/coal. Range remains available initially because archers use food and timber. Useful buildings remain visible when temporarily unaffordable. Discovery survives depletion and saving across islands; hidden deposits and injected stock do not bypass unlocks. Existing buildings and saves remain intact.

The first transport is a playable dock recipe costing 60 wood + 20 timber, taking 20 seconds. It holds four passengers, sails on water, and lands passengers at clear shore. Each island has its own inventory; ships carry an additional 50 resources and share their cargo while stopped at shore. Generation guarantees at least 600 reachable wood, 300 food and 120 stone, including wood to process timber and establish the settlement. Tests cover processing, starter construction costs, three research jobs and villager training within the base budget. Fields consume wood, stone, water and labor for ordinary food.

Remaining acceptance criteria:

1. Implemented: dock-built transport approaches reachable shore for boarding orders and boards/lands existing units, retaining identities and personal carried loads. Water navigation, seat reservations, safe landings, interrupted orders and save compatibility are covered. Departure needs no cloth or metal.
2. Implemented: sailing toward the map frontier generates an adjacent island; continuous ocean crossings reach the persistent second island with iron/coal for steel; further islands introduce clay/bricks and fiber/cloth separately. Their resources complement earlier islands instead of replacing them; preserve productive first-island farms and forests.
3. Implemented: island-local inventories and 50-resource ship holds fund outposts. Cargo stays aboard and is available while stopped at shore; dock controls transfer it directly. All discovered settlements keep running.
4. Show the next useful unlock and its missing input as expansion becomes playable. No age progression, automatic inter-island shipping, combat or adversaries in this foundation PR.

## Released baseline — Gather, build, research, and route

- [x] Persisted 600-cell isometric world with eight biomes and explored/visible/unseen fog privacy.
- [x] Seven biome-compatible raw resources: wood, food, stone, gold, iron, clay, and fiber.
- [x] Bounded villager carrying, deposits at the closest accessible compatible building, resumption through temporary approach congestion, explicit unload-first gather/build reassignment, depletion, construction, training, and five gathering technologies.
- [x] Deterministic four-neighbor routing, occupancy, reserved destinations/build sites, and blocked-spawn rejection.
- [x] Typed sequenced WebSocket commands/snapshots, SQLite round-trip, authoritative 0×/1×/2× speed. At 0×, simulation and NPC presentation freeze and gameplay orders are rejected until resumed; camera and selection remain available.
- [x] Terrain/entity presentation, directional movement/gathering animation, fog memory, capability popover, and desktop/mobile controls (Canvas 2D; superseded by the 3D client).

## Direction change — 3D client and spatial soundness

Status: integrated into the current prototype; the shared Rust renderer has replaced the retired Three.js client.

- [x] One derived occupancy map: buildings (rectangular footprints, foundations included), live resources, unit cells, and step targets are exclusive claims; move destinations are reservations.
- [x] Units claim the next cell before stepping; deterministic eight-neighbor Dijkstra never cuts an occupied corner.
- [x] Building placement preserves every unit’s existing ground routes, preventing builders or bystanders from being enclosed by a foundation.
- [x] Fogged building orders explore first, then recheck the fully visible footprint before placing and paying for a foundation. Stop/replacement cancels exploration without cost.
- [x] Build orders place a foundation immediately; `Construct` lets any villager resume or help; completion releases every builder at once.
- [x] Typed cell coordinates on every command; `GameWorld::validate` runs on load, after accepted commands and ticks (debug), and across deterministic randomized play.
- [x] Shared native/WebGL2 Rust client: painted sprites, island terrain with fog-of-war shader, minimap, and mouse/touch controls. The original Three.js implementation is retired.
- [x] Finer grid: 120×80 half-unit cells (a villager is about 1.5 cells tall), 5×5 town centers (houses/granaries 3×3, watchtowers 2×2, docks 4×4; neighboring plots may share edges), clustered resources (woodlines, berry patches, mine clumps), so workers stand right against their work.
- [x] `Stop` order: abandon the current task, keeping cargo and foundation progress (no refunds needed); a new order replaces a busy villager's task the same way, atomically.
- [x] Villager build menu: town center, house (+5 housing), granary (food/fiber drop-off and +50% nearby field yield), watchtower (sight 20), dock (must touch the sea; the fishing boat comes later). Training respects housing.
- [x] Completed lumber mills accept wood for automatic gathering deliveries and click-to-unload; gatherers use the nearest reachable compatible drop-off and resume work.
- [x] Gatherers move on to the nearest reachable node of the same kind within 10 cells of the connected exhausted wild-resource patch (or individual field) when theirs runs out.
- [x] Seeded island worldgen: deterministic integer-hash noise, sea, beaches, elevation with hills and impassable peaks, rivers with fords, biome-placed resource clusters, and a guaranteed fishing-boat budget (wood 300, food 150, stone 80, iron 60, fiber 60, clay 40, with 50% headroom) reachable from the start; compact terrain snapshots.
- [x] Liveness: head-on standoffs resolve by deterministic yielding (lower index side-steps, highest index wins a contested cell); idle units never rest on another unit's reservation; a destination may be reserved while someone only walks through it.

## Slice A — Expandable domain foundation

Status: domain catalogs and scenario scaffolding are integrated; scenario objectives are not playable.

Deliver the smallest explicit catalogs and persisted state needed by later slices.

Gameplay acceptance:

1. The snapshot exposes a stable catalog of the actual resources, building kinds, unit kinds, recipes, and technologies used by this roadmap; unknown persisted enum values fail explicitly rather than resetting the world.
2. Current-version saves retain units, stockpiles, orders, buildings, fog, navigation, and research. Incompatible store versions reset; backward compatibility is deferred until explicitly requested.
3. Scenario state has an explicit identifier, authoritative tick limit, objective progress, and running/won/lost outcome without yet claiming objectives are playable.
4. Existing released gameplay remains behaviorally unchanged.

Engineering acceptance:

- Split `game.rs` before it or any frontend file grows beyond 1,000 lines; catalogs are direct typed constants/data, not a generic content engine.
- Focused store-reset, catalog-integrity, serialization, and deterministic tick-boundary regressions pass.

## Slice B — Multi-unit control

Status: selected units use a broad ivory ground ring with a blue border in the shared native/WebGL2 renderer; terrain never clips it, while buildings cover it. Desktop selection and group orders are integrated; additive touch selection remains open.

Gameplay acceptance:

1. Intentional mouse Shift-drag from empty ground draws a readable selection rectangle and replaces the selection with visible friendly units whose projected feet are enclosed; Control-drag (Command on Mac) adds enclosed units; ordinary drag still pans and click selection still works.
2. Touch keeps pan/tap semantics and offers additive unit selection without accidental box selection.
3. A group ground order is one typed authoritative command. Validation is atomic: one invalid/busy/member mismatch rejects the whole order without moving any unit.
4. Accepted group movement assigns deterministic distinct reachable destinations, respects reservations/occupancy, and visibly moves every selected unit without stacking.
5. The HUD reports the selected count and group orders survive snapshots/reconnects.

## Building task queues — implemented, awaiting release

Each building runs one task and holds up to five waiting production/research tasks in submission order. Inputs and trainee housing are reserved when ordered; tapping a queued coin cancels that waiting task and refunds the full cost. Active work continues, including waiting for a free spawn cell.

Completed research remains visible with a bronze laurel seal with a green check and disabled ordering; hover or tap explains the upgrade. Completion survives saving and is shared across islands.

## Building expansion — deployed

User-directed implementation activates construction of all 17 catalog buildings. The build menu groups buildings into Town, Gathering, Production and Military, plus a Roads group for dirt and stone surfaces, with distinct portraits from the authored HD sheets. Catalog manifests load at runtime so the cloud build does not require asset files before compilation. Catalog building construction-stage sprites and unit idle/walking sprites are now integrated; military/villager action sheets and base resources now use 512 px cells from recovered sources and reviewed refinements. Five processors make timber/steel/bricks/cloth/rations through the typed `Produce` command. Farms and mining camps provide matching drop-offs and a local, non-stacking 25% gathering bonus. Completed farms also unlock player-built 3×3 fields: 10 wood + 5 stone and 12 villager-seconds create 120 harvestable food, or 180 with a completed granary within six cells of the field edge (non-stacking, evaluated at preparation completion). Fields have dedicated generated cleared-soil, cultivation, seedling and ripe-wheat art, including their menu portrait and placement preview. Preparation workers automatically harvest the completed field using the normal carry/deposit loop and custom hoeing animations. Field placement preserves existing walking and delivery routes, just like building foundations. Exhausted plots persist and require a new explicit order and the same paid labor to replenish; interruption and helpers preserve paid progress. Existing harvesters keep their gathering order while another worker replenishes their field, including when returning with the previous harvest. New worlds now generate only food, wood and stone; the full resource catalog remains available to domain test fixtures. Barracks/range/workshop/infirmary train the four defined non-worker unit types, with housing, idle movement and blocked-spawn handling; wildlife contact attacks are implemented, while general combat and healing remain deferred. Monuments are costly landmarks with extended vision, without scenario victory logic. Matching economy research is available at its building as well as the town center.

The Rust HUD exposes grouped, labelled construction, production costs/progress and all stockpile totals. Compact phone and short-landscape layouts place actions beside the globe/speed controls, with queues and submenus wrapping upward. Catalog building and unit sprites are integrated; all audited unit action/facing frames now use 512 px cells. Costs and recipe quantities are initial balance values.

This completes construction and bounded production portions of C/D, not their extraction gating, upgrades, tools, combat or scenario requirements. The next progression direction is resource-based island expansion: food/wood/stone on the first island, transport ships carrying units and 50-resource movable storage holds, persistent destination islands, and island-local inventories. Continuous sailing, progressive island placement, a cumulative globe overview and simultaneous simulation of discovered settlements are implemented. Profile larger populated worlds before adding streamed terrain or snapshot deltas. Animals/adversaries follow that economy/transport loop.

## Slice C — Steel economy vertical slice

Status: planned.

Gameplay acceptance:

1. Coal appears only in compatible terrain and is extracted by villagers only after a Mining Camp is constructed beside it.
2. Iron extraction also requires a Mining Camp; iron is discoverable but cannot be hand-gathered.
3. A Smelter/Forge can queue steel batches; each batch atomically reserves iron plus coal, progresses visibly, and deposits steel exactly once.
4. Invalid placement, missing inputs, full task queues, and inaccessible spawn/interaction cells reject without partial cost/input mutation.
5. The building popover and stockpile HUD make prerequisites, costs, queue progress, blocked reasons, coal, and steel understandable.

## Slice D — Coherent broader economy and progression

Status: planned.

Target economy (14 total resources/products): water, wood, food, stone, gold, iron ore, coal, clay, fiber, timber, steel, bricks, cloth, and rations.

Gameplay acceptance:

1. Extraction buildings are limited to Mining Camp (iron/coal/gold/stone) and Farm (food/fiber); villagers still directly gather wood and clay.
2. Lumber Mill makes timber from wood; Smelter makes steel from iron+coal; Kiln makes bricks from clay+wood; Weaver makes cloth from fiber; Kitchen makes rations from food.
3. Town Center, Mining Camp, Farm, Lumber Mill, Smelter, Kiln, Weaver, Kitchen, Barracks, Range, Workshop, Infirmary, Watchtower, and Monument form the complete useful building roster. Buildings not yet active in combat may appear only in the slice that makes them useful.
4. Building upgrades are bounded to two levels and improve one visible property. Research remains building-specific and every technology enables or improves an immediately playable action.
5. Costs, ordered building queues, prerequisites, progress, and completion are authoritative, persisted, and visible. No giant recipe/technology matrix.

## Slice E — Bounded scenarios

Status: planned.

Deliver four selectable deterministic challenges, each with visible progress, a tick/time limit, and terminal won/lost state:

1. Foundry Town: construct a Smelter and hold 20 steel.
2. Frontier Survey: reveal at least 70% of terrain and construct a Watchtower before the limit.
3. Monument Works: construct the Monument using timber, bricks, cloth, gold, and steel.
4. Hold the Coast: survive the raid schedule until the final tick with the Town Center alive (enabled with Slice G).

A combined default prototype scenario requires steel production, exploration, and survival. Terminal scenarios reject further simulation-changing commands except reset/select-scenario. HUD presents objective progress and remaining authoritative time.

## Slice F — Combat foundation

Status: wildlife contact combat is implemented; ranged attacks, building health, factions and general combat remain planned.

Gameplay acceptance:

1. Units/buildings have faction and health; snapshots omit unseen hostiles under existing fog privacy.
2. Barracks trains melee Guards; Range trains Archers. Group attack/target commands validate ownership, visibility, range/path viability, and members atomically.
3. Guards close to melee range; Archers hold bounded range. Attack cadence, damage, target loss, and death cleanup are deterministic.
4. Dead units release occupied/reserved cells and are removed from selection/orders/persistence exactly once.
5. The player can train, select, group-order, fight, and win a small deterministic skirmish through the real UI.

## Slice G — Defense, support, siege, and raids

Status: planned.

Gameplay acceptance:

1. Infirmary trains a Healer that restores friendly health without exceeding maximum health and cannot heal hostiles or dead targets.
2. Workshop trains a slow Siege Cart with bonus structure damage; blocked production spawns reject atomically.
3. Watchtowers automatically attack visible hostile units in range with deterministic cadence and target choice.
4. Pirate raids use a seeded fixed schedule plus bounded deterministic interval jitter, spawn at valid coastal/edge cells, and pursue explicit scenario targets.
5. Raid warnings, current wave, losses, and survive progress are visible. The player can build defenses, position a mixed group, survive, and complete Hold the Coast.

## Slice H — Balance and release polish

Status: planned.

Gameplay acceptance:

1. Starting resources and timings let a new run reach steel, field a mixed defense, and finish the default scenario in a bounded play session without developer shortcuts.
2. Desktop and phone controls remain legible; selection, commands, queue state, scenario progress, combat feedback, and raids have clear visual feedback without frontend emojis.
3. New sprites follow the established hand-drawn cel-shaded dark-ink direction, are valid sRGB RGBA PNGs, and share verified ground anchors.
4. README documents the actual playable loop and controls; this roadmap marks only production-verified slices delivered.

## Gate for every released slice

1. Focused Rust regressions for every domain rule and bounded transition.
2. `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets --all-features -- -D warnings`.
3. Browser-binding syntax plus focused presentation/control contract checks; Python asset checks; `git diff --check`.
4. Independent specification review, code-quality review, and thermonuclear maintainability review with all blockers fixed.
5. Real isolated local server plus Chromium: use actual controls/WebSocket, inspect intermediate authoritative state, browser console, desktop and phone screenshots, and play the delivered loop.
6. Push `master`, observe CI, deploy through `python3 scripts/modal_manage.py`, then verify production assets byte-for-byte, state/protocol, hard-reloaded Chromium interactions/screenshots, and restoration of reversible controls.
7. Refresh `OPEN_WORK.md`, README, and this roadmap so committed, pushed, deployed, and production-verified states are never conflated.

## Explicitly deferred

- LLM-controlled villagers, autonomous planning, multiplayer, mod/plugin APIs, generic ECS/content engines, and a large branching tech tree.

- [x] Dedicated lossless 512 px building/construction sprites, translucent placement ghosts, green/red rectangular footprint previews, and Grid toggle (G). Original-style building sprites render without skewing inside cobblestone plots; houses are visually smaller and the completed granary displays stored grain. Plots mark the exact claim, including construction and previews; touching plots share one level foundation height.

## Integrated native presentation fixes

- [x] Control/Command-click additive group selection and existing shared gather/construct commands.
- [x] Globe minimap terrain, camera marker and click/touch navigation share the world view’s isometric orientation and fit the full current map.

- [x] Centered time-control labels and italic resource-gain feedback for gathering, explicit unloading and pre-build drop-offs; brief assignment, drop-off and idle announcements rising/fading at their initial world positions like resource-gain feedback, with plain walking excluded.
- [x] Full authored walking/carrying cycles paced by displayed travel, immediate motion detection and neutral idle.
- [x] Preserve painted sprite colors while retaining current terrain depth, calibrated buildings and translucent ghosts.
- [x] Keep partially visible buildings on screen when their ground anchors pass the near clipping plane at close zoom.
- [x] Shift-drag from empty ground draws a selection box and replaces the selection with enclosed visible units; Control-drag (Command on Mac) adds units in native and WebGL2 clients.
- [ ] Additive touch selection and accessible DOM controls in the wgpu client remain open.
- [ ] Explicit blocking/non-blocking task classification remains open; current domain allows valid replacement orders for all unit tasks.

## Recovered acceptance gaps — reviewed 2026-10-05

- The canvas HUD still lacks accessible DOM button equivalents, and touch still lacks additive selection; these existing items above must survive handoff cleanup. Canvas labels/hit regions do not complete accessibility.
- Native reset-dialog appearance has checked-in Linux/Zenity evidence, but no macOS/Windows appearance evidence was found in the reviewed PRs; physical-phone safe-area behavior remains unverified. Desktop/phone emulation and hosted/local presentation fixtures certify only their recorded platforms and revisions ([presentation verification](docs/PRESENTATION_VERIFICATION.md), [compact HUD limits](docs/COMPACT_HUD_REVIEW.md#boundaries)).
- An idle villager in a one-cell passage and previously placed route-blocking fields remain documented limits; the placement fix does not repair existing worlds ([field audit](docs/FIELD_GATHERING_REVIEW.md#limits)). Reproduce before changing routing behavior.
- Open PR proposals and any remaining release checks belong in [OPEN_WORK.md](OPEN_WORK.md); an open PR's described behavior is not part of the implemented baseline. Native/browser-local persistence is also not implemented; selecting it as new scope requires a task, not an assumption that hosted SQLite already covers it.

- [x] Full-plot initial foundations, cleaned curved roof repeats for non-HQ buildings, and loading animation sharing the lossless 512 px gameplay sheets.

Island storage: stopped shore ships supplement local construction/production costs and accept villager deposits; dock controls transfer cargo directly. Cargo stays aboard between islands, while training/research remain building jobs. Pointer/touch island inspection drives the top resource bar; resource names appear on hover or tap instead of permanent labels. Mobile cargo paging uses inline chevrons and supports horizontal swipe paging.

## Deployment verification follow-up — reconciled 2026-10-05

- [x] `250f3ce` updates `scripts/modal_manage.py::verify_once` for bounded compressed terrain and runtime dimensions, with decoder regressions in CI. The earlier documentation review's fixed-map finding is resolved.
- The verifier still assumes a land unit and unseen terrain. Diagnose those separately if a valid world lacks them; see [release guide](docs/knowledge/build-integration-and-release.md#known-verifier-mismatch).

Island generation uses rounded, square, elongated, lobed and open-bay outlines, with shared relief and downhill drainage; starter resources remain visible and reachable.
