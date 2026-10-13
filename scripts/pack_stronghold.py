#!/usr/bin/env python3
"""Pack the barbarian stronghold into row 1 of assets/sprites/buildings_sanctuary.{png,json}.

The temple row (row 0) is kept as packed. The stronghold is never built, so all
four stage slots hold the same frame. Usage: python3 scripts/pack_stronghold.py
"""
import json, sys
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "assets/sprites/tools"))
from cat_pack import BASE, CELL, FIT_H, FIT_W, cutout, footprint  # noqa: E402

SPRITES = ROOT / "assets/sprites"
SOURCE = SPRITES / "stronghold_sources/cutout.png"

manifest = json.loads((SPRITES / "buildings_sanctuary.json").read_text())
old = Image.open(SPRITES / "buildings_sanctuary.png").convert("RGBA")
atlas = Image.new("RGBA", (CELL * 4, CELL * 2))
atlas.alpha_composite(old.crop((0, 0, CELL * 4, CELL)), (0, 0))
im, _ = cutout(SOURCE)
scale = min(FIT_H / im.height, FIT_W / im.width)
w, h = round(im.width * scale), round(im.height * scale)
off = ((CELL - w) // 2, BASE - h)
frame = im.resize((w, h), Image.Resampling.LANCZOS)
corners = [[round(x * w / im.width + off[0], 4), round(y * h / im.height + off[1], 4)] for x, y in footprint(im)]
for col in range(4):
    atlas.alpha_composite(frame, (col * CELL + off[0], CELL + off[1]))
manifest["size"] = list(atlas.size)
manifest["frames"]["stronghold"] = [[col * CELL, CELL, CELL, CELL] for col in range(4)]
manifest["footprints"]["stronghold"] = [corners] * 4
atlas.save(SPRITES / "buildings_sanctuary.png", optimize=True)
(SPRITES / "buildings_sanctuary.json").write_text(json.dumps(manifest, indent=2) + "\n")
print(atlas.size, corners)
