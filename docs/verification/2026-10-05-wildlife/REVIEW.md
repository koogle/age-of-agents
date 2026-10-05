# Wolves and bears review (2026-10-05)

The user requested dangerous animals instead of timed events, then suggested a
bear. PR #98 replaces the withdrawn drought with territorial wolves and bears.
Timing-based world events are absent from the final implementation.

## Behavior and maintainability review

One focused domain module owns animal generation, typed explicit hunting,
territorial pursuit, contact damage and death cleanup. Animals use the existing
occupancy and PathTree rules, including reserved step targets and no corner
cutting. Friendly units acquire no autonomous retaliation or task selection.

The initial balance gives a wolf 40 HP / 8 damage and a bear 100 HP / 16 damage.
Bears are slower and have a shorter pursuit range. Players can retreat or order
several units to attack; a guard can defeat a bear. These are provisional values.

Health, cooldowns, positions and orders are serialized; phases are not an event
system. Group rejection is atomic. Dead units release claims/reservations and
housing once; their carried load is lost, while stockpiles and paid building work
remain. Passengers retain health and are not attacked at sea. Dead animals clear
all hunting orders. Live animals are sent only under current fog visibility.

Store version 13 resets incompatible saves under the accepted no-migrations
policy, including version 11 and the withdrawn draft's version 12. Corrupt current
states return errors, including missing terrain, instead of panicking/resetting.

No new dependencies, general combat engine, factions, raids, weather, ranged
attacks, healing, building damage, respawns or loot. The new client interaction
module keeps the main client below 1,000 lines. The existing DOM accessibility
gap remains. Reverse-facing and dedicated animal attack sprites remain art work;
this slice has authored idle/walk frames, mirrored as needed, with measured feet
anchors. Friendly attacks reuse existing work/military action sprites.

## Evidence

237 Rust tests passed: 13 server, 76 client, 148 domain; one existing manual
benchmark remains ignored. Ten focused wildlife regressions cover deterministic
safe spawning, visibility/atomic rejection, attacks and cleanup, cargo loss,
pursuit/return, pause/reload, discovery, blocked diagonal contact and corruption.
Native/WASM strict lint and formatting pass. The 286-frame asset audit includes
four original 627px wildlife frames; all four have transparent corners. Field,
transport, icon and Python deployment-verifier checks pass.

The [browser driver](../verify_wildlife.py) uses a real isolated SQLite server,
seed 123, Chromium with software WebGL, desktop mouse at 1280×800 and emulated
phone pinch/touch at 390×844 DPR2. Its controlled fixture places two guards and a
wolf/bear near the camera; this does not represent natural spawn placement.
It asserts explicit UI attack orders, damage to both sides and bear defeat, and
records screenshots plus `results.json`. A follow-up desktop run (`results-desktop.json`) verifies the final damage-label de-duplication; its focused regression also passes. A phone close-up precedes zooming out to
bring the guards into its narrower viewport. Physical phones and native desktop
appearance are not covered by emulation.

Asset prompts, original draft, refinement, request/cost provenance and limitations
are retained in [wildlife sources](../../../assets/sprites/wildlife_sources/README.md).

Direct release remains blocked: Modal 1.5.3 status again returned "Could not
connect to the Modal server" with configured credential bindings ready. No
production state was changed.

## Population tuning (2026-10-05)

User direction: one wild animal on the first island and more on the second.
Generation now chooses one wolf for island zero and two wolves plus one bear
for later islands. Existing placement, fog, combat and persistence rules remain
unchanged. No schema bump or rewrite of saved animals.

Thermonuclear review: one local roster branch in the authoritative domain; no
new abstractions, dependencies, commands, client behavior or save fields. Focused
checks exercise three seeds through the first three islands and preserve earlier
animals during discovery. The browser driver also checks the natural starter
roster in the saved world before constructing its combat fixture.

Population follow-up verification passed: all 237 Rust tests (one manual benchmark
ignored), formatting, strict native/WASM lint, rebuilt server/WASM, JS syntax and
whitespace checks. Fresh desktop Chromium confirms the saved natural starter
roster is one wolf, then exercises hunting, two-way damage and defeat without
page errors. Logs and screenshots: `/tmp/aoa-population-browser`; phone interaction
was previously verified above and is unchanged by this generation-only adjustment.
The earlier Modal connectivity blocker remains; this follow-up is not deployed.

## Seeded population variation (2026-10-05)

The subsequent user request supersedes the fixed three-animal later-island roster:
later islands now choose 2–4 animals using the world seed and island index, with
one bear and the remaining animals wolves. The first island retains one wolf.
Existing saved animals and placement safeguards are unchanged. Focused tests
cover five seeds and three discoveries each, verify deterministic replay and
preservation of earlier animals, and observe all three possible counts.

Thermonuclear review: bounded local generation logic, existing hash mixer, no RNG
dependency or persisted random state, no client or command changes.

Random-count follow-up: all 237 Rust tests pass (one manual benchmark ignored),
strict native/WASM lint, formatting, rebuilt server/WASM, generated JS syntax,
document links and whitespace checks pass. Fresh desktop browser verifies natural
starter count and the combat fixture without page errors; evidence is in
`/tmp/aoa-random-wildlife-browser`. No deployment was performed; the previously
recorded Modal connectivity blocker remains.

## Authorized master integration (2026-10-05)

The user requested PR creation and merge to master. Reused PR #98 and integrated
master `e7c8cb6`, preserving island inventories, 50-resource ship holds and all
four shore-facing dock views. Resolved command variants by retaining both
AttackAnimal and TransferShipCargo; discovery initializes both inventory and
wildlife. Death tests now verify island inventories remain unchanged. Version 14
identifies the combined save schema and rejects incompatible versions via the
established reset policy. Generated WebAssembly is rebuilt from combined source.

Thermonuclear review: no features removed. The existing carriers_for method moves
unchanged to storage.rs to keep the shared client entrypoint under 1,000 lines.
Wildlife remains sheet 13; extended building atlas rows remain intact. Both asset
provenance ledger additions are retained.

Master advanced during verification to `ba51a51` (automatic shore pickup, PR #102).
Integrated that change as well; only the generated WASM conflicted and is rebuilt.
No source behavior from the pickup change was discarded.

Final source passes 251 Rust tests (13 server, 79 client, 159 domain; one ignored
manual benchmark), strict native/WASM lint, formatting and rebuilt WASM/JS checks.
The initial phone browser run exposed a verifier timing assumption: slow screenshot
capture allowed the surviving wolf to kill the guards before the final survivor
health assertion. The verifier now pauses for combat screenshots and records
injury/death throughout the sampled fight. Rerunning both input modes on the final
combined build before merge.

Final combined desktop mouse and DPR2-phone touch runs both pass: explicit attack,
two-way damage, bear defeat and no page errors. Evidence:
`/tmp/aoa-wildlife-final-browser/results.json` and sibling screenshots.
All local merge gates pass; deployment is checked separately after merging PR #98.
