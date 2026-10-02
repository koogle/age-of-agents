"""Pack resource/tree strips into one billboard sheet + JSON (assets/sprites/resources.*)."""
import json, sys
import numpy as np
from PIL import Image
from scipy import ndimage

CELL, BASE_Y, FIT = 256, 248, 232      # cell size, base line, max content size inside a cell
# node: (strip, frame indices in the strip, world size, measured along "h"eight or "w"idth of the first frame)
NODES = {
  "cypress": ("trees", [0], 1.6, "h"), "olive": ("trees", [1], 1.15, "h"), "stump": ("trees", [2], 0.42, "w"),
  "berry": ("berry", [0, 1, 2], 0.62, "w"), "stone": ("stone", [0, 1, 2], 0.75, "w"), "gold": ("gold", [0, 1, 2], 0.62, "w"),
  "iron": ("iron", [0, 1, 2], 0.7, "w"), "clay": ("clay", [0, 1, 2], 0.85, "w"), "fiber": ("fiber", [0, 1, 2], 0.7, "h"),
}

def frames(strip):
    a = np.asarray(Image.open(f"res/{strip}_cut.png").convert("RGBA")).astype(np.float32)
    solid = a[..., 3] > 40
    # Drop thin horizontal ground lines but keep thin upright stalks: opening with a vertical element.
    keep = ndimage.binary_dilation(ndimage.binary_opening(solid, structure=np.ones((7, 1))), iterations=2) & solid
    lab, k = ndimage.label(keep); sizes = ndimage.sum(keep, lab, range(1, k + 1))
    keep = np.isin(lab, 1 + np.nonzero(sizes > keep.sum() * 0.002)[0])
    a[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
    cols = np.nonzero(keep.any(0))[0]
    runs, start = [], cols[0]
    for p, q in zip(cols, cols[1:]):
        if q - p > 12: runs.append((start, p + 1)); start = q
    runs.append((start, cols[-1] + 1))
    while len(runs) > 3:
        i = min(range(len(runs)), key=lambda j: runs[j][1] - runs[j][0])
        j = i - 1 if i == len(runs) - 1 or (i > 0 and runs[i][0] - runs[i - 1][1] < runs[i + 1][0] - runs[i][1]) else i + 1
        lo, hi = sorted((i, j)); runs[lo:hi + 1] = [(runs[lo][0], runs[hi][1])]
    if len(runs) != 3: raise SystemExit(f"{strip}: {len(runs)} frames")
    out = []
    for x0, x1 in runs:
        sub = a[:, x0:x1]; rows = np.nonzero((sub[..., 3] > 40).any(1))[0]
        out.append(sub[rows[0]:rows[-1] + 1])
    return out

cells, meta, cache = [], {}, {}
for node, (strip, idx, size, axis) in NODES.items():
    fr = cache.setdefault(strip, frames(strip))
    subs = [fr[i] for i in idx]
    scale = min(FIT / max(s.shape[0] for s in subs), FIT / max(s.shape[1] for s in subs))   # shared by all stages
    rects = []
    for s in subs:
        img = Image.fromarray(np.clip(s, 0, 255).astype(np.uint8), "RGBA")
        w, h = max(1, round(s.shape[1] * scale)), max(1, round(s.shape[0] * scale))
        cell = Image.new("RGBA", (CELL, CELL)); cell.alpha_composite(img.resize((w, h), Image.LANCZOS), ((CELL - w) // 2, BASE_Y - h))
        cells.append(cell); rects.append(len(cells) - 1)
    first = subs[0]; px = (first.shape[0] if axis == "h" else first.shape[1]) * scale
    meta[node] = {"stages": rects, "unitsPerPixel": round(size / px, 6), "worldSize": size, "measured": "height" if axis == "h" else "width"}
cols = 8; rows = -(-len(cells) // cols)
sheet = Image.new("RGBA", (cols * CELL, rows * CELL))
for i, c in enumerate(cells): sheet.alpha_composite(c, ((i % cols) * CELL, (i // cols) * CELL))
rect = lambda i: [(i % cols) * CELL, (i // cols) * CELL, CELL, CELL]
manifest = {"image": "resources.png", "size": list(sheet.size), "cell": [CELL, CELL], "anchor": [CELL // 2, BASE_Y],
            "note": "stages run full -> nearly empty; one billboard per node; world size = pixel size * unitsPerPixel (shared by all stages)",
            "nodes": {n: {"stages": [rect(i) for i in m["stages"]], "unitsPerPixel": m["unitsPerPixel"],
                          "worldSize": m["worldSize"], "measured": m["measured"]} for n, m in meta.items()}}
sheet.save(sys.argv[1] + ".png", optimize=True); json.dump(manifest, open(sys.argv[1] + ".json", "w"), indent=1)
print(sheet.size, len(cells))
