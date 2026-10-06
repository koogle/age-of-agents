# Two-view dock rendering — 2026-10-06

Replace four independently authored dock facings with two views: south/front and
north/rear. East reflects south and west reflects north in screen space. Apply the
same reflection to registered ground corners before plot fitting, including all
construction stages and the existing shared placement-preview path.

The dock contribution drops from 16 to 8 frames. The whole building atlas drops
from 2048×3584 to 2048×2560 (28 to 20 frames), and PNG size from 6,263,046 to
4,309,636 bytes (31.2% smaller). Its uncompressed RGBA base level saves 8 MiB.
Eight redundant east/west derived PNGs and their corner records are removed;
original generation strips and prompts remain as historical provenance.

## Visual comparison

[Completed docks: before above, after below](complete.jpg).
[All four construction stages](all-stages.jpg).

Baseline: master `0c7b649`. `compare.py` reads that baseline through Git and checks
that every retained frame and footprint record is unchanged before creating the
comparison sheets. South, north and all non-dock building pixels are identical;
east/west use reflected source pixels. No regeneration, resampling of the runtime
source beyond the existing packer, or recoloring was introduced.

Reproduce from the repository root:

```bash
python scripts/pack_building_sprites.py
python docs/verification/dock-mirroring/compare.py
python scripts/check_sprite_resolution.py
python docs/verification/check_dock_facings.py --output /tmp/dock-mirroring
```

## Style and quality review

- **Ink/color:** retained source pixels and fine contours. During this PR review,
  Jakob flagged noticeably darker/greyer timber in the north/rear source. That
  existing material mismatch is also mirrored into west; this reduction preserves
  it and does not establish cross-view color consistency. Matching timber across
  the two source views remains a separate art correction.
- **Light/material:** unchanged painted shading; reflected views also reflect
  the baked highlights. No new world-lighting calculation is introduced.
- **Shape/detail:** completed dock, pier, boat and construction silhouettes are
  retained from the two source views. East's construction appearance now follows
  south's sparse posts/frame/partial-roof progression; west follows north.
- **Camera/scale:** upright horizontal reflection, never flat rotation. Registered
  ground-corner tests verify reflected world-axis positions, fit, scale and anchor
  at every stage. Existing tests continue to cover all other buildings and fields.
- **Integration:** all four facings and stages inspected in desktop and DPR-2
  phone replay; correct shore direction, plot fit and mouse/touch selection.
  Neutral-grey before/after sheets retain the original cutout appearance.

Applied [THERMONUCLEAR_REVIEW](../../THERMONUCLEAR_REVIEW.md): deletes duplicate
art/registration data, reuses the existing UV mirroring facility, and replaces a
five-element return tuple with the existing frame structure. No dependencies,
interaction modes, simulation rules, building availability, costs, commands or
save changes. Existing domain shore selection remains authoritative; both ghosts
and completed/unfinished docks call the same sprite function.

## Verification status

- Full workspace: 294 tests pass; one existing manual benchmark ignored.
- Formatting and strict native/WASM Clippy pass.
- All 295 remaining sprite frames pass the 512px resolution audit.
- All retained building pixels and footprint records match baseline exactly.
- Release WebGL build passes; tested bundle/atlas hashes are in [checks.json](checks.json).
- Browser replay passes: 40 captures, all eight mouse/touch dock-picking checks,
  no JavaScript errors. Desktop 1200×900 at DPR 1; emulated phone 390×844 at DPR 2.
  Actual screenshot pixel dimensions are asserted. Software WebGL logged expected
  ReadPixels performance warnings, with no rendering/application errors.

Selected runtime evidence (normal zoom):

| Facing | Desktop | Emulated phone |
| --- | --- | --- |
| East (mirrored south) | [image](desktop-east-normal.jpg) | [image](phone-east-normal.jpg) |
| West (mirrored north) | [image](desktop-west-normal.jpg) | [image](phone-west-normal.jpg) |

[All desktop construction stages at maximum zoom](desktop-stages.jpg).
Phone maximum zoom intentionally crops the large dock at the viewport edges;
normal zoom shows the whole dock. Full-resolution PNG captures remain reproducible
with the driver; selected retained JPGs are compressed previews.

Browser checks use
isolated snapshot fixtures, not saved games or production. Native windows and
physical phones are outside this check; the PR is not a deployment.
