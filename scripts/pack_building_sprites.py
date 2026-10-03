#!/usr/bin/env python3
"""Pack reviewed source cutouts into a lossless gameplay atlas (512 px cells)."""
import json
from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/sprites/building_sources"
DEST = ROOT / "assets/sprites"
CELL, BASE, FIT_H, FIT_W = 512, 496, 472, 488
KINDS = ("house", "granary", "watchtower", "dock")
STAGES = ("foundation", "walls", "roof", "complete")


def cutout(image):
    pixels = np.array(image.convert("RGBA"))
    solid = pixels[..., 3] > 40
    labels, count = ndimage.label(solid)
    sizes = ndimage.sum(solid, labels, range(1, count + 1))
    keep = np.isin(labels, 1 + np.flatnonzero(sizes > max(60, solid.sum() * 0.002)))
    pixels[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
    y, x = np.nonzero(pixels[..., 3] > 40)
    return Image.fromarray(pixels[y.min():y.max() + 1, x.min():x.max() + 1]), (int(x.min()), int(y.min()))


def main():
    atlas = Image.new("RGBA", (CELL * 4, CELL * len(KINDS)))
    frames = {}
    footprints = {}
    source_corners = json.loads((SOURCE / "footprints.json").read_text())["footprints"]
    corrected = Image.open(SOURCE / "orthogonal_house_granary.png")
    for row, kind in enumerate(KINDS):
        if kind in ("house", "granary"):
            # Fractional sheet dimensions: round shared edges once, with no
            # dropped pixels or neighboring-frame bleed.
            cutouts = [cutout(corrected.crop((
                round(col * corrected.width / 4), round(row * corrected.height / 2),
                round((col + 1) * corrected.width / 4), round((row + 1) * corrected.height / 2),
            ))) for col in range(4)]
        else:
            cutouts = [cutout(Image.open(SOURCE / f"{kind}_{stage}.png")) for stage in STAGES]
        images = [im for im, _ in cutouts]
        scale = min(FIT_H / max(im.height for im in images), FIT_W / max(im.width for im in images))
        frames[kind] = []
        footprints[kind] = []
        for col, (image, origin) in enumerate(cutouts):
            width, height = round(image.width * scale), round(image.height * scale)
            offset = ((CELL - width) // 2, BASE - height)
            footprints[kind].append([
                [round((x - origin[0]) * width / image.width + offset[0], 4),
                 round((y - origin[1]) * height / image.height + offset[1], 4)]
                for x, y in source_corners[kind][col]
            ])
            # Keep one scale and baseline per building, without warping the art.
            image = image.resize((width, height), Image.Resampling.LANCZOS)
            atlas.alpha_composite(image, (col * CELL + offset[0], row * CELL + offset[1]))
            frames[kind].append([col * CELL, row * CELL, CELL, CELL])
    atlas.save(DEST / "buildings_hd.png")
    manifest = {"image": "buildings_hd.png", "size": list(atlas.size), "cell": [CELL, CELL],
                "anchor": [CELL // 2, BASE], "stages": list(STAGES), "frames": frames,
                "footprints": footprints}
    (DEST / "buildings_hd.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Packed {len(KINDS) * len(STAGES)} cutouts into {atlas.size}, lossless RGBA PNG")


if __name__ == "__main__":
    main()
