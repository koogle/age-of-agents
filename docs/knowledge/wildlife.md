# Dangerous wildlife

Read before changing animal behavior, hunting, unit health or animal rendering.

## User direction (2026-10-05)

The user corrected the environmental-danger request: add wolves and dangerous
animals instead of timed events, then suggested a bear. The drought in PR #98
was withdrawn; wolves and bears replace it. Bears replaced the initially proposed boars in that first release. Friendly units remain player-controlled; pursuit is for hostile animals.

The user subsequently requested one animal on the first island and more on the
second, then requested some variation in later-island counts. That release retained one
starter wolf and chose 2–4 animals per later island from the world seed and island
index, with one bear and the remainder wolves. Boars now extend this roster below.

## Boar addition (2026-10-06)

Jakob requested integrating the retained boar concept as another danger, with a
small first-island presence and more on the second. Implementation retains wolves/bears, adds one starter boar and 2–3 boars on later islands using
the existing territorial combat and safe-start rules.

The user explicitly clarified that the supplied concept requires refinement in
the game style. Use it for identity only; the approved NPC-matched animal family
owns rendering style. The first detailed-fur pass is rejected and retained; the
second pass removes hatching in favor of broad cel-painted masses.

## Strength tuning (2026-10-06)

Jakob requested substantially more animal health and damage, with one wolf requiring
3–5 archers. Full-health contact tests yield zero survivors with one/two archers,
one with three, three with four, and four with five. Positioning and staggered
arrival can change casualties. Existing saved current health is preserved; increased
damage applies immediately, and new animals spawn at the increased maximum.
There is no schema change or world reset.

## Lion prides (2026-10-09)

Jakob asked for more monsters, one class per PR with art previews first; lions
are the first class from the [open work](../../OPEN_WORK.md) list. Every island
after the first gets one pride: a maned **Lion** (the leader form) and 2–3
**Lionesses**, homes within 3 cells of the leader, placed with the same safe-start
rules as other wildlife. Pride ids continue the island's `animal-{island}-{n}` numbering.

Pride rule, implemented in `tick_wildlife`: lions whose homes lie within
`PRIDE_RADIUS` (5 cells) form one pride. When any member has a living unit inside
its own aggro radius and territory, every member targets the nearest unit inside
its own territory, so the pride converges together. Each lion still never leaves
its territory, and other species never join a pride alert. Both kinds stay slower
than villagers (3 cells/s off-road), so retreat still works. No schema change: old
saves load; only islands discovered after the update gain prides.

