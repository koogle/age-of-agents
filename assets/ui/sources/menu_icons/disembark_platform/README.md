# Disembark platform correction

Jakob identified the two-board simplification in PR #140 as a regression on
2026-10-06. This replaces that version with a joined wooden landing platform:
several narrow deck boards, a continuous front support and corner timber ends.
The simpler boot, sparse texture and restrained palette remain.

[Before/after at 24/32/48px](comparison.png) (removed 2026-10-10: iteration evidence; final state kept). The retained earlier platform
(`../command_disembark/refined.png`) provided construction cues; original wood
and masonry icons (`../refinement/`) remained the attached style references.
Exact prompt and tool output identifier: [provenance](provenance.json).

| Criterion | Finding |
| --- | --- |
| Ink | Pass: fine brown contours; structural board divisions replace dense decorative marks. |
| Color | Pass: muted oak, ochre boot, pale sand and teal water match the established family. |
| Light/material | Pass: soft watercolor planes without glossy highlights. |
| Shape/detail | Pass: deck boards visibly join across a rectangular landing with a supporting beam and posts; no longer two loose blocks. Boot and landing remain distinct at menu size. |
| Camera/scale | Pass: existing three-quarter viewpoint; standard optical weight and safe-circle normalization. |
| Alpha | Pass: clean corners and intact silhouette on parchment, dark green and blue at 24/32/48px. |
| Runtime | Pass: fresh desktop and DPR-2 phone Land passengers captures and typed Disembark dispatch; no page errors. |

Reproduce the packed 128px RGBA PNG from the retained source:

```bash
python assets/ui/sources/menu_icons/disembark_platform/pack.py
python scripts/normalize_icons.py --check assets/ui/sources/menu_icons/disembark_platform/command_disembark.png
```

[Runtime evidence](../../../../../docs/verification/menu-icons/disembark-platform/README.md).

`before.png` preserves the rejected two-board runtime icon. `refined.png` is the
unchanged image-tool output. Packing only applies the existing optical normalizer.
This correction supersedes the earlier simplification review's Disembark
shape/detail finding; it does not change the other three refined icons.
