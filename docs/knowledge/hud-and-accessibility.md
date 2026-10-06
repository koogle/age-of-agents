# HUD, feedback and accessibility

Read before: Before changing commands, labels, icons, responsive layout, or accessible controls.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with the HUD

Start in [hud.rs](../../crates/client/src/hud.rs),
[layout.rs](../../crates/client/src/hud/layout.rs),
[selection.rs](../../crates/client/src/hud/selection.rs) and
[feedback.rs](../../crates/client/src/feedback.rs). Follow offers from the
snapshot to HUD state to hit dispatch to typed command; the domain revalidates.

1. Enumerate states and copy together: action, unavailable reason, queued,
   active, finished, cancelled/refunded. Do not reuse imperative button text
   as an active-task sentence.
2. Use the existing geometry for drawing and hit testing, including label hits.
   Preserve the compact bottom band and upward expansion for menus/queues.
3. Exercise the affected states at narrow portrait, short landscape and desktop,
   with DPI scaling and actual touch. A hover tooltip needs a touch equivalent.
4. Derive transient feedback from transitions; repeated snapshots must not replay
   announcements or accumulate unbounded labels.

```bash
cargo test -p aoa-client --locked hud::layout_tests
```

The [compact HUD review](../COMPACT_HUD_REVIEW.md) contains the viewport matrix.
Accessible DOM controls and additive touch selection remain implementation gaps;
verify [current open work](../../OPEN_WORK.md) before reporting them complete.

## Mobile time-control spacing

User steering on 2026-10-05 requests a tighter mobile time-control row. The compact layout reduces center spacing from 44px to 36px for the existing 30px coins (6px visible gaps), with separate 36×44px hit regions. Keep the rightmost coin anchored and desktop positioning unchanged.

## Learned constraints and evidence

**Evidence:** [#30](https://github.com/koogle/age-of-agents/pull/30),
[#34](https://github.com/koogle/age-of-agents/pull/34),
[#64](https://github.com/koogle/age-of-agents/pull/64) and
[#76](https://github.com/koogle/age-of-agents/pull/76) progressively fixed crowded
mobile controls. [#35](https://github.com/koogle/age-of-agents/pull/35) added
tap explanations; [#77](https://github.com/koogle/age-of-agents/pull/77) separated
completed research from merely unavailable actions;
[#84](https://github.com/koogle/age-of-agents/pull/84) separated button instructions
from status grammar. [#78](https://github.com/koogle/age-of-agents/pull/78) and
[#82](https://github.com/koogle/age-of-agents/pull/82) distinguished timber and field research.

**Lesson:** Specify available, unaffordable, prerequisite-blocked, queued, active
and completed states together. Buttons express actions, statuses describe ongoing
work, and disabled actions explain why on tap. Use distinct art for distinct
meanings. Rendering and hit testing share layout geometry; a pretty default
screen does not prove full queues, submenus and placement fit.

**Check:** Use the [compact HUD matrix](../COMPACT_HUD_REVIEW.md): narrow portrait,
short landscape, DPR1/DPR2, full queues, research completion and actual hit dispatch.
Canvas hit targets are not accessible DOM buttons. The DOM mirror and additive
touch selection remain open in [ROADMAP.md](../../ROADMAP.md#integrated-native-presentation-fixes).

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).

## Ship cargo controls

Jakob’s PR #94 refinements specify a single row of resource columns, filtered to resources on the connected island or aboard. Show resource icons and onboard quantities only, without resource-name or shore-count labels. Place up/load and down/unload shields at each icon’s lower left/right corners; the final artwork is 24px (26px hovered) with 44×40px hit areas. Extra resources paginate on narrow screens. The FAL bronze/laurel/green-enamel art matches research check seals; provenance is in `assets/ui/sources/cargo_shields/classical/`.

Top resources follow the pointer/touch island, including minimap inspection. Building affordability uses the building’s own island. Dock selection exposes Ship cargo when dock artwork obscures its vessel. Validate mouse/touch transfers against authoritative cargo and island inventories, not only screenshots.

Mobile refinement (Jakob, 2026-10-05): in the compact HUD (width below 600 or height below 500 logical pixels), cargo paging uses 44px-wide chevron targets beside the resource columns, keeping the strip 78px tall. Desktop keeps labeled paging. Mouse/touch drags starting in cargo cancel the pressed action after 8 logical pixels; a predominantly horizontal swipe of at least 32px snaps one page on release, clamped to the available pages. This is swipe paging, without continuous or inertial scrolling. Drawing and hit testing share the strip bounds; reset those bounds every layout to avoid stale gesture capture.

Verification (2026-10-05): `hud::cargo::tests` checks every resource page, nonoverlapping transfer/chevron targets, compact row height, DPI-scaled swipe thresholds and cancelled transfers. [Cargo browser replay](../verification/replay_cargo.py) exercises DPR-2 touch swipes/chevrons and DPR-1 desktop clicks/drags; it inspects outgoing commands against a presentation fixture rather than applying transfers to a save. Run with `--output DIR`, then separately with `--output DIR --desktop`. [Mobile row capture](../verification/2026-10-05/mobile-cargo-row.png). The all-13-resource stress fixture still crowds top-resource labels in short landscape; this change only adjusts cargo layout.

## Resource names on demand

Requested by Jakob (2026-10-05): top-bar resources show icons and quantities without persistent name labels. Resource hints use an initial capital (Jakob’s follow-up, 2026-10-05). Hovering an icon or its count shows its name in the existing explanation pill; tapping shows the same name for three seconds. Existing action/error messages take priority. Resource rows are 66px tall instead of 82px. The icons/counts share one hit area so inspection consumes the input instead of issuing a world order. The mobile-layout matrix includes these top-bar regions in its bounds, overlap and hit-dispatch checks, but excludes them from the bottom-action-band assertion.

Desktop hover/leave and DPR-2 phone tap/expiry were checked without game commands or browser errors; see [resource-name verification](../verification/resource-names/README.md).

## Dedicated menu icons (2026-10-05 audit)

Jakob requested a complete missing-icon audit, creation, and sequential PR merges.
Track coverage in [the menu icon audit](../MENU_ICON_AUDIT.md). Inspect both command
and queued-job mappings: a manifest entry alone does not make an icon available
in the Rust HUD. `hud.rs::ICONS` must load it into the shared runtime atlas.
Stop already had authored artwork but was omitted from that list and used Cancel;
restore the hand for land units and ships, retaining the X for cancellation.

## Text transparency

User feedback on 2026-10-05 identified colored rectangular backgrounds behind speed labels. The shared HUD atlas has only 2px packing gutters but a full mip chain; minified glyphs can sample neighboring content. Glyph quads now use shader mode 4 to sample the level-zero alpha mask with the requested text color, leaving sprite mip filtering unchanged. Verify small HUD labels and white selected-speed labels when changing atlas sampling.

## Field placement costs (2026-10-05)

With water added as a required field input, the placement information pill now
shows the full shared cost continuously: 10 wood, 5 stone, 10 water, plus work
time and food yield. Mouse and touch both get this without relying on hover.
See [water verification](../verification/water/README.md).
