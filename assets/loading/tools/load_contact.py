import json
import numpy as np
from PIL import Image, ImageDraw
m = json.load(open("load/buildings.json")); sheet = Image.open("load/buildings.webp").convert("RGBA")
title = Image.open("load/title.webp").convert("RGBA")
W, H = 1240, 1500
yy, xx = np.mgrid[0:H, 0:W]
r = np.hypot((xx - W * 0.5) / W, (yy - H * 0.4) / H) / 0.7       # radial-gradient(circle at 50% 40%, #f4f0e2 0%, #cfe5f2 70%)
t = np.clip(r, 0, 1)[..., None]
grad = (np.array([0xf4, 0xf0, 0xe2]) * (1 - t) + np.array([0xcf, 0xe5, 0xf2]) * t).astype(np.uint8)
out = Image.fromarray(grad, "RGB").convert("RGBA")
tw = 760; ti = title.resize((tw, round(title.height * tw / title.width)), Image.LANCZOS)
out.alpha_composite(ti, ((W - tw) // 2, 24))
d = ImageDraw.Draw(out)
y0 = 24 + ti.height + 30
for row, (b, rects) in enumerate(m["frames"].items()):
    for c, (x, y, w, h) in enumerate(rects):
        cell = sheet.crop((x, y, x + w, y + h)).resize((240, 240), Image.LANCZOS)
        out.alpha_composite(cell, (40 + c * 300, y0 + row * 250))
    d.text((40, y0 + row * 250 + 236), f"{b}: foundation / walls / roof / complete (240 px cells)", fill=(61, 74, 82))
out.convert("RGB").save("load/contact_sheet.jpg", quality=86); print(out.size)
