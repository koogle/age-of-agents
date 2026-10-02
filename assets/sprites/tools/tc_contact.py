import json, sys
from PIL import Image, ImageDraw
PX = 60
R = "/home/user/age-of-agents/assets"
m = json.load(open(sys.argv[1] + ".json")); sheet = Image.open(sys.argv[1] + ".png")
vm = json.load(open(f"{R}/sprites/villager.json")); vs = Image.open(f"{R}/sprites/villager.png")
crop = lambda img, r: img.crop((r[0], r[1], r[0] + r[2], r[1] + r[3]))
W, H = 5 * 170 + 40, 2 * 200
out = Image.new("RGB", (W, H)); d = ImageDraw.Draw(out)
for g, gname in enumerate(["meadow", "prairie"]):
    tex = Image.open(f"{R}/terrain/{gname}.webp").convert("RGB").resize((240, 240))
    for y in range(g * 200, g * 200 + 200, 240):
        for x in range(0, W, 240): out.paste(tex, (x, y))
    sc = m["unitsPerPixel"] * PX; size = round(m["cell"][0] * sc)
    vsc = 0.78 / vm["figureHeight"] * PX; vil = crop(vs, vm["animations"]["idle"]["front"][0]).resize((round(256 * vsc),) * 2, Image.LANCZOS)
    for i, (name, rect) in enumerate(m["frames"].items()):
        ax, ay = 20 + i * 170 + 75, g * 200 + 140                       # ground position of the footprint centre
        c = crop(sheet, rect).resize((size, size), Image.LANCZOS)
        out.paste(c, (ax - round(m["anchor"][0] * sc), ay - round(m["anchor"][1] * sc)), c)
        # a villager standing one cell in front-right of the footprint centre
        vx, vy = ax + round(0.9 * PX), ay + round(0.55 * PX)
        out.paste(vil, (vx - round(vm["anchor"][0] * vsc), vy - round(vm["anchor"][1] * vsc)), vil)
        if g == 0: d.text((ax - 40, 4), name, fill=(40, 30, 20))
out.save(sys.argv[2], quality=88); print(out.size, "temple px", size)
