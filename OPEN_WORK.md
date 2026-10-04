# Open Work

## Core module extraction (2026-10-04)

Extracted `spatial` (geometry, occupancy, placement), `units` (NPC state and movement), `jobs` (typed activities and implementations), and `resources` (kinds, nodes, cargo, stockpile, chain catalog). Navigation remains reusable; whole-world validation has its own module. Root exports, serialized fields, command handling and tick order are retained. Client placement uses shared core geometry; field dimensions come from resource footprints. Extreme command coordinates are rejected before iteration.

Branch `refactor/core-modules` is based on master a27a564 and preserves upstream building queues, island progression and current art. Integrated verification: all 163 workspace tests pass (104 game, 46 client, 13 server), including three spatial regressions. Formatting, strict native/WASM Clippy, server build, release WASM build and whitespace checks pass; tracked browser artifacts are rebuilt. Thermonuclear review complete: direct modules, no dependencies or new gameplay, existing atomicity and queue behavior retained.

The earlier pre-integration refactor also matched an original-base deterministic replay of 1,800 commands/5,400 ticks across three seeds, and passed desktop/touch placement and construction checks. Fresh integrated Chromium checks select a villager on desktop and DPR-2 phone with no page errors. Screenshots: `/workspace/scratch/aoa-pr-desktop.png` and `/workspace/scratch/aoa-pr-phone.png`; checks and logs: `/workspace/scratch/aoa-pr-*`. Server is running on port 8000 with isolated database `/workspace/scratch/aoa-pr-integrated.db`. User requested a PR and screenshots; publish for review without merging or deploying.

## Retire the unused frontend (2026-10-03)

User requested removal of unused legacy code, then PR creation and merge. `/` and `/play` now share the Rust/WebGL2 client; `?local` still selects the in-page simulation. Removed the complete Three.js frontend/vendor libraries, unused GLBs/model builders, processed `assets/game/` sprites, obsolete asset pipelines/checks, and the standalone image-model probe. Source sheets, concept art, ledgers, and historical provenance remain. Docker/Modal packaging, production verification, CI, and docs now use the Rust client only.

Rebased onto master 54bf023, preserving HD sprites, custom fields, building task queues, progression, favicon and drop-off feedback. The diff against that base contains no changes to `crates/`, persistence, active art, dependencies, or the browser bootstrap/WASM. All 160 workspace tests (13 server, 46 client, 101 game), formatting, strict native/WASM Clippy, server build, browser-binding/Python syntax, UI icon checks, all 268 HD sprite checks, and whitespace pass. Isolated SQLite server: root/play and both local variants serve the exact Rust bootstrap with no-cache; retired URLs return 404. Deployment verifier passes locally. Real Chromium desktop/DPR-2 phone loads all four variants without page errors or missing/legacy requests; actual mouse/touch pause, 2×, and resume controls update authoritative state. Screenshots: `/tmp/aoa-cleanup-{desktop,phone}{,-local}.png`.

Thermonuclear review: deletes the duplicate presentation stack and obsolete pipelines, adds no dependency or gameplay behavior, preserves typed commands/persistence and `/play` links, and retains provenance. PR #69: https://github.com/koogle/age-of-agents/pull/69. Integrated the documentation-only handoff refresh from master 9cfa899. Ready for the user-authorized merge; production verification remains pending until the merge-triggered workflow completes.


Updated 2026-10-03 at 22:52 Europe/Madrid. Keep only outstanding work, blockers and verification follow-ups here. Implemented features are summarized in [README.md](README.md); completed history lives in Git and `docs/`.

Use `master` as the integration branch and PR base.

## Ordered agent backlog

