# Painted ground textures

One seamless, top-down painted texture per biome, sampled by `frontend/ground-paint.js` from `/assets/terrain/<name>.webp`. One repeat covers 4×4 cells, and the shader blends biome borders itself.

| File | Size | Use |
| --- | --- | --- |
| `<name>.webp` | 512×512, q88, 6–47 KB | Runtime texture |
| `masters/<name>.webp` | 1024×1024, q92 | Master for re-processing |
| `manifest.json` | | Names, files, target and measured mean colour, prompt per texture, generation settings, cost |
| `contact_sheet.png` | | Each texture tiled 2×2, with its mean colour |
| `tools/terrain.py` | | Generation and post-processing script |
| `tools/ledger.jsonl` | | Per-request FAL ledger for this branch |

Names: meadow, forest, prairie, highland, wetland, scrubland, heath, clayland (the eight `BIOME_COLORS` biomes), plus beach and shallows for the coastal rim.

## Generation

- **Endpoint:** `fal-ai/nano-banana/edit`, 1:1, one image per texture. It has no seed parameter, so request ids are in the ledger.
- **Style references, attached as images (not just described):** `assets/reference/diorama_primary.webp` at 768 px, and a crop of `assets/reference/mediterranean_4.webp` (pixels 700,900–1500,1500, the painted landscape).
- **Prompt:** the shared prefix below, followed by a ground description per biome (in `manifest.json` and `tools/terrain.py`).

> The attached images are STYLE references only (a sunlit Greek island diorama and a Mediterranean landscape painting). Paint a NEW image: a flat, top-down, perfectly overhead square swatch of ground texture in the style of a hand-painted Studio Ghibli background, gouache and watercolour on paper, soft visible but gentle brush strokes, warm even sunlight, low contrast, the same warm Mediterranean palette as the references. The texture fills the entire frame edge to edge with an even, uniform density, no horizon, no perspective, no vignette, no strong shadows, no objects (no trees, no large rocks, no buildings, no paths, no people), no text. Ground: …

## Post-processing

`finish()` in `tools/terrain.py` is deterministic and free:

1. Crop a 3% border, because the model tends to paint a darker frame or vignette; resample to 1024.
2. Flatten large blotches: subtract 75% of the 56 px Gaussian low-pass deviation. Big light and dark patches are what make a repeat visible.
3. Make it seamless: blend with the half-offset copy using a smoothstep weight that is 1 in the middle and 0 at the edges. The output wraps exactly by construction, and the blend zone is soft enough not to show on these homogeneous textures.
4. Recolour: move the mean onto the target colour and scale deviations by 0.75 (low contrast, so units read on top). The targets are `BIOME_COLORS`, beach (237,209,153) to match the shader's sand, and shallows (120,196,190).

Rejected: inpainting the offset seam with `fal-ai/flux-pro/v1/fill`. It read the cross-shaped mask as an object and painted a raised plank cross into the beach. The first clayland was regenerated once because its pebbles came in two horizontal bands that striped the tile.

## Verification

- Every texture was checked tiled 2×2 (contact sheet) and with 1:1 crops across the wrap edges.
- Mean colours land within ±1 of their targets (measured on the 512 webp).
- In-game, headless Chromium at gameplay and closest zoom: the shader picks up all ten files (no 404s), and villagers, resources and the town center stay readable on the painted ground.

## Cost

About **$0.60**: eleven nano-banana/edit generations (ten textures plus the clayland redo) and three Flux Fill attempts that were thrown away. Branch total so far is about $3.92.

## Known limitations

- Scrubland's dry-earth crackle forms a polygon mosaic that is visible at the closest zoom.
- Highland is very pale and quiet; it may want slightly more texture if it reads as blank.
- Forest's fallen-needle detail mostly turns into small dark specks after recolouring.
