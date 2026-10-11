# Barbarian raid — review evidence (2026-10-11)

Art and behavior for the barbarian monster class: a raider and a chieftain, and
the raid that lands them on the second island ([guide](../../knowledge/wildlife.md#barbarian-raid-and-building-damage-2026-10-11)).

- `concept-to-sprite.png`: the picked Thracian concepts and the four packed poses
  (idle, walk, windup, strike) of each, atlas rows 8–9.
- `style-compare.png`: the shipped villager and guard beside the raider and the
  chieftain (idle, strike) at their in-game relative heights.
- `raid-desktop.png`, `raid-desktop-zoom.png`, `raid-phone.png`: `cargo run
  --release -p aoa-game --example archipelago_preview -- 7 2 raid` sails from the
  home island into the fog, discovers island 2, steers beside a free site, lands
  villager-1 with a completed house there (the only staged parts), which arms the
  raid. The countdown is then skipped; the war band the simulation lands walks to
  the house and strikes it (60 damage, bar over the roof) while villager-1 waits
  aboard. `GameWorld::validate` passes. Captured with the
  [archipelago capture script](../archipelago-plan/capture.py) plus a wheel zoom
  over the fight; the phone view is a 390×844 DPR-2 viewport panned with a mouse
  drag, because CDP touch drags did not pan the paused fixture in this harness.

## Style acceptance (pending Jakob)

| Criterion | Result | Observation |
| --- | --- | --- |
| Ink | Pass | Fine warm-brown contours of the guard's weight on every silhouette and fold. |
| Color | Pass | Rust, ochre, olive and bronze; no blue, so raiders never read as the player's units. |
| Light and material | Pass, watch | One soft cel shadow tone; the chieftain's scale corselet and pelt carry a little more texture than the guard. |
| Shape and detail | Pass | Clear silhouettes at gameplay zoom (fox cap, crescent pelta; crest and bear pelt). |
| Camera and scale | Pass | Same three-quarter camera; raider slightly taller than a villager, chieftain about a fifth taller again. |
| Era fit | Pass | Thracian peltast kit around 400 BC; no horned helmets. |
| Integration | Pass | Clean alpha on meadow; feet registered on the shared baseline in all poses. |

Still open: the chieftain resembles a Greek hoplite more than the raider does
(gilded helmet and crest); no OpenAI image_gen cleanup pass.

## Behavior checks run

- `cargo test --workspace --locked` (game: 205 unit tests plus `tests/full_run.rs`,
  which still wins seed 7; client; server, including store version 20 resets),
  `cargo clippy` native and wasm32 with `-D warnings`, `cargo fmt --check`,
  `scripts/check_sprite_resolution.py` (40 wildlife frames pass).
- `raid_tests`: a landing on island 2 arms a seeded 300–600 s raid (not the home
  island, not discovery alone) that lands 10–20 raiders, chieftain first, at least
  20 cells away, deterministically and once; raiders cut down units in reach before
  buildings; march on a town center, damage it, raze it and idle a `Deposit` order
  aimed at it, then hunt the remaining units; friendly units can attack and kill them.
