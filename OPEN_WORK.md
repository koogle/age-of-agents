# Current work: environmental dangers (2026-10-05)

- Implemented the requested first calamity slice: deterministic escalating drought, food-gathering penalty, saved simulation clock, and shared client forecast. See [environmental dangers](docs/knowledge/environmental-dangers.md) for rules and verification entry points.
- Store version 12 resets incompatible saves, including version 11, under the accepted compatibility policy. No new assets or dependencies.
- All 232 Rust tests pass (one existing manual benchmark ignored); asset checks and rebuilt JS syntax pass. Desktop mouse, DPR2 phone touch and landscape browser checks pass without page errors; native and WASM strict lint pass; evidence is in [drought review](docs/verification/2026-10-05-drought/REVIEW.md). Not deployed. Rust and wasm-bindgen were absent from this fresh environment and are now installed. Modal SDK installation succeeded, but `python3 scripts/modal_manage.py status` failed with "Could not connect to the Modal server" despite ready runtime credential bindings; direct release is blocked on connectivity.
- Remaining danger work: balance, other calamities and eventual defeat; drought currently applies economic pressure without destruction or passive food consumption.

# Previous handoff: test runtime optimization (2026-10-05)

- [PR #97](https://github.com/koogle/age-of-agents/pull/97), branch `perf/test-runtime`, integrates master `3ccc5e8`. Implements the requested parallel soundness seeds, bounded destination searches, and simulation-only test optimization. All eight seeds, 9,600 ticks, debug assertions, route tie breaks and drop-site preferences are retained.
- Final integrated warm suite: 233 passed, one existing manual benchmark ignored, 52.74s wall (0.29s build check, 1.20s server, 15.72s client, 34.79s domain). Investigation baseline: 215 tests in 115.80s at `b054655`; upstream adds tests, so this is not an identical-suite benchmark. Formatting, native/WASM strict lint, three Python verifier tests, generated JS syntax and rebuilt WASM pass. Desktop gathering/deposit and DPR2-phone pinch/tap gathering pass without page errors on isolated seed-123 saves. User authorized merging PR #97. Subsequent integration with master `3ccc5e8` passes 226 tests (13 server, 75 client, 138 domain; one manual benchmark ignored), formatting, strict native/WASM lint and a rebuilt WASM bundle. Fresh isolated desktop mouse and DPR2-phone pinch/tap gathering pass without page errors. Merge/deployment state is tracked by the linked PR and production workflow.
- Thermonuclear review: no dependencies, new gameplay rules, persistence changes or removed assertions. Nearest-goal routes match complete-search paths across all 3×3 obstacle layouts; full connectivity queries stay complete. Many distant compatible drop sites remain a profiling caveat.
- Baseline instrumentation: 45,911 path searches consumed 90.19s of 103.36s in the serial soundness test. Full-map searches remain where complete connectivity is needed. Investigation artifacts: `/workspace/scratch/test-performance/`.

# Previous handoff: remove save backward compatibility (2026-10-05)

- Jakob requested removal of compatibility code and a PR. `STORE_VERSION` 11 transactionally resets different-version SQLite stores; current saves persist and corrupt current data remains an error. Policy and maintained save/archipelago guides are updated.
- Removed inventory pooling, archived island translation/models, legacy home-dock inference and historical deserialization defaults. Discovery writes terrain/resources directly to the shared world. No new dependencies or gameplay changes.
- Rebased onto master `2bddab9`, preserving construction-material unlocks, movement, field-replenishment and maintained knowledge. Final integration passes **224 Rust tests** (13 server, 75 client, 136 domain; one manual benchmark ignored), formatting, strict native/WASM lint, release WASM rebuild and generated JS syntax. Desktop mouse and emulated DPR2-phone touch decode current snapshots, pause and reload without page errors. Thermonuclear review passed: atomic version reset, explicit current corruption errors, deterministic discovery and global validation retained, no dependencies or gameplay additions.
- Isolated server checks verified that a current-version restart preserves the complete paused snapshot and a version-10 old-layout database resets to a playable world. Artifacts/databases: `/workspace/scratch/store-reset`; integration logs: `/tmp/aoa-pr-{tests,lint,wasm-lint,web}.log`.
- PR [#96](https://github.com/koogle/age-of-agents/pull/96) merged as `3ccc5e8`; its production workflow was queued when this integration began. Earlier Modal connection attempt failed; runtime now reports credentials ready, but deployment was not retried for this PR-only request.

## Remaining work and open PRs

- Other module-extraction/island-inventory proposals remain independent work; consult live PR metadata before integration. Construction-material unlocks and the knowledge folder are already in this PR’s master base.
- Profile populated archipelagos and refine the economy per [ROADMAP.md](ROADMAP.md); starter-only budget measurements do not establish busy-world capacity. Combat, destructive calamities and permanent progression remain proposals.
- Preserve [acceptance gaps](ROADMAP.md#recovered-acceptance-gaps--reviewed-2026-10-05): accessible DOM controls, additive touch selection, native macOS/Windows appearance and physical-phone safe-area verification, and documented narrow-passage/existing-blocked-field limits. Native/browser-local games remain in-memory.
- The production verifier still requires land units and unseen terrain; distinguish those fixture assumptions from actual deployment failures. Its old fixed-map/terrain-run bug is resolved.

## Verification and release

Local integration verification is recorded above. All 74 relative links in the eight changed Markdown files resolve; final formatting and whitespace checks pass. The PR does not merge or deploy. Confirm any later master release through workflow/production evidence rather than inferring it from PR state.
