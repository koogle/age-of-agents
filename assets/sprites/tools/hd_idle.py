"""Assemble villager_idle_hd.{png,json}: idle frames of the three people at 2x, layout matching villager.json scaled by 2."""
import json, sys
import numpy as np
from PIL import Image
R = "/home/user/age-of-agents/assets/sprites"
PEOPLE = ["villager", "villager_woman", "villager_elder"]
COLS = [("front", 0), ("front", 1), ("back", 0), ("back", 1)]
base = json.load(open(f"{R}/villager.json"))
CELL = base["cell"][0] * 2
sheet = Image.new("RGBA", (CELL * 4, CELL * 3)); rects = {}
report = []
for r, who in enumerate(PEOPLE):
    hd_m = json.load(open(f"hd/{who}.json")); hd = Image.open(f"hd/{who}.png")
    lo_m = json.load(open(f"{R}/{who}.json")); lo = Image.open(f"{R}/{who}.png")
    rects[who] = {"front": [], "back": []}
    for c, (facing, i) in enumerate(COLS):
        x, y, w, h = hd_m["animations"]["idle"][facing][i]
        cell = hd.crop((x, y, x + w, y + h)); sheet.alpha_composite(cell, (c * CELL, r * CELL))
        rects[who][facing].append([c * CELL, r * CELL, CELL, CELL])
        # check: the HD frame downsampled 2x must match the shipped 1x frame
        lx, ly, lw, lh = lo_m["animations"]["idle"][facing][i]
        a = np.asarray(cell.resize((lw, lh), Image.LANCZOS)).astype(np.float32)
        b = np.asarray(lo.crop((lx, ly, lx + lw, ly + lh))).astype(np.float32)
        m = (a[..., 3] > 128) | (b[..., 3] > 128)
        report.append((who, facing, i, round(float(np.abs(a[..., 3] - b[..., 3]).mean()), 2), round(float(np.abs(a[..., :3] - b[..., :3])[m].mean()), 2)))
manifest = {"image": "villager_idle_hd.png", "size": list(sheet.size), "cell": [CELL, CELL],
  "anchor": [base["anchor"][0] * 2, base["anchor"][1] * 2], "figureHeight": base["figureHeight"] * 2,
  "note": "idle frames of villager, villager_woman, villager_elder at exactly 2x villager.json (same anchor pixel after scaling); rows = people, columns = idle front 1, front 2, back 1, back 2",
  "people": rects}
sheet.save(sys.argv[1] + ".png", optimize=True); json.dump(manifest, open(sys.argv[1] + ".json", "w"), indent=1)
print(sheet.size, manifest["anchor"], manifest["figureHeight"])
for row in report: print("  %-15s %-5s %d  alpha diff %.2f  rgb diff %.2f" % row)
