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
