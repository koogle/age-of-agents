# Open Work

Updated 2026-10-03. Keep this file to current status, remaining work, and blockers; completed implementation and verification history lives in Git and the reviews under `docs/`.

Use `master` as the integration branch and PR base. The branch rename is cancelled; agents should preserve ongoing feature work and continue targeting `origin/master`.

## Activity sprite restoration (verified; publishing)

The farm-field commit `9ff0087` disabled work poses whenever cargo existed, freezing gatherers in the carry pose after their first collected item. The shared renderer now keeps active gathering animated with a partial load, while preserving cargo-first construction/cultivation and stopped/unloading carry poses. No asset or simulation changes.

Verification: all 142 workspace tests, formatting, strict native/WASM lint, frontend syntax and existing asset checks pass; release WASM rebuilt. Actual WebGL2 instance uploads confirm the original bundle holds one carry frame for all eight resource kinds, while the repaired bundle advances the correct chopping/mining/digging/foraging frames on desktop and DPR-2 phone with no page errors. Render tests also cover all three villager variants, empty/partial loads and cargo/work controls. Desktop screenshot visually reviewed; browser evidence/driver: `/workspace/scratch/activity-sprites/`. Review: [activity sprites](docs/ACTIVITY_SPRITES_REVIEW.md). Next: push to master and verify the resulting Modal deployment.

## Loading-screen title cleanup (2026-10-03)

Removed the isolated olive leaf overlapping the first "A" in `assets/loading/title.webp`. Used a generated repair only within a 64×54 px patch; visible pixels outside that patch and the central sprig are unchanged. Title dimensions, transparency, loading behavior and layout remain intact. Provenance is recorded in `assets/loading/README.md`.

Also removed shadow/cutout remnants in the transparent center of the "O" in "of", following user feedback. Used only a generated repair's opening alpha mask within 40×40 px at (568, 107), preserving original marble colors, the ring and central olive sprig. The center is fully transparent; pixel comparison confirms no visible changes outside this region. Desktop and phone `clean-o` previews pass.

Desktop (1280×800) and DPR-2 phone (390×844) loading-screen previews pass, with the WASM download held to inspect the animation and title. Screenshots are in `/workspace/scratch/loading-leaf/`. Existing depleted-resource, activity-asset and icon checks pass; pixel comparison verifies the localized repair bounds and unchanged remainder. Thermonuclear review: asset-only subtraction, no code/dependencies or gameplay changes. Rust tools are unavailable in this environment; merge CI runs quality and Modal deployment. PR #51 was closed when temporary `main` was removed. User authorized merging replacement PR #53; merged into `master` at `aa22862`, preserving concurrent activity-sprite work. Automatic quality/Modal deployment run https://github.com/koogle/age-of-agents/actions/runs/37149591115 is pending behind an earlier release; title cleanup is not yet confirmed live.

## Current baseline

- Native rebuild/launch is available through `./scripts/run_native.sh`; it runs from any working directory and preserves `AGE_OF_AGENTS_DB`. Shell syntax, native launch, formatting, 139 Rust tests, both lint targets, and the code-quality review passed; live-browser verification was unavailable because this session has no connected browser.
- `master` includes the grouped 17-building catalog and production, farm fields, seeded reset, group selection, gathering/drop-off fixes, safe building placement, sprite/roof fixes, near-plane visibility, and distant globe zoom. See [README.md](README.md#implemented-roadmap) for playable features.
- Release `787de26` passed [quality, Modal deployment, and production verification](https://github.com/koogle/age-of-agents/actions/runs/37142115461). This supersedes the old pending-deployment notes.
- PR #49 merged the concise README, Greek roguelike direction, and `decisions.md`. This documentation follow-up addresses its three review comments: an expanding world, this handoff cleanup, and the FAL/ChatGPT refinement workflow. Local link/anchor and whitespace checks plus the documentation quality review cover these edits; no runtime changes.

## Remaining work

- **Island expansion:** starter-resource/research rules and an achievable ship budget; dock-built transport, boarding/unloading, persistent destination islands and founding; then local inventories and trade. Preserve earlier islands as the world expands, and add discoveries to the globe after transport works.
- **Transport constraints:** the current generator retains one island, water blocks units, and each unit claims a land cell. Boarding must preserve unit identity/cargo and update occupancy validation; the global stockpile needs explicit local inventories/transfers, and Reset game cannot serve as island expansion.
- **Threats and progression:** combat, wolves, pirates, mythical creatures, calamities, treasures, and permanent upgrades remain unimplemented. Follow the [proposed gameplay loop](README.md#proposed-gameplay-loop); event timing, upgrades, and balance remain open.
- **HD assets:** several villager and military action/facing sheets remain 256 px and need source recovery or regeneration, refinement, and repacking. The last recorded audit found 183 of 264 frames below 512 px; rerun `python3 scripts/check_sprite_resolution.py --report-only` before asset work, and use the [asset workflow](AGENTS.md#asset-workflow).
- **Controls/accessibility:** additive touch selection and accessible DOM controls in the wgpu client remain open; desktop Shift-click/Shift-drag is implemented. Explicit blocking/non-blocking task classification is absent, though valid replacement orders work.
- **Verification gaps:** native reset-dialog appearance and a separate live-browser check of the deployed globe view remain unverified in the handoff. Local desktop/phone globe checks and automated production verification passed.

## Working constraints

Use an isolated SQLite database for local verification and preserve existing saves. Old recovery work remains on `codex/native-sprite-rendering`; historical root/gukaet edits and saves should remain untouched. Keep changes tied to the [current roadmap](ROADMAP.md#current-direction).
