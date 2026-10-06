#!/usr/bin/env python3
"""Compare the baseline atlas with the two-view dock; no generated or recolored art."""
import io
import json
import subprocess
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
BASE = '0c7b649'

def old(path):
    return subprocess.check_output(['git', 'show', f'{BASE}:{path}'], cwd=ROOT)

before = Image.open(io.BytesIO(old('assets/sprites/buildings_hd.png'))).convert('RGBA')
after = Image.open(ROOT / 'assets/sprites/buildings_hd.png').convert('RGBA')
b = json.loads(old('assets/sprites/buildings_hd.json'))
a = json.loads((ROOT / 'assets/sprites/buildings_hd.json').read_text())

def frame(image, manifest, kind, stage):
    x, y, w, h = manifest['frames'][kind][stage]
    return image.crop((x, y, x + w, y + h))

# Repacking must not change retained pixels or structural registration.
for kind in a['frames']:
    assert a['footprints'][kind] == b['footprints'][kind]
    for stage in range(4):
        assert frame(before, b, kind, stage).tobytes() == frame(after, a, kind, stage).tobytes()
assert after.size == (2048, 2560)
assert sum(k.startswith('dock') for k in a['frames']) == 2
font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 17)
views = [('South', 'dock', 'dock', False), ('East', 'dock_east', 'dock', True),
         ('North', 'dock_north', 'dock_north', False), ('West', 'dock_west', 'dock_north', True)]
for stages, name in [([3], 'complete'), (range(4), 'all-stages')]:
    out = Image.new('RGB', (1120, len(stages)*560), '#e3e5e7')
    draw = ImageDraw.Draw(out)
    for row, stage in enumerate(stages):
        for col, (label, prev, current, mirror) in enumerate(views):
            for version in range(2):
                art = frame(before, b, prev, stage) if version == 0 else frame(after, a, current, stage)
                if version == 1 and mirror:
                    art = ImageOps.mirror(art)
                art.thumbnail((250, 250))
                x, y = col*280+15, row*560+version*280+25
                out.paste(art, (x,y), art)
                draw.text((x,y-22), f'{label} / {b["stages"][stage]} / {"before" if version == 0 else "after"}', font=font, fill='#20252a')
    out.save(OUT / f'{name}.jpg', quality=92)
print('All retained atlas pixels and footprint records match baseline; 16 → 8 dock frames.')
print(f'PNG: {len(old("assets/sprites/buildings_hd.png")):,} → {(ROOT / "assets/sprites/buildings_hd.png").stat().st_size:,} bytes')
