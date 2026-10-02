"""Pack resource variants into resources_variants.{png,json} (same conventions as resources.json)."""
import json, sys
import numpy as np
from PIL import Image
from scipy import ndimage

CELL, BASE_Y, FIT = 256, 248, 232
# node -> (world size, measured axis) — identical to tools/res_pack.py so variants match the base nodes
SIZE = {"cypress": (1.6, "h"), "olive": (1.15, "h"), "berry": (0.62, "w"), "stone": (0.75, "w"), "gold": (0.62, "w"),
        "iron": (0.7, "w"), "clay": (0.85, "w"), "fiber": (0.7, "h")}
# strip file -> (node, frames per variant, number of variants in the strip)
STRIPS = {"cypress": ("cypress", 1, 4), "olive": ("olive", 1, 3)}
for n in ["berry", "stone", "gold", "iron", "clay", "fiber"]:
    for v in (1, 2): STRIPS[f"{n}{v}"] = (n, 3, 1)

def frames(path, count):
    a = np.asarray(Image.open(path).convert("RGBA")).astype(np.float32)
    solid = a[..., 3] > 40
    keep = ndimage.binary_dilation(ndimage.binary_opening(solid, structure=np.ones((7, 1))), iterations=2) & solid
    lab, k = ndimage.label(keep); sizes = ndimage.sum(keep, lab, range(1, k + 1))
    keep = np.isin(lab, 1 + np.nonzero(sizes > keep.sum() * 0.002)[0])
    a[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
    cols = np.nonzero(keep.any(0))[0]
    runs, start = [], cols[0]
    for p, q in zip(cols, cols[1:]):
        if q - p > 12: runs.append((start, p + 1)); start = q
    runs.append((start, cols[-1] + 1))
    while len(runs) > count:
        i = min(range(len(runs)), key=lambda j: runs[j][1] - runs[j][0])
        j = i - 1 if i == len(runs) - 1 or (i > 0 and runs[i][0] - runs[i - 1][1] < runs[i + 1][0] - runs[i][1]) else i + 1
        lo, hi = sorted((i, j)); runs[lo:hi + 1] = [(runs[lo][0], runs[hi][1])]
    if len(runs) != count: raise SystemExit(f"{path}: {len(runs)} frames, expected {count}")
    out = []
    for x0, x1 in runs:
        sub = a[:, x0:x1]; rows = np.nonzero((sub[..., 3] > 40).any(1))[0]
        out.append(sub[rows[0]:rows[-1] + 1])
    return out

cells, nodes = [], {}
for strip, (node, per, nvar) in STRIPS.items():
    fr = frames(f"var/{strip}_cut.png", per * nvar)
    for v in range(nvar):
        subs = fr[v * per:(v + 1) * per]
        scale = min(FIT / max(s.shape[0] for s in subs), FIT / max(s.shape[1] for s in subs))
        idx = []
        for s in subs:
            img = Image.fromarray(np.clip(s, 0, 255).astype(np.uint8), "RGBA")
            w, h = max(1, round(s.shape[1] * scale)), max(1, round(s.shape[0] * scale))
            c = Image.new("RGBA", (CELL, CELL)); c.alpha_composite(img.resize((w, h), Image.LANCZOS), ((CELL - w) // 2, BASE_Y - h))
            cells.append(c); idx.append(len(cells) - 1)
        size, axis = SIZE[node]; first = subs[0]
        px = (first.shape[0] if axis == "h" else first.shape[1]) * scale
        nodes.setdefault(node, []).append({"stages": idx, "unitsPerPixel": round(size / px, 6)})
cols = 8; rows = -(-len(cells) // cols)
sheet = Image.new("RGBA", (cols * CELL, rows * CELL))
for i, c in enumerate(cells): sheet.alpha_composite(c, ((i % cols) * CELL, (i // cols) * CELL))
rect = lambda i: [(i % cols) * CELL, (i // cols) * CELL, CELL, CELL]
manifest = {"image": "resources_variants.png", "size": list(sheet.size), "cell": [CELL, CELL], "anchor": [CELL // 2, BASE_Y],
  "note": "extra individuals per node, same conventions as resources.json (stages full -> nearly empty, base anchor, per-variant unitsPerPixel); pick one per node by id hash, alongside the base node in resources.json",
  "nodes": {n: [{"stages": [rect(i) for i in v["stages"]], "unitsPerPixel": v["unitsPerPixel"]} for v in vs] for n, vs in nodes.items()}}
sheet.save(sys.argv[1] + ".png", optimize=True); json.dump(manifest, open(sys.argv[1] + ".json", "w"), indent=1)
print(sheet.size, len(cells), {n: len(v) for n, v in nodes.items()})
