# Open work — 2026-10-08

Compact handoff: current state, what comes next, and what is still unresolved.
Finished work lives in merged PRs, `README.md` and the
[knowledge guides](docs/knowledge/INDEX.md), not here.

## Current state

- Master is deployed through the merge-triggered
  [production workflow](https://github.com/koogle/age-of-agents/actions/workflows/deploy.yml);
  the latest release includes [#159](https://github.com/koogle/age-of-agents/pull/159).
  Store version 19.
- A run is playable end to end: a seeded 5–7 island archipelago under fog with only
  the temple island marked, player-steered ships with one-tap returns, the run-down
  Sanctuary of the Gods, artifact capture and return-home victory.
  `crates/game/tests/full_run.rs` wins seed 7 through player commands alone in about
  455 simulated seconds. Guides: [archipelago](docs/knowledge/archipelago-and-transport.md),
  [temple and artifact](docs/knowledge/temple-and-artifact.md).
- The run is still far too easy: no wildlife on the route and a short walk from the
  landing to the temple.

## Next steps to finish the capture-the-flag loop

Proposed order, one PR each unless noted; nothing below is implemented. Each step
keeps `tests/full_run.rs` passing and extends it once the step changes the run.

1. **Combat foundations the monsters need.** Ranged attacks for archers, healing,
   and building health so raiders can threaten settlements. Test against today's
   wolves, bears and boars first.
2. **Per-island difficulty.** Monster counts and strength scale with each island's
   distance from home; the temple island gets guardians, so landing beside the
   temple is no longer a free win.
3. **Monster classes, one PR each with art previews first:** lions, giant snakes,
   skeletons, barbarians, centaurs, cyclops and minotaur, each with its own
   behavior. Every class can come in several forms (Jakob, 2026-10-08: for
   example a cyclops general or a barbarian chieftain); these are illustrations
   of the idea, not a fixed list of variants.

## Future work (not needed to finish the loop)

Moved here by Jakob on 2026-10-08.

- **Hero unit.** A special unit the player must build, unlocked only once the
  right resources are discovered and paid; only a hero could claim the Artifact
  of the Gods (today any unit can). Open: which building trains it and which
  resources unlock it.
- **Return-trip escalation.** Claiming the artifact raises difficulty: new
  monsters spawn on the temple island and along the way home.
- **Island types.** Distinct kinds of islands (for example volcanic, forested,
  ruined, barbarian-held), each with its own terrain look, resources and monster
  mix, chosen per planned site from the seed.

Not planned yet (Jakob, 2026-10-08): a loss condition. The one-hour placeholder
`Lost` stays a recorded value only and does not block victory.

## Still unresolved

- **Art alternatives ([#145](https://github.com/koogle/age-of-agents/pull/145)).** The
  map, military shield, steel and cloth calibrations are wired in
  ([evidence](docs/verification/alternatives-integration/README.md)). Not wired: the
  wolf (recolor failed), the sandal Disembark (the joined platform is approved) and
  the earlier batches the archive review marks not ready.
- **Performance and economy.** Populated archipelagos are unprofiled; starter-only
  measurements in [the map budget](docs/CONTINUOUS_MAP.md) do not certify busy
  worlds. The economy still needs refinement per [the roadmap](ROADMAP.md).
- **Production verifier.** `scripts/modal_manage.py` expects a land unit and unseen
  terrain; a valid world without them would fail the check. See
  [release guide](docs/knowledge/build-integration-and-release.md#known-verifier-mismatch).
- **Local Modal account.** The development machine's Modal profile is `radiantai`,
  not production. Release only through the GitHub Actions workflow.
- `docs/RUN_PLAN.md` is an earlier written plan Jakob did not want; the lists above
  supersede it.
