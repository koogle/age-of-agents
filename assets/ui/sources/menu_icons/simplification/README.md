# Menu icon simplification

Requested and authorized for sequential PR merges on 2026-10-06. The four
refinements preserve icon keys, meaning, camera and the approved illustrated
family while removing secondary props and repeated fine detail.

- [Before/after at 128/24/32px on three backgrounds](comparison.png)
- [Style findings and reproducible packing](STYLE_REVIEW.md)
- [Exact prompts, sources and hashes](provenance.json)
- [Combined runtime verification](../../../../../docs/verification/menu-icons/simplification/README.md)

The original drafts are preserved unchanged. `before/` contains the previous
runtime icons, `packed/` the normalized refinements. The rejected three-icon
pass is negative evidence, not a style reference. Approved family references
are the original wood and masonry icons under `../refinement/`.

| Runtime replacement | PR state |
| --- | --- |
| Cargo | Refined runtime PNG in this revision |
| Gathering | Refined runtime PNG in this revision |
| Town | Refined runtime PNG in this revision |
| Disembark | Verification in progress |

The earlier per-icon source folders describe the first dedicated-icon release.
This folder owns this subsequent refinement and its packing provenance.
