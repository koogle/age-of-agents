import json, sys
from PIL import Image, ImageDraw
PX = 60                                              # screen px per world cell at gameplay zoom
m = json.load(open(sys.argv[1] + ".json")); sheet = Image.open(sys.argv[1] + ".png")
vm = json.load(open("out_spr/villager.json")); vs = Image.open("out_spr/villager.png")
def crop(img, r): return img.crop((r[0], r[1], r[0] + r[2], r[1] + r[3]))
rows = list(m["nodes"].items())
W, H = 130 + 4 * 110, len(rows) * 120 + 10
out = Image.new("RGB", (W * 2, H), "white"); d = ImageDraw.Draw(out)
vil = crop(vs, vm["animations"]["idle"]["front"][0]); vsc = 0.78 / vm["figureHeight"] * PX
vil = vil.resize((round(256 * vsc),) * 2, Image.LANCZOS)
for g, gname in enumerate(["meadow", "prairie"]):
    tex = Image.open(f"/home/user/age-of-agents/assets/terrain/{gname}.webp").convert("RGB").resize((240, 240))
    for y in range(0, H, 240):
        for x in range(0, W, 240): out.paste(tex, (g * W + x, y))
    for r, (name, n) in enumerate(rows):
        base_y = 10 + r * 120 + 105
        d.text((g * W + 6, base_y - 40), name, fill=(40, 30, 20))
        sc = n["unitsPerPixel"] * PX
        for i, rc in enumerate(n["stages"]):
            c = crop(sheet, rc); s = round(256 * sc)
            if s < 2: continue
            c = c.resize((s, s), Image.LANCZOS)
            cx = g * W + 130 + i * 110 + 50
            out.paste(c, (cx - s // 2, base_y - round(m["anchor"][1] * sc)), c)
        out.paste(vil, (g * W + 130 + 3 * 110 + 50 - vil.width // 2, base_y - round(vm["anchor"][1] * vsc)), vil)
out.save(sys.argv[2], optimize=True); print(out.size)
