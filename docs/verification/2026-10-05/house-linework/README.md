# House linework refinement — 2026-10-05

User feedback identified a style mismatch in the new houses. Refined house and
granary roof/complete frames against the retained high-resolution clean-roof
originals: sparse warm contours, broad muted straw masses and soft two-tone
shading replace dense bright individual strands and heavy seams. Granary wheat
emblem and original grain sacks are restored. The material tiers remain intact.

[Equal-size comparison](comparison.jpg): previous thatch / refined thatch /
approved original style. Current catalog previews were refreshed; earlier browser
captures under `material-tiers/` remain historical evidence of the previous art.

Sources: `assets/sprites/material_sources/`, exact `linework-prompts.json`, updated
provenance and `superseded_linework/` originals. Both new strips are 1774×887;
512px packed cells use downsampling only. The completed house source is an
unscaled crop of the strip. All other HD atlas cells are pixel-identical to
`0d0a395`; masonry atlases and manifests are unchanged. Repacking is reproducible.

Validation: 273 Rust tests pass (one manual benchmark ignored), formatting and
strict native/WASM lint pass, and the resolution audit passes all 383 frames.
The linework commit itself changes no runtime source or bundle. The thermonuclear review found no new
abstraction, gameplay, save or input behavior; the change is confined to assets,
provenance, visual verification and the stronger style guidance.

Browser procedure:

```sh
python3 docs/verification/check_house_linework.py --output /tmp/house-linework
```

Uses a controlled loopback WebSocket fixture, current WebGL bundle and actual
new atlases on Chromium desktop 1200×900/DPR1 and emulated phone 390×844/DPR2.
It captures both buildings' roof/complete stages at default gameplay and maximum
zoom. Phone zoom is wheel-controlled for this visual check; it does not establish
physical-device behavior, touch pinch dispatch or upgrade payment acceptance.

All 16 browser captures completed without page errors; both stages were visually
inspected. Maximum zoom intentionally exceeds the narrow phone viewport, while
default gameplay shows the complete silhouette. Final captures use the rebuilt combined source (`e421b50` + master `92aaf7e`);
atlas and WASM hashes are retained in [result.json](result.json).

## Style acceptance review

Reference revision: `0d0a395`, `building_sources/clean_roofs/{house,granary}_{roof,complete}.png`.
Each generation attached those approved family originals; granary also attached
the corrected house for matching the new roof material, not as an architectural
reference. [Target backgrounds](backgrounds.jpg) show original/refined pairs.

| Criterion | Result | Observation |
| --- | --- | --- |
| Ink | Pass | Warm fine contours and sparse roof marks replace black straw clusters; wall/door line weight remains consistent with original siblings. |
| Color | Pass | Muted ochre replaces bright golden straw; cream walls and muted teal shadows remain. |
| Light/material | Pass | Broad soft cel-shaded straw masses replace glossy strand highlights. |
| Shape/detail | Pass | House blue door/window/plant and granary wheat/sacks remain recognizable; roof and complete stages share the same treatment. |
| Camera/scale | Pass | Fixed original isometric view, plot registration and 512px cells; no upscaling. |
| Integration | Pass | Alpha is clean on dark green, parchment and blue; default and maximum-zoom desktop/phone captures inspected. |

Icon-size (24/32px) checks are inapplicable to these world building sprites.
Style acceptance is a visual judgment from these references and comparisons,
not a claim that passing Rust tests establishes an art match.

Final integration: master `92aaf7e` merged without discarding terrain-first
generation, resource-name hints or reviewed UI art. All 276 combined Rust tests,
formatting, strict native/WASM lint, web/server builds, six verifier tests and
icon/field/transport checks pass. The 16 captures were repeated with the rebuilt
combined WASM; no page errors. This local review does not claim deployment.