Order by prerequisites first, then estimated complexity within each stage. Small means a bounded fix/check; medium means one substantial subsystem; large means coordinated simulation, persistence and client work. These are relative sizes, not time estimates. Independent tasks may run alongside the main chain. No task below is implemented merely because it is listed here; see [README.md](README.md#implemented-roadmap) for completed features.

### Stage 1 — Independent work, smallest first

| ID | Task / complexity | Dependencies | Acceptance criteria |
| --- | --- | --- | --- |
| A1 | Remaining release and platform verification — small | Successful runtime deployment; target desktops for native checks | Independently check deployed starter economy, camera controls, gathering animations, loading title and NPC selection-ring occlusion. Check native reset-dialog appearance on macOS/Windows when those desktops are available. Follow the release checklist below; use isolated worlds and never reset the shared production save. |
| A2 | Field placement safety — small | None | Make `plant_field` preserve existing ground routes as building placement does. Cover trapped bystanders and the last exit with regressions; preserve wood/stone charges, shared labor and manual replenishment. |
| A3 | Touch selection and accessibility — medium | None; coordinate client edits with B5 | Add additive touch selection and accessible keyboard/screen-reader equivalents for canvas controls. Preserve desktop Shift-click/Shift-drag, mouse/touch parity and camera gestures. Explicit blocking/non-blocking task classification remains open; scope it separately if it requires new domain semantics. |
| A4 | Ship action/facing sprites — asset workload | B1/B2 ship-state requirements | Generate/refine ship-state art once action/facing requirements are agreed. Require detailed authored sources, at least 512×512 frames and provenance. Run resolution/asset checks and inspect maximum-zoom desktop and DPR-2 phone. |

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

## Remaining release and platform checks

- **Current release verification:** [run 37152725466](https://github.com/koogle/age-of-agents/actions/runs/37152725466) for `54bf023` (merged [PR #68](https://github.com/koogle/age-of-agents/pull/68)) is running its quality job. Confirm quality, Modal deployment and automated production verification for this combined release, including building task queues and one-shot unloading feedback. Follow a newer superseding release if this run is cancelled. The earlier HD merge `1f61c21` passed both quality and deployment in [run 37152283578](https://github.com/koogle/age-of-agents/actions/runs/37152283578); HD assets, custom field art and the favicon no longer need implementation or merge work.
- **Independent live browser acceptance:** verify fresh starter-island resources/build menu in an isolated world using deployed assets; check hosted camera controls, partial-load gathering animations, loading-title cleanup, HD sprites and field stages at maximum zoom, and selected NPC rings above raised terrain but behind opaque buildings. Check building queues/cancellation refunds and the one-shot unloading announcement alongside resource-gain feedback. Record desktop/DPR-2 phone evidence and the checked bundle; the final combined phone replay of unloading feedback remains unconfirmed. Do not reset the shared production save. Existing scoped evidence: [HD sprites](docs/HD_SPRITES_REVIEW.md), [field art](docs/FIELD_ART_REVIEW.md), [building queues](docs/BUILDING_QUEUES_REVIEW.md), and `/workspace/scratch/status-check/`.
- **Native platform appearance:** macOS and Windows reset-dialog appearance remains unverified. Check the warning, seed entry, invalid-input retry, cancellation and confirmation on those target desktops.

Already-closed Linux/Zenity reset, live desktop globe and browser zigzag direction/pose checks are recorded in [presentation verification](docs/PRESENTATION_VERIFICATION.md) (merged PR #60); they are not outstanding tasks. Their platform/bundle scope does not certify the newer combined release. Fresh runs still cannot depart until B2/B3 are implemented.

## Working constraints

Use an isolated SQLite database for local verification and preserve existing saves. Old recovery work remains on `codex/native-sprite-rendering`; historical root/gukaet edits and saves should remain untouched. Keep changes tied to the [current roadmap](ROADMAP.md#current-direction).

Each implementation agent uses its own branch/worktree from current `origin/master` and opens a scoped PR against `master`; do not auto-merge agent PRs. Reserve an integration owner for B1–B4 and coordinate overlapping client edits. Keep deterministic authoritative rules in `crates/game`, typed atomic commands, valid occupancy and compatible saves. Follow [AGENTS.md](AGENTS.md) and [the review gate](docs/THERMONUCLEAR_REVIEW.md): focused tests, formatting/lint, relevant asset checks, and real desktop/phone verification for runtime changes. State verification, assets and limitations in each PR; remove completed tasks from this file and keep README/ROADMAP synchronized.
