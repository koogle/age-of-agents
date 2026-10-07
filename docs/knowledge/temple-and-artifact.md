# Temple and Artifact of the Gods

Read before: changing the run's goal, the temple, the artifact bearer, win/loss
state, or the final island's generation.
Status: current guide; implemented on `claude/laughing-goodall-10lotl` (2026-10-07).
Last source review: 2026-10-07, `crates/game/src/game/temple.rs` and client wiring.

## What this system does

Jakob's direction (2026-10-06): the final island holds a temple with the Artifact
of the Gods, revealed from the start; the run becomes capture the flag. The
temple's location is the globe marker from the
[planned archipelago](archipelago-and-transport.md#planned-run-archipelago-2026-10-06).

- `BuildingKind::Temple` (5×5, no cost, not in `BUILDABLE`) is placed once, when
  the final planned site is discovered (`islands.rs::discover_site` calls
  `temple.rs::place_temple`). It takes the free plot nearest the island centre
  whose 7×7 ring is open walkable land, before wildlife spawns.
- The temple gives **no sight**: the player does not own it, so it lifts no fog
  (`visible_cells` skips it). It appears in snapshots once any footprint cell is
  explored, like other buildings.
- `GameWorld::artifact_bearer: Option<String>` (persisted, in snapshots) names the
  unit carrying it. `None` means it rests in the temple.
- `Command::ClaimArtifact { unit_ids, building_id }` validates the group atomically
  (temple exists, artifact still inside, every unit can reach beside the temple).
  Units walk there with `UnitAction::ClaimArtifact`; the first to arrive becomes
  the bearer, the rest go idle. Any unit kind can claim.
- `tick_artifact` keeps the artifact with a bearer aboard a ship; if the bearer no
  longer exists (death), the artifact returns to the temple. A bearer within
  three cells of a completed town center on the home island (island index 0) sets
  `scenario.outcome = Won` with objective progress 1/1.

The client sends `ClaimArtifact` when units are selected and the temple is
tapped (`lib.rs`). The temple sheet `assets/sprites/buildings_sanctuary.*` has
the emptied temple in its first three stage slots and the full temple last;
`view.rs` passes "artifact still inside" as the temple's `working` flag and
`view/buildings.rs` picks the frame. `hud/run.rs` draws the globe marker and a
28 px artifact icon above the bearer's health bar.

**Steering (Jakob, 2026-10-07):** after the first temple, Jakob asked for a more
run-down building whose linework matches the existing buildings, and for victory
to appear like the status messages above NPC heads instead of a HUD pill. His
note about the dome was cut off; the dome is now cracked and partly collapsed
verdigris, pending his confirmation. Claim and victory now use unit status
feedback (`feedback.rs`): "Claimed the Artifact of the Gods" and "Victory · the
Artifact of the Gods is home" rise above the bearer and, unlike ordinary
statuses, linger for eight seconds. Do not reintroduce a victory pill.

## Learnings and failure modes

- The existing Slice A placeholder marks a run `Lost` after 36,000 ticks (one
  hour) without enforcing anything. The win check therefore overrides any
  non-`Won` outcome rather than requiring `Running`; revisit when real loss rules
  arrive.
- An exact "touching the town center" win rule was unusable: tapping the town
  center selects it instead of moving units. Three cells is reachable by tapping
  nearby ground.
- `game.rs` reached the 1,000-line limit; `CommandError` moved to `errors.rs`
  and the building constructor to `construction.rs`.

## Verification and open questions

- `cargo test -p aoa-game --locked temple_tests`: single temple on the final
  island only, no fog lifted, hidden until explored, claim/reject atomicity,
  passenger bearer, fallen bearer, home reach and save round trip, and temple
  placement plus on-foot reachability from the sea for 24 seeds.
- `hud::run::tests` covers the marker; `feedback::tests` covers the claim and
  lingering victory status.
- `cargo test -p aoa-game --locked --test full_run` plays seed 7 to victory using
  only player commands (about 24 s in a debug build). Set `AOA_RUN_SNAPSHOTS=<dir>`
  to keep each milestone's snapshot for browser replay. It found no wildlife on
  the route and a five-second walk from landing to the temple: the run is easy
  until return-trip escalation and monsters exist.
- [Browser evidence](../verification/temple-artifact/README.md) uses staged
  snapshots from `archipelago_preview <seed> 1 found|carried|won`, which place a
  villager by the temple as a fixture; a full live run to the temple is not yet
  verified.
- Open: return-trip escalation once the artifact is claimed (Jakob, 2026-10-07:
  difficulty rises and new monsters spawn on the way home), guardians around the
  temple, loss rules, and what happens after victory (the game keeps simulating;
  Reset game starts a new run).

## Keep this document current

Update when the win/loss rules, the bearer's abilities, temple placement or the
temple art change, or when monsters start guarding it.
