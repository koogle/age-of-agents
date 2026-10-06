#!/usr/bin/env python3
"""Pack the retained water jug into the existing resource atlas and HUD kit."""
import json
from pathlib import Path
from PIL import Image
from normalize_icons import normalize

ROOT = Path(__file__).resolve().parents[1]
source = Image.open(ROOT / 'assets/ui/sources/water/refined.png').convert('RGBA')
normalize(source).save(ROOT / 'assets/ui/icons/resource_water.png')
# Ignore near-transparent diffusion fringe when locating the ground contact.
sprite = source.crop(source.getchannel('A').point(lambda a: 255 if a > 40 else 0).getbbox())
sprite.thumbnail((464, 464), Image.Resampling.LANCZOS)
frame = Image.new('RGBA', (512, 512))
frame.alpha_composite(sprite, ((512 - sprite.width) // 2, 496 - sprite.height))
path = ROOT / 'assets/sprites/resources.json'
manifest = json.loads(path.read_text())
atlas = Image.open(path.with_suffix('.png')).convert('RGBA')
# An unused cell of the current atlas. Leave every existing frame untouched.
rect = [2560, 1024, 512, 512]
assert not any(r == rect for node in manifest['nodes'].values() for r in node['stages'] if node is not manifest['nodes'].get('water'))
atlas.paste(frame, tuple(rect[:2]))
atlas.save(path.with_suffix('.png'))
manifest['nodes']['water'] = {'stages': [rect], 'unitsPerPixel': 0.0018, 'worldSize': 0.84, 'measured': 'height'}
path.write_text(json.dumps(manifest, indent=1) + '\n')
print('Packed water icon and 512px source marker')
