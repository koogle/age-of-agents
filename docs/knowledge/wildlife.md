# Dangerous wildlife

Read before changing animal behavior, hunting, unit health or animal rendering.

## User direction (2026-10-05)

The user corrected the environmental-danger request: add wolves and dangerous
animals instead of timed events, then suggested a bear. The drought in PR #98
was withdrawn; wolves and bears replace it. Bears replace the initially proposed
boars. Friendly units remain player-controlled; pursuit is for hostile animals.

## Implemented rules

`crates/game/src/game/wildlife.rs` owns deterministic generation, explicit group
attack orders, animal pursuit/damage and cleanup. Each generated island gets two
wolves and one bear where valid cells exist. Starting animals are at least 26
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
Selection details show friendly health, floating text reports damage, and red
rings identify wildlife. Authored sprites are documented in
[provenance](../../assets/sprites/wildlife_sources/README.md). Idle/walk frames
are authored and mirrored; reverse-facing and animal attack poses are future art
work. Friendly hunting reuses existing villager chopping / military action art.

Store version 13 adds required health and wildlife state. Incompatible stores
(including 11 and the withdrawn drought's 12) reset under the existing
[save policy](server-and-saves.md). Current corrupt health, steps, cooldowns,
claims or orders fail validation. Passengers retain health and remain safe at sea.

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
