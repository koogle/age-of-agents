# Monster sprites

The mythical monster classes, four poses each (idle, walk, attack windup,
strike), facing right and mirrored in the renderer:

- **Skeleton warrior** (Spartoi): bone-white, bronze Corinthian helmet with a
  crimson crest, round bronze shield, short sword, teal eye glow.
- **Centaur archer**: ivy-wreathed bearded rider on a chestnut horse body, leopard
  skin, recurve bow.
- **Cyclops**: a mandrill-inspired beast with one tall vertical slit eye, red and
  blue ridged muzzle, fangs, olive-grey fur and a rock-studded club (Jakob,
  2026-10-12: scarier, eye turned 90 degrees, animal-like).
- **Cyclops general**: the same beast in a bronze gorgon breastplate, crested
  helmet and dark red cloak, with a bronze war maul.
- **Minotaur**: black bull head with ivory horns and a brass nose ring, red
  Minoan kilt, bronze labrys.

Sheets were drawn with FAL `fal-ai/nano-banana-pro/edit`, using the shipped guard
family (`../barbarian_sources/guard_style_reference.png`) as the style reference
and the concept in `concepts/` as the design reference. The cyclops general used
the cyclops sheet as its sibling reference. Cutouts come from `fal-ai/birefnet/v2`.
Prompts, request IDs, reviews and removed passes are in `provenance.json`. Run
`python3 scripts/pack_wildlife.py` to repack.
