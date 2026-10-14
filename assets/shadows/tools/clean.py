#!/usr/bin/env python3
"""Turn a raw model mask into a shadow-only mask: drop the object's own pixels and the guide pole."""
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

sys.path.insert(0, str(Path(__file__).resolve().parent))
import generate as g


def clean(raw, sheet, rect, mirror):
    mask = np.array(raw.convert('L').resize((g.CANVAS, g.CANVAS)), float) / 255
    # The model's black is not exactly black; drop that floor so quads stay clear.
    mask = np.clip((mask - 0.2) / 0.6, 0, 1)
    alpha = np.zeros((g.CANVAS, g.CANVAS))
    a = np.array(g.frame_image(sheet, rect, mirror).split()[-1], float) / 255
    x, y = g.CELL_OFFSET
    alpha[y:y + g.CELL_PX, x:x + g.CELL_PX] = a
    mask[alpha > 0.5] = 0
    (bx, by), h, _ = g.POLE
    mask[by - h - 20:, : bx + h + 80] = 0
    image = Image.fromarray((mask.clip(0, 1) * 255).astype('uint8'))
    return image.filter(ImageFilter.GaussianBlur(3))


if __name__ == '__main__':
    key = sys.argv[1]
    job = next(j for j in g.jobs() if j[0] == key)
    raw = Image.open(g.WORK / (key.replace('/', '__') + '.png'))
    clean(raw, *job[1:]).save(sys.argv[2])
