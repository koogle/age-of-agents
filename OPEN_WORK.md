# Current handoff: remove save backward compatibility (2026-10-05)

- Jakob requested removal of compatibility code and a PR. `STORE_VERSION` 11 transactionally resets different-version SQLite stores; current saves persist and corrupt current data remains an error. Policy and maintained save/archipelago guides are updated.
- Removed inventory pooling, archived island translation/models, legacy home-dock inference and historical deserialization defaults. Discovery writes terrain/resources directly to the shared world. No new dependencies or gameplay changes.
- Rebased onto current master `2bddab9`, preserving construction-material unlocks, movement, field-replenishment and maintained knowledge. Final integration checks and combined browser rebuild are in progress; earlier isolated verification passed 208 Rust tests, native/WASM lint and desktop/phone pause/reload. A server restart preserved the complete paused snapshot; a version-10 old-layout database reset to a playable world.
- Earlier Modal connection attempt failed. Runtime now reports credentials ready, but this request is to open a PR; no merge or production deployment performed.
- Evidence and isolated databases: `/workspace/scratch/store-reset`. Final integration results and PR URL will replace the pending status before handoff.

## Remaining work and open PRs

- Open PR metadata checked for this merge: [#72 module extraction](https://github.com/koogle/age-of-agents/pull/72), [#92 material-based construction](https://github.com/koogle/age-of-agents/pull/92) and [#94 island inventories/ship storage](https://github.com/koogle/age-of-agents/pull/94). These proposals are not the current baseline; review their live state before integration. This task authorizes only the documentation PR's merge.
- Profile populated archipelagos and refine the economy per [ROADMAP.md](ROADMAP.md); starter-only budget measurements do not establish busy-world capacity. Combat, calamities and permanent progression remain proposals.
- Preserve [acceptance gaps](ROADMAP.md#recovered-acceptance-gaps--reviewed-2026-10-05): accessible DOM controls, additive touch selection, native macOS/Windows appearance and physical-phone safe-area verification, and documented narrow-passage/existing-blocked-field limits. Native/browser-local games remain in-memory.
- The production verifier still requires land units and unseen terrain; distinguish those fixture assumptions from actual deployment failures. Its old fixed-map/terrain-run bug is resolved.

## Verification and release

Final documentation checks pass: 25 Markdown-only files, 182 local links/anchors, all twelve indexed guides, source references, parsed YAML, the protocol example, whitespace, and a documentation/scope review. No runtime tests or browser checks were rerun for Markdown-only integration. The upstream handoff reports 228 passing Rust tests, three verifier tests and combined desktop/phone acceptance; those are inherited results, not fresh checks by this task. Current runtime files and generated output are preserved exactly from the integrated base.

The authorized merge runs the existing master quality/deployment workflow. Confirm its actual result separately; do not infer deployment from merge or carry stale PR-body status forward.
