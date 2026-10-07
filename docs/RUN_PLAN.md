# Run plan — archipelago, artifact and monsters

Status: proposal written 2026-10-06 from Jakob's direction; nothing below is
implemented. Phases are ordered so each one is playable on its own and the next
one builds on it. Code paths are relative to the repository root and were
inspected at `dcc7ac0`.

## Jakob's direction (2026-10-06)

Close the loop on a single complete run:

- Every run has 5–7 islands spread over a map, not in a line. The whole
  archipelago layout is chosen first; each island's terrain is generated as the
  player progresses.
- The final island holds the goal: a temple with a treasure chest, the
  Artifact of the Gods. It is revealed to the player at the start of the run.
- Each island is harder than the last: more monsters, stronger monsters.
- New monster classes, all in the existing NPC-matched cel style: barbarians,
  skeletons, cyclops, giant snakes, minotaur, centaur, lions. Classes need
  variety (a cyclops general, a barbarian chieftain) and different behaviors.
- The run becomes a capture-the-flag game: reach the artifact and bring it home.

## What exists today

| System | Current behavior | Source |
| --- | --- | --- |
| Island placement | Deterministic square spiral, 64-cell ocean gaps, no island cap; the next island generates when a ship is within 12 cells of the current island's frontier edge | `crates/game/src/game/islands.rs` |
| Island content | Terrain-first worldgen per island index; resources by discovery tier (iron/coal, clay, fiber) | `crates/game/src/game/worldgen.rs` |
| Danger | One `Animal` entity type with three kinds (wolf, bear, boar), territorial pursuit, contact damage, explicit group attack orders, no respawn, no building damage, no ranged attacks | `crates/game/src/game/wildlife.rs` |
| Friendly military | Guards, archers, healers, siege carts train; all attacks are contact attacks, healing is unimplemented | `crates/game/src/game/domain.rs` |
| Run state | `ScenarioState` with `Running`/`Won`/`Lost` exists in the snapshot but no objective ever sets it | `crates/game/src/game/domain.rs` |
| Globe minimap | Shows discovered terrain and the camera marker; nothing for undiscovered islands | `crates/client/src/hud/minimap.rs` |
| Saves | Store version 17; incompatible versions reset the hosted world | `src/store.rs` |

## Decisions Jakob should confirm before Phase 1

1. **Bounded run.** A run's archipelago has a fixed size of 5–7 islands. This
   replaces the current "no island-count cap" statement in README, AGENTS and
   ROADMAP. Recommendation: keep "the world keeps expanding" as the between-run
   promise (later runs, bigger or different archipelagos), not within one run.
2. **Win rule.** Recommendation: the run is won when a unit carrying the artifact
   boards a ship and that ship leaves the final island's waters. Phase 2 ships
   the simpler "pick up the artifact" win first and adds the escape in the same
   phase, so the capture-the-flag feel arrives before monsters do.
3. **Loss rule.** Recommendation: the run is lost when the player has no living
   units on land or at sea and cannot train one (no completed town center with
   food). Monsters do not attack buildings until Phase 3 makes that possible.
4. **Save impact.** Phases 1, 2 and 4 each change persisted state (planned
   islands, temple/artifact, creature kinds). Each bumps `STORE_VERSION` and
   resets incompatible hosted worlds under the existing policy; no migrations.

## Phase order

### Phase 1 — Archipelago layout and reveal

Goal: at run start the player sees where the islands are and which one holds
the temple. Islands still generate on approach.

Domain (`crates/game`):

- Replace `island_origin(id)` with a seeded `ArchipelagoPlan` stored in
  `GameWorld`: 5–7 island slots on a jittered 3×3 or 4×3 slot grid of
  `120×80` regions with the existing 64-cell gaps. Slots stay axis-aligned so
  the dense terrain rectangle, snapshot codec, minimap and camera bounds keep
  working; measured budgets in [continuous map](CONTINUOUS_MAP.md) cover a
  3×4 slot grid (about 240k cells).
