# Open work

Only work that is still open: tasks, pending decisions and blockers. Delete an
item when it is done; finished work, history and facts belong in git, `README.md`,
`decisions.md` and the [knowledge guides](docs/knowledge/INDEX.md).

## Waiting on Jakob

- **Giant snakes style call** ([#164](https://github.com/koogle/age-of-agents/pull/164)):
  [comparison sheets](docs/verification/snakes/README.md). Delete the sheets once decided.

## Next steps to finish the capture-the-flag loop

One PR each; each keeps `crates/game/tests/full_run.rs` passing and extends it
once the step changes the run. The run is still too easy: a short walk from the landing to the temple.

1. **Combat foundations the monsters need.** Healing (archers already shoot from six cells).
2. **Make the run winnable again (monster balance).** Jakob merged the monsters on
   2026-10-14 knowing level 3+ islands are currently unwinnable. The
   [live test](docs/knowledge/wildlife.md#live-test-against-player-squads-2026-10-13):
   squads of 10–12 beat level-2 skeletons but lose outright to level-3 centaurs,
   level-4 cyclopes and the minotaur, who also kills the artifact bearer the moment
   he wakes. Options: lower per-level scaling, move the minotaur's lair away from the
   temple or give the bearer a head start, and healing (step 1); re-test with archer squads.
   Re-run the live squad test and record a video after the change. Crowded
   high-level islands can drop some planned monsters for lack of spaced lairs.

## Future work (not needed to finish the loop)

- **Hero unit.** Built only once the right resources are discovered and paid; only
  a hero could claim the artifact. Open: which building trains it and which
  resources unlock it.
- **Return-trip escalation.** Claiming the artifact spawns new monsters on the
  temple island and along the way home.
- **Island types.** Volcanic, forested, ruined or barbarian-held islands with their
  own terrain look, resources and monster mix, chosen per site from the seed.

## Unresolved

- **Branches on the pre-rewrite history.** Open PR #144 needs rebasing onto the
  rewritten `master`; stale remote branches (including closed #72 and #103) should be deleted.
- **Performance.** Populated archipelagos are unprofiled; the
  [map budget](docs/CONTINUOUS_MAP.md) covers only the starter world.
- **Economy refinement** per [the roadmap](ROADMAP.md).
- **Barbarian raid follow-ups:** a raider ship instead of beach spawns, defenses
  that fire back, and playtested balance for raiders, building health and repair.
- **Production verifier.** `scripts/modal_manage.py` expects a land unit and unseen
  terrain ([release guide](docs/knowledge/build-integration-and-release.md#known-verifier-mismatch)).
- **Delete `docs/RUN_PLAN.md`:** an earlier plan Jakob did not want; the lists above
  supersede it.
