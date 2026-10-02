"""Pack town-center stages into one 512 px-cell billboard sheet + JSON.

The stages are edits of the complete temple and keep its framing, so they are
placed in shared raw-image coordinates; only the foundation, which the model
drew about 9% larger, is rescaled to the complete temple's width."""
import json, sys
import numpy as np
from PIL import Image
from scipy import ndimage

CELL, BASE_Y, MARGIN = 512, 470, 12
WORLD_WIDTH = 1.9                           # complete temple, full sprite width in world units
ORDER = ["foundation", "build33", "build66", "complete", "working"]

def load(k):
    a = np.asarray(Image.open(f"tc/{k}_cut.png").convert("RGBA")).astype(np.float32)
    solid = a[..., 3] > 40
    lab, n = ndimage.label(solid); sizes = ndimage.sum(solid, lab, range(1, n + 1))
    keep = np.isin(lab, 1 + np.nonzero(sizes > 60)[0])
    a[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
    return a

def bbox(a):
    ys, xs = np.nonzero(a[..., 3] > 40); return xs.min(), xs.max() + 1, ys.min(), ys.max() + 1

raw = {k: Image.fromarray(np.clip(load(k), 0, 255).astype(np.uint8), "RGBA") for k in ORDER}
cx0, cx1, _, cy1 = bbox(np.asarray(raw["complete"]).astype(np.float32))
fx0, fx1, _, fy1 = bbox(np.asarray(raw["foundation"]).astype(np.float32))
f = (cx1 - cx0) / (fx1 - fx0)
fc = ((fx0 + fx1) / 2, fy1)                  # scale the foundation about its bottom centre onto the complete one's
big = raw["foundation"].resize((round(raw["foundation"].width * f), round(raw["foundation"].height * f)), Image.LANCZOS)
fixed = Image.new("RGBA", raw["foundation"].size)
fixed.alpha_composite(big, (round((cx0 + cx1) / 2 - fc[0] * f), round(cy1 - fc[1] * f)))
raw["foundation"] = fixed
# One shared transform: union bbox -> cell, bottom of the complete temple on BASE_Y.
boxes = [bbox(np.asarray(raw[k]).astype(np.float32)) for k in ORDER]
ux0, ux1 = min(b[0] for b in boxes), max(b[1] for b in boxes); uy0 = min(b[2] for b in boxes)
s = min((CELL - 2 * MARGIN) / (ux1 - ux0), (BASE_Y - MARGIN) / (cy1 - uy0))
ox = CELL / 2 - (cx0 + cx1) / 2 * s; oy = BASE_Y - cy1 * s
cells = []
for k in ORDER:
    img = raw[k].resize((round(raw[k].width * s), round(raw[k].height * s)), Image.LANCZOS)
    cell = Image.new("RGBA", (CELL, CELL)); cell.alpha_composite(img, (round(ox), round(oy))) if ox >= 0 and oy >= 0 else None
    if ox < 0 or oy < 0:
        tmp = Image.new("RGBA", (CELL, CELL)); tmp.paste(img, (round(ox), round(oy)), img); cell = tmp
    cells.append(cell)
fa = np.asarray(cells[0])[..., 3] > 40; fy = np.nonzero(fa.any(1))[0]
anchor_y = int(round((fy.min() + fy.max()) / 2))              # middle of the flat foundation platform = footprint centre
cw = np.nonzero((np.asarray(cells[3])[..., 3] > 40).any(0))[0]; cw = cw.max() - cw.min() + 1
sheet = Image.new("RGBA", (CELL * len(cells), CELL))
for i, c in enumerate(cells): sheet.alpha_composite(c, (i * CELL, 0))
manifest = {"image": "towncenter.png", "size": list(sheet.size), "cell": [CELL, CELL],
  "anchor": [CELL // 2, anchor_y], "baseBottom": [CELL // 2, BASE_Y],
  "unitsPerPixel": round(WORLD_WIDTH / cw, 6),
  "note": "anchor = footprint centre on the ground (middle of the foundation platform); baseBottom = lowest point of the front step. All frames share one scale and anchor.",
  "frames": {k: [i * CELL, 0, CELL, CELL] for i, k in enumerate(ORDER)},
  "constructionStages": [["foundation", 0.0], ["build33", 0.15], ["build66", 0.5]]}
sheet.save(sys.argv[1] + ".png", optimize=True); json.dump(manifest, open(sys.argv[1] + ".json", "w"), indent=1)
print(sheet.size, "anchor", manifest["anchor"], "upp", manifest["unitsPerPixel"], "complete width px", cw, "foundation fix", round(f, 3))
