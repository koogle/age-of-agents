# Rendering, camera and input

Read before: Before changing projection, terrain, map bounds, picking, selection, zoom, or depth.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with this system

Read [camera.rs](../../crates/client/src/camera.rs) for projection/inverse picking,
[gestures.rs](../../crates/client/src/gestures.rs) for mouse/touch dispatch,
[view.rs](../../crates/client/src/view.rs) for sprites, and
[minimap.rs](../../crates/client/src/hud/minimap.rs) for its shared forward/inverse map.
Drawing lives in [render.rs](../../crates/client/src/render.rs) and
[shaders](../../crates/client/src/shaders).

1. Identify coordinate units: simulation cells, world units, logical screen
   pixels, device pixels and atlas pixels. Trace conversions rather than copying
   a constant from another layer.
2. List consumers affected by the change before editing. Preserve fixed camera
   heading, terrain-aware picking and pointer/pinch anchoring.
3. Pair mathematical projection/round-trip tests with visible interaction checks;
   a correctly drawn object can still have the wrong hit region.
4. Rebuild WebGL after source changes and inspect near/far views and actual mouse
   and touch events. Native and browser share the renderer.

```bash
cargo test -p aoa-client --locked camera::tests
cargo test -p aoa-client --locked gestures::tests
cargo test -p aoa-client --locked minimap
```

PR #87 is now merged (integration checked at `0a863a4`): Shift-drag replaces,
Control-click/drag adds on Windows/Linux and Command does so on Mac. Modifier
state is captured at press time; touch keeps its existing tap/pan path.

PR #91 also changed movement presentation: local interpolation follows the
simulation accumulator; remote playback buffers two ticks and catches up smoothly at at most 1.1× for
short gaps. Merged PR #93 adds explicit resynchronization when lag exceeds eight
ticks, resetting history/velocity without advancing gait before rebuilding the buffer. Full authored gait
cycles replace the earlier two-pose gait, and work animation is gated by arrival
at the authoritative interaction cell. Read `view/movement_tests.rs` and
`view/activity_tests.rs` before modifying timing or work poses. See [movement verification](../MOVEMENT_VERIFICATION.md) for the merged seeded
frame-scenario suite and the scope of its historical verification.

Reconnect explicitly resets playback even for short gaps or paused worlds,
and an excessive remote inbox backlog collapses to the newest snapshot before
presentation. These corrections do not reframe the camera or clear selection
for the same world; see [recovery contract and replay](runtime-debugging.md#reconnect-recovery).

## Learned constraints and evidence

**Evidence:** [#26](https://github.com/koogle/age-of-agents/pull/26) corrected flat
ground picking after terrain gained height;
[#40](https://github.com/koogle/age-of-agents/pull/40) corrected taps on tall
sprites. [#52](https://github.com/koogle/age-of-agents/pull/52) anchored zoom but
[#75](https://github.com/koogle/age-of-agents/pull/75) still had to correct pan
speed across zoom/directions. [#89](https://github.com/koogle/age-of-agents/pull/89)
found stale 30×20 bounds and unrotated minimap axes;
[#90](https://github.com/koogle/age-of-agents/pull/90) made all bounds dynamic.

**Lesson:** A change to camera, terrain scale or map size needs a consumer audit:
projection/inverse picking, sprite hit regions, selection, placement, minimap
sampling/marker/clicks, fog textures, camera bounds and generated verification
fixtures. Use runtime dimensions and shared transforms; avoid copies of old map
constants. Measure screen movement in logical pixels or viewport fractions with
frame time and DPI accounted for.

**Check:** Tests in `camera.rs`, `gestures.rs`, `hud/minimap.rs` and
`hud/layout_tests.rs`; inspect both world axes, hills, near/far/globe zoom, map
expansion, mouse and DPR-2 touch. For depth changes include front/behind walls,
raised terrain and partially visible roofs: see
[#41](https://github.com/koogle/age-of-agents/pull/41),
[#48](https://github.com/koogle/age-of-agents/pull/48) and
[#59](https://github.com/koogle/age-of-agents/pull/59).

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).

## Rust subtraction pass (2026-10-06)

Jakob clarified that the refactor should reduce source lines. Repeated
GPU bind-group descriptor construction is consolidated with one local helper for the renderer's
contiguous bindings. Resource order, texture views, layout, filtering and shader
behavior are preserved. This is GPU setup glue, not a renderer/engine framework.
Desktop/phone verification is tracked in [refactor evidence](../verification/rust-refactor/README.md).
