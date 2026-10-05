# Current handoff: field-research icon (2026-10-05)

- Replaced Agriculture’s sickle-and-grain icon with a tilled field, seedlings and hoe through the existing FAL nano-banana/edit → BiRefNet → normalize_icons pipeline. The existing `tech_agriculture` key supplies available/disabled research buttons and queued research; no Rust, gameplay or WASM changes.
- Preserved the previous icon, generated source, cutout, prompt, references, request IDs and small-size comparison in `assets/ui/field_research_sources`. Updated the generation subject and contact sheet; estimated FAL cost $0.0448.
- Icon normalization, PNG/RGBA/transparency and 282-frame resolution checks pass. All 183 workspace tests, formatting and strict native/WASM lint pass. Desktop and DPR2-phone available/queued research inspected using staged snapshots; both browser checks pass with no page errors. QA artifacts are in `/workspace/scratch/field-research`.
- Thermonuclear review: asset replacement only, no new dependencies or game behavior; no source file growth beyond the existing prompt.
- PR https://github.com/koogle/age-of-agents/pull/82 (`feat/field-research-icon`) starts from master, separate from timber PR #78. Modal remains unavailable in this environment; not deployed.
## Authorized icon PR merge

User authorized merging PRs #78 and #82. Timber PR #78 merged as dda155f; field-research PR #82 is now integrated with that master, retaining both generated icons and rebuilding the combined contact sheet. Combined icon checks and a fresh DPR2-phone town-center/queued-research preview pass without page errors; no Rust or browser-bundle diff from the verified timber merge. Ready for the authorized field-research merge. Integrating current master (field-route fixes and action flashes), preserving both changes and rebuilding the browser bundle. Combined verification on master 9393efe passed all 190 tests, formatting, both strict lint targets, asset checks and a fresh-browser lumber-mill smoke. Compact HUD integration passed all 57 client tests, both lint targets and a fresh-browser smoke. Integrated fog-placement PR #79 on master ddf2123; rebuilt bundle, client tests, formatting, strict native/WASM lint and fresh-browser lumber-mill smoke all pass. Ready for the authorized timber merge.

# Current handoff: timber icon (2026-10-05)

- Generated a distinct sawn-timber resource icon through the existing FAL nano-banana/edit → BiRefNet → normalize_icons pipeline, using the existing wood art as reference. Source render, cutout, prompt, request IDs and preview are retained in `assets/ui/timber_sources`; estimated FAL cost $0.0448.
- Registered the icon in the shared Rust HUD and UI manifest; used it for timber stockpiles, lumber-mill production and queued timber jobs. Regenerated the UI contact sheet. Raw wood and simulation rules are unchanged.
- All icon normalization checks and the 282-frame sprite resolution audit pass; PNG/RGBA/transparent corners and light/dark/blue preview reviewed. All 183 workspace tests, formatting, native and WASM strict lint pass. Rebuilt the tracked browser bundle. Desktop and DPR2 phone lumber-mill selection with active/queued timber passes with no page errors; inspected desktop HUD at normal/close zoom and phone HUD at normal zoom using a paused fixture. The additional phone close-zoom screenshot timed out under software rendering; normal phone selection and screenshot passed. QA captures and fixture are in `/workspace/scratch/timber`.
- Thermonuclear review: small presentation-only mapping change, no dependencies or domain/persistence changes. No source files cross 1,000 lines.
- Deployment unavailable: this environment has no Modal tooling/profile. Review PR: https://github.com/koogle/age-of-agents/pull/78 (`feat/timber-icon`); not deployed.

# Current handoff: explore fogged building sites (2026-10-05)

- User requested a PR for building placement outside current sight. Branch `fix/explore-before-building` is based on master `61fc87c`.
- Fogged orders retain a typed, persisted exploration assignment. Villagers unload cargo first, explore, and create/pay for a foundation only after the whole footprint is currently visible and all existing placement checks pass. Stop/replacement, blocked/unreachable sites and lost affordability leave no foundation or charge. Visible placement remains immediate.
- Mouse/touch placement accepts fogged building sites; selection shows “Exploring build site.” Field preparation retains its existing behavior. README/roadmap and tracked browser bundle updated; no new assets or dependencies.
- Validation covers 199 tests: the full integration suite passed (13 server, 56 client, 128 domain), followed by all 58 client tests after the client-only compact-HUD merge. Strict native/WASM lint, formatting and generated JS syntax checks pass; browser bundle rebuilt. Seven new regressions cover exploration and its action feedback.
- Thermonuclear review: construction code is consolidated in a 140-line domain module; atomic placement is reused, pending sites own no cells or costs, bounds and worker type validate on reload, and no idle behavior or new navigation machinery was added.
- Real browser menu → fog placement → exploration → completed house passes on desktop 1280×800/DPR1 and phone 390×844/DPR2 with touch input. Authoritative snapshots confirm no early foundation/cost, exactly one house and exactly 15 wood spent; no page errors. QA scripts, fixture and screenshots are under `/workspace/scratch/fog-build`.
- User authorized merging PR #79. Integrated master `7aaafdb` (action flashes, field-route safety and compact HUD); added exploration status/reassignment coverage. Final checks pass, including the compact-HUD touch menu → exploration → completed house flow with exactly 15 wood spent and no browser errors. Ready for the authorized merge. The existing Modal workflow deploys master merges.

