# Open Work

Updated 2026-10-03. Keep this file to current status, remaining work, and blockers; completed implementation and verification history lives in Git and the reviews under `docs/`.

Use `master` as the integration branch and PR base. The branch rename is cancelled; agents should preserve ongoing feature work and continue targeting `origin/master`.

## First-island economy PR #55

User requested a PR for timber-only starter processing and complementary later islands. Branch `feat/first-island-economy` targets `master`; the user has now authorized merging this PR. Fresh worlds generate only food/wood/stone and use authoritative resource-discovery unlocks. Starter menu: town center, house, granary, farm/fields, lumber mill, dock; research: Forestry/Agriculture/Masonry. Rations/infirmaries deferred. Old saves default to unrestricted rules and keep their world. Ship budget now uses wood/timber with reachable raw-budget coverage; ships, destination islands and local inventories remain explicit follow-ups in ROADMAP.md. No new sprites or asset transformations.

Verification: all 147 workspace tests pass (13 server, 39 client, 95 game), including 24-seed resource reachability, budget/processing and discovery/persistence/atomicity regressions. Native/WASM lint, formatting and release server/WASM builds pass. Desktop actual UI lumber-mill placement, exact 40 wood/10 stone construction cost, 10 wood processing charge and 5 timber completion passed without runtime errors; DPR-2 phone also passes all three menu groups with no runtime errors; screenshots /tmp/aoa-progression-*.png visually reviewed. PR opened: https://github.com/koogle/age-of-agents/pull/55 (base master); merging after preserving the newer activity-sprite fix. Review: docs/STARTER_ECONOMY_REVIEW.md. Rebased onto the native launch-script update dddcc08 without changing verified runtime code. Fresh runs still cannot depart until the transport follow-up lands; README and PR must state this clearly.

## Activity sprite restoration (integrated)

The farm-field commit `9ff0087` disabled work poses whenever cargo existed, freezing gatherers in the carry pose after their first collected item. The shared renderer now keeps active gathering animated with a partial load, while preserving cargo-first construction/cultivation and stopped/unloading carry poses. No asset or simulation changes.

Verification: all 142 workspace tests, formatting, strict native/WASM lint, frontend syntax and existing asset checks pass; release WASM rebuilt. Actual WebGL2 instance uploads confirm the original bundle holds one carry frame for all eight resource kinds, while the repaired bundle advances the correct chopping/mining/digging/foraging frames on desktop and DPR-2 phone with no page errors. Render tests also cover all three villager variants, empty/partial loads and cargo/work controls. Desktop screenshot visually reviewed; browser evidence/driver: `/workspace/scratch/activity-sprites/`. Review: [activity sprites](docs/ACTIVITY_SPRITES_REVIEW.md). Included in master at 44769d8 and preserved in the #55 integration.

## Current baseline

- Native rebuild/launch is available through `./scripts/run_native.sh`; it runs from any working directory and preserves `AGE_OF_AGENTS_DB`. Shell syntax, native launch, formatting, 139 Rust tests, both lint targets, and the code-quality review passed; live-browser verification was unavailable because this session has no connected browser.
- `master` includes the grouped 17-building catalog and production, farm fields, seeded reset, group selection, gathering/drop-off fixes, safe building placement, sprite/roof fixes, near-plane visibility, and distant globe zoom. See [README.md](README.md#implemented-roadmap) for playable features.
- Release `787de26` passed [quality, Modal deployment, and production verification](https://github.com/koogle/age-of-agents/actions/runs/37142115461). This supersedes the old pending-deployment notes.
- PR #49 merged the concise README, Greek roguelike direction, and `decisions.md`. This documentation follow-up addresses its three review comments: an expanding world, this handoff cleanup, and the FAL/ChatGPT refinement workflow. Local link/anchor and whitespace checks plus the documentation quality review cover these edits; no runtime changes.

## Remaining work

- **Island expansion:** starter rules and the reserved ship budget are implemented in #55. Remaining: dock-built transport, boarding/unloading, persistent destination islands and founding; then local inventories and trade. Preserve earlier islands as the world expands, and add discoveries to the globe after transport works.
- **Transport constraints:** the current generator retains one island, water blocks units, and each unit claims a land cell. Boarding must preserve unit identity/cargo and update occupancy validation; the global stockpile needs explicit local inventories/transfers, and Reset game cannot serve as island expansion.
- **Threats and progression:** combat, wolves, pirates, mythical creatures, calamities, treasures, and permanent upgrades remain unimplemented. Follow the [proposed gameplay loop](README.md#proposed-gameplay-loop); event timing, upgrades, and balance remain open.
- **HD assets:** several villager and military action/facing sheets remain 256 px and need source recovery or regeneration, refinement, and repacking. The last recorded audit found 183 of 264 frames below 512 px; rerun `python3 scripts/check_sprite_resolution.py --report-only` before asset work, and use the [asset workflow](AGENTS.md#asset-workflow).
- **Field placement safety:** `plant_field` checks free space and builder reachability, but does not call the route-preservation check used by building placement. Add bystander/last-exit regressions and prevent field-induced traps.
- **Useful advanced progression:** connect steel/bricks/cloth to tools, buildings and ship upgrades; the existing discovery gates and processing recipes alone do not complete that progression. Keep rations deferred until provisioning is useful.
- **Controls/accessibility:** additive touch selection and accessible DOM controls in the wgpu client remain open; desktop Shift-click/Shift-drag is implemented. Explicit blocking/non-blocking task classification is absent, though valid replacement orders work.
- **Verification gaps:** native reset-dialog appearance and a separate live-browser check of the deployed globe view remain unverified in the handoff. Local desktop/phone globe checks and automated production verification passed.

## Working constraints

Use an isolated SQLite database for local verification and preserve existing saves. Old recovery work remains on `codex/native-sprite-rendering`; historical root/gukaet edits and saves should remain untouched. Keep changes tied to the [current roadmap](ROADMAP.md#current-direction).
