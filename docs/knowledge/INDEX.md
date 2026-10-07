# Maintained project knowledge

Read this index at task start and whenever the task moves into another system.
**Before making changes, open the relevant guides below and their necessary
dependencies.** Reading the index alone is not consultation. Check the source
when a guide's claim matters to the task; source-review dates are not guarantees
of current behavior.

| When working on | Read first | Also consult when affected |
| --- | --- | --- |
| Wildlife, hunting, unit health | [Dangerous wildlife](wildlife.md) | Placement/routes; rendering/input; assets; server/saves |
| Island generation, discovery, sailing, passengers, expanding maps | [Archipelago and transport](archipelago-and-transport.md) | Server/saves; rendering/input |
| Run goal, temple, artifact bearer, win state | [Temple and artifact](temple-and-artifact.md) | Archipelago; server/saves; HUD |
| Gathering, cargo, reassignment, field work | [Tasks and cargo](tasks-and-cargo.md) | Placement/routes; HUD |
| Water resource, riverbank collection, irrigation | [Water integration notes](water-resource.md) | Tasks/cargo; economy; archipelago; server/saves |
| Foundations, fields, roads, occupancy, reachability | [Placement and routes](placement-and-routes.md) | Tasks/cargo; rendering/input |
| Production, research, refunds, housing, unlocks | [Economy and queues](economy-and-queues.md) | Server/saves; HUD |
| Camera, terrain, picking, selection, minimap | [Rendering and input](rendering-and-input.md) | HUD; archipelago |
| Frozen worlds, stuck workers, delayed input | [Runtime debugging](runtime-debugging.md) | Server/saves; affected domain guide |
| FAL, image refinement, sprites, icons, packing | [Asset pipeline](asset-pipeline.md) | Rendering/input; HUD; build/release |
| Commands, copy, feedback, responsive UI, accessibility | [HUD and accessibility](hud-and-accessibility.md) | Economy/queues; rendering/input |
| HTTP/WebSocket, snapshots, SQLite, migrations, local fixtures | [Server and saves](server-and-saves.md) | Archipelago; runtime debugging |
| Native/WebGL build, branch integration, Modal/Actions | [Build, integration and release](build-integration-and-release.md) | Verification; GitHub history |
| Browser fixtures, evidence, platform limits, handoffs | [Verification and handoffs](verification-and-handoffs.md) | Affected system guide |
| PR state, review discussions, historical investigation | [GitHub history](github-history.md) | Verification; affected system guide |
| Rust duplication, shared constraints, simplification | [Rust simplification audit](rust-simplification-audit.md) | Affected domain, HUD and server guides |

The linked guides contain procedures, system entry points, reusable findings,
failure modes and evidence. Existing detailed specs, benchmarks, asset provenance
and historical reviews remain linked from those guides; do not copy them wholesale.

## Maintaining this folder

Use the [project-documentation skill](../../.agents/skills/project-documentation/SKILL.md)
automatically while working. Update the relevant guide when developer steering,
a system interaction, investigation or verification teaches something reusable.
Do this at meaningful checkpoints, not only at the end of the conversation.

Give a new system or independently reusable procedure its own descriptive file;
split a guide when its sections have different reading triggers. Use the
[document scaffold](../../.agents/skills/project-documentation/references/knowledge-document.md)
as a starting point. Add its reading trigger here, link dependencies, and repair
inbound links when moving a topic. No document should be discoverable only through
conversation history. Keep this index a route map, not the accumulated knowledge.

Stable behavior, how-to instructions and research conclusions belong in guides;
task status belongs in [OPEN_WORK.md](../../OPEN_WORK.md), concise accepted choices
in [decisions.md](../../decisions.md), and proposed product scope in
[ROADMAP.md](../../ROADMAP.md). The [PR audit](../REWORK_LESSONS.md) records historical
coverage and attribution, not the live system instructions.