- Each planned island has an origin, a difficulty tier (1 = start, N = final),
  and a role (`Start`, `Waypoint`, `Final`). Tier is the island's rank by sea
  distance from the start; the farthest slot is final. Keep the current
  resource-tier progression (iron/coal, clay, fiber) keyed by tier, not by
  discovery order, so a player who skips an island still finds a coherent
  economy.
- Generation trigger: a ship within 12 cells of any undiscovered slot's
  bounding rectangle generates that island. The Explore shortcut sails to the
  nearest undiscovered slot. `validate_islands` checks origins against the plan.
- Snapshot adds the plan (slot rectangles, tier, role, discovered flag) so the
  client can draw it. Bump `STORE_VERSION`.

Client (`crates/client`):

- Globe minimap draws undiscovered slots as faint island silhouettes (one
  painted generic outline per slot, same for every slot) and a temple marker on
  the final slot; discovered islands draw as today.
- A small text line in the resource bar area: "Island 2 of 6". No run briefing
  screen yet.

Verification: `islands_tests` cover plan determinism per seed, that all slots
are generated after sailing to each, that the final slot is the farthest,
that no two runs with the same seed differ; the ignored `archipelago_budget`
measurement rerun on a full 7-island plan; browser check of the globe reveal on
desktop and DPR-2 phone.

### Phase 2 — Final island, temple, artifact, win and loss

Goal: a run can be won and lost through the real UI without any new monster.

- Worldgen for the `Final` role places a **Temple** (a generated, pre-complete
  building kind, 5×5, on a level plot near the island center) with the
  **Artifact of the Gods** inside. The artifact is a typed world entity, not a
  resource, so it never enters a stockpile.
- Orders: tapping the temple with a villager selected issues `ClaimArtifact`.
  The villager walks to the temple's front cell and picks the artifact up,
  which replaces its cargo (cargo is dropped). A unit holding the artifact
  cannot gather or build and shows a carry pose with the artifact sprite.
- Win: `ScenarioOutcome::Won` when the artifact bearer boards a ship that then
  leaves the final slot's rectangle (decision 2). Loss per decision 3. A
  terminal outcome rejects gameplay commands except reset, as Slice E already
  specifies, and the client shows a victory or defeat panel with a New run
  button that reuses the existing seeded reset.
- Art: temple (two states: sealed, opened), treasure chest, artifact carry
  sprite, globe temple marker. All through the FAL pipeline against the
  approved building and villager families; style acceptance is a merge gate.
- Bump `STORE_VERSION`.

Verification: domain tests for claim, drop, death of the bearer (artifact
returns to the temple), ship escape win, loss detection, and terminal command
rejection; browser run from first island to the final island with a developer
seed that places the final slot adjacent to the start.

### Phase 3 — Combat rules the monsters need

Goal: the friendly military and the danger system support everything the
monster classes in Phase 5 require, tested with the existing wildlife.

1. **Ranged attacks.** Archers hold a bounded range (6 cells), need line of
   sight through the existing visibility rules, and fire on a deterministic
   cadence. Centaurs and barbarian archers reuse the same rule.
2. **Building health and creature attacks on buildings.** Buildings get
   health; a destroyed building releases its plot and refunds nothing.
   Needed for barbarian raids and for a meaningful loss rule.
3. **Healing.** Healers restore friendly health within a short radius, never
   above maximum, never hostiles. This is the counterweight to tougher islands.
4. **Death and corpse cleanup** stay as today; no respawn for creatures.

Each rule lands as its own PR with focused tests; none changes the save schema
except building health.

### Phase 4 — Creature framework and difficulty tiers

Goal: one creature entity with a kind table and a small set of behaviors,
tuned per island tier, with wolves, bears and boars as the tier-1 roster.

- Rename `Animal` to `Creature`; `CreatureKind` gains a `rank`
  (`Regular`, `Veteran`, `Leader`) that scales health and damage and gives a
  leader a retinue at spawn. This is data, not a hierarchy.
