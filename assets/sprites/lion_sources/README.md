# Lion sprites

Lioness (pride member) and maned lion (pride leader), four poses each: idle,
walk, attack windup, attack strike, facing right and mirrored in the renderer.

The integrated sheets (`*_family_render_*`, cut out as `*_family_cutout.png`) were
generated 2026-10-10 with FAL `fal-ai/nano-banana/edit` from the shipped square
wolf/bear sheet alone, with the selected lioness as the lion's pride reference,
then cut out with `fal-ai/birefnet/v2`. Jakob rejected the earlier drafts
(`lion*_render_*`, `lion*_refined_render_*`) because their colour and shading
did not match the game. Editing an off-style sheet kept its airbrushed fills, so
generate new species fresh from the approved family sheet.

Prompts, references, request IDs, measurements and rejected passes are in
`provenance.json`. Run `python3 scripts/pack_wildlife.py` to repack. No OpenAI
image_gen refinement pass yet. [Review](../../../docs/verification/lions/README.md).
