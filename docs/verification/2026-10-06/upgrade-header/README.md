# Consistent building upgrade header — 2026-10-06

Every completed building uses the same brick medallion at the top-right of its
description. Available, locked, pending and completed upgrades retain that
placement and a 44px target. Production/research stay below; action hover preserves
building identity. Specialist descriptions name the research upgrade payoff.

No hard building or upgrade count limit is added. Rare-resource costs for future
advanced upgrades remain proposed. Current payment uses island-local stores and
stopped connected ships; individual building inventories are not introduced.

Initial verification: 291 Rust tests pass (13 server, 97 client, 181 domain; one
manual benchmark ignored), formatting, strict native/WASM lint and web/server
builds pass. The all-building layout matrix covers four states, 320/390px portrait,
844px landscape and 1440px desktop, checking target position, size, hover stability
and dispatch. Existing layout checks cover overlaps.

Actual Chromium desktop mouse and DPR2 phone touch both pass locked research,
upgrade, paid research and persisted completion with no page errors. Reproduce
with `docs/verification/check_brick_research.py` and the research fixture example.
[Results and tested bundle hash](result.json) · [Preview](preview.jpg).
Physical-device/native appearance and accessible DOM controls remain unverified.

Thermonuclear review: upgrade copy extracted from the near-limit selection file;
layout uses the existing typed action and shared input path. No new dependency,
authoritative gameplay rule, limit, resource tier or save-model change is added by
the header. Approved brick artwork is reused. Title wrapping reserves the button
space and full-width body text sits below it. All changed client files stay below
1,000 lines.

## Integration with current master

Integrated `0c7b649`, preserving roads, required water balances, stronger wildlife,
removed build-subcategory cards and refined menu icons. Reconciled typed road and
upgrade commands and kept both field-cost and upgrade-control tests. Selection
tests now live in their own file to keep the runtime module comfortably below
1,000 lines. Snapshot fixtures include roads/water and the material/research
catalog. Generated browser files are rebuilt from the combined source.

Master already uses save version 16 for water. The combined version is 17, with
explicit reset coverage for version 16; corrupt current saves remain errors.
This is an integration requirement, not a persistence change for button placement.

Combined checks: 307 Rust tests pass (13 server, 99 client, 195 domain; one
manual benchmark ignored), formatting, strict native/WASM lint and rebuilt
web/server pass. All 388 sprite frames, transport/field asset checks and six
release-verifier tests pass. Relative documentation links resolve.

Final combined desktop mouse and DPR2 phone touch upgrade/research/payment/reload
flows both pass with no page errors. [Final results](merged/result.json),
[combined checks](combined-checks.json) and [final preview](final-preview.jpg).

The subsequent [brick-arrow refinement](brick-arrow/README.md) supersedes the
round brick button and persistent description copy shown in earlier captures.
