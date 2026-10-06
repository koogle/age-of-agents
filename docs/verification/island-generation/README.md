# Terrain-first islands

Historical evidence: feature-specific generation/capture scripts were retired
on 2026-10-06. Old commands below record how these results were obtained;
consult [the retirement record](../RETIRED_TOOLS.md) for current checks and exact historical code.

Generation change, 2026-10-05. See the [generation guide](../../knowledge/archipelago-and-transport.md#terrain-first-generation-2026-10-05).

## Reproduction

```bash
cargo test -p aoa-game --locked worldgen
cargo build -p aoa-game --example island_preview
python3 docs/verification/island-generation/preview.py /tmp/island-previews
./scripts/build_web.sh
python3 docs/verification/island-generation/browser.py /tmp/island-previews
```

The preview exports actual fully explored snapshots for seeds 2 (rounded), 0
(square), 8 (long), 17 (bay) and 18 (lobed). The relief image is a diagnostic map,
not a screenshot. The browser driver checks the exported bay in the WebGL renderer,
then starts a real `?local&seed=17` simulation. It uses desktop 1280×800 and emulated
phone 390×844 at DPR 2 with touch. It does not establish physical-device acceptance
or a complete sailing/construction playthrough.

## Review

The thermonuclear review removed the old independent meander field and per-river
bed flattening instead of adding corrections around them. Shape and drainage
are small domain modules; rendering, commands, saves and the resource catalog
retain their existing contracts. Depressions receive a spill gradient, runoff is
conserved, and confluences share the same elevation. Tests cover those properties,
shape proportions, open bays, deterministic seeds and reachable starter budgets.

The first full-suite run exposed a starting visibility regression: resources
within the old 16-cell "near start" radius could all be outside town-center sight.
Generation now requires visible wood and food and tests that invariant, rather
than weakening the affected presentation tests. Coastline coordinate warping was
added after inspecting the first relief preview's overly straight bay mouth.

This is not physical erosion or a lake simulation. Height quantization in snapshots
can flatten tiny gradients visually; drainage uses the authoritative float heights.
Existing saved terrain is unchanged, and new discoveries use the new generator.

## Local verification results

- Workspace suite: 234 passed; one manual benchmark ignored.
- Formatting and strict native/WASM Clippy passed; the WebGL bundle was rebuilt.
- Desktop and DPR-2 emulated-phone browser checks passed for snapshot rendering,
  mouse/touch input and fresh in-browser generation, without JavaScript errors.
- [Relief comparison](relief.png), [desktop view](desktop-bay.png),
  [phone view](phone-bay.png) and [browser log](browser.log) record the visual checks.
- Modal's read-only status request returned no response before a 30-second timeout
  (after an earlier longer attempt was interrupted). No deployment was attempted;
  production is unverified. No production save was reset.

## Integration for PR #111

Integrated master `079d348` before the user-authorized merge, preserving wildlife,
shore pickup, pause/reconnect, automatic resource continuation and the current UI.
The combined suite passes 267 tests with one manual benchmark ignored. Strict
native/WASM Clippy, formatting, the rebuilt WASM bundle, all six release-verifier
tests and the 298-frame/field/transport/icon audits pass. Combined desktop and DPR-2 phone snapshot and fresh-WASM browser checks pass
without JavaScript errors; screenshots and the browser log are refreshed. The terrain change does not alter master’s store version
14 or its current save model. Release uses the master GitHub Actions workflow;
the earlier local Modal timeout is not evidence of a production failure.
