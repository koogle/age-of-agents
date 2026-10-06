# Candidate style review — not approved for runtime

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
