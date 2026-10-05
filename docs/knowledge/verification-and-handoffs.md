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

`python docs/verification/menu_icons.py --name stop --output /tmp/menu-stop`
serves a controlled current-schema snapshot on loopback :8011, loads the current
WASM and assets, captures desktop/DPR-2 phone views and checks Stop wire dispatch.
It never touches hosted saves. This verifies presentation and pointer/touch
dispatch, not domain acceptance. Use native domain tests for authoritative behavior.
In this cloud executor, source `/workspace/.cloud-setup/activate.sh` when Rust is
not on PATH; check/install the matching WASM target and wasm-bindgen version.
Software WebGL can take roughly a minute for initial shaders; a real-time snapshot
feed avoids clock instrumentation blocking first-frame initialization.

For sequential icon verification, `menu_icon_batch.py --queue DIR` keeps Chromium
alive and accepts a single `request.json` with `name` and `output`. It writes
`done.json`, screenshots and a bundle hash; the source lists supported menu
fixtures. It uses loopback :8012 and a fresh page for each rebuilt bundle, with
verified desktop/phone metrics and actual mouse/touch events. Reusing the browser
keeps shader initialization costs manageable; this remains presentation/wire
verification rather than a persisted-world gameplay test.
