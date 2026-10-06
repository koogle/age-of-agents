# Overhead health bars — 2026-10-06

Requested behavior: replace numeric HP with small thin overhead bars on villagers
and animals. Combat damage is visible through shrinking health; floating numeric
damage labels are removed. Friendly military units use the same display.

Implementation: 24×3 logical-pixel fill, one-pixel dark border, green above 50%,
orange at 25–50%, red at/below 25%. Bars use interpolated friendly positions and
rendered animal anchors, terrain height and the camera's billboard projection.
Species maximum health comes from the domain. No save or combat-rule changes.

Reproduce after rebuilding WebAssembly:

```sh
python3 docs/verification/replay_health.py --output docs/verification/health-bars
```

The replay supplies controlled snapshots over a loopback WebSocket, observes
actual uploaded HUD geometry at full/half/quarter health and captures desktop
1280×800 and emulated phone 390×844 at DPR 2. It covers a villager, guard and all
three animal species. It checks bar widths/colors and browser errors. This is
presentation evidence, not physical-device or live-combat acceptance.

## Code-quality review

The shared client owns all presentation; simulation, commands, costs and saves
are unchanged. Reusing HUD rectangles adds no dependency or input hit regions.
Bars follow the same interpolated/terrain-adjusted anchors as sprites. Removing
damage text deletes the entity-following feedback branch; activity and cargo
messages preserve their existing behavior. No changed client file exceeds 1,000
lines. No unrelated feature changes or art generation.

Initial desktop review lowered the wolf/boar anchors to match their painted
heights. Health fills use a one-pixel radius: the existing rounded-rectangle
shader leaves a zero-radius interior at half opacity, which muted the fill.

Validation passed: 303 workspace tests (one existing ignored benchmark), refreshed
101 client tests after the final visual adjustment, formatting and strict
native/WASM lint. The rebuilt WebGL replay passes all three health fractions on
desktop and DPR2 phone with zero page errors; PNG dimensions are 1280×800 and
780×1688 respectively. `results.json` records the tested bundle SHA-256 and
uploaded geometry. Final screenshots are the six PNGs alongside this file.
User authorized PR creation and merge after reviewing the preview. Merge and
production release status are tracked by [PR #152](https://github.com/koogle/age-of-agents/pull/152)
and its merge-triggered Actions run; these local checks do not establish deployment.
