# Lion prides — review evidence (2026-10-09)

Draft-1 art and behavior for the first monster class. Captures come from the
current web build rendering a temple-island snapshot from `tests/full_run.rs`
(seed 7) with a pride injected beside the landed villager: the leader idle,
one lioness striking (the villager's bar is at 55%), one winding up, one walking.
The injected scene is for rendering only and was not validated by the simulation.

- `style-compare.png`: shipped wolf, bear and boar beside the lioness and lion
  (idle and strike), at one common scale.
- `poses.png`: all four poses of each, on flat grass.
- `desktop.png`, `desktop-zoom.png`: 1280×800, DPR 1.
- `phone.png`: 390×844, DPR 2.

## Style acceptance (pending Jakob)

Matches: fine warm brown contours, two-tone cel shading, no fur hatching,
consistent identity across poses, grounded paws, same three-quarter camera.

Differences, still open: the lions are brighter and flatter than the wolf and
bear, with fewer silhouette fur tufts. The windup poses differ only slightly from
idle. Sources are 512px cells (the other animals use 627px), so at maximum zoom
they are a little softer. No OpenAI refinement pass has been made yet. If Jakob
wants a closer match, the next step is an image_gen refinement against the
wolf/bear sheet.

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
