# Current handoff: authorized knowledge-folder merge (2026-10-05)

- User authorized merging the documentation work. Branch `docs/maintained-project-knowledge` integrates master `0a863a4`, preserving merged movement #91, harvesting #88, selection #87 and the repaired terrain-run deployment verifier. The documentation diff changes no runtime source, art or generated browser output relative to that base.
- [The knowledge folder](docs/knowledge/INDEX.md) contains twelve focused system/procedure guides; the [skill](.agents/skills/project-documentation/SKILL.md) requires consulting relevant guides before changes and updating them throughout work, especially after developer steering. PR audit evidence remains in [the source record](docs/REWORK_LESSONS.md). Other ChatGPT/linked Claude transcripts were unavailable during that audit.
- Reconciled the guides with current master: #87/#88 are implemented; #86 was closed as superseded by #90; fixed-map/run-decoder verification was repaired in `250f3ce`. The release guide retains only actual remaining verifier limitations. Skill YAML description is quoted and validated with a YAML parser.

## Remaining work and open PRs

- Open PR metadata checked for this merge: [#72 module extraction](https://github.com/koogle/age-of-agents/pull/72), [#92 material-based construction](https://github.com/koogle/age-of-agents/pull/92), [#93 movement recovery](https://github.com/koogle/age-of-agents/pull/93), and [#94 island inventories/ship storage](https://github.com/koogle/age-of-agents/pull/94). These proposals are not the current baseline; review their live state before integration. This task authorizes only the documentation PR's merge.
- Profile populated archipelagos and refine the economy per [ROADMAP.md](ROADMAP.md); starter-only budget measurements do not establish busy-world capacity. Combat, calamities and permanent progression remain proposals.
- Preserve [acceptance gaps](ROADMAP.md#recovered-acceptance-gaps--reviewed-2026-10-05): accessible DOM controls, additive touch selection, native macOS/Windows appearance and physical-phone safe-area verification, and documented narrow-passage/existing-blocked-field limits. Native/browser-local games remain in-memory.
- The production verifier still requires land units and unseen terrain; distinguish those fixture assumptions from actual deployment failures. Its old fixed-map/terrain-run bug is resolved.

## Verification and release

Documentation checks cover local links/anchors, all twelve indexed guides, source references, YAML parsing, the protocol example, whitespace, and a documentation/scope review. No runtime tests or browser checks were rerun for Markdown-only integration. The upstream handoff reports 228 passing Rust tests, three verifier tests and combined desktop/phone acceptance; those are inherited results, not fresh checks by this task. Current runtime files and generated output are preserved exactly from the integrated base.

The authorized merge runs the existing master quality/deployment workflow. Confirm its actual result separately; do not infer deployment from merge or carry stale PR-body status forward.
