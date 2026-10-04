#!/usr/bin/env python3
"""Check transport registration, transparency and retained source resolution."""
from pathlib import Path
import json
from PIL import Image
root=Path(__file__).resolve().parents[1]/'assets/sprites'
meta=json.loads((root/'transport.json').read_text())
im=Image.open(root/'transport.png')
assert im.format=='PNG' and im.mode=='RGBA' and list(im.size)==meta['size']
for name,rect in meta['frames'].items():
    x,y,w,h=rect
    assert w>=512 and h>=512
    frame=im.crop((x,y,x+w,y+h))
    assert all(frame.getpixel(p)[3]==0 for p in [(0,0),(w-1,0),(0,h-1),(w-1,h-1)])
    assert frame.getchannel('A').getbbox()[3]==480
    source=Image.open(root/'transport_sources'/f'{name}-cutout.png')
    assert min(source.size)>=768
print('PASS transport: two registered transparent 512px frames from retained high-resolution sources')
