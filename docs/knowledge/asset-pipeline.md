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
2. For FAL generation, inspect the approved reference files and retained provenance.
   Historical batch generators are retired; recover an old recipe only when
   needed through the retirement record.
   Do not launch an old paid batch to repair one frame. The existing
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
Approved kit and rejected drafts were compared side by side; rejected work is
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
| Era fit | Ancient Greek Mediterranean subjects/material depiction; simple abstract UI glyphs | Magnetic compasses, modern navigation instruments, medieval heraldry or clearly modern equipment |
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
and DPR-2 phone checks; both passed.
Road materials occupy layers 11/12 of the existing texture array and use its
mirrored, mip-clamped sampling; retain full-resolution originals and record exact
prompts/reference hashes before integration.

## New species from the family sheet (2026-10-10)

Jakob rejected the first lion drafts: colour and shading did not match the game.
Measured on opaque pixels, the lioness coat sat at value 0.54–0.77 (flat,
airbrushed), while the wolf spans 0.44–0.87 and the bear 0.35–0.64 with painted
light and shadow sides. Editing the off-style sheet with the wolf attached barely
changed its rendering, and repaints drifted in colour between top and bottom
cells. Generating fresh from the square shipped sheet
`wildlife_sources/npc-style-refined.png` alone, then using the chosen new animal as
the identity reference for its sibling form, matched palette and shading with one
colour across all cells. Keep references square: a 2:1 reference produced
1472×704 sheets with cells below the 512px minimum. Check per-cell saturation and
value percentiles before packing.

