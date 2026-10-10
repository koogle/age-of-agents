# Lion prides — review evidence (2026-10-09, art updated 2026-10-10)

Art and behavior for the first monster class. Captures come from the
current web build rendering a temple-island snapshot from `tests/full_run.rs`
(seed 7) with a pride injected beside the landed villager: the leader idle,
one lioness striking (the villager's bar is at 55%), one winding up, one walking.
The injected scene is for rendering only and was not validated by the simulation.

- `style-compare.png`: shipped wolf, bear and boar beside the lioness and lion
  (idle, idle, strike, strike). Not to game scale.
- `before-after.png`: rejected draft 1 (top) against the integrated family pass.
- `poses.png`: all four poses of each, on flat grass.
- `desktop.png`, `desktop-zoom.png`: 1280×800, DPR 1.
- `phone.png`: 390×844, DPR 2.

## Style acceptance (pending Jakob)

Draft 1 was rejected on 2026-10-10: colour and shading were not consistent with
the game. The integrated sheets were generated fresh from the shipped wolf/bear
sheet ([lesson](../../knowledge/asset-pipeline.md#new-species-from-the-family-sheet-2026-10-10)).

Matches now: muted earthy coat (lioness saturation 0.42 and lion 0.45, between the
boar's 0.36 and the bear's 0.51), the wolf's two-tone cream underside, painted
shadow on far legs and belly, dark grey claws, brush-stroke mane strands, thin
warm brown contours, one colour across all four cells of each sheet, a clearly
crouched windup and an open-jaw strike.

Still open: fur texture is softer than the wolf's strokes at close zoom; sources
are 512px cells (other animals 627px); no OpenAI image_gen cleanup pass yet.

## Behavior checks run

- `cargo test -p aoa-game` (199 unit tests plus `tests/full_run.rs`, which
  still wins seed 7), `cargo test -p aoa-client`, `cargo clippy --workspace
  --all-targets -D warnings`, `cargo fmt --check`.
- `one_lion_sighting_alerts_the_whole_pride`: an intruder seen only by the far
  lioness sends the whole pride, which kills it; a wolf beside the pride stays home.
- `prides_do_not_chase_beyond_their_territory`: after the intruder leaves, every
  lion returns home and validation holds every tick.
- `discovery_adds_wildlife_without_replacing_existing_animals`: every later island
  gets one lion and 2–3 lionesses within 3 cells of him, deterministically.
- `python3 scripts/check_sprite_resolution.py`: wildlife 20 frames at 627px;
  the existing wolf, bear and boar pixels are byte-identical after repacking.
