#!/usr/bin/env python3
"""Pack cleaned shadow masks into assets/sprites/shadows.png and map sprite frames to them.

Four masks share each 256 px cell, one per colour channel. Values are stored
sRGB-encoded because the client samples sheets as sRGB; sampling returns coverage.
Every drawn frame of a character points at the shadow of its idle pose for the
same facing, so animation never makes the shadow pop.
"""
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
import clean as c
import generate as g

CELL = 256
COLUMNS = 8
OUT = g.SPRITES / 'shadows'
# Sheet indices, in the order the client uploads them (`sheet_images` in lib.rs).
SHEETS = {
    'villager': 0, 'villager_woman': 1, 'villager_elder': 2, 'towncenter': 4, 'villager_idle_hd': 5,
    'buildings_hd': 6, 'buildings_economy': 7, 'buildings_crafts': 8, 'buildings_civic': 9,
    'units': 10, 'villager_field_preparation': 12, 'wildlife': 13, 'buildings_sanctuary': 14,
}
PEOPLE = ['villager', 'villager_woman', 'villager_elder']


def srgb(linear):
    return np.where(linear <= 0.0031308, linear * 12.92, 1.055 * np.power(linear, 1 / 2.4) - 0.055)


def flat(key):
    """Foundation plots and crop fields lie on the ground and cast no shadow."""
    return key.startswith('building/field/') or key.endswith('/0') or '/0/' in key \
        or key == 'building/towncenter/foundation'


def frame_entries():
    """(sheet name, rect, uv mirror, shadow key) for every frame the client draws."""
    out = []
    for name in ['buildings_hd', 'buildings_economy', 'buildings_crafts', 'buildings_civic', 'buildings_sanctuary']:
        for row, stages in g.load(name)['frames'].items():
            for stage, rect in enumerate(stages):
                out.append((name, rect, False, f'building/{row}/{stage}'))
                if row in ('dock', 'dock_north'):
                    out.append((name, rect, True, f'building/{row}/{stage}/mirror'))
    for frame, rect in g.load('towncenter')['frames'].items():
        out.append(('towncenter', rect, False, f'building/towncenter/{frame}'))

    def views(sheet, person, frames_by_view, mirrored=lambda view, i: False):
        for view, frames in frames_by_view.items():
            for i, rect in enumerate(frames):
                for uv_mirror in (False, True):
                    # A frame stored mirrored faces the other way for the same uv flag.
                    facing = uv_mirror ^ mirrored(view, i)
                    out.append((sheet, rect, uv_mirror, f'npc/{person}/{view}' + ('/mirror' if facing else '')))

    villager = g.load('villager')
    for variant, person in enumerate(PEOPLE):
        for anim, frames_by_view in villager['animations'].items():
            mirrors = villager.get('frameMirrors', {}).get(anim, {})
            views(person, person, frames_by_view,
                  lambda view, i: i in (mirrors.get(view) or [[], [], []])[variant])
    for person, frames_by_view in g.load('villager_idle_hd')['people'].items():
        views('villager_idle_hd', person, frames_by_view)
    for person, frames_by_view in g.load('villager_field_preparation')['people'].items():
        views('villager_field_preparation', person, {'front': frames_by_view['front']})
    for unit, spec in g.load('units')['units'].items():
        for frames_by_view in spec['animations'].values():
            views('units', unit, frames_by_view)
    for frame, rect in g.load('wildlife')['frames'].items():
        species = next(s for s in species_names() if frame.startswith(s + '_'))
        for uv_mirror in (False, True):
            out.append(('wildlife', rect, uv_mirror, f'npc/{species}' + ('/mirror' if uv_mirror else '')))
    return out


def species_names():
    # Longest first so "cyclops_general" wins over "cyclops".
    names = [f[:-len('_idle')] for f in g.load('wildlife')['frames'] if f.endswith('_idle')]
    return sorted(names, key=len, reverse=True)


def main():
    jobs = {j[0]: j for j in g.jobs()}
    ready = [k for k in jobs if not flat(k) and (g.WORK / (k.replace('/', '__') + '.png')).exists()]
    slots = {}
    cells = (len(ready) + 3) // 4
    rows = max(1, (cells + COLUMNS - 1) // COLUMNS)
    atlas = np.zeros((rows * CELL, COLUMNS * CELL, 4), float)
    for i, key in enumerate(ready):
        _, sheet, rect, mirror = jobs[key]
        raw = Image.open(g.WORK / (key.replace('/', '__') + '.png'))
        mask = np.array(c.clean(raw, sheet, rect, mirror).resize((CELL, CELL), Image.LANCZOS), float) / 255
        cell, channel = divmod(i, 4)
        x, y = cell % COLUMNS * CELL, cell // COLUMNS * CELL
        atlas[y:y + CELL, x:x + CELL, channel] = mask
        slots[key] = {'rect': [x, y, CELL, CELL], 'channel': channel}
    # Only colour channels are sRGB-decoded when sampled; alpha stays linear.
    atlas[..., :3] = srgb(atlas[..., :3].clip(0, 1))
    image = (atlas.clip(0, 1) * 255).round().astype('uint8')
    Image.fromarray(image, 'RGBA').save(str(OUT) + '.png', optimize=True)
    sizes = {}
    entries = []
    missing = set()
    for sheet, rect, uv_mirror, key in frame_entries():
        if key not in slots:
            if not flat(key):
                missing.add(key)
            continue
        if sheet not in sizes:
            meta = g.load(sheet)
            sizes[sheet] = meta['size']
        entries.append({'sheet': SHEETS[sheet], 'rect': rect, 'mirror': uv_mirror, 'shadow': key})
    manifest = {
        'note': 'Generated ground-shadow masks (assets/shadows/README.md). A mask covers twice its '
                'sprite quad; the sprite cell sits at cellOffset (fractions of the mask, from the top-left).',
        'size': [COLUMNS * CELL, rows * CELL],
        'cellOffset': [g.CELL_OFFSET[0] / g.CANVAS, g.CELL_OFFSET[1] / g.CANVAS],
        'sheetSizes': {str(SHEETS[s]): v for s, v in sizes.items()},
        'shadows': slots,
        'frames': entries,
    }
    Path(str(OUT) + '.json').write_text(json.dumps(manifest, separators=(',', ':')))
    print(f'packed {len(ready)} masks into {COLUMNS * CELL}x{rows * CELL}; {len(entries)} frame entries; '
          f'{len(missing)} shadow keys missing')
    for key in sorted(missing):
        print('  missing', key)


if __name__ == '__main__':
    main()
