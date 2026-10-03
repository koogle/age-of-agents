# Custom crop field sprites

Generated 2026-10-03 through fal.ai (`fal-ai/nano-banana/edit`), using the completed farm in `buildings_economy.png` as the style reference. `generation.json` records the exact selected prompts; `requests.jsonl` records generation and BiRefNet cutout request IDs and estimated costs (including the rejected tall-seedling iteration).

The four 1024×1024 source renders and their RGBA cutouts are retained here. Cleared soil doubles as the depleted state; cultivation shows bare furrows; seedlings show short, separated sprouts; ripe wheat is the completed/harvestable state. All stages retain the same diamond soil base, camera, lighting and registration. A field has no farm shed or perimeter wall.

`python3 scripts/pack_field_sprites.py` packs the four stages into 512×512 cells in the fifth row of the existing economy atlas. It preserves the original four building rows pixel-for-pixel, uses one crop/scale for all field frames, records the shared soil corners, removes segmentation specks and extends edge colors under transparent pixels for clean mip filtering. Requires Pillow, NumPy and SciPy. The source is downsampled, never enlarged.

The client fits these ground corners to its 3×3 field footprint using the same anchoring/depth calculation as buildings. Preparation switches at one-third and two-thirds of paid work; depleted plots return to cleared soil. The ripe cell also supplies the Field button and placement ghost. Simulation, costs, duration and harvest behavior are unchanged.
