# Active menu icon audit

User requested missing dedicated menu icons and sequential PR merges. Audit: [coverage](docs/MENU_ICON_AUDIT.md). First change restores the existing Stop hand in land/ship commands. The complete audit covers 20 sequential fixes. Fourteen FAL originals and OpenAI refinements, plus four portraits from approved unit sprites, are prepared under `/workspace/scratch/menu-icons/generated/`; all 18 packed icons pass normalization. The Stop patch passes 232 Rust tests (one manual benchmark ignored); native/WASM lint, rebuilt browser bundle, asset checks and desktop/phone Stop dispatch all pass. Evidence is retained in `docs/verification/menu-icons/stop/`. Integrated upstream dock orientation and shore pickup through `ba51a51`; combined tests and lint pass, regenerated WASM resolves the only conflict. Replaying browser verification before first PR merge; remaining icons are pending. Existing unrelated handoff follows; #94 is merged and release `37379045726` for `75e7540` succeeded.

# Open work

- Automatic transport shore pickup: [PR #102](https://github.com/koogle/age-of-agents/pull/102), `feat/transport-shore-pickup`; user authorized PR creation and merge to master. Board orders choose a mutually reachable berth, preserve group reservations, and wait for the ship to stop. Explicit ship orders cancel pickup; no save schema changes. Integrated master `e7c8cb6` (shore-facing docks). Combined verification: 240 Rust tests passed (one manual benchmark ignored), strict native/WASM lint, formatting, 294-frame asset audit, transport/verifier checks and web/server rebuild. Isolated Chromium desktop mouse (1100×750) and DPR-2 phone touch (430×932) both issued Board, moved the ship to shore and boarded only once stopped, with no browser errors. Thermonuclear review passed for atomicity, deterministic routes, seat ownership, cancellation and crowded-shore waiting. QA server stopped. The PR tracks merge status; production release uses the merge-triggered GitHub Actions workflow, whose result must be checked separately.

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
