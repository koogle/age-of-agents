# Current handoff: maintained knowledge folder (2026-10-05)

- Checkout: `work` at `b054655`, the merge of [PR #90](https://github.com/koogle/age-of-agents/pull/90), plus local documentation edits. Runtime baseline remains the continuous archipelago. This review did not verify production deployment; the old PR body saying “not merged” is superseded by live merged metadata.
- Developer clarification: preserve system documentation, interaction procedures, research and reusable learnings in individual files, consult them before changes, and maintain them throughout work, especially after steering. Created [docs/knowledge/](docs/knowledge/INDEX.md) with twelve focused guides and a reading-trigger index; the [skill](.agents/skills/project-documentation/SKILL.md) now defines ongoing consultation/update checkpoints and a document scaffold. The former large retrospective is now only a [source record and topic routes](docs/REWORK_LESSONS.md). Other ChatGPT conversations/full linked Claude transcripts remained unavailable during the earlier history review.

## Open PRs checked during the earlier 2026-10-05 review

| PR | State and next action |
| --- | --- |
| [#72: module extraction](https://github.com/koogle/age-of-agents/pull/72) | Open, unmerged. Reassess its scope and overlap against current domain/archipelago code before further integration; the proposed module tree is not the current source layout. |
| [#86: original dock return](https://github.com/koogle/age-of-agents/pull/86) | Open, unmerged. #90 already incorporated the home-dock behavior for continuous sailing; reconcile remaining differences before deciding this older map-exchange PR's disposition. Do not merge its obsolete travel implementation. |
| [#87: selection modifiers](https://github.com/koogle/age-of-agents/pull/87) | Open, unmerged. Proposes Shift replacement and Control/Command additive selection; current checkout still uses the older Shift behavior. Review/integration remains separate work. |
| [#88: harvesting during replenishment](https://github.com/koogle/age-of-agents/pull/88) | Open, unmerged. Proposes retaining an existing harvester's assignment during paid field preparation; current gathering still treats the empty node as exhausted. Review/integration remains separate work; no automatic replanting or recruitment is authorized by that proposal. |

The user authorized creating and merging the documentation PR. Integrate current master and reconcile the dated PR statuses below before merging.

## Remaining work

- Repair the deployment verifier's stale fixed-map/uncompressed-terrain assumptions before relying on it for current archipelago acceptance; details and evidence are in the [release guide](docs/knowledge/build-integration-and-release.md#known-verifier-mismatch) and roadmap. This task documented the source mismatch; it did not change runtime code or reproduce a production incident.

- Profile populated archipelagos and refine the economy per [ROADMAP.md](ROADMAP.md). [Map budgets](docs/CONTINUOUS_MAP.md) are starter-population measurements; dense terrain/search arrays, full-map textures and `u16` coordinates remain scaling limits. Combat, calamities and permanent progression are still proposals.
- Preserve [recovered acceptance gaps](ROADMAP.md#recovered-acceptance-gaps--reviewed-2026-10-05): accessible DOM controls, additive touch selection, native macOS/Windows appearance and physical-phone safe-area verification, and the documented one-cell-traffic/existing-blocked-field limits. Native/browser-local games are in-memory; hosted SQLite does not imply local persistence.
- For any requested release verification, inspect the relevant current workflow/deployed revision and affected real flow. Historical test counts, PR descriptions and expired scratch paths are not fresh release evidence.

## Verification of this documentation change

Reviewed current server envelope/commands, persistence, terrain codec, client source, build scripts, FAL helper, Modal verifier and CI definitions to ground the guides. Validation passed across 25 Markdown files: 186 local links/anchors (including 37 direct source links), all twelve guides indexed with reading triggers, skill metadata/references, the WebSocket example against the current command fields, and whitespace. Documentation review confirms topic ownership, preserved historical evidence and no claim that the verifier mismatch is fixed. Example commands and historical test results are not freshly executed runtime verification. No application tests or browser checks were rerun for the documentation edits. Commit/push/merge are now authorized; release state will be checked separately.
