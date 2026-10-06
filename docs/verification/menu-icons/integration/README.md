# Dedicated menu icon integration

The audit and sequential art PRs are recorded in [the menu ledger](../../../MENU_ICON_AUDIT.md).
[Checks](checks.json) and [browser results](browser-results.json) describe the final
combined bundle. Each scene lives in `../<scene>/` with desktop/phone captures,
captured commands and results. Per-icon style reviews and retained source pixels
live in `assets/ui/sources/menu_icons/` at the repository root.

## Review

- The first illustration batch drifted toward black contours, stronger orange
  washes and glossy metal. It was rejected; approved wood/masonry references
  were attached to the corrective passes. Comparison evidence is retained with
  each asset. Unit portraits preserve the approved world sprites.
- Resource and production/queue mappings are exhaustive. Category medallions,
  active-category information and hovered-command thumbnails share the intended
  art. The Stop hand and Cancel X remain distinct. No command, cost, unlock,
  input geometry or save-model behavior changes.
- Thermonuclear review: direct mappings and one shared product helper; no new
  runtime dependencies or framework; all client source files remain below 1,000
  lines. New regression checks exercise atlas loading, distinct resource art,
  production/queue consistency, specialist selection, ship commands and category
  navigation/info thumbnails.
- Final code validation: 276 Rust tests pass, one existing manual benchmark is
  ignored; strict native/WASM lint, formatting, rebuilt JS/WASM, 298-frame audit,
  normalization, exact reproduction of the 18 additions and six release-verifier
  unit tests pass. Logs are retained here.

## Browser scope and corrected fixtures

The actual rebuilt WebGL client receives controlled snapshots on loopback port
8012. The verifier captures mouse/touch commands without forwarding them to a
real game server. It runs desktop scenes first, then closes that context and runs
phone scenes, keeping one large software WebGL renderer active at a time.
Desktop is 1280×800 DPR1; phone is 390×844 DPR2. Both browser metrics and captured
pixel dimensions (1280×800 / 780×1688) are asserted. Unit scenes cover production
and selected-unit portraits; the category scene covers all four category coins.

Earlier CDP-only phone captures returned desktop-sized images and do not count
as phone visual evidence. The corrected contexts replace that evidence. Training
fixtures include an off-screen house: queued and active unit jobs consume housing,
so a barracks-only fixture correctly disabled training. The fixture runs at 1×
because paused worlds reject gameplay commands.

This verifies presentation and dispatch, not authoritative gameplay acceptance,
physical phones, native-window appearance or performance on real hardware.
Merge and production release remain separate facts.

## Reproduce

Build the web client following the repository build guide. On Linux, the driver
uses `/usr/bin/chromium` and Python `aiohttp`, `playwright` and `Pillow`.
Start the loopback worker:

```bash
python docs/verification/menu_icon_batch.py --queue /tmp/menu-icon-qa
```

Submit one request at a time, remove the previous `done.json` first, and wait for
its replacement before submitting another:

```json
{"name":"unit_guard","mode":"phone","output":"/tmp/menu-icon-captures/unit_guard"}
```

Write that object to `/tmp/menu-icon-qa/request.json` via a temporary file and
atomic rename. A result containing `error` fails the check. Modes are `desktop`
and `phone`; omit mode to exercise both. Run all desktop cases before all phone
cases to reuse each renderer. Stop the worker after verification.

Cases: `category_town`, `command_back`, `resource_coal`, `resource_steel`,
`resource_bricks`, `resource_cloth`, `resource_rations`, `unit_guard`, `unit_archer`,
`unit_healer`, `unit_siege_cart`, `transport_queue`, `command_disembark`,
`command_sail`, `command_explore`, `command_cargo`.

## Availability integration

The complete 16-family / 40-capture icon suite ran on the bundle recorded in
`before-availability/checks.json`. Master then advanced with the approved
productive-building availability change (#131). The client change there only
adjusts existing tests; icon-rendering code and accepted art are unchanged.
Preserve that complete suite as baseline evidence. Final combined validation
reruns the workspace suite, lint and web build, then exercises starter Production
filtering, guard production/portrait and sailing on desktop and DPR-2 phone.
The starter fixture retains the ten currently available buildings instead of
forcing the full catalog; the Production menu must contain Lumber mill, Kitchen
and Back. Final combined evidence is recorded separately so bundle hashes cannot
be mistaken for the earlier complete suite.

## Final granary integration (2026-10-06)

Master advanced again with #132's granary field-yield bonus and concise HUD
description. Preserve its behavior and verifier branch, while keeping Stop
dispatch fixtures at 1×. Rebuilt the combined source, passed all 276 Rust tests
and strict lint, then captured granary selection/description on desktop and
DPR-2 phone; both pixel dimensions are asserted. `latest-combined/` and the
final bundle hash in `checks.json` identify this last verification. The earlier
40-capture suite and eight-capture availability replay retain their own hashes
and evidence; the icon art and mapping code did not change between these builds.
