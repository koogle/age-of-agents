# Barbarian sprites

The raid's war band, four poses each (idle, walk, attack windup, strike), facing
right and mirrored in the renderer: the **barbarian raider** (Thracian: fox-skin
cap, zigzag cloak, crescent wicker pelta, curved sica) and the **barbarian
chieftain** (gilded helmet with a dark red crest, bear-pelt cloak, scale
corselet), plus the **barbarian torchbearer** Jakob asked for (bare tattooed
chest, zigzag loincloth, fox tail, hand axe, burning pitch torch). Raider and
chieftain were picked from eight concepts on 2026-10-11 (`concepts/`).

Each sheet was drawn with FAL `fal-ai/nano-banana-pro/edit` using the shipped
guard sprites as the style reference (`guard_style_reference.png`); the chieftain
and the torchbearer used the accepted raider sheet as their sibling reference. Cutouts by
`fal-ai/birefnet/v2`. Prompts, request IDs, reviews and removed passes:
`provenance.json`. Run `python3 scripts/pack_wildlife.py` to repack.
