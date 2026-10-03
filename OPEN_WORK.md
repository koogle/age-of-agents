# Open Work

Updated 2026-10-03. Keep only outstanding work, blockers and verification follow-ups here. Implemented features are summarized in [README.md](README.md); completed history lives in Git and `docs/`.

Use `master` as the integration branch and PR base.

## Pending release verification

- Merge user-authorized [NPC highlight PR #56](https://github.com/koogle/age-of-agents/pull/56), then confirm its production release. The final combined build passes all 151 workspace tests, strict native/WASM lint, formatting, asset checks and desktop/phone browser verification. Selected units use larger ivory rings with blue borders; existing marks and gameplay retain their paths. Structural review passed; no new dependencies or rendering layers. Evidence: `/workspace/scratch/highlight-*`.

- Confirm [starter-economy production run 37150108528](https://github.com/koogle/age-of-agents/actions/runs/37150108528) succeeds, then verify the fresh-island economy on live `/play`. PR #55 is merged; deployment is pending. Fresh runs still cannot depart until transport is implemented.
- Confirm [camera/activity production run 37149731725](https://github.com/koogle/age-of-agents/actions/runs/37149731725) succeeds and verify the hosted camera controls, gathering animations and loading-title cleanup. This combined run is in progress; the earlier title-only run was cancelled. Local desktop/phone verification already passed.

## Remaining work

- **Island expansion:** starter rules and the reserved ship budget are implemented in #55. Remaining: dock-built transport, boarding/unloading, persistent destination islands and founding; then local inventories and trade. Preserve earlier islands as the world expands, and add discoveries to the globe after transport works.
- **Transport constraints:** the current generator retains one island, water blocks units, and each unit claims a land cell. Boarding must preserve unit identity/cargo and update occupancy validation; the global stockpile needs explicit local inventories/transfers, and Reset game cannot serve as island expansion.
- **Threats and progression:** combat, wolves, pirates, mythical creatures, calamities, treasures, and permanent upgrades remain unimplemented. Follow the [proposed gameplay loop](README.md#proposed-gameplay-loop); event timing, upgrades, and balance remain open.
- **HD assets:** several villager and military action/facing sheets remain 256 px and need source recovery or regeneration, refinement, and repacking. The last recorded audit found 183 of 264 frames below 512 px; rerun `python3 scripts/check_sprite_resolution.py --report-only` before asset work, and use the [asset workflow](AGENTS.md#asset-workflow).
- **Field placement safety:** `plant_field` checks free space and builder reachability, but does not call the route-preservation check used by building placement. Add bystander/last-exit regressions and prevent field-induced traps.
- **Useful advanced progression:** connect steel/bricks/cloth to tools, buildings and ship upgrades; the existing discovery gates and processing recipes alone do not complete that progression. Keep rations deferred until provisioning is useful.
- **Controls/accessibility:** additive touch selection and accessible DOM controls in the wgpu client remain open; desktop Shift-click/Shift-drag is implemented. Explicit blocking/non-blocking task classification is absent, though valid replacement orders work.
- **Verification gaps:** native reset-dialog appearance and a separate live-browser check of the deployed globe view remain unverified in the handoff. Local desktop/phone globe checks and automated production verification passed. Focused browser verification of villager direction/pose hold on zigzag routes also remains unrecorded.

## Working constraints

Use an isolated SQLite database for local verification and preserve existing saves. Old recovery work remains on `codex/native-sprite-rendering`; historical root/gukaet edits and saves should remain untouched. Keep changes tied to the [current roadmap](ROADMAP.md#current-direction).
