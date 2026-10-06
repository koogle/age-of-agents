# PR #103 rebase onto main — 2026-10-06

User requested rebasing against main; this repository uses `master`. Final
feature changes are replayed onto `50d02e1`, including gathering-loop preservation
when selecting buildings, territorial boars, neutral stone roads and stationary
NPC status messages. A local backup retains previous head `74f4705`. The feature
was consolidated against its prior merged base before replay so specialist
research written in merge resolutions remains intact.

Only handoff notes and generated browser output conflicted. Both handoffs are
preserved, and JS/WASM is rebuilt from combined source. Upstream storage-selection
logic, wildlife simulation/renderer/art, feedback source and stone-road texture
match main exactly. The building-selection call sites retain main's shared
carrier filter. Schema 17 and clay/material/research rules remain unchanged.

309 tests pass (13 server, 100 client, 196 domain; one ignored benchmark),
formatting and strict native/WASM lint pass, and web/server builds pass. All 392
sprite frames, icon normalization and six release-verifier tests passed after
the boar integration; the subsequent selection fix changes no assets or release
verification code. Desktop mouse and DPR2-phone touch building selection pass across all four
gathering phases, building switching and stopped-carrier deposit controls.
[Selection results and final bundle](selection-results.json). Desktop mouse and DPR2-phone touch inspection, upgrade, locked/pending research
rejection, payment, completion and reload pass without page errors.
[Final upgrade results and bundle](final-result.json) ·
[Final desktop hover](final-desktop-upgrade-hover.png) ·
[Final phone card](final-phone-locked.png). No merge or deployment is requested.

Earlier checks on base `806512c` passed 307 tests and 388-frame audits, desktop
mouse/DPR2-phone upgrade flows and stationary-label anchor/rise/fade/expiry replay.
[Historical upgrade results and bundle](result.json) ·
[Historical desktop hover](desktop-upgrade-hover.png) ·
[Historical phone card](phone-locked.png) ·
[Historical status results and bundle](status-results.json).
The status replay's bundle hash belongs to that earlier base; feedback source
remains identical through the final rebase. Intermediate base `de49ab4` also
passed 308 tests, 392-frame audits and desktop/phone upgrade flows before main's
building-selection fix arrived.

Thermonuclear review: no added runtime behavior or dependency during rebase;
feature source/assets survive, main's independent changes are retained, and
generated artifacts are rebuilt. Typed domain payment and save validation remain
covered by the full suite. Push uses an explicit force-with-lease against the
previously observed remote head, preserving concurrent remote changes.
