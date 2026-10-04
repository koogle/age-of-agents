#!/usr/bin/env python3
"""Pack retained high-resolution transport cutouts into registered 512px views."""
import json
from pathlib import Path
from PIL import Image
ROOT = Path(__file__).resolve().parents[1] / 'assets/sprites'
canvas = Image.new('RGBA', (1024, 512))
for i, name in enumerate(('front', 'rear')):
    source = Image.open(ROOT / 'transport_sources' / f'{name}-cutout.png').convert('RGBA')
    bounds = source.getchannel('A').point(lambda a: 255 if a > 16 else 0).getbbox()
    source = source.crop(bounds)
    source.thumbnail((464, 448), Image.Resampling.LANCZOS)
    canvas.alpha_composite(source, (i * 512 + (512-source.width)//2, 480-source.height))
canvas.save(ROOT / 'transport.png')
(ROOT / 'transport.json').write_text(json.dumps({'size':[1024,512], 'cell':[512,512], 'anchor':[256,410], 'frames':{'front':[0,0,512,512],'rear':[512,0,512,512]}},indent=2)+'\n')
