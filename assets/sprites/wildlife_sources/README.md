# Wolf and bear sprites

Superseded and rejected images named below (`refined.png`, `draft.png`,
`wolf-bear-draft.png`, `style-pass-1-rejected.png`, `boar-refined.png`,
`boar-detailed-rejected.png`, `boar-approved-animation-edge-draft.png`,
`boar-silhouette-study.png`) were removed on 2026-10-10 under Jakob's rule against
keeping intermediate art; their request IDs stay in the provenance files. The
packer reads `npc-style-refined.png`, `npc-attack-refined.png` and
`boar-approved-animation.png`; `boar-npc-study.png` is the approved identity.

The original four-frame atlas (`refined.png`, removed) was superseded. The integrated
`../wildlife.png` / `../wildlife.json` now have eight 627px frames; see the
NPC-style refinement and authored attack sections below. Mirrored to face
left; reverse-facing poses are not authored. The attack extension below adds windup/strike frames.

`wolf-bear-draft.png` was generated with FAL `fal-ai/nano-banana/edit` using
`assets/reference/diorama_primary.webp`; exact prompt/response are retained in
`wolf-bear-provenance.json`, with request ID/cost in `assets/ui/tools/ledger.jsonl`.
The earlier wolf/boar draft is retained as an unused source: the user requested a
bear before integration, replacing the proposed boar.

`refined.png` (removed) was the unmodified OpenAI image_gen refinement of that draft,
generated 2026-10-05: four square cells, consistent wolf/bear identities,
standing and walking poses, corrected anatomy, no text or numbers, transparent
background, painted cel shading and fine ink outlines. Original tool output:
`/workspace/generated_images/exec-b51c3a46-89ac-49c4-a244-206e2e8d496a.png`.
The original atlas copied this output without resizing or claiming recovered detail.
The source is 1254×1254 despite a prompt requesting at least 1536×1536; each
627px frame still exceeds the project's 512px minimum. The original renderer used measured per-frame
paw baselines (611/589/541/545 pixels) to keep movement frames grounded. Run `python3 scripts/check_sprite_resolution.py` to audit bounds,
RGBA format, transparency and original cell sizes.

## NPC style refinement, 2026-10-05

The integrated atlas now uses `npc-style-refined.png`, generated with OpenAI
image_gen with the shipped villagers and original guard art attached alongside
the world reference. `style-pass-1-rejected.png` (removed) showed that softer
painted fur alone still looked too textured beside the NPCs. The final pass
uses broad cel color regions, restrained warm brown contours and sparse fur
tufts. Exact final prompt, reference revision and outputs are recorded in
`npc-style-provenance.json` and `npc-style-prompt.txt`.

Run `python3 scripts/pack_wildlife.py` from the repository root to reproduce
`wildlife.png`. All four original 627px cells are downsampled equally to 589px
inside the existing 627px frame layout, with the opaque paw baseline at y=590.
The renderer uses that common baseline. Sources are never enlarged. Existing
world sprite sizes, simulation behavior, manifests and facing scope are retained.

[NPC comparison and review](../../../docs/verification/2026-10-05-animal-style/REVIEW.md)
contains the actual shipped villager/woman/elder/guard alongside old and new animals.

## Authored attacks, 2026-10-05

`npc-attack-refined.png` adds a wolf braced windup / open-jaw bite and a bear
raised-paw windup / forward swipe. The original NPC-matched animal source and
approved villager/guard references were attached to the edit. The exact prompt,
source output and reference hashes are in `npc-attack-provenance.json`.

The integrated atlas is now **2508×1254, four 627px columns per species**:
idle, walk, attack windup, attack strike. The packer preserves idle/walk pixels
and their y=590 baseline. Attack frames use the same scale but register measured
planted rear-paw landmarks to idle, preserving body position when the front paw
lifts. The moving front paw does not determine whole-body registration.
The packer writes both PNG and JSON; all eight frames exceed the 512px minimum.

[Attack comparison and integration review](../../../docs/verification/2026-10-05-animal-attacks/REVIEW.md)
records style, grounded registration and desktop/phone evidence.

## Boars, 2026-10-06

Jakob requested the earlier `draft.png` boar (removed) as an additional danger.
`boar-refined.png` (removed) was the unmodified transparent second OpenAI image_gen result,
refined against that original and the approved NPC-matched wildlife sheet.
Exact prompts, source outputs and reference hashes are in `boar-provenance.json`.
`boar-detailed-rejected.png` (removed) was the first pass, rejected for dense fur
strokes; the final pass matches the approved family’s broad cel masses.
Four 627px source cells hold idle, walk, lowered-head windup and tusk strike.
The packer appends them as a third atlas row (2508×1881), preserving all eight
wolf/bear frames byte-for-byte. No original source pixels are enlarged.

[Boar style and integration evidence](../../../docs/verification/boars/REVIEW.md)
records the final comparison, desktop and phone hunting, and verification scope.

## Reopened style review

The user rejected the integrated result as not there yet. Two follow-up idle
studies and exact prompts/reference hashes are retained in the `boar-npc-study`
and `boar-silhouette-study` files. Neither is accepted or integrated: the first
still has large angular patches; the second has more natural anatomy but too
much fine fur texture. Prior style acceptance is superseded. Gameplay tests
remain valid; no runtime pixels changed during this review.

## Selected boar family (2026-10-06)

Jakob selected the first follow-up study, `boar-npc-study.png`, superseding the
agent’s earlier assessment. `boar-approved-animation.png` extends that exact
light taupe design to idle, walk, windup and strike, with a second tool pass
restoring safe transparent margins around the snouts. The edge draft and both
exact prompts are retained; see `boar-approved-animation-provenance.json`.
This is now the packed third atlas row. The prior `boar-refined.png` was removed
as rejected history. Wolf and bear pixels, frame dimensions and gameplay rules
are unchanged.
