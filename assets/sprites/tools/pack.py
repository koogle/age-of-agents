"""Split cut-out strips into frames, normalize scale and feet anchor, pack a sprite sheet + JSON."""
import json, sys
import numpy as np
from PIL import Image
from scipy import ndimage

import os as _os
_K = int(_os.environ.get("HD_SCALE", "1"))      # 2 = same layout at twice the resolution (villager_idle_hd)
CELL, FOOT_Y, FIG_H = 256 * _K, 240 * _K, 176 * _K   # cell size, feet baseline, idle_front head-to-feet height
STRIPS = {  # name: (animation, facing, frames)
  "idle_front": ("idle", "front", 2), "idle_back": ("idle", "back", 2),
  "walk_front": ("walk", "front", 4), "walk_back": ("walk", "back", 4),
  "carry_front": ("carry", "front", 4), "carry_back": ("carry", "back", 4),
  "chop": ("chop", "front", 3), "mine": ("mine", "front", 3), "forage": ("forage", "front", 2),
  "dig": ("dig", "front", 3), "build": ("build", "front", 3),
}

def frames_of(path, n):
    im = Image.open(path).convert("RGBA"); a = np.asarray(im).astype(np.float32)
    alpha = a[..., 3]; solid = alpha > 40
    lab, k = ndimage.label(solid); sizes = ndimage.sum(solid, lab, range(1, k + 1))
    keep = np.isin(lab, 1 + np.nonzero(sizes > solid.sum() * 0.004)[0])   # drop specks (berries, dirt, ground lines)
    a[..., 3] *= ndimage.binary_dilation(keep, iterations=3)
    cols = keep.any(0); xs = np.nonzero(cols)[0]
    # column runs = frames; merge runs separated by tiny gaps, then pick the n widest
    runs, start = [], xs[0]
    for p, q in zip(xs, xs[1:]):
        if q - p > 6: runs.append((start, p + 1)); start = q
    runs.append((start, xs[-1] + 1))
    while len(runs) > n:   # merge the narrowest run into its nearest neighbour
        i = min(range(len(runs)), key=lambda j: runs[j][1] - runs[j][0])
        j = i - 1 if i == len(runs) - 1 or (i > 0 and runs[i][0] - runs[i - 1][1] < runs[i + 1][0] - runs[i][1]) else i + 1
        lo, hi = sorted((i, j)); runs[lo:hi + 1] = [(runs[lo][0], runs[hi][1])]
    if len(runs) < n: raise SystemExit(f"{path}: found {len(runs)} frames, expected {n}")
    out = []
    for x0, x1 in runs:
        sub = a[:, x0:x1]; rows = np.nonzero((sub[..., 3] > 40).any(1))[0]
        out.append((sub, rows[0], rows[-1] + 1))
    return out

def blue_area(sub):
    r, g, b, al = sub[..., 0], sub[..., 1], sub[..., 2], sub[..., 3]
    return ((b > r + 50) & (b > g + 20) & (b > 120) & (al > 128)).sum()

import os
PREFIX = os.environ.get("VARIANT", "") and os.environ["VARIANT"] + "_"
data = {k: frames_of(f"spr/{PREFIX}{k}_cut.png", v[2]) for k, v in STRIPS.items()}
ref = data["idle_front"]
H0 = np.median([y1 - y0 for _, y0, y1 in ref]); B0 = np.median([np.sqrt(blue_area(s)) for s, _, _ in ref])
sheet_cells, meta = [], {}
for name, frames in data.items():
    anim, facing, n = STRIPS[name]
    by_blue = (B0 / np.median([np.sqrt(blue_area(s)) for s, _, _ in frames])) * (FIG_H / H0)
    by_height = FIG_H / np.median([y1 - y0 for _, y0, y1 in frames])
    # Upright strips without raised tools scale by head-to-feet height (the scarf looks smaller from behind and
    # the sack hides it). Forage is a crouch at about 62% of standing height.
    # Each estimate fails differently (raised tools inflate height; a headscarf or the pose hides scarf area),
    # so tool strips use their geometric mean. TOOL_SCALE=height forces height (the woman's headscarf hides
    # most of her scarf).
    scale = {"idle": by_height, "walk": by_height, "carry": by_height, "forage": by_height * 0.62}.get(anim, by_height if os.environ.get("TOOL_SCALE") == "height" else (by_blue * by_height) ** 0.5)
    print(f"{name:12s} blue {by_blue:.3f} height {by_height:.3f}")
    ground = np.median([y1 for _, _, y1 in frames])          # shared ground line of the strip
    for sub, y0, y1 in frames:
        img = Image.fromarray(np.clip(sub, 0, 255).astype(np.uint8), "RGBA")
        al = sub[..., 3] > 40
        foot_rows = al[int(ground - 0.12 * (ground - y0)):int(ground)]
        fx = np.nonzero(foot_rows.any(0))[0]; cx = (fx.min() + fx.max()) / 2 if len(fx) else sub.shape[1] / 2
        w, h = int(round(sub.shape[1] * scale / _K)) * _K, int(round(sub.shape[0] * scale / _K)) * _K   # 2x the 1x size exactly
        img = img.resize((w, h), Image.LANCZOS)
        cell = Image.new("RGBA", (CELL, CELL))
        # Place at _K x the rounded 1x position so an HD frame lands exactly on 2x the shipped 1x frame.
        px = int(round(CELL / _K / 2 - cx * scale / _K)) * _K; py = int(round(FOOT_Y / _K - ground * scale / _K)) * _K
        cell.alpha_composite(img, (px, py))  # clips at the cell edge
        sheet_cells.append(cell)
        meta.setdefault(anim, {}).setdefault(facing, []).append(len(sheet_cells) - 1)
cols = 8; rows = -(-len(sheet_cells) // cols)
sheet = Image.new("RGBA", (cols * CELL, rows * CELL))
for i, c in enumerate(sheet_cells): sheet.alpha_composite(c, ((i % cols) * CELL, (i // cols) * CELL))
rect = lambda i: [(i % cols) * CELL, (i // cols) * CELL, CELL, CELL]
manifest = {"image": os.path.basename(sys.argv[1]) + ".png", "cell": [CELL, CELL], "anchor": [CELL // 2, FOOT_Y], "figureHeight": FIG_H,
            "facings": "front = three-quarter front (facing viewer-left), back = three-quarter back (facing viewer-right); mirror for the other two",
            "fps": {"idle": 2, "walk": 8, "carry": 7, "chop": 5, "mine": 5, "forage": 3, "dig": 5, "build": 6},
            "animations": {a: {f: [rect(i) for i in idx] for f, idx in fs.items()} for a, fs in meta.items()}}
sheet.save(sys.argv[1] + ".png", optimize=True)
json.dump(manifest, open(sys.argv[1] + ".json", "w"), indent=1)
print(sheet.size, len(sheet_cells), "frames")
