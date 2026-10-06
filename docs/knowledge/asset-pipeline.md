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

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).

## Directional dock integration (2026-10-05)

The building packer appends three directional rows to the existing atlas, keeping
all original pixels and UV rectangles. Use the retained 2172×724 strips in
`assets/sprites/building_sources/directions`; the earlier 1254×1254 construction
sheet was rejected because its cells were only 418px. Each integrated source
cell is 724×724 before packing down to 512×512. Manual structural deck corners
register each construction stage, and the atlas manifest drives texture dimensions.
The new geometry remains upright in the fixed camera. Domain facing and preview
behavior are documented in [placement](placement-and-routes.md#dock-orientation-2026-10-05).

## Wildlife style correction (2026-10-05)

Jakob found the shipped wolves and bears too generic and not fully aligned with the game. Refine against `assets/reference/diorama_primary.webp`: simplify dense fur into painted masses, soften dark contours and contrast, and preserve readable species silhouettes, existing idle/walk identities, camera and ground anchors. The current atlas is the edit target, not an approved style reference. Validate against the world reference and actual gameplay before release.

Follow-up direction: compare wildlife directly with existing NPCs, not only the world mood board. Attach `villager_idle_hd.png` and `hd_sources/units/guard_idle_front_cut.png` to refinement; judge the same sparse contours, broad quiet color areas and cel shadow treatment side by side at matched gameplay scale. First softer fur pass still has excessive faceted texture and is rejected as the final style.
