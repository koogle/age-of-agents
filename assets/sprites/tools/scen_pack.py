"""Pack scenery and effect billboards into scenery.{png,json} with explicit rects, anchors and suggested world sizes."""
import json, sys
import numpy as np
from PIL import Image
from scipy import ndimage

def split(path, count, min_gap=12):
    a = np.asarray(Image.open(path).convert("RGBA")).astype(np.float32)
    solid = a[..., 3] > 40
    lab, k = ndimage.label(solid); sizes = ndimage.sum(solid, lab, range(1, k + 1))
    keep = np.isin(lab, 1 + np.nonzero(sizes > 25)[0]); a[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
    cols = np.nonzero(keep.any(0))[0]; runs, start = [], cols[0]
    for p, q in zip(cols, cols[1:]):
        if q - p > min_gap: runs.append((start, p + 1)); start = q
    runs.append((start, cols[-1] + 1))
    while len(runs) > count:
        i = min(range(len(runs) - 1), key=lambda j: runs[j + 1][0] - runs[j][1])     # merge across the smallest gap
        runs[i:i + 2] = [(runs[i][0], runs[i + 1][1])]
    if len(runs) != count: raise SystemExit(f"{path}: {len(runs)} parts, expected {count}")
    out = []
    for x0, x1 in runs:
        sub = a[:, x0:x1]; rows = np.nonzero((sub[..., 3] > 40).any(1))[0]; out.append(sub[rows[0]:rows[-1] + 1])
    return out

def trim_stem(a):
    """Drop a narrow stem under a puff: cut where the row width falls below 45% of the widest row in the lower half."""
    al = a[..., 3] > 40; widths = al.sum(1); h = len(widths); peak = widths.max()
    for y in range(int(h * 0.5), h):
        if widths[y] < 0.45 * peak: return a[:y]
    return a

# name -> (strip, index, target px (w,h) box, anchor kind, suggested world width)
ITEMS = []
ITEMS += [("volcano", "volcano", 0, 1, (1024, 512), "bottom", 24.0)]
ITEMS += [(f"ship_{n}", "ships", i, 3, (256, 256), "bottom", w) for i, (n, w) in enumerate([("small", 1.0), ("merchant", 1.5), ("striped", 1.3)])]
ITEMS += [(f"cloud_{i + 1}", "clouds", i, 4, (512, 256), "center", w) for i, w in enumerate([5.0, 2.5, 5.5, 2.5])]
ITEMS += [(n, "fx", i, 4, (128, 128), "center", w) for i, (n, w) in enumerate([("fx_wood_chips", 0.3), ("fx_stone_dust", 0.3), ("fx_berry_leaves", 0.25), ("fx_smoke_puff", 0.4)])]
cache, sprites = {}, []
for name, strip, i, n, (bw, bh), kind, world in ITEMS:
    parts = cache.setdefault(strip, split(f"{'scen'}/{strip}_cut.png", n))
    a = parts[i]
    if name == "fx_smoke_puff": a = trim_stem(a)
    img = Image.fromarray(np.clip(a, 0, 255).astype(np.uint8), "RGBA")
    s = min((bw - 8) / img.width, (bh - 8) / img.height)
    img = img.resize((max(1, round(img.width * s)), max(1, round(img.height * s))), Image.LANCZOS)
    sprites.append((name, img, kind, world))
# Shelf packing, 2048 wide.
W, x, y, shelf, placed = 2048, 0, 0, 0, []
for name, img, kind, world in sorted(sprites, key=lambda t: -t[1].height):
    if x + img.width > W: x, y, shelf = 0, y + shelf + 4, 0
    placed.append((name, img, kind, world, x, y)); x += img.width + 4; shelf = max(shelf, img.height)
H = y + shelf
sheet = Image.new("RGBA", (W, H)); meta = {}
for name, img, kind, world, px, py in placed:
    sheet.alpha_composite(img, (px, py))
    anchor = [img.width / 2, img.height] if kind == "bottom" else [img.width / 2, img.height / 2]
    meta[name] = {"rect": [px, py, img.width, img.height], "anchor": [round(v, 1) for v in anchor], "anchorKind": kind,
                  "worldWidth": world, "unitsPerPixel": round(world / img.width, 6)}
order = [it[0] for it in ITEMS]
manifest = {"image": "scenery.png", "size": [W, H],
  "note": "rect = [x,y,w,h] in the sheet; anchor in sprite pixels (bottom = base/waterline centre, center = sprite centre); worldWidth is a suggestion, unitsPerPixel = worldWidth / rect width",
  "sprites": {n: meta[n] for n in order}}
sheet.save(sys.argv[1] + ".png", optimize=True); json.dump(manifest, open(sys.argv[1] + ".json", "w"), indent=1)
print(sheet.size, {n: meta[n]["rect"][2:] for n in order})
