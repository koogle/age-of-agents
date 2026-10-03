# Open Work

Updated 2026-10-03. Keep this file to current status, remaining work, and blockers; completed implementation and verification history lives in Git and the reviews under `docs/`.

Use `master` as the integration branch and PR base. The branch rename is cancelled; agents should preserve ongoing feature work and continue targeting `origin/master`.

## Mouse camera controls — PR #52

Branch `fix/pointer-anchored-zoom`, PR https://github.com/koogle/age-of-agents/pull/52, targets `master`. Wheel zoom stays anchored under the mouse, pinch zoom under the fingers’ midpoint. Mouse hover within a 32-logical-pixel edge band gently pans in that direction; corners pan diagonally with bounded speed. The speed ramps near the border and follows frame time. HUD hover, dragging, touch input, pointer exit and focus loss suspend edge panning. Existing camera bounds and globe view remain; no simulation, save, asset or dependency changes.

Verification: all 143 workspace tests, formatting, strict native/WASM Clippy and rebuilt release WASM pass. Four focused camera/input regressions cover zoom anchoring, globe/limits, edge/corner directions, speed ramp and DPR/outside/interior positions. Actual desktop wheel in/out drift stays below 0.004 CSS pixels and DPR-2 phone pinch below 0.008. Browser corner panning passes at DPR 1 and 2, with zero drift in the interior, outside the view, over HUD controls, on focus loss and while holding a drag. DPR-2 phone touch at the edge produces zero camera drift. No browser runtime errors; screenshots reviewed. Evidence: `/workspace/scratch/pointer-zoom/`. Existing asset/syntax checks passed during zoom work.

Thermonuclear review: one zoom method and one direct edge-direction function reuse existing camera picking/drag/nudge; native/browser share the implementation, no new engine layer, all changed client files below 1,000 lines. Integrated latest master `dddcc08` documentation cleanup and native launch script; merge/deployment pending through the existing merge-triggered Modal workflow.

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
