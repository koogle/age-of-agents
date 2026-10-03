# Open Work

Updated 2026-10-03. Keep only outstanding work, blockers and verification follow-ups here. Implemented features are summarized in [README.md](README.md); completed history lives in Git and `docs/`.

Use `master` as the integration branch and PR base.

## Building task queues and selection portraits — ready for review

Branch `fix/building-task-queues` adds one active task plus five paid waiting tasks per building. Cancelling a waiting coin refunds full inputs and releases trainee housing; stable IDs protect against repeated/stale clicks, and existing saves default to empty queues. Portraits stay inside the rounded progress pill; queued controls fit desktop and DPR-2 phone layouts. Current master’s starter-resource/discovery gates and selection rendering are preserved. Review: [Building queues](docs/BUILDING_QUEUES_REVIEW.md).

Verified: all 157 workspace tests, strict native/WASM Clippy, formatting, relevant existing asset checks, rebuilt release WASM and the native server. Real desktop clicks and phone taps verify full queues, immediate costs, exact production/research refunds, housing reuse, stale cancellation, reconnect, and completion without charging twice or activating trained units. No browser errors. Screenshots and drivers: `/workspace/scratch/building-queues/`. No new assets or dependencies. Only waiting tasks can be cancelled; active work continues.

Pending: review/merge into `master`, then confirm the automated Modal release. Do not auto-merge this implementation PR. Direct Modal credentials are unavailable in this environment; no deployment was performed.

## Ordered agent backlog

