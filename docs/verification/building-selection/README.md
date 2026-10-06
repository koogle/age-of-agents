# Building selection preserves gathering

User request, 2026-10-06: selecting or switching buildings while a gathering
villager is selected must not force an early unload or replace its gathering loop.

The client building-click carrier filter excludes all `UnitAction::Gather`
phases. The same filter controls the unload hover ring. Building selection still
opens the menu; other carriers still receive a compatible manual `Deposit`.
Automatic gathering deliveries remain authoritative simulation behavior.
No protocol or save-model change.

## Reproduce

After rebuilding the browser client:

```bash
python3 docs/verification/building_selection.py --output docs/verification/building-selection
```

The driver serves a controlled snapshot on loopback :8017 and exercises real
Chromium mouse clicks and emulated DPR2 phone touch taps. For each gathering
phase it selects a villager carrying seven wood, selects the town center and
switches to a lumber mill, checking selection counts and outgoing commands.
It also checks that an idle carrier still issues exactly one town-center deposit.
Phone camera setup uses two wheel steps to fit both buildings; the tested
selection events use real touch taps. Every target must lie within the viewport.
An idle-carrier deposit to the mill additionally proves its target was picked
rather than leaving the town center selected after an offscreen tap.
The fixture records command dispatch rather than executing a persisted-world
simulation. It never contacts production. The driver sends each phase snapshot explicitly;
a continuous feed during software-WebGL startup left the client observing an
earlier phase and produced contradictory stopped/gathering checks.

## Code-quality review

One shared candidate filter owns both building-click orders and hover previews;
the candidates are a plain data function so mixed-group selection can be tested
without a renderer. Removed the duplicate building-selection branch. No new
input mode, dependency, autonomous behavior or domain/save changes. Unit tests
cover all four gathering phases in a mixed selection, compatible/incompatible
storage, missing cargo and empty selection/snapshot. All 296 combined workspace tests pass (one existing manual benchmark ignored), along
with formatting, strict native and WASM lint, rebuilt WebGL output and whitespace
checks. Desktop mouse and DPR2 phone touch pass all four gathering phases plus
stopped-carrier controls with no page errors; results and bundle hash are in
[results.json](results.json). The corrected phone replay zooms out to verify both
storage targets inside the viewport. Native-window appearance and physical
phones are unverified. Integrated master `de49ab4`, preserving boars and stationary
status feedback; 307-frame/field/transport/icon audits and six release-verifier
tests pass. User authorized creation and merge of [PR #149](https://github.com/koogle/age-of-agents/pull/149)
on 2026-10-06; merge and deployment status are tracked by that PR and the
[production workflow](https://github.com/koogle/age-of-agents/actions/workflows/deploy.yml).
