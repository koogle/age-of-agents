# Painted ground textures

Historical generation/contact scripts were retired in the 2026-10-06 cleanup;
their references below describe provenance, not current reproduction commands.
Active offline packers and original art are retained; retired scripts are
recoverable from git at `1984e9b`.

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
2. Flatten only very large blotches: subtract 50% of the 160 px Gaussian low-pass deviation (85% for forest and clayland, 75% for scrubland, whose big patches showed the repeat at 3×3). Brush-scale strokes are kept, because the shader lifts contrast about 1.8× itself.
3. Make it seamless: blend with the half-offset copy using a smoothstep weight that is 1 in the middle and 0 at the edges. The output wraps exactly by construction, and the blend zone is soft enough not to show on these homogeneous textures.
4. Recolour: move the mean onto the target colour at full contrast (1.0). The targets are `BIOME_COLORS`, except meadow, which uses a softer, warmer (172,196,100) as asked after the in-game review instead of (176,204,92). Beach is (237,209,153) to match the shader's sand, and shallows is (120,196,190).

Revision 2: after an in-game review, the first pass (contrast 0.75, 56 px flattening at 75%) read as flat lime. Meadow was regenerated with a prompt asking for clearly visible brushed grass strokes, and all ten were re-processed with the settings above.

Rejected: inpainting the offset seam with `fal-ai/flux-pro/v1/fill`. It read the cross-shaped mask as an object and painted a raised plank cross into the beach. The first clayland was regenerated once because its pebbles came in two horizontal bands that striped the tile.

## Verification

- Every texture was checked tiled 2×2 (contact sheet) and with 1:1 crops across the wrap edges.
- Mean colours land within ±1 of their targets (measured on the 512 webp).
- Repeat checked on 3×3 tilings; forest keeps a faint medium-scale pattern, which the shader's second rotated sample breaks up.
- In-game, headless Chromium at gameplay and closest zoom: the shader picks up all ten files (no 404s), and villagers, resources and the town center stay readable on the painted ground.

## Cost

About **$0.64**: twelve nano-banana/edit generations (ten textures plus the clayland and meadow redos) and three Flux Fill attempts that were thrown away.

## Known limitations

- Scrubland's dry-earth crackle forms a polygon mosaic that is visible at the closest zoom.
- Highland is very pale and quiet; it may want slightly more texture if it reads as blank.
- Forest keeps a faint medium-scale repeat in a plain 3×3 tiling.

## Building plots

`cobblestone.png` is an opaque 1254×1254 illustrated limestone swatch with warm cream stone, light sandy mortar and fine brown outlines matching the original house art. Generation provenance is in `cobblestone.provenance.json`. FAL nano-banana/edit rejected the initial request with a content-check error; the replacement was generated with OpenAI image_gen.

The renderer mirrors the swatch in world space and clamps sampling inside each mip level, so repeated tiles meet at identical pixels without blurring or doubling the painted contours. It still covers exact building claims and placement previews, independently of the optional grid overlay. Original `assets/sprites/tile_stone.png` is retained but no longer used for paving.

## Roads (2026-10-06)

`road_dirt.png` and `road_stone.png` are dedicated opaque 1254×1254 painted
swatches, refined with OpenAI image_gen from approved clay/paving and attached
meadow/primary-diorama references. [Exact prompts and provenance](roads.provenance.json)
record the reference revision, hashes and output originals. The initial dirt used
sandy ochre brushwork; the readability correction below deepens it to packed-earth
brown. Stone uses irregular limestone with fine brown joints.

The Rust asset manifest loads these after the unchanged plot material (array
layers 11 and 12). Sources remain at full resolution; upload resamples to 512×512
with mipmaps, using the same mirrored/clamped sampling as plots. This guarantees
continuous repeat edges even though the generated source is not assumed to wrap.
No biome or building-plot pixels were changed.

### Stone readability correction

The stone material was subsequently cooled toward neutral limestone and given
slightly stronger joints and soft face shading after Jakob found it too sandy.
The layout, scale and sampling are unchanged. The initial sandy output is retained
in `road_sources/stone-sandy-before.png`; [follow-up provenance](road_sources/stone-contrast.provenance.json)
records the exact prompt and reference hashes.

### Dirt readability correction

The pale sandy dirt was recreated as deeper muted ochre-brown packed earth after
Jakob reported it was hard to see. The previous source is retained in
`road_sources/dirt-sandy-before.png`; [follow-up provenance](road_sources/dirt-readability.provenance.json)
records approved references and the exact prompt. The generated full-resolution
PNG replaces only the dirt material; sampling and stone/biome/plot assets remain
unchanged.