# Previous handoff: compact mobile HUD (2026-10-05)

The requested shared Rust layout places actions bottom-left beside an 80px globe and a single row of 44px pause/1×/2× targets above it. Selection text, full queues and building submenus wrap upward within the left column; picking a building collapses the menu. Painted artwork is preserved. Earlier stacked mobile studies are superseded.

Rebased onto current master `61fc87c`, retaining transport, islands, field harvesting and zoom-scaled edge panning. All 185 workspace tests, formatting, strict native/WASM Clippy and rebuilt browser bundle pass. Layout tests include ship controls and pass after the landscape queue-width refinement. Browser-local phone/menu/landscape replay has no page errors; authoritative combined touch checks pass for speed changes, full queues and cancellation/refunds. Final phone, submenu, placement, narrow-phone, landscape and desktop captures are saved in `docs/verification/2026-10-05/`; the browser replay has no page errors. Review: `docs/COMPACT_HUD_REVIEW.md`. Direct Modal deployment fails with “Token missing”; no production changes have been made. PR #76: https://github.com/koogle/age-of-agents/pull/76. User authorized merging with pause aligned beside 1× and 2×. Integrated master `9393efe` (field route safety and action flashes); all 192 combined tests, formatting, strict native/WASM lint and rebuilt-browser DPR2 pause smoke pass. Ready for the user-authorized merge; release verification follows the merge-triggered workflow.

# Previous handoff: field gathering audit (2026-10-05)

- Dedicated branch `fix/field-gathering-routes`; user has now authorized merging PR #80. Integrated master `4b6fe9e` (action flashes); source merges cleanly, handoffs preserved, combined bundle rebuilt; all 190 workspace tests, formatting and strict native/WASM lint pass. Fresh-browser smoke of the combined bundle also passes; ready for the authorized merge.
- Confirmed fields already share ordinary gathering, deposits, resumption and same-kind continuation. Found and reproduced a field-placement gap: a permanent plot could cut off delivery routes while remaining reachable for preparation. Reuses building placement's route-preservation guard before mutation/spending.
- Added regressions for atomic rejection from either side of a bottleneck, complete harvesting around a legal bypass across reload, and field/wild-food continuation in both directions. All ten focused field tests pass; the rejection test failed before the fix. All 186 workspace tests, formatting and strict native/WASM lint pass; rebuilt the tracked browser bundle. Desktop mouse and DPR2 phone touch each completed the prepare → harvest → six deliveries → idle flow with no page errors, using an isolated SQLite fixture under `/workspace/scratch/field-audit`. Rebuilt-client smoke also passes. Dedicated PR: https://github.com/koogle/age-of-agents/pull/80 (open, unmerged). No production deployment performed.
- Audit and thermonuclear review: `docs/FIELD_GATHERING_REVIEW.md`. Existing blocked layouts are not migrated. Separately observed the existing idle-villager blockage in one-cell traffic; no general movement behavior change in this PR. User's precise saved layout is unavailable.

# Previous handoff: villager action flashes (2026-10-05)

- Extended the existing 1.4-second floating italic drop-off feedback to gathering (with resource name), building, field preparation, boarding, and idle transitions. Plain movement does not flash. Labels follow the villager and rapid assignment changes replace the previous status; resource gains retain separate labels with vertical separation on simultaneous status changes.
- Feedback compares authoritative snapshots and assignment targets; repeated snapshots and movement-to-work gathering phases do not replay it. Initial snapshots/new units stay quiet, and reset/island changes clear labels. No simulation, persistence, command, asset, or dependency changes.
- Added four focused client tests and adapted drop-off/gain coverage. All 187 workspace tests (55 client, 13 server, 119 domain), formatting, strict native and WASM lint pass. After the final viewport correction, reran all 55 client tests and both lint targets; rebuilt the tracked browser bundle. Desktop 1280×800 and phone 390×844/DPR2 gather/stop captures show the new labels, accepted commands and no page errors; walking captures stay quiet. QA uses an isolated database and a controlled presentation clock for stable captures, under `/workspace/scratch/status-flashes`.
- Thermonuclear review: retained the existing feedback module and animation, with one assignment-label helper; typed snapshot comparison stays presentation-only. Source remains below 1,000 lines, common mouse/touch command paths unchanged, no autonomous behavior. Floating text now shifts inside viewport edges and fits narrow screens to prevent clipped messages; reviewed desktop and phone previews after the correction.
- User approved the visual result and authorized creating a PR and merging into the default branch (`master`). Branch `feat/villager-action-flashes` contains the verified change; merging triggers the repository quality/deploy workflow. Direct Modal deployment remains unavailable in this environment.

# Previous handoff: cursor edge panning (2026-10-04)

