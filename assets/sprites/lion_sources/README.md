# Lion sprites

Lioness (pride member) and maned lion (pride leader), four poses each: idle,
walk, attack windup, attack strike, facing right and mirrored in the renderer.

Integrated designs, chosen by Jakob on 2026-10-10 from `concepts/`: the **island
huntress** (faint rosettes, black-backed ears, white throat) as the lioness and
the **Nemean lion** as the leader. Each was adapted with FAL
`fal-ai/nano-banana-pro/edit` (2K, 1024px cells) from the wolf/bear sheet and its
concept (renders removed; the Nemean sheet had generated cell dividers painted
out before cutout). Jakob then asked for thinner line work and a more
real lion, so both were repainted as naturalistic wildlife illustrations
(removed), then given cel shading at a medium detail level
(now removed), and finally an ink
pass with the wolf sheet attached that redraws the outlines in his ink
(`ink_prompt.txt`; `nemean_ink_render_1.png`, `huntress_ink_render_1.png`, cut
out as `*_ink_cutout.png`). The packer only registers and downsamples.

Earlier passes (draft, wolf-matched, warm redraw, concept adaptation,
naturalistic, cel, first ink) were rejected in turn: generic Disney-like, then
grey, then off in line work and colour. Their images were removed on 2026-10-10
under Jakob's rule; `provenance.json` keeps every request ID and review.
Prompts, references, request IDs and reviews are in `provenance.json`. Run
`python3 scripts/pack_wildlife.py` to repack. No OpenAI image_gen cleanup pass.
[Review](../../../docs/verification/lions/README.md).
