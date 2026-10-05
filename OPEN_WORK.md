# Current fix: automatic bush gathering continuation (2026-10-05)

- Branch `fix/food-gathering-continuation`, integrating master `e7c8cb6` (water-facing docks). Reproduced automatic idling after delivery with nearby bushes still live: continuation used only the last exhausted node, making the 10-cell search depend on harvesting order.
- Fixed `next_resource` to measure from the connected exhausted wild-resource patch. Keeps same-kind/reachable candidates, deterministic ordering, bounded range, individual field behavior and explicit Stop. No save schema or inventory changes; already-idle workers still need a new player order.
- A minimal regression fails before the fix and passes at 1×/2×. Seven generated seeds exercise repeated two-worker deliveries at 2×. Reconstructed visible production geometry also continues into the formerly missed patch. The snapshot reconstruction is not a full save; direct Modal API reads failed to connect, and production has not been mutated in diagnosis.
- Full workspace: 233 passed, one existing benchmark ignored; strict native/WASM lint, formatting and generated-JS syntax checks pass. Browser baseline (isolated hosted Chromium, 2×, one gather order and no later commands) delivers 20 food then idles with 40 nearby food remaining. The rebuilt fixed browser delivers all 60, leaves 30 distant food untouched and has no page errors. [Evidence](docs/verification/2026-10-05/bush-delivery-2x.json). Thermonuclear review passes: one domain search change, no new dependencies/state, bounded same-kind reachability and feature preservation. PR #100 is open. Release run 37383382481 passed quality but was cancelled before deployment when PR #99 merged; integrating its dock changes before rebuilding and rerunning release checks.

# Integrated upstream handoff

## Shore-facing docks (current workspace, 2026-10-05)

- Implemented automatic water-edge facing shared by domain coast validation,
  placement ghosts and placed docks. Added east/north/west sprites, all four
  construction stages, retained source/provenance and a reproducible packer.
  Original building atlas rows are pixel-identical; no save-model change.
- Verification: full workspace 235 tests passed (one manual benchmark ignored),
  including snapshot edge/fog regression; all five
  building geometry tests pass across every facing/stage. Native/WASM lint,
  294-frame sprite audit and rebuilt WebGL passed. Desktop four-shore captures
  and selection pass; DPR-2 phone replay also passes all four directions and
  construction stages (32 captures, no browser errors). Evidence and review:
  [dock-facing verification](docs/verification/dock-facing/README.md).
- User authorized merging the dock changes on 2026-10-05: [PR #99](https://github.com/koogle/age-of-agents/pull/99),
  branch `feat/water-facing-docks`. Production checks use the GitHub Actions
  release workflow after merge; inspect its result before claiming deployment.
  Modal 1.6.1 needed its API proxy extra;
  installing `modal[api-proxy-support]` fixed connectivity. The configured
  workspace is `radiantai`, not the repository's production `koogle-frick`, and
  no local profiles exist. Direct deployment was not attempted; the existing
  GitHub release workflow uses the production credentials.

## Earlier release follow-up

- Branch `feat/island-storage-ships`, PR #94: https://github.com/koogle/age-of-agents/pull/94. Integrating master through `58813c1`. Implements island-local inventories and 50-resource ship holds in addition to four passengers. Cargo stays aboard on arrival; stopped shore ships supply local construction/production/research and accept partial villager deposits. Training/research still require buildings.
- Storage sites unify completed compatible buildings and available ship holds. Costs reserve once from shore stores then connected ships; outputs/refunds go to the job’s island. Explicit load/unload transfers require a completed dock, reject atomically, and do not require NPC labor. Storage targets use `storage_id`; incompatible stores reset rather than migrate.
- Top resources follow pointer/touch island inspection, including minimap hover. Selected building costs use its own island. Ship cargo is now a single horizontal strip with one column per resource, resource icons without name labels, onboard counts and Load/Unload shields at each icon’s lower corners (24px art, generous tap targets); cargo-only types remain visible and additional columns paginate on compact screens. Dock selection also exposes moored ship cargo when artwork obscures the vessel.
- User authorized merging PR #94. Integrating master `58813c1` before release: preserve construction unlocks, movement recovery, bounded route searches and removal of save backward compatibility. Store version 12 resets incompatible saves; no inventory migrations or legacy aliases. Integrated verification: 231 Rust tests pass (13 server, 77 client, 141 domain; one manual benchmark ignored), strict native/WASM lint, rebuilt web/server, verifier/asset checks and documentation links. Combined desktop mouse and DPR-2 phone touch load/unload acceptance passed without browser errors. Task-owned QA server stopped. Captures/logs: `/workspace/scratch/island-storage/merge-browser.log`, `desktop-cargo-merged.png`, `phone-cargo-merged.png`. PR #94 merged as `330803f`. Its first release run was cancelled before deployment because the verifier still read the removed shared-stockpile field. Verifier repair `75e7540` is now deployed and verified: [production run 37379045726](https://github.com/koogle/age-of-agents/actions/runs/37379045726) succeeded.

## Remaining work and open PRs

- Other module-extraction proposals remain independent work; consult live PR metadata before integration. Island inventories are delivered by PR #94. Construction-material unlocks and the knowledge folder are already in this PR’s master base.
- Profile populated archipelagos and refine the economy per [ROADMAP.md](ROADMAP.md); starter-only budget measurements do not establish busy-world capacity. Combat, calamities and permanent progression remain proposals.
- Preserve [acceptance gaps](ROADMAP.md#recovered-acceptance-gaps--reviewed-2026-10-05): accessible DOM controls, additive touch selection, native macOS/Windows appearance and physical-phone safe-area verification, and documented narrow-passage/existing-blocked-field limits. Native/browser-local games remain in-memory.
- The production verifier still requires land units and unseen terrain; distinguish those fixture assumptions from actual deployment failures. Its old fixed-map/terrain-run bug is resolved.

## Verification and release

Local integration verification is recorded above. All 74 relative links in the eight changed Markdown files resolve; final formatting and whitespace checks pass. PR #94 is merged; the subsequent verifier repair released successfully in the workflow linked above. Dock merge/release status is tracked at the top.
