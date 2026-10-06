# Verification and durable evidence

Read before: Before designing acceptance checks, recording results, or preparing a handoff.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Reproducing and recording checks

Choose tests for the changed behavior, then apply the required
[quality gates](../../README.md#contributing) and
[review](../THERMONUCLEAR_REVIEW.md). Commands documented here are recipes,
not claims that this documentation edit ran application tests.

For controlled WebGL presentation reproduction, inspect the checked-in driver
and its assumptions, then run from the repository root with Python `aiohttp`,
`playwright` and Chromium installed:

```bash
python docs/verification/replay_presentation.py --output /tmp/aoa-presentation-check
```

The driver uses loopback port 8001, a retained fixture and controlled time. See
[presentation verification](../PRESENTATION_VERIFICATION.md) for its atlas decoder
and interpretation limits. Its assertions are not an end-to-end gameplay proof;
UI commands against an isolated hosted save answer a different question.

`python3 docs/verification/replay_reconnect.py --output /tmp/aoa-reconnect-check`
uses loopback :8002 to close/reopen an actual browser WebSocket and deliver a
40-snapshot burst while rendering is suspended. It checks the rendered villager
anchor on desktop and emulated DPR2 phone, including a paused sequence restart.
Both replay drivers share `presentation-fixture.json`; the inbox Rust tests
parse it so missing required snapshot metadata fails before a browser load timeout.
The reconnect driver keeps real WebSocket/retry timing and holds only animation
frames for the background-tab scenario. It throttles rendering to 10 FPS for
software WebGL; high-frame-rate motion remains covered by the Rust movement
suite. Advancing a synthetic clock through every expensive WebGL frame made this
reconnect test impractically slow, so it does not use the presentation driver's
blanket clock control.

Store reusable fixtures/drivers and selected evidence under `docs/verification/`
when they support a lasting claim. A result should identify revision/bundle,
mode, fixture/seed, viewport/DPI/platform, procedure, observed state and remaining
limits. Label emulated phones, native appearance and production checks separately.

At each milestone and before handoff, update the relevant knowledge guide with
what was learned and `OPEN_WORK.md` with what remains. Follow the
[documentation skill](../../.agents/skills/project-documentation/SKILL.md) for
capturing developer steering and reconciling stale knowledge throughout work.

## Learned constraints and evidence

**Evidence:** [#50](https://github.com/koogle/age-of-agents/pull/50),
[#57](https://github.com/koogle/age-of-agents/pull/57),
[#58](https://github.com/koogle/age-of-agents/pull/58),
[#61](https://github.com/koogle/age-of-agents/pull/61) and
[#70](https://github.com/koogle/age-of-agents/pull/70) repeatedly cleaned or reordered
the handoff. [#60](https://github.com/koogle/age-of-agents/pull/60) supplied missing
native/live/presentation evidence. PR #90's body still said “Not deployed or
merged” when live PR metadata already marked it merged.

**Lesson:** A handoff should say what remains and link evidence, not retell every
release. On cleanup, check each unresolved item before dropping it. Distinguish
implemented, tested, merged, deployed and independently verified; confirm PR
status from metadata and deployment from workflow/production evidence. Record
target platform, revision, bundle and limits. A fixture, phone emulation and a
physical device answer different questions.

**Recovered gaps:** Live open PRs are recorded in the handoff; the four from the original history review are a dated snapshot. The current code
still lacks accessible DOM controls and additive touch selection. Native
macOS/Windows reset appearance and physical-phone safe areas remain unverified
in reviewed evidence. These are not reopened completed fixes. Store useful
replay scripts/screenshots in the repository; temporary paths and old test
counts alone are insufficient future evidence.

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).

## Menu icon verification

Use the maintained batch driver below for menu presentation and pointer/touch
dispatch. The former single-icon driver and dated feature-specific captures are
retired; see [the retirement record](../verification/RETIRED_TOOLS.md). Domain
tests remain the authority for gameplay, and real-server road/wildlife drivers
exercise acceptance against an isolated save.

For sequential icon verification, `menu_icon_batch.py --queue DIR` keeps Chromium
alive and accepts a single `request.json` with `name` and `output`. It writes
`done.json`, screenshots and a bundle hash; the source lists supported menu
fixtures. It uses loopback :8012 and a fresh page for each rebuilt bundle, with
verified desktop/phone metrics and actual mouse/touch events. Reusing the browser
keeps shader initialization costs manageable; this remains presentation/wire
verification rather than a persisted-world gameplay test.

Menu command fixtures use simulation speed 1× after the pause-contract fix (#105); their mocked positions remain controlled. A paused fixture is appropriate for frozen-frame checks, but gameplay orders must respect the authoritative pause contract.

## Phone screenshot dimensions (2026-10-05)

The menu integration review caught a false phone capture: changing dimensions
through CDP on a Playwright context with a fixed desktop viewport satisfied
`innerWidth/innerHeight/devicePixelRatio` checks, but `page.screenshot()` still
returned desktop-sized pixels. Use separate contexts configured with the actual
viewport and `device_scale_factor`, and assert captured pixel dimensions as well
as browser metrics. The pre-integration batch-driver phone captures are
superseded by the corrected #107 suite; do not use metadata alone as visual
proof. Run all desktop scenes then all phone scenes, closing the previous context so only one large software WebGL world stays active.

Unit-training fixtures must provide housing for active and queued unit jobs, even when the visible unit list is empty. A barracks-only fixture correctly disables training and cannot prove enabled-command dispatch; the menu suite supplies a separate house.

Canvas command medallions lift by 3 physical pixels on hover. When selecting them from captured quads, group nearby y-coordinates into a row before sorting left-to-right; a strict y-first sort moves the hovered command to index zero and can click the wrong category.

For unrelated menu fixtures, request `"fresh_context": true` in the batch worker
to close the previous context and initialize the camera/selection from that
fixture. The 2026-10-06 icon refinement replay found center-screen selection
unreliable when reusing a context across dock, land-unit and open-water scenes.
Fresh contexts isolate the scenes; do not accept an empty-action screenshot as
proof merely because it produced no browser errors.

## Build-feedback browser checks (2026-10-06)

`docs/verification/check_build_feedback.py --output DIR` exercises the menu and overhead complaints on desktop and DPR-2 touch; `--mode desktop` or `--mode phone` isolates a viewport. It advances the game's presentation clock in steps below the 250ms client frame clamp and waits for HUD uploads. When throttling animation with timeout IDs, replace cancellation with `clearTimeout` too: winit cancels pending frames during input, and native cancellation cannot cancel a timeout. Pair browser creation with `finally` cleanup. See [the feedback evidence](../verification/build-feedback/README.md).
## Task-selection fixtures (2026-10-06)

The [building-selection replay](../verification/building_selection.py) sends
three startup snapshots, then explicit updates for each controlled task phase.
A continuous feed during SwiftShader startup left the test observing an earlier
phase; task-only fixtures do not need a continuously advancing snapshot stream.
Check projected tap coordinates against the viewport and use a positive command
case to prove the intended target ID: an offscreen phone mill tap initially left
the town center selected while selection counts still looked correct. The fixed
fixture zooms out for setup, uses touch for selection, and verifies a stopped
carrier's deposit names the mill before testing gathering exclusions.

## Subtraction policy (2026-10-06)

Jakob explicitly requested cutting repetitive tests/fixtures and substantially
reducing Python. Retire one-off historical capture/generation scripts whose
results and provenance are already retained; keep current acceptance drivers,
release checks and offline rebuild paths. Preserve useful behavioral regressions;
fixture-generator self-tests and duplicated setup are candidates for removal.
This does not authorize removing gameplay, art sources, provenance or save safety.
Retired tools and their replacement/recovery paths are listed in
[the retirement record](../verification/RETIRED_TOOLS.md).
