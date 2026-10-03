#!/usr/bin/env python3
"""Pack unit strips with villager conventions; HD_SCALE=2 recovers 512px cells."""
import json, sys, os
import numpy as np
from PIL import Image
from scipy import ndimage

K = int(os.environ.get("HD_SCALE", "1"))
CELL, FOOT_Y, FIG_H, CART_W = 256 * K, 240 * K, 176 * K, 200 * K
SOURCE = os.environ.get("UNIT_SOURCE", "units")
UNITS = ["guard", "archer", "healer", "siege_cart"]
STRIPS = [("idle", "front", "idle_front", 2), ("idle", "back", "idle_back", 2), ("walk", "front", "walk_front", 4),
          ("walk", "back", "walk_back", 4), ("action", "front", "action", 3)]
SELECT = {("guard", "walk_back"): [0, 1, 2, 3]}            # the model drew five frames; the first four form the cycle
GROUND_LINE = {("guard", "walk_back")}                     # strip has a drawn ground line to remove
FPS = {"idle": 2, "walk": 8, "action": 5}

def frames_of(path, n, select=None, ground=False):
    a = np.asarray(Image.open(path).convert("RGBA")).astype(np.float32)
    solid = a[..., 3] > 40
    if ground:
        solid &= ndimage.binary_dilation(ndimage.binary_opening(solid, structure=np.ones((7, 1))), iterations=2)
    lab, k = ndimage.label(solid); sizes = ndimage.sum(solid, lab, range(1, k + 1))
    keep = np.isin(lab, 1 + np.nonzero(sizes > solid.sum() * 0.004)[0])
    a[..., 3] *= ndimage.binary_dilation(keep, iterations=3)
    xs = np.nonzero(keep.any(0))[0]
    runs, start = [], xs[0]
    for p, q in zip(xs, xs[1:]):
        if q - p > 6: runs.append([start, p + 1]); start = q
    runs.append([start, xs[-1] + 1])
    width = xs[-1] - xs[0]
    i = 0                                                   # merge slivers (separated arrow tips, sparks) into a neighbour
    while i < len(runs):
        if runs[i][1] - runs[i][0] < width * 0.04 and len(runs) > 1:
            j = i - 1 if i > 0 else i + 1
            lo, hi = sorted((i, j)); runs[lo:hi + 1] = [[runs[lo][0], runs[hi][1]]]; i = 0
        else: i += 1
    if select: runs = [runs[s] for s in select]
    while len(runs) > n:
        i = min(range(len(runs) - 1), key=lambda j: runs[j + 1][0] - runs[j][1])
        runs[i:i + 2] = [[runs[i][0], runs[i + 1][1]]]
    if len(runs) < n:                                       # overlapping props (spears): cut at the emptiest column near each boundary
        x0, x1 = xs[0], xs[-1] + 1; col = keep.sum(0).astype(float); cuts = [x0]
        for i in range(1, n):
            c = x0 + (x1 - x0) * i / n; lo, hi = int(c - (x1 - x0) / (3 * n)), int(c + (x1 - x0) / (3 * n))
            cuts.append(lo + int(np.argmin(col[lo:hi])))
        cuts.append(x1); runs = [[cuts[i], cuts[i + 1]] for i in range(n)]
    if len(runs) != n: raise SystemExit(f"{path}: {len(runs)} frames, expected {n}")
    out = []
    for x0, x1 in runs:
        sub = a[:, x0:x1].copy()
        if K > 1:
            # Splitting overlapping props can leave a tiny neighboring tip in
            # the cell. Remove isolated cut fragments after splitting, too.
            solid = sub[..., 3] > 40
            labels, count = ndimage.label(solid)
            areas = ndimage.sum(solid, labels, range(1, count + 1))
            keep = np.isin(labels, 1 + np.nonzero(areas > areas.max() * 0.001)[0])
            sub[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
        rows = np.nonzero((sub[..., 3] > 40).any(1))[0]
        out.append((sub, rows[0], rows[-1] + 1))
    return out

cells, units = [], {}
for u in UNITS:
    units[u] = {"figureHeight": FIG_H, "fps": FPS, "animations": {}}
    for anim, facing, strip, n in STRIPS:
        fr = frames_of(f"{SOURCE}/{u}_{strip}_cut.png", n, SELECT.get((u, strip)), (u, strip) in GROUND_LINE)
        if u == "siege_cart":
            scale = CART_W / np.median([s.shape[1] for s, _, _ in fr])
        else:
            scale = FIG_H / np.median([y1 - y0 for _, y0, y1 in fr])
        ground = np.median([y1 for _, _, y1 in fr])
        # Long spears must fit around the feet anchor, including action poses.
        # Bound one shared scale for the whole strip so animation size stays stable.
        if K > 1:
            scale = min(scale, 1.0)
            for sub, y0, y1 in fr:
                foot = (sub[..., 3] > 40)[int(ground - 0.12 * (ground - y0)):int(ground)]
                fx = np.nonzero(foot.any(0))[0]
                cx = (fx.min() + fx.max()) / 2 if len(fx) else sub.shape[1] / 2
                scale = min(scale, (CELL / 2 - 8) / max(cx, sub.shape[1] - cx),
                            (FOOT_Y - 8) / (ground - y0),
                            (CELL - FOOT_Y - 8) / max(1, y1 - ground))
        for sub, y0, y1 in fr:
            al = sub[..., 3] > 40
            foot = al[int(ground - 0.12 * (ground - y0)):int(ground)]
            fx = np.nonzero(foot.any(0))[0]; cx = (fx.min() + fx.max()) / 2 if len(fx) else sub.shape[1] / 2
            img = Image.fromarray(np.clip(sub, 0, 255).astype(np.uint8), "RGBA")
            img = img.resize((round(sub.shape[1] * scale), round(sub.shape[0] * scale)), Image.LANCZOS)
            c = Image.new("RGBA", (CELL, CELL))
            c.alpha_composite(img, (round(CELL / 2 - cx * scale), round(FOOT_Y - ground * scale)))
            cells.append(c)
            units[u]["animations"].setdefault(anim, {}).setdefault(facing, []).append(len(cells) - 1)
cols = 8; rows = -(-len(cells) // cols)
sheet = Image.new("RGBA", (cols * CELL, rows * CELL))
for i, c in enumerate(cells): sheet.alpha_composite(c, ((i % cols) * CELL, (i // cols) * CELL))
rect = lambda i: [(i % cols) * CELL, (i // cols) * CELL, CELL, CELL]
for u in units:
    units[u]["animations"] = {a: {f: [rect(i) for i in idx] for f, idx in fs.items()} for a, fs in units[u]["animations"].items()}
manifest = {"image": "units.png", "size": list(sheet.size), "cell": [CELL, CELL], "anchor": [CELL // 2, FOOT_Y],
  "facings": "front = three-quarter front (facing viewer-left), back = three-quarter back (facing viewer-right); mirror for the other two",
  "note": f"Per unit, the same shape as villager.json (figureHeight, fps, animations.<anim>.<facing> = cell rects). The siege cart is scaled to a {CART_W} px-wide body instead of a figure height.",
  "units": units}
sheet.save(sys.argv[1] + ".png", optimize=True); json.dump(manifest, open(sys.argv[1] + ".json", "w"), indent=1)
print(sheet.size, len(cells), "frames")
