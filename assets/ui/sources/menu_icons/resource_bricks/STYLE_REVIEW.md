# resource_bricks: style review

Approved family references: original wood and masonry UI kit, retained in `../refinement/`; approved revision and exact prompts in provenance.

[Comparison](style-review.png) shows 128px, 24px and 32px views on parchment, dark green and blue. Dusty terracotta blocks use fine brown contours and restrained warm planes, distinct from raw clay.

| Criterion | Finding |
| --- | --- |
| Ink | Pass: fine brown contours; heavy black draft edges removed |
| Color | Pass: restrained warm washes and muted accents match the family |
| Shading | Pass: quiet watercolor planes; no strong glossy gradient |
| Shape/detail | Pass: Dusty terracotta blocks use fine brown contours and restrained warm planes, distinct from raw clay. |
| Camera/scale | Pass: existing three-quarter family view; normalized optical weight and safe circle |
| Alpha/background | Pass: silhouette inspected on three backgrounds, transparent corners, no visible white matte |
| Runtime integration | Pending in [#107](https://github.com/koogle/age-of-agents/pull/107); this PR adds unused art only |

The larger source is retained alongside provenance. Rejected earlier refinement and full sheet are retained as negative evidence, not approved references.

Technical validation: PNG signature, RGBA, 128×128 output, transparent corners and icon normalization. Runtime mapping, desktop/phone use and full combined game validation belong to #107 before these assets appear in gameplay.
