#!/usr/bin/env python3
"""Pack generated wood/thatch frames against retained masonry atlas registration.

Generated sources remain untouched. This only trims near-transparent cutout noise,
splits the authored two-frame strips and packs down into existing 512px cells.
No recoloring, roof synthesis or enlargement of source detail is performed.
"""
import json
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SPRITES = ROOT / 'assets/sprites'
SOURCE = SPRITES / 'material_sources'
GROUPS = {
    'buildings_hd': ['house', 'granary', 'watchtower', 'dock', 'dock_east', 'dock_north', 'dock_west'],
    'buildings_economy': ['mining_camp', 'farm', 'lumber_mill', 'smelter'],
    'buildings_crafts': ['kiln', 'weaver', 'kitchen'],
    'buildings_civic': ['barracks', 'range', 'workshop', 'infirmary'],
}


def clean(image):
    pixels = np.array(image.convert('RGBA'))
    # Generated PNGs can carry saturated RGB under alpha 1/2. Discard that
    # invisible fringe before downsampling to avoid bleeding into opaque edges.
    pixels[pixels[..., 3] <= 8] = 0
    return Image.fromarray(pixels)


def pair(name):
    image = clean(Image.open(SOURCE / f'{name}.png'))
    width = image.width // 2
    if min(width, image.height) < 512:
        raise ValueError(f'{name}: source cells below 512px; regenerate, do not upscale')
    return [image.crop((i * width, 0, (i + 1) * width, image.height)) for i in range(2)]


def pack_frame(atlas, reference, rect, source):
    x, y, width, height = rect
    old = clean(reference.crop((x, y, x + width, y + height)))
    bounds = old.getchannel('A').point(lambda a: 255 if a > 40 else 0).getbbox()
    src_bounds = source.getchannel('A').point(lambda a: 255 if a > 40 else 0).getbbox()
    if not bounds or not src_bounds:
        raise ValueError('empty frame')
    painted = source.crop(src_bounds)
    left, top, right, bottom = bounds
    scale = min((right - left) / painted.width, (bottom - top) / painted.height)
    if scale > 1:
        raise ValueError('source detail would be enlarged')
    size = (round(painted.width * scale), round(painted.height * scale))
    packed = painted.resize(size, Image.Resampling.LANCZOS)
    atlas.paste((0, 0, 0, 0), (x, y, x + width, y + height))
    atlas.alpha_composite(packed, (x + round((left + right - size[0]) / 2), y + bottom - size[1]))


def main():
    for sheet, kinds in GROUPS.items():
        reference_path = SOURCE / 'buildings_crafts_original.png' if sheet == 'buildings_crafts' else SPRITES / f'{sheet}_masonry.png'
        reference = Image.open(reference_path).convert('RGBA')
        atlas = reference.copy()
        manifest = json.loads((SPRITES / f'{sheet}.json').read_text())
        for kind in kinds:
            frames = manifest['frames'][kind]
            stages = pair('kiln_late' if kind == 'kiln' else f'{kind}_pair')
            if kind == 'house':
                stages[1] = clean(Image.open(SOURCE / 'house_complete.png'))
            for stage, source in zip([2, 3], stages):
                pack_frame(atlas, reference, frames[stage], source)
            if kind == 'weaver':
                pack_frame(atlas, reference, frames[1], clean(Image.open(SOURCE / 'weaver_walls.png')))
            if kind == 'kiln':
                for stage, source in enumerate(pair('kiln_early')):
                    pack_frame(atlas, reference, frames[stage], source)
        atlas.save(SPRITES / f'{sheet}.png')
        if sheet == 'buildings_crafts':
            masonry = reference.copy()
            pack_frame(masonry, reference, manifest['frames']['monument'][3],
                       clean(Image.open(SOURCE / 'monument_masonry.png')))
            masonry.save(SPRITES / f'{sheet}_masonry.png')
    reference = Image.open(SPRITES / 'towncenter_masonry.png').convert('RGBA')
    atlas = reference.copy()
    manifest = json.loads((SPRITES / 'towncenter.json').read_text())
    for name, source in zip(['complete', 'working'], pair('towncenter_complete_pair')):
        pack_frame(atlas, reference, manifest['frames'][name], source)
    atlas.save(SPRITES / 'towncenter.png')
    print('Packed starter materials; original masonry atlases and manifest coordinates retained.')


if __name__ == '__main__':
    main()
