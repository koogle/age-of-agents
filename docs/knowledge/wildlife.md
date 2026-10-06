# Dangerous wildlife

Read before changing animal behavior, hunting, unit health or animal rendering.

## User direction (2026-10-05)

The user corrected the environmental-danger request: add wolves and dangerous
animals instead of timed events, then suggested a bear. The drought in PR #98
was withdrawn; wolves and bears replace it. Bears replace the initially proposed
boars. Friendly units remain player-controlled; pursuit is for hostile animals.

The user subsequently requested one animal on the first island and more on the
second, then requested some variation in later-island counts. Generation retains one
starter wolf and chooses 2–4 animals per later island from the world seed and island
index, with one bear and the remainder wolves, preserving safe-start placement.

## Implemented rules

`crates/game/src/game/wildlife.rs` owns deterministic generation, explicit group
attack orders, animal pursuit/damage and cleanup. The first island gets one wolf; the second and later
islands get 2–4 animals (one bear and 1–3 wolves) where valid cells exist. Starting animals are at least 26
cells from friendly units/buildings, with homes 18 cells apart. They never respawn.
Idle animals at home skip occupancy/path reconstruction. Animals pursue nearby land units and return when targets
leave their territory; they do not roam randomly or attack buildings/ships.

| Kind | Health | Damage / second in contact | Speed (cells/s) | Aggro / territory radius |
| --- | --- | --- | --- | --- |
| Wolf | 40 | 8 | 2.5 | 6 / 10 |
| Bear | 100 | 16 | 1.8 | 4 / 6 |

All friendly units have 100 health. Villagers, guards, archers and siege carts
can receive `AttackAnimal { unit_ids, animal_id }`; healers reject attack orders.
Their contact damage is 10/25/18/35 per second respectively. This is a bounded
wildlife combat slice: ranged attacks, healing, building damage, factions/raids,
loot and run-ending rules remain unimplemented. Two villagers or a guard can
fight a bear more safely than a lone villager. Retreat is possible because
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

## Presentation and persistence

The shared mouse/touch path selects a friendly unit then taps an animal to attack;
without units selected, tapping reports species, health and instructions.
Selection details show friendly health, floating text keeps only the latest damage
feedback per entity, and red
rings identify wildlife. Authored sprites are documented in
[provenance](../../assets/sprites/wildlife_sources/README.md). Idle/walk frames use the shared simulation-speed animation clock, so pause freezes
the current stride and resume continues it. Attack orders are rejected while paused.
Frames are authored and mirrored; reverse-facing poses remain future art work.
Both animals now have authored attack windup and strike poses. Friendly hunting reuses existing villager chopping / military action art.

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
after rebuilding server/WebAssembly. This real-server fixture places guards and
animals near the starting camera, then uses actual desktop mouse and emulated
phone touch to select a guard and attack a bear. It verifies damage both ways,
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
