# Candidate style review — not approved for runtime

**Update 2026-10-09:** at Jakob's request four candidates are now wired into the
game: the 12c map (`command_explore`), the 11d shield (`category_military`), steel A
from 11a (`resource_steel`) and the 10 cel cloth (`resource_cloth`).
Everything else below remains candidate-only.

Current decision (2026-10-06): Jakob preferred the latest calibrations, including
the thin-line map, and explicitly authorized merging PR #145 as a candidate
archive. Earlier rejection findings remain valid; this does not establish runtime
readiness or select a steel A/B variant.

Human-visible inspection of all batch sheets, 24/32px icon comparisons and world
source previews completed on 2026-10-06. Model sameness did not guarantee style:
several failures below survived explicit prompts. This batch is for selection,
not a passing integration gate.

| Criterion | Finding | Integration verdict |
| --- | --- | --- |
| Ink | Resource objects retain thin brown contours. Some menu/portrait detail is denser than approved original kit; back-arrow edge is softer. | Mixed; cleanup required. |
| Color | Blue cloth, blue-grey steel, red bricks, teal arrow and greener agriculture offer greater material separation. Cloth is stronger than the quiet original UI. Stone differs only subtly from already-refined #143. | Useful directions, not blanket approval. |
| Light/material | Wolf/bear use broad cel planes. Shield/portrait metal and rations apple introduce brighter highlights. Military retry reduces white highlight but retains a gradient. | Reject glossy candidates for integration. |
| Shape/detail | Cargo repair removes invented emblems. Agriculture repair makes seedlings legible. Town roof/basket have excess repeated detail; explore is busier. Portrait identity is not verified against unit masters. | Mixed; simplify before integration. |
| Camera/scale | Rear dock repair preserves a rear facade and one sail, with brown timber matching front direction. Animals change facing/proportions; dock supports/material details change. | No animation/anchor or geometry approval. |
| Era fit | Magnetic compass rejected in runtime and candidate; lace-up boot and heraldic shield candidates require correction. See ERA_REVIEW.md for all subjects. | Blocks runtime acceptance. |
| Alpha/integration | BiRefNet cutouts exist, PNG/alpha checks pass, multiple-background sheets inspected. Source images are not normalized runtime icons or packed frames. Shields differ in silhouette/arrow treatment. | Not integration-ready; paired shield normalization/regeneration required. |

## Retained failures and preferred review variants

- `01-calibration/resource_water`: rejected blurred low-resolution copy on black.
  Compare `01b-calibration-repair/resource_water` instead; this is clean but taller
  than the shipped squat jug, so it changes silhouette.
- `01-calibration/dock_pair`: rejected; both views were front/side rather than rear.
- `05-world/dock_rear`: rejected grey-green walls; did not solve timber mismatch.
- `06-dock-material-repair/dock_rear_matched_timber`: corrected timber but rejected
  duplicate sail. `07-targeted-repairs/dock_rear_single_sail` removes it and is the
  preferred rear comparison. Dense roof seams still need cleanup.
- `03-menu/command_cargo`: rejected invented emblems. Prefer the plain crate in
  `07-targeted-repairs/command_cargo` for review.
- `02-resources/tech_agriculture`: seedlings too small. Prefer larger seedlings in
  `07-targeted-repairs/tech_agriculture` for review.
- `03-menu/category_military`: glossy shield. Retry removes the white highlight
  but remains too gradient-heavy; neither passes final style acceptance.
- `03-menu/command_back`: bevel/gradient too strong. Retry is simpler but soft-edged.
- Unit portraits: rejected for replacing full-body silhouettes with busts and adding
  face/fold detail. Final full-figure batch attaches each approved HD unit identity
  strip directly; guard/healer restore quiet full-figure styling. The final archer
  loses its bow and is rejected for that missing identity/tool cue; original unit
  runtime sprites remain unchanged.
- Cargo shields: up/down silhouettes and arrowheads do not form a matched pair;
  regenerate from one approved shield before runtime consideration.

## Verification limits and quality review

The original villagers and all runtime files remain byte-identical to base.
This candidate-only branch adds no game behavior, dependency, renderer or save
change. Thermonuclear review applied to scope/scripts/docs: direct explicit batch
manifest, one paid generation runner, offline contact-sheet utilities. No generic
asset framework added. Prompts and outputs are retained rather than overwriting
failed attempts. Secret values are not stored.

No Rust test/build, browser flow, physical phone, desktop maximum-zoom run or Modal
deployment is claimed for candidates that are not wired into the game. World
comparisons use equal thumbnail bounds, not authoritative in-game relative scale.
Future integration must regenerate construction/animation variants, register
anchors, normalize icons, refine artifacts and pass the complete style gate.