That family pass was itself rejected: copying the wolf made the lions grey, and
the earlier drafts looked like generic animated-movie lions. What worked: first
explore distinctive concepts with `fal-ai/nano-banana-pro` (text to image), let
Jakob pick, then adapt each with `fal-ai/nano-banana-pro/edit` using the approved
family sheet as style reference and the concept as design and colour reference;
generate the second form with the first as its sibling reference. Ask for the
facing, identical colours in every cell and two-thirds cell width explicitly,
and check for generated cell divider lines before cutout.
Prompting for thinner lines barely changes line weight; `thin_lines()` in
`scripts/pack_wildlife.py` thins strokes deterministically instead (1px shave at
2K; 2px broke silhouettes). Ask for realism as a "naturalistic wildlife
field-guide illustration" with concrete anatomy; that removed the fantasy look.
Then ask for cel shading as "flat base colour plus one shadow tone of the same hue
per colour region", and name the distinguishing markings to keep: a plain cel
request stripped the lioness's ear backs and rosettes (Jakob: a fine balance
between too much and not enough detail). A cel pass can also bleach colour and
fade outlines (the lioness went from value 0.75 to 0.87 and from 10% to 4% dark
pixels; Jakob: beige, lost linework). Measure saturation, value and the share of
pixels under luminance 0.3 against the sibling and the wolf before packing, and
anchor the weaker sheet to the stronger sibling's render with "as deep as", "as
crisp and dark as" wording naming the reference. When frames still read muddled
beside the wolf, measure line work at atlas scale (stroke width, ink luminance
and colour, line coverage, silhouette edge gradient) to name the gap, then close
it with an image model edit that attaches the wolf sheet and asks for its ink
colour, continuous crisp outlines and short interior lines. Jakob (2026-10-10):
line work and shading fixes go through an image model, per the asset workflow;
a packer re-ink step (snap strokes to the wolf's ink, harden alpha, sharpen) and
a 2K stroke-shave were built, matched the numbers, and were then removed for
that reason. Pixel code in the packer is for registration and packing only. Jakob (2026-10-10,
after the lion folder reached 117 MB): do not commit rejected or intermediate
renders. Keep the integrated source, cutout, approved references and prompts;
record every other pass as request IDs and a one-line review in `provenance.json`
and delete its images before merging. Evidence folders keep final comparisons. The model pass
that replaced them (`lion_sources/ink_prompt.txt`: keep everything, redraw the
outlines in the wolf's deep brown-black ink, continuous and crisp, with his short
interior lines) landed on the wolf's numbers in one round when the ask named
ink colour and continuity rather than "thinner"; an earlier "thinner lines"
prompt had barely moved line weight. Provenance:
[lion sources](../../assets/sprites/lion_sources/provenance.json).

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
Desktop and DPR2 phone normal/max-zoom review passed.

## Alternative batches (2026-10-06, reviewed archive)

Jakob requested alternatives for recently merged art using the original villager
model and similar prompts, to restore consistency and controlled color accents.
The retained originals use `fal-ai/nano-banana/edit` (see sprite README and
`tools/strips.py`); preserve villagers and runtime assets. Candidates live under
`assets/alternatives/2026-10-06/`, with exact references and request provenance.
Newer assets supply subject geometry only; approved villagers, primary diorama
and original UI kit supply style. Review a small calibration batch before
scaling; rear dock timber should share the front material rather than turn grey.
Jakob preferred the latest calibrations including the thin-line map and explicitly
authorized merging the candidate archive in PR #145 on 2026-10-06. Rejected
attempts remain rejected; these are not integration-ready animations.

The first batches confirm that reusing the same model is insufficient: small
128px subject references can be copied as blurred enlargements, explicit rear-view
prompts can produce another front view, and palette corrections can invent props.
Keep approved style references attached, use retained HD subject identity when
needed, inspect each batch and retain failures. The candidate gallery and explicit
[review](../../assets/alternatives/2026-10-06/STYLE_REVIEW.md) separate successful
color directions from unfinished linework, material, silhouette and alpha work.

## Era fit (developer steering, 2026-10-06)

Jakob explicitly rejected anachronistic visuals, citing the compass. Review every
subject for the broad ancient Greek Mediterranean setting, without inventing an
exact year or banning plausible metallurgy/tools. The shipped Explore magnetic
compass and map-plus-compass alternative are historically rejected; runtime
replacement remains pending integration. Prefer simple coastline parchment with
a celestial star, not a compass rose, magnetic needle or modern navigation marks.
Check footwear, weapons, armor, heraldic ornament and material depiction as well
as palette. Abstract directional/check UI glyphs are interface conventions, not
claims of period artifacts. Record uncertain identifications as questions.
See the [candidate era audit](../../assets/alternatives/2026-10-06/ERA_REVIEW.md).

## Cel shading correction (developer steering, 2026-10-06)

Jakob liked some alternatives but found others too realistic, losing the
cel-shaded Studio Ghibli look. Stop broad expansion; calibrate at most three
representative subjects before any further batch. Original villagers and primary
diorama are visual authority. Restore soft two-tone cel shading, broad quiet flat
colors, sparse brown ink, restrained highlights and simplified painted forms.
Color pop comes from subject hue/value separation and silhouette, not gloss,
specular gradients, PBR-like material modeling, microtexture or dense fur.
Original UI wood/food may support readability but must not pull material rendering
toward realism. Earlier glossy steel/cloth/shield, realistic bread and detailed
wolf alternatives are not preferred defaults; label their review status visibly.

## Bounded painted-cel refinement (2026-10-06)

After seeing comparisons, Jakob asked to keep working on them. The next bounded
review covers steel, wolf, Explore and military shield only; cloth and sandal
remain tentative. Preserve hand-painted Ghibli cel character, not sterile flat
vector diagrams. Restrained painted variation and a small soft highlight are
allowed when consistent with the original villagers; avoid gloss/PBR and noise.
Lock original wolf right-facing silhouette/camera with retained HD subject art,
while simplifying fur. Keep map contours fine and its celestial star distinct
from a compass rose. Inspect the representative before any targeted retries;
maximum one retry per subject for a concrete defect, no broad regeneration.

Check actual reference dimensions before submission: `refined.png` in the menu
kit can be a 313px crop while `generated.png` retains 1024px. File names do not
establish HD source quality. In this bounded follow-up, strong wolf geometry
references were copied without requested recoloring; preserve that failure and
stop at the retry bound rather than calling near-identical output an improvement.

## Map line-weight correction (2026-10-06)

Jakob likes the visible calibration direction, except the map linework remains
too thick. Refine only the current Explore map contours: thinner outer edge,
coastline, star and fold ink matching approved villagers/UI kit. Preserve layout,
silhouette, palette, paper/islands/star and existing texture; no wider redraw.
This initial preference did not approve runtime integration or accept the failed
wolf recolor; later explicit archive merge authorization is recorded above. Keep originals and compare at 128/24/32px; at most one
targeted retry after the initial map-only pass.

Both original-model narrow map edits failed: one redesigned the image, one
retained the thick stroke. Root then disclosed OpenAI image_gen refinement and
used the preferred map as its sole direct image input. The resulting contour is
visibly finer at 128/24/32px; retain the exact direct-input distinction, source
chain and unknown OpenAI cost/backend. Never relabel an external refinement as
a FAL illustration or infer runtime approval from the user liking a direction.

## Dirt road readability follow-up (2026-10-06)

Jakob reports the sandy dirt road is visually hard to see and requests recreation.
The earlier dirt integration pass does not establish sufficient readability.
The local replacement uses deeper ochre-brown packed earth with restrained painted
variation, retaining the diorama/meadow family, fine pebble contours and mirrored
sampling. Desktop and emulated DPR2 phone default/maximum zoom show clearer separation
from meadow, retaining fine detail without visible repeat seams. Judge the result under actual terrain lighting;
source contrast alone is insufficient.
