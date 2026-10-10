# Lion prides — review evidence (2026-10-09, art updated 2026-10-10)

Art and behavior for the first monster class. Captures:

- **Real spawn** (`real-spawn-*.png`): `cargo run -p aoa-game --example
  archipelago_preview -- 7 2 pride` sails a ship from the home island into the fog,
  discovers island 2, steers to the water nearest the pride the simulation spawned
  there (lion `animal-1-4` and two lionesses), lands villager-1 four cells from the
  leader, and exports the snapshot; the ship's vision shows the pride on the shore.
  Nothing animal is placed by hand; `GameWorld::validate` passes.
- **Posed** shots were removed on 2026-10-10; `poses.png` shows every frame.

- `style-compare.png`: shipped wolf, bear and boar beside the lioness and lion
  (idle, idle, strike, strike). Not to game scale.
- `poses.png`: all four poses of each, on flat grass.

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

- `ink-before-after.png`  and `ink-zoom.png` : the model ink pass
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
