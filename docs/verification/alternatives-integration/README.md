# Wiring #145 art alternatives into the game (2026-10-09)

Jakob asked to wire in the [#145](https://github.com/koogle/age-of-agents/pull/145)
alternatives. Following his preference for the latest calibrations, four HUD icons
now use them; the previous icons are kept in `assets/ui/sources/alternatives-integration/previous/`
and compared at 128/32/24 px in `assets/ui/sources/alternatives-integration/comparison.png`.

| Icon | Source in `assets/alternatives/2026-10-06/` | Where it shows |
| --- | --- | --- |
| `command_explore` | `12c-openai-map-refinement` (thin-line map) | Not shown in-game since Explore was removed (#155); replaces the anachronistic compass file. |
| `category_military` | `11d-map-shield-targeted-retries` (blue cel shield) | Build menu Military button and info pill. |
| `resource_steel` | `11a-painted-cel-steel` (steel A) | Resource bar, smelter production and queues. Steel B (`11c`) stays available. |
| `resource_cloth` | `10-cel-calibration` (cloth without satin shine) | Resource bar, weaver production and queues. |

Not wired: the wolf (its recolor failed and the candidate is near-identical), the
sandal Disembark (Jakob later approved the joined-platform Disembark), and the
earlier batches the archive review marks not ready (glossy, busy or identity
failures).

Checks: `scripts/normalize_icons.py --check` passes for all icons; client tests and
the sprite audit pass. Captures from a real seed-7 snapshot with steel and cloth
added to the island inventory: `desktop-1`/`phone-1` show the resource bar with
the new steel and cloth icons, `desktop-2`/`phone-2` the build menu with the new
Military shield. Desktop 1280×800 and phone 390×844 at DPR 2; no page errors.
