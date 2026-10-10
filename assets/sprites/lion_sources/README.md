# Lion sprites

Lioness (pride member) and maned lion (pride leader), four poses each: idle,
walk, attack windup, attack strike, facing right and mirrored in the renderer.

Integrated designs, chosen by Jakob on 2026-10-10 from `concepts/`: the **island
huntress** (faint rosettes, black-backed ears, white throat) as the lioness and
the **Nemean lion** as the leader. Each was adapted with FAL
`fal-ai/nano-banana-pro/edit` (2K, 1024px cells) from the wolf/bear sheet and its
concept (`huntress_render_0.png`, `nemean_render_3_clean.png`, with generated
cell dividers painted out). Jakob then asked for thinner line work and a more
real lion, so both were repainted as naturalistic wildlife illustrations
(`*_natural_render_0.png`), then given cel shading at a medium detail level
(`nemean_cel_render_0.png`, `huntress_cel3_render_1.png`, cut out as
`*_cel_cutout.png`). The packer only registers and downsamples; a model pass to
match the wolf's ink is pending (`ink_prompt.txt`), blocked on FAL balance.

Earlier passes (`lion*_render_*`, `*_refined_*`, `*_family_*`, `*_pro_*`) were
rejected: generic Disney-like, then grey, then off in line work and colour.
Prompts, references, request IDs and reviews are in `provenance.json`. Run
`python3 scripts/pack_wildlife.py` to repack. No OpenAI image_gen cleanup pass.
[Review](../../../docs/verification/lions/README.md).