## User correction: preserve cel-shaded Ghibli look

On 2026-10-06 Jakob found some alternatives too realistic. Broad expansion is
paused. Steel gradients, satin cloth, bread highlights, the domed shield and
dense wolf fur are rejected/need correction. A three-subject steel/cloth/wolf
calibration uses the original villager and primary diorama only as style authority;
color separation is retained while simplifying to two-tone cel forms. No further
broad regeneration is authorized by this calibration.

Calibration outcome: cloth loses satin shine and wolf uses much quieter fur
masses, closer to retained NPCs. Wolf facing/proportions still differ. Steel is
simpler but retains edge glints/gradients and is NOT a style pass. No further
generation expanded from this calibration. Era map removes compass/rose but has
a heavier contour and baked shadow requiring cleanup; sandal removes modern
boot/cuff. All remain unapproved. See cel/era correction comparison sheets.

## Four-subject painted-cel refinement (user requested continued work)

Bounded to steel, wolf, Explore and military shield, with exactly one targeted
retry each. Cloth/sandal were not regenerated. That batch stopped; the later
map-only user request below is separate.

| Subject | Concrete result | Remaining limit |
| --- | --- | --- |
| Steel A | Richer cool-grey painted planes and three-ingot silhouette. | Dense edge marks/strong contour; initial subject reference was 313px, discovered after submit. Not an HD recovery claim. |
| Steel B | Retry uses actual 1024px original, quieter pale painted material. | Very close to original; not automatically better than darker A. Both shown as choices. |
| Wolf | Original right-facing stance/camera/proportions restored. | Both geometry pass and explicit cool-coat retry retained tan colors. Requested recolor FAILED; no meaningful style/color improvement claimed. |
| Explore | Simpler parchment instead of thick scroll rollers, no magnetic device. | Still has strong contour and drop shadow; no final style pass. |
| Military shield | Final retry has quiet blue cel face and spear occluded behind shield. | Most promising concrete correction, but optical/alpha/gameplay normalization unverified. |

No unlimited polishing: eight illustration attempts total (four initial, four
targeted retries), estimated $0.3584 across sixteen successful FAL calls. Prior
outputs/prompts retained. The lead comparison and separate steel A/B sheet show
source-size and 24/32px evidence, not a claim that newest is best.

## Latest user preference and map-only correction

Jakob likes the calibrations shown, but wants thinner map linework. Treat current
calibration direction as preferred, not individual runtime approval. Keep all
other candidates fixed; correct only the outer/map/star/fold ink of latest
`11d-map-shield-targeted-retries/command_explore`. No palette/layout/texture change
authorized. Existing wolf recolor failure and other verification limits remain.

Map line-only first attempt rejected: it rotated the composition, changed teal
islands to green, added islands and enlarged/moved the star. The preferred 11d map
is unchanged. One narrowly specified retry uses only the approved HD villager
style strip and exact map subject to reduce reference confusion.

Second FAL map-only attempt preserves composition but does not visibly reduce
the thick outer stroke at 128/24/32px. It is a failed correction, not promoted over
the preferred original. A dark-brown-pixel diagnostic is effectively unchanged
(11,559 before versus 11,591 after); this is supporting evidence only, not a style
score or certified line-width measurement. Two bounded attempts, four successful
FAL calls, estimated $0.0896. No automatic model substitution performed.

## Disclosed OpenAI map refinement outcome

After both bounded FAL line edits failed, root disclosed the repository refinement
route and used OpenAI image_gen on the preferred 11d map ONLY. No villager/UI
images were directly attached to that call; source-chain references and this
review remain available. Generation ID: `exec-219b5bfa-102b-45fb-8ea0-477804f3049d`.
Unknown backend/version/cost are recorded as unknown, not fabricated.

The 128px comparison shows materially finer outer, fold, coastline and star ink.
At 24/32px the paper, islands and star still read on grey/parchment/green/blue.
Palette/composition remain close; 1254px output versus 1024px source is not a claim
of pixel-exact preservation. Existing shadow is preserved, not newly solved.
This is the latest user-preferred map correction; other liked calibrations remain
unchanged. Runtime normalization/gameplay approval remains pending.

Map follow-up subtotal: two FAL illustrations/two cutouts ($0.0896 estimate),
plus one BiRefNet cutout of OpenAI result ($0.005 estimate). OpenAI cost unknown.
The FAL runner refuses to regenerate externally refined entries as FAL outputs.