- Increased full edge speed to 0.75 viewport heights per second (about 5× the previous horizontal speed). Screen-space conversion follows orthographic zoom and compensates for vertical isometric foreshortening; diagonal speed remains normalized.
- Preserved the 32-logical-pixel ramp, focus/HUD/drag/touch guards, camera bounds, and keyboard/drag controls. Updated README and roadmap controls descriptions.
- Added projection-based zoom/DPR/direction coverage plus frame-rate and pan-bound checks. All 183 workspace tests (13 server, 51 client, 119 domain), formatting, and strict native/WASM lint pass. Rebuilt the tracked browser bundle. Browser mouse checks passed for horizontal/vertical panning at three zoom levels on DPR1 and DPR2 (about 581–595 CSS pixels per game second at one pixel from the edge of an 800px-high viewport). DPR2 phone drag and stationary edge-touch checks pass with no page errors. QA scripts/logs are in `/workspace/scratch/edge-pan`; measurements account for the existing 0.25-second frame cap under software rendering.
- Thermonuclear review: small client-only change, no dependency or simulation/persistence changes, no files over 1,000 lines, existing input guards retained.
- User authorized merging this change into master through a PR. Branch `fix/zoom-scaled-edge-pan` is based on current master `fe75673`; local verification is complete. Direct Modal deployment is unavailable because this environment has no Modal profile; merging triggers the repository production workflow.

# Previous handoff: persistent islands (2026-10-04)

## Integration for authorized merge of PR #73

User explicitly requested merging PR #73. Integrated master `3003c75`, including merged field-preparation PR #74: preparing a field now proceeds into harvesting, with twelve HD hoeing poses. Transport retains sprite slot 11; field preparation uses slot 12. Source changes merge cleanly; rebuilt generated browser assets instead of choosing either conflicting bundle. Combined verification passed: 181 workspace tests (13 server, 49 client, 119 domain), formatting, both strict lint targets, rebuilt WebGL bundle, all 282 HD frames, and field/transport asset checks. Ready for the explicitly authorized merge. See `docs/FIELD_PREPARATION_REVIEW.md` for field-specific evidence and limitations.

## Implemented on `feat/island-discovery`

- Based on merged transport PR #71 (`0864057`). Fresh island PR: https://github.com/koogle/age-of-agents/pull/73 (merge explicitly authorized; integration and checks underway).
- First completed transport generates island 2 exactly once. Existing ships in old saves can discover it on departure. Explicit previous/next voyage controls carry a stopped ship, its passengers and goods; frontier voyages generate further deterministic islands without a fixed cap.
- Discovery-order IDs and root-seed mixing; persistent terrain, fog, resources, buildings, units, ships, orders and island-local inventories. Away islands pause; research, discovered unlocks and entity counters are global. Cross-island duplicate research is rejected.
- Complementary, biome-appropriate reachable deposits: island 2 iron/coal, island 3 clay, island 4 fiber; repeat thereafter. Initial arrival is beside open ocean and a shore connected to resources.
- Shore unloading supports founding from transported wood (town center 20, dock 30); loading still requires a completed dock. Voyages are atomic and reject moving ships, unreachable ocean, invalid destinations and blocked arrival.
- Shared Rust mouse/touch controls show current island/count, previous/next voyage buttons and discovery details. Camera and terrain interpolation reset on island changes. Island terrain/resource sprites reused; transport visual refinement is recorded below.

## Transport visual follow-up

- User requested the transport match the small boat beside the wharf. Replaced the bulky striped-sail design with a low timber hull, plain cream sail and simple benches using OpenAI image_gen with the approved dock as reference. Original output and provenance retained.
- Render size reduced from 2.8 to 1.6 world units; selection ring from 0.7 to 0.4. Updating existing PR #73. Gameplay capacities and simulation unchanged.
- All 47 client tests, sprite packing/transparency/resolution/icon checks and strict native/WASM lint passed. Desktop and DPR2 phone selection/sailing passed; inspected default and maximum zoom, including real touch pinch. Final previews are in `/workspace/scratch/ship-refinement`. Reviewed the visual-only diff against the thermonuclear gate; no simulation or capacity changes.

## Interior layout correction

- User identified inconsistent bench counts and moving sacks in the authored views. Current art has exactly three benches, a mast mounted on the middle bench, and one sack in the stern bay in both views. Bow/stern views naturally place that compartment at opposite ends of the image.
- Retained draft/final image_gen sheets and exact prompts. Existing packer produces two registered 512px frames from native 887px panels; no renderer or simulation change.
- Resolution/transparency/registration checks pass. Desktop/phone four-facing previews checked at normal and close zoom under `/workspace/scratch/ship-layout`, with no browser errors. Manually checked three benches and the same stern sack compartment in the authored and mirrored views. Updating PR #73.

## Ship rendering correction

- Fixed sea-depth clipping of the lower hull by applying the same proportional footprint depth pull used by other billboards. Fixed reverse-view mirroring so the bow follows projected movement.
- Two regression tests cover projected bow direction and the complete keel clearing maximum wave height. All 49 client tests and strict native/WASM lint pass; rebuilt browser bundle. Desktop/phone four-facing previews checked at normal and close zoom in `/workspace/scratch/ship-render`.
- This correction updates open PR #73; it is not deployed. No sprite-generation or simulation changes.

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
