# Current handoff: field-research icon (2026-10-05)

- Replaced Agriculture’s sickle-and-grain icon with a tilled field, seedlings and hoe through the existing FAL nano-banana/edit → BiRefNet → normalize_icons pipeline. The existing `tech_agriculture` key supplies available/disabled research buttons and queued research; no Rust, gameplay or WASM changes.
- Preserved the previous icon, generated source, cutout, prompt, references, request IDs and small-size comparison in `assets/ui/field_research_sources`. Updated the generation subject and contact sheet; estimated FAL cost $0.0448.
- Icon normalization, PNG/RGBA/transparency and 282-frame resolution checks pass. All 183 workspace tests, formatting and strict native/WASM lint pass. Desktop and DPR2-phone available/queued research inspected using staged snapshots; both browser checks pass with no page errors. QA artifacts are in `/workspace/scratch/field-research`.
- Thermonuclear review: asset replacement only, no new dependencies or game behavior; no source file growth beyond the existing prompt.
- Branch `feat/field-research-icon` starts from master, separate from timber PR #78. Modal remains unavailable in this environment; not deployed.

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
