#!/usr/bin/env python3
"""Pack catalog building cutouts into buildings_<group>.{png,json}, same format as buildings_hd.

Usage: python cat_pack.py <source dir with <kind>_<stage>_cut.png> <dest dir>
Footprint corners [left, front, right, rear] are estimated per frame from the base of the
alpha mask (no hand placement): front = lowest opaque point, left/right = outermost opaque
points in the lower 40% of the sprite, rear = left + right - front (a parallelogram).
"""
import json, sys
from pathlib import Path
import numpy as np
from PIL import Image
from scipy import ndimage

CELL, BASE, FIT_H, FIT_W = 512, 496, 472, 488          # identical to scripts/pack_building_sprites.py
STAGES = ("foundation", "walls", "roof", "complete")
GROUPS = {
    "economy": ("mining_camp", "farm", "lumber_mill", "smelter"),
    "crafts": ("kiln", "weaver", "kitchen", "monument"),
    "civic": ("barracks", "range", "workshop", "infirmary"),
}

def cutout(path):
    px = np.array(Image.open(path).convert("RGBA"))
    solid = px[..., 3] > 40
    lab, n = ndimage.label(solid); sizes = ndimage.sum(solid, lab, range(1, n + 1))
    keep = np.isin(lab, 1 + np.flatnonzero(sizes > max(60, solid.sum() * 0.002)))
    px[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
    y, x = np.nonzero(px[..., 3] > 40)
    return Image.fromarray(px[y.min():y.max() + 1, x.min():x.max() + 1]), (int(x.min()), int(y.min()))

def footprint(image):
    """Corners in the cutout's own pixel space."""
    a = np.asarray(image)[..., 3] > 40
    h, w = a.shape
    ys, xs = np.nonzero(a)
    bottom = ys.max()
    front = (float(np.median(xs[ys >= bottom - 2])), float(bottom))
    low = ys >= h * 0.6
    lx, ly = xs[low], ys[low]
    li = np.argmin(lx); ri = np.argmax(lx)
    # among the outermost columns take the lowest point (the ground corner, not an overhang)
    left = (float(lx[li]), float(ly[lx == lx[li]].max()))
    right = (float(lx[ri]), float(ly[lx == lx[ri]].max()))
    rear = (left[0] + right[0] - front[0], left[1] + right[1] - front[1])
    return [left, front, right, rear]

def pack(src, dest, group, kinds):
    atlas = Image.new("RGBA", (CELL * 4, CELL * len(kinds)))
    frames, footprints = {}, {}
    for row, kind in enumerate(kinds):
        cuts = [cutout(src / f"{kind}_{s}_cut.png") for s in STAGES]
        imgs = [im for im, _ in cuts]
        scale = min(FIT_H / max(im.height for im in imgs), FIT_W / max(im.width for im in imgs))
        frames[kind], footprints[kind] = [], []
        for col, (im, _) in enumerate(cuts):
            w, h = round(im.width * scale), round(im.height * scale)
            off = ((CELL - w) // 2, BASE - h)
            footprints[kind].append([[round(x * w / im.width + off[0], 4), round(y * h / im.height + off[1], 4)]
                                     for x, y in footprint(im)])
            atlas.alpha_composite(im.resize((w, h), Image.Resampling.LANCZOS), (col * CELL + off[0], row * CELL + off[1]))
            frames[kind].append([col * CELL, row * CELL, CELL, CELL])
    name = f"buildings_{group}"
    atlas.save(dest / f"{name}.png", optimize=True)
    manifest = {"image": f"{name}.png", "size": list(atlas.size), "cell": [CELL, CELL], "anchor": [CELL // 2, BASE],
                "stages": list(STAGES), "frames": frames, "footprints": footprints,
                "note": "Same format as buildings_hd.json. Footprint corners [left, front, right, rear] are estimated automatically from each frame's base outline."}
    (dest / f"{name}.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return name, atlas.size

if __name__ == "__main__":
    src, dest = Path(sys.argv[1]), Path(sys.argv[2])
    for g, kinds in GROUPS.items(): print(pack(src, dest, g, kinds))
