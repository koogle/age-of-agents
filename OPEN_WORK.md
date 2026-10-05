# Wildlife integration (2026-10-05)

- PR #98 merged into master as `0b5ce8b`, including `ba51a51` (island inventories, ship holds, shore-facing docks and automatic shore pickup). Final desktop mouse and DPR2-phone touch combat pass without page errors. Final integration suite: 251 tests pass (13 server, 79 client, 159 domain; one manual benchmark ignored); native/WASM strict lint and 298-frame asset audit pass. Combined save schema is version 14; release through GitHub Actions with production credentials.
- Review: [PR #98](https://github.com/koogle/age-of-agents/pull/98), **Add territorial wolves and bears**. Wildlife implementation `2d275e5`, followed by damage-feedback cleanup; merged, with production release tracked separately in GitHub Actions.
- Latest population tuning: one wolf on newly generated first islands; 2–4 animals (one bear and 1–3 wolves) on the second and later islands. Existing saved animals are preserved. Seeded variation verified across five seeds and three discoveries each; all 237 tests, strict native/WASM lint, rebuilt WASM and fresh desktop browser combat pass.
- User correction: wolves and other dangerous animals, instead of timed events. Drought changes in PR #98 have been reverted on its branch; replacing them with wolves and bears. See [wildlife](docs/knowledge/wildlife.md).
- Wildlife implemented with generated/refined wolf and bear art. All 237 Rust tests pass (one manual benchmark ignored), native/WASM strict lint and all 286 sprite-frame audits pass. Real desktop mouse and DPR2-phone pinch/touch hunting verify attack orders, two-way damage and bear defeat without page errors; final frame-anchor screenshots pass too. See [review/evidence](docs/verification/2026-10-05-wildlife/REVIEW.md). Store version 14 resets incompatible saves under existing policy. PR #98 is merged; confirm deployment through the release workflow.
- Historical direct release check (superseded by the Modal proxy/account findings below): Modal 1.5.3 status again reports "Could not connect to the Modal server" despite ready runtime bindings. No production state was changed.

# Open work

- Branch `fix/mobile-cargo-chevron-swipe`: mobile boat cargo now uses inline chevrons in a single 78px row, including 320px portrait and short landscape. Shared mouse/touch swipe paging cancels underlying transfers. Verified 252 combined workspace tests (one manual benchmark ignored), final cargo regressions, formatting, strict native/WASM lint, rebuilt browser bundle, DPR-2 touch paging and DPR-1 mouse paging/dragging. [Replay and visual evidence](docs/knowledge/hud-and-accessibility.md#ship-cargo-controls). Thermonuclear review completed: shared input path, unchanged authoritative cargo commands/save model, no added runtime dependency, all changed client files below 1,000 lines. User authorized merging [PR #101](https://github.com/koogle/age-of-agents/pull/101). Integrated master `0b5ce8b` (water-facing docks, shore pickup and wildlife), rebuilt the combined browser bundle, and passed the full suite, strict native/WASM lint and combined DPR-2 cargo replay. Production delivery follows the GitHub Actions release workflow after the authorized merge; see the Modal workspace guidance below.

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
