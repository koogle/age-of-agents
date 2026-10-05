#!/usr/bin/env python3
"""Render relief maps from the real generator. Build island_preview first.

cargo build -p aoa-game --example island_preview
python3 docs/verification/island-generation/preview.py /tmp/island-previews
"""
import json
import re
import subprocess
import sys
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(sys.argv[1])
OUT.mkdir(parents=True, exist_ok=True)
MASK = (1 << 64) - 1

def mix(a, b):
    z = a ^ ((b * 0x9E3779B97F4A7C15) & MASK)
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
    return z ^ (z >> 31)

def unpack(s):
    return re.sub(r'~(\d+):(.)', lambda m: m[2] * int(m[1]), s)

names = ['Rounded', 'Square', 'Long', 'Bay', 'Lobed']
seeds = {}
for seed in range(100):
    seeds.setdefault(mix(seed, 0x5348415045) % 5, seed)
canvas = Image.new('RGB', (1200, 660), '#192c36')
draw = ImageDraw.Draw(canvas)
draw.text((20, 12), 'Generated island relief - coast shape first, shared terrain and drainage', fill='white')
for k, name in enumerate(names):
    seed = seeds[k]
    raw = subprocess.check_output([str(ROOT / 'target/debug/examples/island_preview'), str(seed)])
    (OUT / f'{name.lower()}.json').write_bytes(raw)
    state = json.loads(raw)
    t = state['terrain']
    cells, heights = unpack(t['cells']).lower(), unpack(t['heights'])
    digits = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_'
    width, height = t['columns'], t['rows']
    image = Image.new('RGB', (width, height))
    for i, c in enumerate(cells):
        h = max(0, digits.index(heights[i]) / 63 * 2 - 1)
        color = (37, 90, 113) if c == 'j' else (int(108 + h * 130), int(145 + h * 90), int(79 + h * 145))
        if c == 'l': color = (62, 175, 242)
        if c == 'i': color = (220, 203, 148)
        image.putpixel((i % width, i // width), color)
    x, y = (k % 3) * 400 + 20, (k // 3) * 310 + 55
    draw.text((x, y), f'{name} / seed {seed}', fill='white')
    canvas.paste(image.resize((360, 240), Image.Resampling.NEAREST), (x, y + 25))
draw.text((820, 410), 'Blue: sea / rivers\nSand: coast / fords\nLight ground: high relief\n\nActual generated terrain;\nnot an in-game screenshot.', fill='white', spacing=10)
canvas.save(OUT / 'relief.png')
