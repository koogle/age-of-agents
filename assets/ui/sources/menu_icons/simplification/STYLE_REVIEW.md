# Simpler menu icons — style review

User direction, 2026-10-06: refine selected icons and make them slightly simpler;
subsequently show the results and merge. This is the follow-up to PR #107, with
four independent runtime replacements: Cargo, Gathering, Town and Disembark.

[Before/after and 24/32px review](comparison.png). Original wood and masonry
references were attached to every image refinement; food is also shown as an
unchanged sibling. The approved sources are retained in `../refinement/`.
Exact prompts, inputs and rejected pass are in [provenance](provenance.json).
The first three-icon draft is superseded for excessive weave and heavier ink.

| Criterion | Finding |
| --- | --- |
| Ink | Pass: fine brown contours at normalized size; repeated interior lines reduced. The initially heavy three-icon pass was corrected. |
| Color | Pass: restrained oak/ochre, cream, terracotta, muted apple red and pale teal; coherent with the reference family. |
| Light/material | Pass: quiet watercolor texture and broad soft planes; no glossy gradients. |
| Cargo shape/detail | Pass: one closed shipping crate; removes sack, twisted rope and tiny fasteners. Broad plank divisions preserve wood construction. |
| Gathering shape/detail | Pass: recognizable handled harvest basket, apple and wheat; three broad body bands replace dense weave, with no extra leaves. |
| Town shape/detail | Pass: pediment, two columns, dark entry and one base; removes emblem, curls, greenery and scattered masonry. |
| Disembark shape/detail | Pass: boot, two boards and a small sand/water landing; removes posts, pebbles, grain and foam. |
| Camera/scale | Pass: family three-quarter view, standard optical normalization and safe circle. Subject identities remain distinct. |
| Alpha/background | Pass: normalized PNGs have clean transparent corners; parchment, dark green and blue comparisons show intact silhouettes and no visible colored/white halo at 24/32px. |
| Runtime integration | Pass: six fresh desktop/DPR-2 phone captures show all four replacements in their menus; no page errors and Disembark wire dispatch passes. See the linked runtime evidence in README. |

Packing only crops the three equal source cells and uses the existing optical
normalizer; it does not repaint the generated illustrations. Sources are 1254px
(Cargo) and 724px per cell (the other three). Runtime UI art is the documented
128px exception to the world-sprite minimum. Reproduce with:

```bash
python assets/ui/sources/menu_icons/simplification/pack.py --output /tmp/simple-icons
python scripts/normalize_icons.py --check /tmp/simple-icons/*.png
```

`before/` preserves the previous runtime PNGs. `packed/` holds the proposed
normalized replacements; individual PRs update the corresponding runtime PNG.
The pairwise visual review applies to the isolated art in each PR. Browser
captures with all four candidates are identified as combined preview evidence.
