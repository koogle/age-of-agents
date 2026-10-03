#!/usr/bin/env python3
"""Append dedicated FAL field stages to the economy atlas without changing its buildings."""
import json
from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'assets/sprites'
SOURCE = DEST / 'field_sources'
STAGES = ('cleared', 'tilled', 'seedlings', 'ripe')
CELL = 512
# All edits preserve the master's four soil corners, in source pixels.
# Back, right, front, left: use the ground contact, not the wheat silhouette.
CORNERS = ((512, 350), (995, 683), (511, 949), (29, 684))
# Shared source crop and downscale retain one registration across every stage.
CROP = (12, 0, 1012, 1000)


def main():
    atlas = Image.new('RGBA', (2048, 2560))
    original = Image.open(DEST / 'buildings_economy.png').convert('RGBA')
    atlas.paste(original.crop((0, 0, 2048, 2048)), (0, 0))
    for col, stage in enumerate(STAGES):
        pixels = np.array(Image.open(SOURCE / f'{stage}.png').convert('RGBA'))
        assert pixels.shape == (1024, 1024, 4), (stage, pixels.shape)
        # Strip only tiny segmentation noise, then extend edge RGB below alpha
        # so mip filtering cannot introduce white/black matte fringes.
        solid = pixels[..., 3] > 40
        labels, count = ndimage.label(solid)
        sizes = ndimage.sum(solid, labels, range(1, count + 1))
        keep = np.isin(labels, 1 + np.flatnonzero(sizes > 100))
        pixels[..., 3] *= ndimage.binary_dilation(keep, iterations=2)
        frame = Image.fromarray(pixels).crop(CROP).resize((CELL, CELL), Image.Resampling.LANCZOS)
        pixels = np.array(frame)
        transparent = pixels[..., 3] == 0
        nearest = ndimage.distance_transform_edt(transparent, return_distances=False, return_indices=True)
        pixels[transparent, :3] = pixels[nearest[0][transparent], nearest[1][transparent], :3]
        atlas.paste(Image.fromarray(pixels), (col * CELL, 2048))
    atlas.save(DEST / 'buildings_economy.png')
    manifest = json.loads((DEST / 'buildings_economy.json').read_text())
    manifest['size'] = list(atlas.size)
    manifest['frames']['field'] = [[col * CELL, 2048, CELL, CELL] for col in range(4)]
    corners = [[round((x - CROP[0]) * CELL / 1000, 4), round(y * CELL / 1000, 4)] for x, y in CORNERS]
    manifest['footprints']['field'] = [corners for _ in STAGES]
    manifest['fieldStages'] = list(STAGES)
    (DEST / 'buildings_economy.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print('Packed four 512px field stages; original 16 building cells preserved.')


if __name__ == '__main__':
    main()
