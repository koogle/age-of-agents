# Granary field yield verification — 2026-10-05

Completed granaries give fields within six cells (square/Chebyshev distance,
nearest footprint cells) +50% food per preparation cycle: 180 instead of 120.
The bonus does not stack, and applies when planting or replenishment finishes.
Already-growing fields keep their current harvest. Storage, farm gathering
speed, explicit orders, costs and carrying limits are preserved.

## Checks

- `cargo test --workspace --locked`: 269 passed, one existing manual benchmark ignored.
- Strict native workspace and WASM Clippy passed without warnings.
- Rebuilt the release WASM and regenerated bindings with wasm-bindgen 0.2.129.
- Desktop mouse and DPR2 emulated phone touch selected the granary successfully,
  with zero page errors. Both descriptions are readable and unclipped.
- Formatting, whitespace and changed Markdown file links pass.

## Domain coverage

`cargo test -p aoa-game --locked granary_yield` passes. Cases cover the inclusive
diagonal boundary, opposite footprint edges, out-of-range and unfinished
buildings, overlapping granaries, capacity/amount, save round-trip and complete
harvest/delivery loops. The second cycle recalculates yield and charges normal
preparation costs. Existing field/gathering tests cover work sharing and orders.

## Browser reproduction

After `./scripts/build_web.sh`, run:

```bash
python docs/verification/menu_icons.py --name granary --output docs/verification/granary
```

This serves an isolated, paused snapshot on loopback port 8011. Actual mouse or
touch selects the granary; the client selection hook asserts a selected building.
Inspect the captured concise description for the food/fiber drop-off role and
+50% nearby field yield. The user requested removing detailed rule qualifiers. The fixture tests presentation,
not authoritative harvest outcomes, which are covered by the domain tests.
Screenshots use Chromium software WebGL: desktop 1280×800 DPR1 and an emulated
390×844 DPR2 phone. Physical phones and native appearance are not verified.

## Code-quality review

Reviewed against `docs/THERMONUCLEAR_REVIEW.md`: the rule lives only at the existing
preparation-to-harvest transition in the Rust domain, uses existing field
capacity, and adds no persisted fields, command variants or automatic orders.
Costs, routing and storage are untouched. The bounded footprint scan runs only
when preparation completes; one boolean prevents stacking. No feature removal,
new abstraction or asset change is needed. All changed frontend files remain
below 1,000 lines.

## Release status

User authorized PR creation and merge; release uses the merge-triggered GitHub
Actions workflow after local verification. Modal's
configured profile is `radiantai`, while the documented production target is
`koogle-frick`; `python -m modal profile list` failed to connect even with Modal
1.6.1 and its proxy support installed. No production mutation was attempted.

## Copy refinement

The user requested shorter text without “no stacking.” The HUD now says
“Food/fiber drop-off · +50% nearby field yield.” The release WASM was rebuilt and
strict native/WASM client lint and formatting passed. This is a text-only change;
the 269-test gameplay result above remains from the preceding implementation.
Desktop and DPR2 phone selection were rerun successfully; refreshed screenshots
and the bundle hash are retained here, with no page errors.

## PR integration

[PR #132](https://github.com/koogle/age-of-agents/pull/132) integrates master
`8bf00b4`, preserving PR #131’s productive-building visibility and the newer menu
artwork. Documentation conflicts were reconciled and WASM regenerated from both
source changes. The combined suite passes 271 tests (one manual benchmark ignored),
formatting and strict native/WASM lint. Desktop and DPR2 phone granary selection
were rerun with no page errors; captures and `result.json` reflect this bundle.
Six release-verifier tests, the 298-frame audit, field/transport checks and icon
normalization also pass. Code-quality review found no further changes needed.
