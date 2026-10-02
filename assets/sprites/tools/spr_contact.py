import json, sys
from PIL import Image, ImageDraw
base = sys.argv[1]; out = sys.argv[2]
m = json.load(open(base + ".json")); sheet = Image.open(base + ".png")
scale = 56 / m["figureHeight"]                       # figure about 56 px tall, as in game
cw = int(m["cell"][0] * scale)
rows = [(a, f, rects) for a, fs in m["animations"].items() for f, rects in fs.items()]
grounds = ["meadow", "prairie", "beach"]
W = 150 + 8 * (cw + 4); H = len(rows) * (cw + 6) + 10
s = Image.new("RGB", (W * len(grounds), H), "white"); d = ImageDraw.Draw(s)
for g, gname in enumerate(grounds):
    tex = Image.open(f"/home/user/age-of-agents/assets/terrain/{gname}.webp").convert("RGB").resize((256, 256))
    for y in range(0, H, 256):
        for x in range(0, W, 256): s.paste(tex, (g * W + x, y))
    for r, (a, f, rects) in enumerate(rows):
        y = 5 + r * (cw + 6)
        d.text((g * W + 6, y + cw // 2 - 6), f"{a} {f}", fill=(40, 30, 20))
        for i, (x0, y0, w, h) in enumerate(rects):
            c = sheet.crop((x0, y0, x0 + w, y0 + h)).resize((cw, cw), Image.LANCZOS)
            s.paste(c, (g * W + 150 + i * (cw + 4), y), c)
s.save(out, optimize=True); print(s.size)
