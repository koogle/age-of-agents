# Current handoff: remove save backward compatibility (2026-10-05)

- Jakob requested removal of compatibility code and a PR. `STORE_VERSION` 11 transactionally resets different-version SQLite stores; current saves persist and corrupt current data remains an error. Policy and maintained save/archipelago guides are updated.
- Removed inventory pooling, archived island translation/models, legacy home-dock inference and historical deserialization defaults. Discovery writes terrain/resources directly to the shared world. No new dependencies or gameplay changes.
- Rebased onto master `2bddab9`, preserving construction-material unlocks, movement, field-replenishment and maintained knowledge. Final integration passes **224 Rust tests** (13 server, 75 client, 136 domain; one manual benchmark ignored), formatting, strict native/WASM lint, release WASM rebuild and generated JS syntax. Desktop mouse and emulated DPR2-phone touch decode current snapshots, pause and reload without page errors. Thermonuclear review passed: atomic version reset, explicit current corruption errors, deterministic discovery and global validation retained, no dependencies or gameplay additions.
- Isolated server checks verified that a current-version restart preserves the complete paused snapshot and a version-10 old-layout database resets to a playable world. Artifacts/databases: `/workspace/scratch/store-reset`; integration logs: `/tmp/aoa-pr-{tests,lint,wasm-lint,web}.log`.
- PR [#96](https://github.com/koogle/age-of-agents/pull/96) is open against master on `refactor/drop-save-backward-compatibility`; no merge or production deployment performed. Earlier Modal connection attempt failed; runtime now reports credentials ready, but deployment was not retried for this PR-only request.

## Remaining work and open PRs

- Other module-extraction/island-inventory proposals remain independent work; consult live PR metadata before integration. Construction-material unlocks and the knowledge folder are already in this PR’s master base.
- Profile populated archipelagos and refine the economy per [ROADMAP.md](ROADMAP.md); starter-only budget measurements do not establish busy-world capacity. Combat, calamities and permanent progression remain proposals.
- Preserve [acceptance gaps](ROADMAP.md#recovered-acceptance-gaps--reviewed-2026-10-05): accessible DOM controls, additive touch selection, native macOS/Windows appearance and physical-phone safe-area verification, and documented narrow-passage/existing-blocked-field limits. Native/browser-local games remain in-memory.
- The production verifier still requires land units and unseen terrain; distinguish those fixture assumptions from actual deployment failures. Its old fixed-map/terrain-run bug is resolved.

## Verification and release

Local integration verification is recorded above. All 74 relative links in the eight changed Markdown files resolve; final formatting and whitespace checks pass. The PR does not merge or deploy. Confirm any later master release through workflow/production evidence rather than inferring it from PR state.
