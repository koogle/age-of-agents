# Asset generation and integration

Read before: Before generating, refining, repacking, replacing, or diagnosing game art.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with FAL and image refinement

Read [the asset policy](../../AGENTS.md#asset-workflow) and the relevant
[sprite](../../assets/sprites/README.md), [UI](../../assets/ui/README.md),
[terrain](../../assets/terrain/README.md) or [loading](../../assets/loading/README.md)
provenance. Start from approved art and retained originals.

1. Identify the asset family, manifest, packer, render size and target states.
   Check whether existing high-resolution sources solve the problem first.
2. For FAL generation, inspect the appropriate script under `assets/*/tools/`
   and its reference files before running it. Scripts can make multiple paid
   calls and assume a particular working directory; do not launch the whole
   generator to repair one frame. The existing
   [falcall.py](../../assets/ui/tools/falcall.py) uses `FAL_KEY`, submits to the
   queue, polls and records request/cost provenance. Check current credentials
   and network readiness without printing secrets; old environment notes are
   not current access checks.
3. Use the documented nano-banana generation/edit and BiRefNet cutout path;
   refine diffusion defects with the available image tool under its current
   instructions. Preserve identity, palette, geometry, anchor and transparent
   output. Retain the actual model, prompts, source references and request IDs.
4. Repack with the existing script for the affected family. For example,
   `python3 scripts/pack_hd_sprites.py` repacks retained HD sources offline and
   needs Pillow, NumPy and SciPy; it rewrites multiple atlases, so inspect its
   scope first. `scripts/normalize_icons.py --check` only checks;
   `--write` changes the icon set.
5. Run relevant checks, integrate manifest/renderer changes, rebuild the client
   when needed, and show actual previews at gameplay and maximum zoom.

```bash
python3 scripts/check_sprite_resolution.py
python3 scripts/check_field_preparation_assets.py
python3 scripts/check_transport_assets.py
python3 scripts/normalize_icons.py --check
```

Run only checks relevant to the asset family plus required integration gates.
Midjourney is an alternative with separate tooling/account requirements in
[its setup guide](../MIDJOURNEY.md); that document's environment observations are
historical. Do not assume a local login or provider access exists in a new session.

## Style acceptance is a merge gate

**Developer correction, 2026-10-06:** Jakob identified the two-board Disembark
refinement in #140 as a regression. Restore a recognizable joined wooden landing
platform with several deck boards; simplification must preserve structural cues,
not merely the boot and shoreline. Retain the restrained palette, simpler boot
and sparse texture. The [platform correction](../../assets/ui/sources/menu_icons/disembark_platform/README.md)
passes visual, alpha and fresh desktop/DPR-2 phone dispatch checks; the prior
Disembark shape/detail pass is superseded by this user feedback.

**Developer steering, 2026-10-06:** After comparing the rejected and merged
menu icons, Jakob requested further refinement and slightly simpler icons.
Keep the established thin brown ink and restrained watercolor family. Reduce
secondary props, repeated texture strokes and tiny decoration where they compete
with the primary silhouette at 24/32px; preserve each command's meaning. Start
with a representative Cargo refinement, then review Gathering, Town and
Disembark. The four replacements now pass the style review, optical/alpha checks
and six desktop/DPR-2 phone menu captures; their independent runtime replacements
are tracked in the [refinement ledger](../../assets/ui/sources/menu_icons/simplification/README.md).
Jakob subsequently authorized showing the four refinements and merging them.
The retained [comparison](../../assets/ui/sources/menu_icons/simplification/comparison.png)
shows the normalized candidates against the previous art and approved references
at 128/24/32px on parchment, dark green and blue. Simplification removes props
and repeated interior marks rather than increasing contour weight.

**Developer steering, 2026-10-05:** Jakob flagged visual style drift during the
menu-icon audit and asked for clearer enforcement. Technical image checks did
not catch heavier contours, brighter washes and stronger shading in the draft
refinement. Those initial illustrations were rejected; the corrected set was subsequently
reviewed and merged individually in #112–#129, with runtime integration in #107.
The [comparison](../verification/art-style/rejected-menu-drafts.png) records the
approved kit on the left and rejected drafts on the right; rejected work is
negative evidence, never a new style reference.

The existing art direction remains authoritative: use
[`diorama_primary.webp`](../../assets/reference/diorama_primary.webp) for world
art, with older Mediterranean references supporting palette only. UI object
icons use the established
[wood](../../assets/ui/icons/resource_wood.png),
[food](../../assets/ui/icons/resource_food.png) and
[research](../../assets/ui/icons/tech_masonry.png) kit as their family reference.
Use the existing seal/coin family for medallions and approved unit originals
for portraits. This makes the existing rules explicit; it does not authorize
restyling the shipped game or promoting the newest generated asset to a standard.

Before generating or refining, record the exact approved reference files and
revision in provenance. Attach those approved references to **every** refinement
pass alongside the draft being edited. A sheet of new drafts alone is not an
adequate style reference. Refine one representative asset first and inspect it
against the approved kit before expanding a batch. Preserve rejected passes and
record why they failed; model names and a matching prompt are not visual evidence.

An art PR must include a completed style review with these acceptance criteria:

| Criterion | Accept | Reject |
| --- | --- | --- |
| Ink | Fine dark-brown contours comparable to the approved sibling | Heavy black outlines, bold sticker edges |
| Color | Restrained ochre, olive, terracotta, limestone and muted teal; subject-specific accents | Stronger saturation across the whole icon or a new palette |
| Light and material | Soft, restrained watercolor/cel shading consistent with the family | Glossy bevels, dramatic gradients, hard high-contrast bands |
| Shape and detail | Clear silhouette, sparse hatching, coherent geometry at display size | Dense decoration, tiny scattered details, distorted anatomy or perspective |
| Camera and scale | Existing family viewpoint, optical weight and safe bounds | A new camera, oversized art, inconsistent anchors or proportions |
| Integration | Clean alpha on dark green, parchment and blue; legible alongside siblings | White fringe, accidental shadow, frame or background, ambiguous meaning |

Include reference/result comparisons at source size and **24/32px for icons**,
plus the actual coin or resource-pill context on desktop and DPR-2 phone when
wired into the game. World assets need gameplay and maximum-zoom comparisons.
Show these previews in chat as well as linking retained evidence from the PR.
Record a pass/fail and a concrete observation for every relevant criterion;
fix failures before merging. Mark genuinely inapplicable checks with a reason.
An art-only PR may defer runtime evidence to its named integration PR, which
must supply that evidence before the new art is used in the game.

PNG, alpha, normalization and resolution checks remain necessary but **cannot
approve style**. Do not invent a numeric image similarity threshold or claim CI
has judged appearance. The PR author must inspect the comparison; user approval
is required only when otherwise mandated by the workflow or requested by the
user. A failed style review blocks merging even when automated checks pass.

## Learned constraints and evidence

**Evidence:** [#9](https://github.com/koogle/age-of-agents/pull/9),
[#12](https://github.com/koogle/age-of-agents/pull/12) and
[#15](https://github.com/koogle/age-of-agents/pull/15) repeatedly corrected stride,
variant and depletion readability. [#23](https://github.com/koogle/age-of-agents/pull/23)
improved idle alone; [#62](https://github.com/koogle/age-of-agents/pull/62) later
recovered 183 undersized action/resource frames. Building scale needed successive
corrections in [#36](https://github.com/koogle/age-of-agents/pull/36) and
[#37](https://github.com/koogle/age-of-agents/pull/37).

**Lesson:** Agree on identity, scale relative to villagers, ground anchor,
construction/depletion stages and supported facings before packing. Fixing one
pose is not a claim that its variants or the full animation set are fixed. Prefer
recovering retained high-resolution sources over regenerating or enlarging a
small atlas. Manifests own atlas dimensions; an art-only update can still require
client integration. Preserve originals, rejected refinements and exact provenance.

**Check:** Use [the asset workflow](../../AGENTS.md#asset-workflow), strict frame
checks, contact sheets, real gameplay/max zoom and target-background alpha checks.
Compare siblings and unaffected pixels. Show images during progress as required
by [#66](https://github.com/koogle/age-of-agents/pull/66); a prose assurance of
quality gives the user no opportunity to catch the next mismatch.

## Requested building material progression (2026-10-05)

Implemented locally from Jakob’s request: preserve the existing building
identities, replace base-building clay/tile roofs with wood or thatch, and add
clay/brick upgraded versions. Preserve approved sources for upgraded art where
suitable. Keep structural kiln/furnace stone distinct from clay roof tiles; a
starter kiln must remain buildable before bricks can be produced. Thatch is an
art/material treatment here, not a request to add another inventory resource or
to require island-4 fiber for starter buildings. Cover construction stages,
completed sprites, working variants and menu portraits consistently.

Sources, provenance and packing instructions: [material sources](../../assets/sprites/material_sources/README.md). Both tiers, construction stages and selection/menu portraits are integrated. The base monument retains its stone finish; its upgraded pedestal uses brick.

### Material-tier generation findings

The 2026-10-05 material pass found that whole 4×4/4×5 atlas edits returned roughly
1254px square/1121×1403 images: only about 280–313 authored pixels per cell.
Those drafts were rejected, not enlarged to pass the 512px gate. Two-frame strips
returned roughly 1774×887 with enough per-frame detail. Retain originals and pack
down with `scripts/pack_material_sprites.py`; `_masonry.png` sheets share original
manifest coordinates. Inspect actual dimensions rather than trusting the prompt.

Giving a whole catalog atlas while asking for a numbered row can change building
identity: farm became a mine, range became barracks, and dock lost its sailboat.
Use an isolated crop of the exact stage pair (or retained high-resolution source)
for corrections. Inspect silhouette/props as well as roofs before integration.
Near-transparent pixels can contain saturated RGB at alpha 1/2; previews may
exaggerate them. Packing discards alpha ≤8 before downsampling, while preserving
opaque art. No procedural roof recoloring or texture generation is used.

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).

## Directional dock integration (2026-10-05)

The building packer appends three directional rows to the existing atlas, keeping
all original UV rectangles (the later material pass replaces roof pixels). Use the retained 2172×724 strips in
`assets/sprites/building_sources/directions`; the earlier 1254×1254 construction
sheet was rejected because its cells were only 418px. Each integrated source
cell is 724×724 before packing down to 512×512. Manual structural deck corners
register each construction stage, and the atlas manifest drives texture dimensions.
The new geometry remains upright in the fixed camera. Domain facing and preview
behavior are documented in [placement](placement-and-routes.md#dock-orientation-2026-10-05).

The material-tier integration also covers all four dock facings: masonry preserves the upstream directional atlas, while three isolated-reference generated strips replace only the starter roofs. Both HD atlases share 2048×3584 dimensions and unchanged directional deck corners. Run the material packer after the original building packer. The directional browser fixture now includes the required material flag and captures both tiers.

## House linework correction (2026-10-05)

User feedback on the material-tier preview: the new houses' linework differs;
enforce the approved style more strongly. Implemented for house and granary roof/complete stages: compare each new
house stage directly with `building_sources/clean_roofs/house_*.png` at equal
rendered size. Keep fine warm ink, sparse material marks and soft two-tone shading;
reject dense straw hatching, heavy black roof seams and glossy strand highlights.
Changing roof material must not introduce another illustration style. Validate
construction and completion together, then inspect desktop and DPR2-phone views.

The accepted correction uses broad muted thatch masses, retaining sparse warm ink
and the original architectural details. Original high-resolution clean-roof art
is the style master; the corrected house additionally guides the granary thatch.
Both authored strips have 887px cells, packed down to 512px. Earlier material
sources and provenance remain in `material_sources/superseded_linework/`.

House correction evidence and criterion-by-criterion visual acceptance are in
[the linework review](../verification/2026-10-05/house-linework/README.md).
The follow-up integrates master `92aaf7e`, retaining terrain-first generation,
resource-name hints and individually reviewed menu icons before rebuilding WASM.

## Water icon correction (2026-10-06)

User rejected the initial water icon as inconsistent with the established style.
The earlier water review overstated the match: the jug has heavy contours,
saturated orange shading and broad highlights beside the finer, restrained
resource kit. That style pass is superseded. Use approved wood/food/masonry
references directly for the correction; the rejected jug is subject reference
only. Retain the rejected source and compare the replacement at 24/32px before
integration. Technical validity does not establish visual consistency.

## Road surface refinement (2026-10-06)

Jakob requested closer style matching for both roads after PR #133. The initial
clay-biome reuse reads too orange and building-plot paving reads too regular.
Dedicated painted road swatches were refined against `diorama_primary.webp`,
`terrain/meadow.webp` and the approved `terrain/cobblestone.png`: muted worn earth,
irregular warm limestone, restrained shading and fine brown joints. Preserve
building plots and biome textures. Acceptance requires normal/max-zoom desktop
and DPR-2 phone evidence; both pass in the [material review](../verification/roads/style/README.md).
Road materials occupy layers 11/12 of the existing texture array and use its
mirrored, mip-clamped sampling; retain full-resolution originals and record exact
prompts/reference hashes before integration.


## Wildlife style correction (2026-10-05)

Jakob found the shipped wolves and bears too generic and not fully aligned with the game. Refine against `assets/reference/diorama_primary.webp`: simplify dense fur into painted masses, soften dark contours and contrast, and preserve readable species silhouettes, existing idle/walk identities, camera and ground anchors. The current atlas is the edit target, not an approved style reference. Validate against the world reference and actual gameplay before release.

Follow-up direction: compare wildlife directly with existing NPCs, not only the world mood board. Attach `villager_idle_hd.png` and `hd_sources/units/guard_idle_front_cut.png` to refinement; judge the same sparse contours, broad quiet color areas and cel shadow treatment side by side at matched gameplay scale. First softer fur pass still has excessive faceted texture and is rejected as the final style.

The corrected water art was explicitly accepted by the user on 2026-10-06
("Looks much better"). It uses pale clay, muted blue water and finer ink; the
comparison and desktop/DPR-2 previews are in the water verification folder.

## Stone road readability follow-up (2026-10-06)

After merging roads, Jakob said the stone street had drifted too far toward sand
and requested slightly stronger material definition. Refine only stone toward
neutral limestone grey with modestly deeper fine joints and clearer soft face
shading; retain the irregular layout and painted style. Dirt stays unchanged.
Jakob preferred the revised neutral-limestone texture after seeing the comparison.
Desktop and DPR2 phone normal/max-zoom review passes; see the [follow-up review](../verification/roads/stone-contrast/README.md).
