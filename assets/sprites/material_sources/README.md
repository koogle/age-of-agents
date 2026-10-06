# Building material sources

Generated edits from OpenAI image_gen on 2026-10-05; exact backend model/request IDs were not exposed. `provenance.json` records artifact identifiers, hashes and source dimensions. Prompt summaries are summaries, not verbatim prompts. PNG originals are retained without modification.

The reference art is the original building catalog retained in the adjacent `_masonry.png` atlases (crafts before pedestal editing is `buildings_crafts_original.png`). `references/` retains isolated crops used to correct ambiguous atlas references. House and dock also referenced the existing high-resolution clean-roof sources under `../building_sources/clean_roofs/`.

Edits preserve Greek isometric identity, lighting and transparent background. House, granary, farm, weaver and range use straw thatch; other roofed buildings use wood shingles/planks. Kiln and kitchen ovens use natural stone before the brick upgrade. Monument retains its original stone statue, gaining a brick pedestal after upgrade. The town-center pair is complete/working; other pairs are roof-stage/complete. House uses only the left pair frame and separate `house_complete.png`. Weaver additionally replaces its partially tiled wall-stage frame. Original early construction frames are otherwise retained.

Run `python3 scripts/pack_material_sprites.py` from the repository root. Packing trims alpha ≤8, aligns to the original opaque frame bounds, and only downsamples. No detail is synthesized or upscaled. Both tiers share existing atlas layouts/plot registration; HUD portraits use the same packed art.

Rejected approaches: whole-sheet edits had only 280–313 pixels per cell and were rejected rather than enlarged. Whole-atlas references also confused farm/mine and range/workshop/infirmary/barracks identities, and one dock lost its boat. Those drafts remain in the session generated-image archive, are excluded from production, and were replaced using isolated exact references. A separate town-center roof/complete draft was superseded by complete/working frames.

Final catalog and browser evidence: [material-tier verification](../../../docs/verification/2026-10-05/material-tiers/README.md).

Directional dock integration from upstream `ba51a51`: both HD atlases use the expanded 2048×3584 layout. Original terracotta directional rows remain in the masonry atlas. Three new wood-roof pairs use isolated directional references and original deck registration. Run the material packer after `pack_building_sprites.py` when rebuilding the complete catalog.

Linework correction: the initial house/granary thatch had dense straw strokes,
heavy dark seams and bright highlights. `superseded_linework/` retains those
sources and their original provenance. Current pairs were regenerated against
the approved high-resolution clean-roof originals with sparse warm ink and soft
two-tone thatch. Exact refinement prompts are in `linework-prompts.json`.
`house_complete.png` is now the unscaled right-hand crop of `house_pair.png`.
The corrected granary also restores its original wheat emblem and grain sacks.
See the [equal-size comparison](../../../docs/verification/2026-10-05/house-linework/comparison.jpg).
