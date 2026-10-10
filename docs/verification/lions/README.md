# Lion prides — review evidence (2026-10-09, art updated 2026-10-10)

Art and behavior for the first monster class. Captures come from the
current web build rendering a temple-island snapshot from `tests/full_run.rs`
(seed 7) with a pride injected beside the landed villager: the leader idle,
one lioness striking (the villager's bar is at 55%), one winding up, one walking.
The injected scene is for rendering only and was not validated by the simulation.

- `style-compare.png`: shipped wolf, bear and boar beside the lioness and lion
  (idle, idle, strike, strike). Not to game scale.
- `before-after.png`: rejected draft 1 (top) against the rejected grey family pass.
- `poses.png`: all four poses of each, on flat grass.
- `desktop.png`, `desktop-zoom.png`: 1280×800, DPR 1.
- `phone.png`: 390×844, DPR 2.

## Style acceptance (pending Jakob)

Rejected along the way (2026-10-10): draft 1 (generic, Disney-like), the family
pass (grey), and a warm painterly redraw (line work and colours off). Jakob then
picked the suggested concepts: the Nemean lion as leader and the island huntress
as lioness, then asked for thinner line work and a more real lion, then for more
cel shading; both are now naturalistic, cel-shaded at a medium detail level, with
packer-thinned lines ([lesson](../../knowledge/asset-pipeline.md#new-species-from-the-family-sheet-2026-10-10)).

- `concept-to-sprite.png`: both concepts and the four packed poses of each.
- `style-compare.png` now shows wolf, bear, boar, lioness and lion idle, then both
  strikes, at their true in-game relative scale.

- `thin-before-after.png`: wolf for reference; Nemean/huntress adaptation (top)
  against the naturalistic, thinned rows (bottom).
- `cel-before-after.png`: wolf for reference; naturalistic rows (top) against the
  first cel-shaded rows (bottom).
- `cel-fix-before-after.png`: that first cel lioness (top) had drifted beige with
  faded outlines (value 0.87, 3.6% dark pixels); the regenerated one (bottom) is
  anchored to the lion (S0.57, 11% dark pixels in the packed frame).
- `reink-before-after.png` and `reink-zoom.png`: a packer re-ink step that matched
  the wolf's numbers and was removed the same day: Jakob ruled that line work
  goes through an image model. Kept as the measured target.
- `ink-before-after.png` and `ink-zoom.png`: the model ink pass that replaced it
  (`nano-banana-pro/edit`, wolf sheet attached). Packed frames: ink luminance
  0.16–0.18 against the wolf's 0.18–0.21, line coverage 2.7–3.9% against 2.5–3.0%,
  silhouette edge 0.98–1.25 against 0.80–0.87, with no packer processing.

Matches: flat cel colour areas with one soft-edged shadow tone, the wolf's ink colour, stroke width and continuous hard-edged contour, consistent identity and colour across
poses, crouched windups, open-jaw strikes. Still open: both lions are slightly more
realistic than the wolf; no OpenAI image_gen cleanup pass yet.

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
