# Alternatives from the original villager model — unapproved

55 retained illustration attempts, including rejected/repaired passes, from 108
successful FAL requests (estimated $2.4540, excluding one API validation failure).

Candidate art only. No runtime image, atlas, manifest, code or retained villager
has changed. These are review choices, not permission to replace existing art.

Latest bounded follow-up: [four-subject comparison](painted-cel-refinement-comparison.jpg)
and [steel A/B options](painted-steel-options-comparison.jpg). Shield is promising;
wolf recolor failed, map retains shadow/line issues, and steel remains a choice.

Earlier user corrections: [era audit](ERA_REVIEW.md), [era comparison](era-correction-comparison.jpg),
and [three-subject cel calibration](cel-correction-comparison.jpg). Broad expansion
is paused; all remain unapproved. The steel calibration still needs less shine.

Open [the gallery](index.html), [resource comparison](02-resources-comparison.jpg),
[menu comparison](03-menu-comparison.jpg), [world comparison](world-comparison.jpg)
and [terrain comparison](terrain-comparison.jpg). Icon sheets include 128px and
24/32px views on neutral grey, parchment, dark green and blue. World sheets show
source-size and small-scale comparisons; they are not in-game screenshots.

## Scope and source selection

Base: `1984e9b` (including stone-road contrast PR #143). The original audit covered
October 5 07:26 UTC–October 6 07:26 UTC, master `0c7b649`. Its 414 changed image
files include drafts, retained originals and screenshots. This work targets the
24 changed runtime icons/buttons, plus representative alternatives for the
changed docks, wildlife, water and two roads. It does not regenerate rejected
historical drafts, screenshots, every construction stage or complete animations.
Water has one shared UI/world candidate. Docks use front and rear only; opposite
orientations remain a mirroring concern tracked separately in PR #144.

The unchanged original villagers are the identity/style anchor. All generation
requests attach the approved villager master; world requests also attach the
primary diorama, while UI requests attach the original wood/food/masonry kit
(or established blank coin for seal-family candidates). Newer runtime images
provide subject geometry only. A low-resolution subject copy failed in calibration
and was removed from subsequent icon requests.

## Exact original model and prompt provenance

The retained base villager identity and action strips were generated with
**`fal-ai/nano-banana/edit`**, confirmed by
[the sprite README](../../sprites/README.md),
[original strip prompts](../../sprites/tools/strips.py) and
[the original ledger](../../sprites/tools/ledger.jsonl). The ledger includes
`spr:master1`, request `01a0fa27-a532-78f1-9984-90f5057e46c7`, alongside the two
other master candidates; the strip script selects `spr/master_1.png`.
The exact identity-master prompt is not retained in the repository; its reference
chain and selection are documented. The exact animation prompt IS retained:
“the same fine thin dark-brown ink linework and flat watercolour wash,” stable
identity/proportions, three-quarter RTS camera and isolated white background.
Those original prompt constraints are adapted to each new subject here.

All new illustration calls use that same exact endpoint. BiRefNet v2 removes
backgrounds, as in the original pipeline. No generator was silently substituted.
No second-model refinement, runtime packing or upscaling has been performed;
these original 1024px-class renders remain initial candidates requiring cleanup
and integration validation before any future replacement.

## Batches and reproducibility

[batches.json](batches.json) retains every exact prompt and ordered reference path.
Each candidate directory holds `original.png`, optional `cutout.png`, model,
reference SHA256 hashes, source revision and raw provider response. The
[ledger](ledger.jsonl) records request IDs, tags, elapsed time and estimated costs.
[Calibration notes](calibration-notes.md) preserve rejected approaches.
`reference_revision` names the base for repository references; references under
this alternatives directory are generated in this branch and are identified by
content hash, not present at that base commit.

From the repository root:

```bash
# PAID: one named batch only, skips complete outputs
python3 assets/alternatives/2026-10-06/generate.py 02-resources
# Offline review sheets only
python3 assets/alternatives/2026-10-06/world_review.py
python3 assets/alternatives/2026-10-06/review.py
```

The model has no seeded reproducibility guarantee. Exact original outputs and
request IDs are retained. Costs are estimates using the repository's historical
$0.0398/image and $0.005/cutout rates, not a retrieved billing statement. One
invalid `2:1` dock request returned API validation failure without an image;
its billable cost is unknown and excluded from successful-request estimates.

## Acceptance limits

See [the explicit review](STYLE_REVIEW.md) and [validation report](validation.json).
Candidate inclusion is not approval. In particular, stronger color does not
excuse glossy shading, extra ornament, changed geometry or poor small-size
readability. Rejected attempts remain visible and labelled for comparison.
Construction states, wildlife gait/attack continuity, registration, normalized
runtime alpha, actual desktop/phone gameplay and maximum-zoom acceptance remain
future integration work. No deployment was performed.

Future NEW generation requests append `era-prompt.txt` and `cel-prompt.txt`;
completed/resumed historical responses preserve the exact prompts originally
used. This does not rewrite historical generation instructions or hashes.

The era/cel follow-up adds six illustration attempts and twelve successful calls,
estimated $0.2688. Only two Explore concepts and three cel calibration subjects
were generated; broader expansion remains paused.
