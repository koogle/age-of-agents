# Lion sprites

Lioness (pride member) and maned lion (pride leader), four poses each: idle,
walk, attack windup, attack strike, facing right and mirrored in the renderer.

Integrated designs, chosen by Jakob on 2026-10-10 from `concepts/`: the **island
huntress** (faint rosettes, black-backed ears, white throat) as the lioness and
the **Nemean lion** (bronze-gold hide, scarred muzzle, heavy mane) as the leader.
Each was adapted with FAL `fal-ai/nano-banana-pro/edit` (2K, 1024px cells) using
the shipped wolf/bear sheet as the style reference and its concept as the design
reference, then cut out with `fal-ai/birefnet/v2` (`huntress_cutout.png`,
`nemean_cutout.png`). The Nemean sheet had generated cell divider lines painted
out first (`nemean_render_3_clean.png`).

Earlier passes (`lion*_render_*`, `*_refined_*`, `*_family_*`, `*_pro_*`) were
rejected: generic Disney-like, then grey, then off in line work and colour.
Prompts, references, request IDs and reviews are in `provenance.json`. Run
`python3 scripts/pack_wildlife.py` to repack. No OpenAI image_gen cleanup pass.
[Review](../../../docs/verification/lions/README.md).