Art: `assets/sprites/lion_sources/` (prompts, renders, cutouts, provenance) packed
as atlas rows 3–4 by `scripts/pack_wildlife.py`. Designs (Jakob, 2026-10-10): the
lioness is an island huntress (faint rosettes, black-backed ears, white throat),
the leader a Nemean lion; at Jakob's request both are naturalistic and
cel-shaded at a medium detail level, with a final model ink pass that matches
the wolf's outlines. Line work is the image model's job: Jakob removed the
packer's thinning and re-ink steps on 2026-10-10 under the asset rules. Sources are 1024px cells downsampled
into 627px frames; quads are 0.95 and 1.0, so the leader stands about a fifth
taller ([asset lesson](asset-pipeline.md#new-species-from-the-family-sheet-2026-10-10)).
Style review: [lion review](../verification/lions/README.md).

## Implemented rules

`crates/game/src/game/wildlife.rs` owns deterministic generation, explicit group
attack orders, animal pursuit/damage and cleanup. The first island gets one wolf and one boar; the second and later
islands get one bear, 1–3 wolves, 2–3 boars and one lion pride where valid cells exist. Starting animals are at least 26
cells from friendly units/buildings, with homes 18 cells apart. They never respawn.
Idle animals at home skip occupancy/path reconstruction. Animals pursue nearby land units and return when targets
leave their territory; they do not roam randomly or attack buildings/ships.

| Kind | Health | Damage / second in contact | Speed (cells/s) | Aggro / territory radius |
| --- | --- | --- | --- | --- |
| Boar | 60 | 10 | 2.2 | 4 / 8 |
| Wolf | 300 | 35 | 2.5 | 6 / 10 |
| Bear | 600 | 50 | 1.8 | 4 / 6 |
| Lioness | 250 | 30 | 2.8 | 6 / 12 |
| Lion | 500 | 45 | 2.4 | 6 / 12 |

All friendly units have 100 health. Villagers, guards, archers and siege carts
can receive `AttackAnimal { unit_ids, animal_id }`; healers reject attack orders.
Their contact damage is 10/25/18/35 per second respectively. This is a bounded
wildlife combat slice: ranged attacks, healing, building damage, factions/raids,
loot and run-ending rules remain unimplemented. Wolves require a squad; bears
are stronger still and defeat three archers in the contact fixture. Retreat is possible because
villagers move faster than predators. No friendly automatic retaliation.

Attack orders validate all members and current target visibility before mutation.
Stop/reassignment remains explicit. Personal cargo stays with a living hunter;
when a unit dies, its cargo is lost and its occupied/reserved cells and housing
are released by removal. Stockpiles and paid building work remain unchanged.
Dead animals are removed once and all orders for that animal become idle.

Animals claim both their current cell and step target through the existing
occupancy map. PathTree routes stay inside home territory, respect reservations,
and cannot cut corners. Contact attacks cannot strike through blocked diagonal
corners. Snapshot animals require current visibility at both step endpoints;
remembered terrain does not expose live hostile positions. Hidden targets cancel
hunting rather than enabling pursuit into fog using private coordinates.

## Health display direction (2026-10-06)

Jakob requested thin overhead health bars for villagers and animals, green at high
health then orange and red as it falls, replacing numeric health text. Implemented in the shared client: a 24×3 logical-pixel fill with a dark one-pixel
border follows each visible friendly unit and animal. Green is above 50%, orange
is 25–50%, and red is at or below 25%; animal fractions use species maximums.
Combat damage is shown by the shrinking bar instead of floating numbers;
status and cargo feedback retain their existing rise/fade behavior.
[Replay and review](../verification/health-bars/README.md) cover desktop and
emulated DPR2 phone presentation.

## Presentation and persistence

The shared mouse/touch path selects a friendly unit then taps an animal to attack;
without units selected, tapping reports species and attack instructions.
Overhead bars show health, and red rings identify wildlife. Authored sprites are documented in
[provenance](../../assets/sprites/wildlife_sources/README.md). Idle/walk frames use the shared simulation-speed animation clock, so pause freezes
the current stride and resume continues it. Attack orders are rejected while paused.
Frames are authored and mirrored; reverse-facing poses remain future art work.
All three animals have authored attack windup and strike poses. Friendly hunting reuses existing villager chopping / military action art.

Store version 14 combines island inventories and ship cargo with required health and wildlife state. Incompatible stores
(including 12 and the earlier branch-only wildlife schema 13) reset under the existing
[save policy](server-and-saves.md). Current corrupt health, steps, cooldowns,
claims or orders fail validation. Population tuning does not change the save schema:
existing animals remain intact on reload; the roster applies when an island is generated. Passengers retain health and remain safe at sea.

## Verification

Focused tests: `cargo test -p aoa-game --locked wildlife_tests`.
They cover generation/safe starts, fog, atomic rejection, hunting, death/cargo,
pursuit/return, pause/reload, discovery, corner attacks and malformed saves.
Existing randomized soundness, queues, cargo and transport suites remain active.

Run `python3 docs/verification/verify_wildlife.py --output /tmp/aoa-wildlife-check`
after rebuilding server/WebAssembly. This real-server fixture places four archers and full-health
animals near the starting camera, then uses actual desktop mouse and emulated
phone touch to select an archer and attack a wolf. A subsequent explicit WebSocket
group order recruits the squad (additive touch selection is not implemented). It verifies damage both ways,
defeat and cleanup, captures screenshots and checks page errors. The controlled
fixture is not evidence of natural spawn placement (domain tests cover that).
See [review/evidence](../verification/2026-10-05-wildlife/REVIEW.md).

Browser-verifier timing: software-rendered screenshot capture can take enough
wall time for combat and subsequent attacks to finish. Pause authoritative
simulation while capturing combat frames, and track unit injury/death across
polls rather than requiring an injured survivor at the final sample.

## NPC-matched art (2026-10-05)

The animal style correction uses the shipped villager family and guard originals as direct references, in addition to the island diorama. Broad cel color areas and fine brown contours replace the original detailed fur. `scripts/pack_wildlife.py` registers the four idle/walk poses at y=590 in 627px cells; attack frames align planted rear paws to idle while the renderer retains that baseline. Sources, the rejected first pass and exact final prompt remain in `assets/sprites/wildlife_sources/`. No animal rules or save schema change. The browser verifier accepts `--closeups` to capture each animal at the camera minimum distance in both desktop and DPR-2 phone viewports, then reloads the paused fixture before interaction acceptance.

Maximum-zoom capture detail: wheel input clamps each event to a 0.5 scale factor in `lib.rs`; use multiple events (eight cover the full 140-to-5 distance range), not one extreme delta. A large delta alone is not proof of maximum zoom.

## Attack-frame extension (2026-10-05)

At Jakob's request, the animal art refinement adds wolf bite and bear swipe windup/strike poses in the NPC-matched style. The existing authoritative `attack_seconds` timer drives them, so idle/proximity alone never triggers attacks and pause freezes the combat pose. Combat balance and save schema are preserved; heading faces the contacted target even when the animal did not need to move first.

The renderer uses columns 0/1 for idle/walk, column 2 for `0 < attack_seconds < 0.6`,
and column 3 for `attack_seconds >= 0.6`. The damage-timer reset supplies an idle
recovery beat; a moving animal uses locomotion. No client proximity heuristic
starts an attack. The simulation updates heading on clear melee contact, so a
stationary animal faces its target. Damage cadence, balance and save schema are
unchanged. Paused snapshots preserve attack poses independent of wall time.

`docs/verification/replay_animal_attacks.py` serves controlled paused snapshots
on loopback :8013 and observes actual WebGL instance UVs for both species,
windup/strike/recovery and mirroring. It captures normal and maximum zoom on
desktop/DPR-2 phone. This isolates visual acceptance; the real-server wildlife
verifier and domain tests still establish combat behavior.

Boars append a third 627px atlas row with idle, walk, windup and tusk-strike
poses. Existing wolf/bear pixels are preserved. Existing saves load unchanged;
new boars appear only when generating an island (reset for a new starter roster).
Run the browser verifier with `--target boar --closeups` for boar mouse/touch
hunting and maximum-zoom evidence.

The pose replay observes all three species. On the 2026-10-06 managed cloud
workspace, SwiftShader exceeded the old 30-second UV-update wait despite
correct rendered frames; allow 120 seconds with UV/error/screenshot diagnostics.
Run browser replays sequentially to avoid competing software renderers. This
verification timeout is not a measurement of performance on physical devices.

## Boar style review reopened (2026-10-06)

Jakob rejected the integrated boar style as not there yet after viewing the
gameplay preview. The previous style-pass assessment is superseded; gameplay
verification remains valid. Re-evaluate against approved villager/guard originals
and the primary diorama directly, with the supplied boar as identity reference.
The selected replacement is described below; the prior assessment is superseded.

## Boar direction selected (2026-10-06)

Jakob explicitly selected the first of the two follow-up studies,
`boar-npc-study.png` (lighter taupe coat and broad cel shading). This user choice
supersedes the agent’s earlier negative assessment of that study. Preserve its
identity, proportions, palette and fur shapes while extending walk and attack
poses; do not substitute the second, more textured silhouette study. The selected family is packed into the third atlas row from
`boar-approved-animation.png`; original 627px cells are never enlarged.
Stronger-wolf browser fixtures start the attack target outside its six-cell aggro
radius. Otherwise resuming simulation before clicking can move the wolf away
from its captured screen position and dispatch a ground move instead of an attack.
Balance evidence and reproduction: [stronger wildlife](../verification/stronger-wildlife/README.md).
