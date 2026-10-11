# Open work

Only work that is still open: tasks, pending decisions and blockers. Delete an
item when it is done; finished work, history and facts belong in git, `README.md`,
`decisions.md` and the [knowledge guides](docs/knowledge/INDEX.md).

## Waiting on Jakob

- **Giant snakes style call** ([#164](https://github.com/koogle/age-of-agents/pull/164)):
  [comparison sheets](docs/verification/snakes/README.md). Delete the sheets once decided.
- **Barbarian raid review and style call** ([#165](https://github.com/koogle/age-of-agents/pull/165)):
  raider, chieftain and torchbearer art (agent's pick of eight concepts plus Jakob's
  torchbearer), building health and free repair, and the island-2 raid. Store version 20.
  Not in it: a raider ship, defenses that fire back, playtested balance.

## Next steps to finish the capture-the-flag loop

One PR each; each keeps `crates/game/tests/full_run.rs` passing and extends it
once the step changes the run. The run is still far too easy: a short walk from
the landing to the temple, and wildlife strength does not scale with distance.

1. **Combat foundations the monsters need.** Ranged attacks for archers (stale PR
   [#142](https://github.com/koogle/age-of-agents/pull/142)) and healing. Building
   health and repair are in #165.
2. **Per-island difficulty.** Monster counts and strength scale with each island's
   distance from home; the temple island gets guardians.
3. **Monster classes, art previews first:** skeletons, barbarians (#165), centaurs,
   cyclops and minotaur, each with its own behavior and possibly several forms
   (Jakob, 2026-10-08: for example a cyclops general or a barbarian chieftain).

## Future work (not needed to finish the loop)

- **Hero unit.** Built only once the right resources are discovered and paid; only
  a hero could claim the artifact. Open: which building trains it and which
  resources unlock it.
- **Return-trip escalation.** Claiming the artifact spawns new monsters on the
  temple island and along the way home.
- **Island types.** Volcanic, forested, ruined or barbarian-held islands with their
  own terrain look, resources and monster mix, chosen per site from the seed.

## Unresolved

- **Branches on the pre-rewrite history.** Open PRs #72, #103, #142 and #144 need
  rebasing onto the rewritten `master`; stale remote branches should be deleted.
- **Performance.** Populated archipelagos are unprofiled; the
  [map budget](docs/CONTINUOUS_MAP.md) covers only the starter world.
- **Economy refinement** per [the roadmap](ROADMAP.md).
- **Production verifier.** `scripts/modal_manage.py` expects a land unit and unseen
  terrain ([release guide](docs/knowledge/build-integration-and-release.md#known-verifier-mismatch)).
- **Delete `docs/RUN_PLAN.md`:** an earlier plan Jakob did not want; the lists above
  supersede it.
