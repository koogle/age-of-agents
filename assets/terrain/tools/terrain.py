"""Seamless painted ground textures for assets/terrain (one per biome)."""
import json, sys, os
import numpy as np
from PIL import Image, ImageDraw, ImageFilter
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri

REFS = [data_uri("dio/ref.jpg"), data_uri("terrain/ref_med4.jpg")]
STYLE = ("The attached images are STYLE references only (a sunlit Greek island diorama and a Mediterranean landscape painting). "
  "Paint a NEW image: a flat, top-down, perfectly overhead square swatch of ground texture in the style of a hand-painted Studio Ghibli "
  "background, gouache and watercolour on paper, soft visible but gentle brush strokes, warm even sunlight, low contrast, "
  "the same warm Mediterranean palette as the references. The texture fills the entire frame edge to edge with an even, uniform density, "
  "no horizon, no perspective, no vignette, no strong shadows, no objects (no trees, no large rocks, no buildings, no paths, no people), no text. Ground: ")
BIOMES = {
  "meadow":   ((176, 204, 92),  "lush yellow-green meadow grass with tiny scattered painted wildflowers in white and yellow"),
  "forest":   ((108, 156, 74),  "darker green forest-floor grass with fallen pine needles and small leaf litter"),
  "prairie":  ((230, 200, 104), "golden dry summer grass with soft ochre tones"),
  "highland": ((214, 206, 182), "pale cream limestone ground with tiny pebbles and thin sparse grass"),
  "wetland":  ((118, 184, 128), "damp lush green grass with tiny reeds and small soft puddle glints"),
  "scrubland":((206, 180, 112), "ochre dry soil with sparse small dry shrub tufts"),
  "heath":    ((166, 176, 104), "olive-sage low heather with tiny purple heather specks"),
  "clayland": ((214, 142, 92),  "warm terracotta clay soil with faint cracks and tiny pebbles"),
  "beach":    ((237, 209, 153), "warm pale fine beach sand with very faint ripples and a few tiny shell specks"),
  "shallows": ((120, 196, 190), "clear turquoise shallow sea water over pale sand, soft light ripples, seen from directly above"),
}
CONTRAST = 0.75           # keep detail but soften it so units stay readable
FLATTEN = 0.75            # share of large-scale blotchiness removed
OUT = "terrain"

def make(name):
    _, ground = BIOMES[name]
    if not os.path.exists(f"{OUT}/{name}_raw.png"):
        o = run("fal-ai/nano-banana/edit", {"prompt": STYLE + ground, "image_urls": REFS, "num_images": 1, "output_format": "png", "aspect_ratio": "1:1"}, 0.0398, "terrain:" + name)
        download(o["images"][0]["url"], f"{OUT}/{name}_raw.png")
    return finish(name)

def finish(name):
    """Deterministic post-process of the raw painting: crop, flatten, tile, recolour."""
    target, _ = BIOMES[name]
    raw = Image.open(f"{OUT}/{name}_raw.png").convert("RGB")
    w, h = raw.size; m = int(min(w, h) * 0.03)                  # models like to paint a darker border
    img = raw.crop((m, m, w - m, h - m)).resize((1024, 1024), Image.LANCZOS)
    a = np.asarray(img).astype(np.float32)
    # Flatten large blotches (they make the repeat visible); keep brush-scale detail.
    low = np.asarray(img.filter(ImageFilter.GaussianBlur(56))).astype(np.float32)
    a = a - FLATTEN * (low - low.reshape(-1, 3).mean(0))
    # Seamless: blend with the half-offset copy, whose seams sit where this copy is fully opaque.
    rolled = np.roll(np.roll(a, 512, 0), 512, 1)
    t = np.abs(np.linspace(-1, 1, 1024)); w1 = np.clip((1 - t) / 0.45, 0, 1); w1 = w1 * w1 * (3 - 2 * w1)
    wgt = np.minimum.outer(w1, w1)[..., None]
    tile = wgt * a + (1 - wgt) * rolled
    # Mean colour onto the biome colour; deviations softened.
    mean = tile.reshape(-1, 3).mean(0)
    tile = np.array(target, np.float32) + CONTRAST * (tile - mean)
    tile = Image.fromarray(np.clip(tile, 0, 255).astype(np.uint8))
    tile.save(f"{OUT}/{name}.png", optimize=True)
    tile.resize((512, 512), Image.LANCZOS).save(f"{OUT}/{name}.webp", quality=88, method=6)
    return name, [round(v) for v in np.asarray(tile).reshape(-1, 3).mean(0)]

if __name__ == "__main__":
    with ThreadPoolExecutor(3) as ex:
        for r in ex.map(make, sys.argv[1:]): print(r, flush=True)
