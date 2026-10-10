# Giant snakes — review evidence (2026-10-10)

Art and behavior for the second monster class: rock python (near wild
resources), sea serpent (shoreline) and marble viper (temple island), picked by
Jakob from six concepts and adapted to the wolf/bear family sheet.

- `concept-to-sprite.png`: the three concepts and the packed poses (rest, move,
  windup, strike) of each, atlas rows 5–7.
- `style-compare.png`: wolf and bear beside the three snakes at true in-game
  relative scale (idle, then the three strikes).
- Spawn check: `cargo run -p aoa-game --example archipelago_preview -- 7 2 snakes`
  (or the `vipers` stage) sails into the fog and lands beside a snake the
  simulation spawned; it stays hidden until the landing reveals it. Captures were
  shown in the PR, not committed.

## Style acceptance (pending Jakob)

Matches: the family's thin dark contours (measured on the wolf's ink luminance at
atlas scale), flat cel colour areas with one shadow tone, consistent identity and
colour across poses. The adaptation came out on target in one pass; the ink pass
changed little. Still open: the python's wide coils fill its frame, so it reads
slightly larger than a wolf; no OpenAI image_gen cleanup pass.

## Behavior checks run

- `cargo test -p aoa-game` (201 unit tests plus `tests/full_run.rs`, which still
  wins seed 7), `cargo test -p aoa-client`, clippy with `-D warnings`, `cargo fmt
  --check`, `scripts/check_sprite_resolution.py`.
- `ambushers_stay_hidden_until_a_unit_comes_close`: a viper is absent from the
  snapshot and rejects attack orders until a unit is within three cells, then
  appears, takes the attack and bites back.
- `snakes_strike_hard_but_never_leave_their_lair`: a python pursuing a unit stays
  within three cells of home and re-conceals once the unit leaves.
- `discovery_adds_wildlife_without_replacing_existing_animals`: every later island
  gets 1–2 pythons and one sea serpent within four cells of water; only the
  temple island gets two vipers.
