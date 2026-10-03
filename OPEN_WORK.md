# Open Work

## Overhead resource drop-off status (2026-10-03)

Replaced the initial persistent pill with a one-shot “Dropping off wood” announcement using the exact existing resource-gain font, stroke, upward drift and 1.4-second fade. It appears when a villager enters unloading, including gather reassignment, construction, field preparation and explicit deposits. Repeated snapshots and Returning → Depositing do not replay it; actual delivery retains its separate +resource message. Loading an existing state does not replay old announcements. Removed the dedicated status HUD/view code; no domain, dependency or art changes.

Thermonuclear review: one transition check feeds the existing bounded feedback list and animation; no second visual/timer system. Regression coverage checks unloading actions, one-shot transitions, repeat snapshots and subsequent resource gains. Initial implementation passed all 140 workspace tests. Revised verification passes all 37 client tests, strict native/WASM Clippy, formatting, diff whitespace and release WASM rebuild. Timed desktop and DPR-2 phone WebGL2 snapshot fixtures visibly show the matching italic feedback and its expiry while the unit remains in Returning; repeat snapshots do not replay it, and both browsers report zero errors. Evidence: /workspace/scratch/status-check/. User authorized PR merge. Rebased onto master fef088f, preserving newer gameplay/rendering/favicon changes and rebuilding combined WASM. Full integrated checks and browser replay are running before merge; merging will trigger the configured Modal deployment workflow.

## AoA favicon (2026-10-03)

User requested creating an AoA favicon from the title style, opening a PR and merging into master. Added transparent cream-marble/terracotta monogram assets (256px PNG and 16/32/48/64px ICO), linked from both `/` and `/play` through the existing asset route. Production verification now compares both favicon files. Generation provenance is in `assets/loading/README.md`.

Verification: Chromium desktop and DPR-2 phone loaded both pages, resolved both icon URLs to exact local bytes and decoded the PNG; ICO sizes and transparent corners pass. Visually reviewed 16/32/64px icons on light and dark tab backgrounds (`/workspace/scratch/favicon/tab-sizes.png`). Existing frontend syntax, asset checks, Python compilation and diff whitespace pass. Thermonuclear review: direct static asset links, no dependencies, server routes or gameplay changes. Rust tooling is unavailable locally; the existing master deployment workflow runs the Rust quality gates and deploys/verifies Modal after merge. Next: create and merge the authorized PR; monitor the deployment workflow.

Updated 2026-10-03 at 22:30 Europe/Madrid. Keep only outstanding work, blockers and verification follow-ups here. Implemented features are summarized in [README.md](README.md); completed history lives in Git and `docs/`.

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

## Custom field artwork (2026-10-03)

User requested fal-generated field/construction sprites, a PR and merge into `master`. Dedicated cleared, cultivated, seedling and ripe frames replace the farm artwork for fields, their placement ghost and Field menu portrait. Four retained 1024px fal renders/cutouts are packed as 512px cells in the economy atlas; existing building rows remain intact. Shared plot fitting preserves registration through preparation/depletion. Simulation and farm buildings are unchanged. Delivery: [PR #63](https://github.com/koogle/age-of-agents/pull/63), preserving master `76ee262` and the NPC ring fix. All 152 workspace tests pass (13 server, 44 client, 95 game), including field stage/registration/footprint/preview coverage. Formatting, asset checks, deterministic repacking and strict WASM lint pass; integrated WASM rebuilt. Desktop and DPR-2 phone WebGL2 stage galleries have no runtime errors; original building pixels are unchanged. Strict native lint and actual desktop mouse/DPR-2 phone touch placement pass: each creates a distinct field and reserves exactly 10 wood + 5 stone. Final desktop and phone captures are reviewed, with no browser runtime errors. The user authorized merging this PR into master; that merge triggers production deployment. The release is not yet confirmed live. Source/provenance: `assets/sprites/field_sources/`; review: `docs/FIELD_ART_REVIEW.md`; evidence: `/workspace/scratch/field-art/`.

## Remaining release and platform checks

- **HD sprite release:** [PR #62](https://github.com/koogle/age-of-agents/pull/62) is ready for review and merge, then deployment verification. All 183 undersized audited frames are repacked from recovered sources and reviewed refinements; strict audit passes 268/268. Workspace tests, native/WASM lint, desktop/DPR-2 phone rendering and actual UI gathering checks pass. Latest selection-ring fixes are preserved. Review: [HD sprites](docs/HD_SPRITES_REVIEW.md).

- **Deployment status:** [runtime run 37150837150](https://github.com/koogle/age-of-agents/actions/runs/37150837150) succeeded for `aba4c6d`, including the starter economy, camera/activity/title changes and NPC ring terrain-occlusion fix. Deployment and automated production verification passed; independent browser acceptance of those combined changes is still open.
- **Current workflow:** [verification-record run 37151048374](https://github.com/koogle/age-of-agents/actions/runs/37151048374), for merged PR #60 at `d7dd0de`, is still running its quality job at this update. Confirm its final outcome; it adds documentation/evidence, with no runtime changes. Follow any newer superseding release instead of cancelled runs.
- **Independent live browser acceptance:** verify fresh starter-island resources/build menu in an isolated world using deployed assets; check hosted camera controls, partial-load gathering animations, loading-title cleanup, and selected NPC rings above raised terrain but behind opaque buildings. Record desktop/phone evidence and the checked bundle. Do not reset the shared production save.
- **Native platform appearance:** macOS and Windows reset-dialog appearance remains unverified. Check the warning, seed entry, invalid-input retry, cancellation and confirmation on those target desktops.

Already-closed Linux/Zenity reset, live desktop globe and browser zigzag direction/pose checks are recorded in [presentation verification](docs/PRESENTATION_VERIFICATION.md) (merged PR #60); they are not outstanding tasks. Their platform/bundle scope does not certify the newer combined release. Fresh runs still cannot depart until B2/B3 are implemented.

## Working constraints

Use an isolated SQLite database for local verification and preserve existing saves. Old recovery work remains on `codex/native-sprite-rendering`; historical root/gukaet edits and saves should remain untouched. Keep changes tied to the [current roadmap](ROADMAP.md#current-direction).

Each implementation agent uses its own branch/worktree from current `origin/master` and opens a scoped PR against `master`; do not auto-merge agent PRs. Reserve an integration owner for B1–B4 and coordinate overlapping client edits. Keep deterministic authoritative rules in `crates/game`, typed atomic commands, valid occupancy and compatible saves. Follow [AGENTS.md](AGENTS.md) and [the review gate](docs/THERMONUCLEAR_REVIEW.md): focused tests, formatting/lint, relevant asset checks, and real desktop/phone verification for runtime changes. State verification, assets and limitations in each PR; remove completed tasks from this file and keep README/ROADMAP synchronized.