Order by prerequisites first, then estimated complexity within each stage. Small means a bounded fix/check; medium means one substantial subsystem; large means coordinated simulation, persistence and client work. These are relative sizes, not time estimates. Independent tasks may run alongside the main chain. No task below is implemented merely because it is listed here; see [README.md](README.md#implemented-roadmap) for completed features.

### Stage 1 — Independent work, smallest first

| ID | Task / complexity | Dependencies | Acceptance criteria |
| --- | --- | --- | --- |
| A1 | Release and presentation verification — small | Latest runtime deployment | Confirm the latest release containing #55 succeeds, then check fresh-island `/play`, camera controls, gathering animations and loading-title cleanup. Also record native reset-dialog appearance, deployed globe behavior, and villager direction/pose holds on zigzag routes. Use isolated worlds; never reset the shared production save. |
| A2 | Field placement safety — small | None | Make `plant_field` preserve existing ground routes as building placement does. Cover trapped bystanders and the last exit with regressions; preserve wood/stone charges, shared labor and manual replenishment. |
| A3 | Touch selection and accessibility — medium | None; coordinate client edits with B5 | Add additive touch selection and accessible keyboard/screen-reader equivalents for canvas controls. Preserve desktop Shift-click/Shift-drag, mouse/touch parity and camera gestures. Explicit blocking/non-blocking task classification remains open; scope it separately if it requires new domain semantics. |
| A4 | HD action/facing sprites — large asset workload | None for existing units; B1/B2 for ship states | Audit and recover/regenerate/refine remaining low-resolution art, then repack with provenance. Last audit: 183 of 264 frames below 512 px. Require detailed authored sources and at least 512×512 frames, not DPI metadata or a larger canvas alone. Run resolution/asset checks and inspect maximum-zoom desktop and DPR-2 phone. Add ship art once action/facing requirements are agreed. |

### Stage 2 — Core dependency chain

Use one integration owner for shared domain, occupancy and save-schema changes. Agree on B1 before parallel implementation; merge B2 → B3 → B4. B5 can develop against agreed command/snapshot interfaces, but each core PR still needs enough actual UI to exercise its own playable slice.

| ID | Task / complexity | Dependencies | Acceptance criteria |
| --- | --- | --- | --- |
| B1 | Transport/island contracts — small design task, high coordination | None | Agree on island IDs, land-versus-aboard unit location, passenger/resource manifests, inventory ownership, typed commands and save compatibility. Resolve the trigger discrepancy: the conversation proposes generation on first ship completion; ROADMAP currently says first departure reveals island two. Recommended contract: completion generates a destination once; departure travels there. Synchronize ROADMAP/decisions when adopting it. Keep this a minimal contract, not a generic engine. |
| B2 | Dock-built transport and passengers — large | B1 | Implement the planned first transport at 60 wood + 20 timber, water navigation, bounded goods/passenger capacity, boarding and safe unloading. Preserve original NPC IDs and carried goods; boarding releases land claims, unloading reserves valid shore cells. Invalid commands are atomic; blocked landing, interruption and save/reload cannot lose or duplicate goods/units. No metal or cloth prerequisite. |
| B3 | Persistent destination islands and founding — large | B1, B2 | Generate destinations deterministically once, preserving prior islands, settlements and fog. Support travel and initial landing without requiring an existing destination dock; allow enough transported supplies to found an outpost. Introduce complementary iron/coal, then clay and fiber on further islands. Do not replace the old world via Reset or make island two supply everything. Avoid a fixed long-term island limit. |
| B4 | Local inventories and trading posts — large | B1–B3 | Replace global spending/deposits with explicit settlement inventories; construction and processing consume local inputs. Ship loading/unloading transfers goods atomically between inventories, with conservation and save/reload tests. Remote stock cannot fund local construction. Demonstrate a round trip that makes trade necessary; no automatic shipping in this slice. |
| B5 | Transport UI, globe and guidance — medium/large | B1 interfaces; B2–B4 for completion | Ship selection, manifests, passenger/goods controls, destinations, local stocks, missing-input/blocked-action explanations and cumulative globe discoveries. Verify build → board/load → travel → land/found → return/trade through actual desktop and touch controls. Coordinate accessible controls with A3. |

### Stage 3 — Progression after working transport and trade

| ID | Task / complexity | Dependencies | Acceptance criteria |
| --- | --- | --- | --- |
| C1 | Useful advanced resources — medium | B2–B5 | Give steel/bricks/cloth meaningful uses in tools, buildings and ship improvements, using existing recipes/discovery gates. Balance complementary islands so first-island food/timber remain useful. Keep timber the sole starter processed resource; defer rations until provisioning has a playable purpose. |
| C2 | Combat and local animal threats — large | Working B2–B5 economy; coordinate with C1 | Start with bounded combat and wolves, including clear feedback and persistence. Existing guards, archers, healers and siege carts only move/stop; their combat/healing behavior remains unfinished. Ship as a separate playable milestone after transport/trade. |

Later, split pirates, mythical creatures, calamities, treasures and permanent upgrades into separate proposals/PRs after C2. Timing, balance and upgrade rules remain open; follow the [proposed gameplay loop](README.md#proposed-gameplay-loop).

## Release verification handoff

Verify the next production release for the NPC ring terrain-occlusion correction: selection rings draw above terrain/raised plots without writing depth, before billboards so buildings and units still cover them. All 151 workspace tests, formatting, strict native/WASM lint and desktop/phone browser checks pass; rebuilt WASM included. Before/after pixels confirm repaired ground arcs and unchanged opaque building regions. Structural review: six lines in the existing shader, no new passes, dependencies or gameplay changes. Evidence: `/workspace/scratch/highlight-depth-*`. User authorized PR merge into `master`.

At this edit, [run 37150225436](https://github.com/koogle/age-of-agents/actions/runs/37150225436), containing #55, is pending behind [camera/activity run 37149731725](https://github.com/koogle/age-of-agents/actions/runs/37149731725). The original #55 run 37150108528 was cancelled/superseded. A1 must follow the latest successful runtime release rather than wait on a superseded run. Local desktop/phone checks passed; merged, deployed and live-verified states must stay distinct. Fresh runs cannot depart until B2/B3 land.

## Working constraints

Use an isolated SQLite database for local verification and preserve existing saves. Old recovery work remains on `codex/native-sprite-rendering`; historical root/gukaet edits and saves should remain untouched. Keep changes tied to the [current roadmap](ROADMAP.md#current-direction).

Each implementation agent uses its own branch/worktree from current `origin/master` and opens a scoped PR against `master`; do not auto-merge agent PRs. Reserve an integration owner for B1–B4 and coordinate overlapping client edits. Keep deterministic authoritative rules in `crates/game`, typed atomic commands, valid occupancy and compatible saves. Follow [AGENTS.md](AGENTS.md) and [the review gate](docs/THERMONUCLEAR_REVIEW.md): focused tests, formatting/lint, relevant asset checks, and real desktop/phone verification for runtime changes. State verification, assets and limitations in each PR; remove completed tasks from this file and keep README/ROADMAP synchronized.