- Behaviors as a typed enum with the fields each needs, implemented directly
  in `wildlife.rs` (split the file when it nears 1,000 lines):
  - `Territorial { aggro, territory }` — today's wolves, bears, boars.
  - `Pack` — one member's aggro pulls the whole group; retreats together.
  - `Ambush { reveal_radius }` — hidden from snapshots until a unit is very
    close, then attacks with high damage.
  - `Ranged { range, keep_distance }` — shoots and backs away from melee.
  - `Raider { camp }` — patrols between camp and nearby points; attacks the
    nearest visible friendly building, then units.
  - `Charger { windup, dash }` — straight-line charge with a telegraphed
    windup; bonus damage on the first contact.
  - `Guardian { post }` — never leaves its post; blocks the temple approach.
  - `Spawner { kind, count }` — a lair structure that releases a group once
    when disturbed.
- Spawn tables per tier: a small constant table mapping tier → list of
  `(kind, rank, count range, placement rule)`. Placement rules reuse the
  existing safe-start distances and territory spacing; raider camps and lairs
  are footprint structures placed by worldgen.
- Fog, occupancy, pathing, health bars and snapshot privacy stay exactly as
  implemented for wildlife.

Bump `STORE_VERSION` once for the rename and rank field.

### Phase 5 — Monster classes, one PR each

Ordered by how much new behavior each needs, so early classes are cheap and
later ones build on proven rules. Each PR ships behavior, tests, FAL art in the
NPC-matched style (idle, walk, windup, strike, mirrored facings) and a browser
replay, with the style review as the merge gate.

| Order | Class | Tier | Behavior | Variants |
| --- | --- | --- | --- | --- |
| 1 | Lions | 2 | `Pack` of 2–4, fast, wolf-like stats | Lioness (regular), maned lion (leader) |
| 2 | Giant snakes | 2–3 | `Ambush` near water and resource clusters, slow, high damage | Python (regular), hydra-scaled serpent (veteran) |
| 3 | Skeletons | 3 | `Spawner` barrow ruins release 4–8 weak melee skeletons | Warrior, archer (`Ranged`), bone champion (leader) |
| 4 | Barbarians | 3–4 | `Raider` camp with patrols; attack buildings | Warrior, archer, chieftain (leader, retinue) |
| 5 | Centaurs | 4–5 | `Ranged` kiting herd, fast | Hunter (regular), centaur lord (leader) |
| 6 | Cyclops | 5–6 | `Territorial` boss, slow, sweep hits every adjacent unit | Cyclops, cyclops general (leader with barbarian retinue) |
| 7 | Minotaur | Final | `Guardian` at the temple; `Charger` when units enter its court | Minotaur, labyrinth keeper (veteran) |

Tier tables stack: a tier-4 island keeps some tier-2 and tier-3 creatures and
adds the new ones at higher counts. Existing wildlife stays the tier-1 roster.

### Phase 6 — Run balance, run framing and release

- Balance a whole run on three fixed seeds: starter island economy to a dock,
  first crossings, military build-up, the final assault and escape. Target a
  bounded play session without developer shortcuts (Slice H criterion 1).
- Run framing in the UI: a short start panel naming the goal and showing the
  globe reveal, per-island difficulty shown on the globe, the victory and
  defeat panels from Phase 2 with the run's island count and time.
- README, ROADMAP, AGENTS and the wildlife guide updated to the bounded run;
  the wildlife guide becomes the creature guide.
- Release through the existing Modal workflow after the thermonuclear review.

## Out of scope for this plan

Permanent progression between runs, calamities, pirates at sea, day/night,
autonomous friendly units and any pathfinding or AI framework. Add nothing of
this before Phase 6 ships.

## Suggested first PRs

1. Phase 1 domain: `ArchipelagoPlan`, approach-based generation, Explore to the
   nearest unknown slot, tests.
2. Phase 1 client: globe silhouettes, temple marker, island counter.
3. Phase 2 domain: temple, artifact, claim order, win and loss.
4. Phase 2 art: temple, chest, artifact carry sprite.
