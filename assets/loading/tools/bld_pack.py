"""Pack loading-screen building stages into buildings.webp + buildings.json (bottom-centre anchored cells)."""
import json, os, sys
import numpy as np
from PIL import Image
from scipy import ndimage
CELL, BASE_Y, FIT_H, FIT_W = 256, 248, 236, 244
ORDER = ["towncenter", "house", "granary", "watchtower", "dock"]
STAGES = {"towncenter": ["foundation", "build33", "build66", "complete"]}
def src(b, s): return f"tc/{s}_cut.png" if b == "towncenter" else f"bld/{b}_{s}_cut.png"
def load(path):
    a = np.asarray(Image.open(path).convert("RGBA")).astype(np.float32)
    solid = a[..., 3] > 40
    lab, n = ndimage.label(solid); sizes = ndimage.sum(solid, lab, range(1, n + 1))
    keep = np.isin(lab, 1 + np.nonzero(sizes > max(60, solid.sum() * 0.002))[0])
    a[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
    ys, xs = np.nonzero(a[..., 3] > 40)
    return a[ys.min():ys.max() + 1, xs.min():xs.max() + 1]
cells, frames = [], {}
for b in ORDER:
    names = STAGES.get(b, ["foundation", "walls", "roof", "complete"])
    subs = [load(src(b, s)) for s in names]
    # one scale per building so stages keep their relative size; the largest stage fits the cell
    scale = min(FIT_H / max(s.shape[0] for s in subs), FIT_W / max(s.shape[1] for s in subs))
    frames[b] = []
    for s in subs:
        img = Image.fromarray(np.clip(s, 0, 255).astype(np.uint8), "RGBA")
        w, h = round(s.shape[1] * scale), round(s.shape[0] * scale)
        c = Image.new("RGBA", (CELL, CELL)); c.alpha_composite(img.resize((w, h), Image.LANCZOS), ((CELL - w) // 2, BASE_Y - h))
        cells.append(c); frames[b].append(len(cells) - 1)
cols = 4; rows = len(ORDER)
sheet = Image.new("RGBA", (cols * CELL, rows * CELL))
for i, c in enumerate(cells): sheet.alpha_composite(c, ((i % cols) * CELL, (i // cols) * CELL))
rect = lambda i: [(i % cols) * CELL, (i // cols) * CELL, CELL, CELL]
manifest = {"image": "buildings.webp", "size": list(sheet.size), "cell": [CELL, CELL], "anchor": [CELL // 2, BASE_Y],
  "stages": ["foundation", "walls", "roof", "complete"],
  "note": "frames.<building> = 4 cell rects in stage order (foundation, walls/scaffolding, roof going on, complete); anchor = bottom centre of the building in every cell; cells are drawn for display at about 240 px tall",
  "frames": {b: [rect(i) for i in idx] for b, idx in frames.items()}}
sheet.save(sys.argv[1] + ".png"); sheet.save(sys.argv[1] + ".webp", quality=int(os.environ.get("Q", 82)), method=6, alpha_quality=90)
json.dump(manifest, open(sys.argv[1] + ".json", "w"), indent=1)
print(sheet.size, os.path.getsize(sys.argv[1] + ".webp"))
