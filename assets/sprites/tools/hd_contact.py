import json
from PIL import Image, ImageDraw
R = "/home/user/age-of-agents/assets/sprites"
hm = json.load(open("out_spr/villager_idle_hd.json")); hd = Image.open("out_spr/villager_idle_hd.png")
tex = Image.open("/home/user/age-of-agents/assets/terrain/meadow.webp").convert("RGB").resize((256, 256))
def frame(who, facing, i, hdq):
    if hdq:
        x, y, w, h = hm["people"][who][facing][i]; return hd.crop((x, y, x + w, y + h)), hm["figureHeight"]
    m = json.load(open(f"{R}/{who}.json")); s = Image.open(f"{R}/{who}.png")
    x, y, w, h = m["animations"]["idle"][facing][i]; return s.crop((x, y, x + w, y + h)), m["figureHeight"]
cols = [(w, f, i) for w in ["villager", "villager_woman", "villager_elder"] for f, i in [("front", 0), ("back", 0)]]
W = 40 + len(cols) * 150; H = 520
out = Image.new("RGB", (W, H))
for y in range(0, H, 256):
    for x in range(0, W, 256): out.paste(tex, (x, y))
d = ImageDraw.Draw(out)
for row, (tall, y0) in enumerate([(60, 10), (200, 120)]):
    for c, (who, f, i) in enumerate(cols):
        for k, hdq in enumerate([False, True]):
            img, fig = frame(who, f, i, hdq); s = tall / fig
            img = img.resize((round(img.width * s), round(img.height * s)), Image.LANCZOS)
            x = 40 + c * 150 + k * (70 if tall == 60 else 72) - (0 if tall == 60 else 60)
            out.paste(img, (x + 20 - img.width // 2 + 20, y0 + (tall + 20) - img.height + 10 if tall == 60 else y0 + 250 + 120 - img.height), img)
d.text((6, 92), "60 px tall: left old 1x, right new HD (each pair)", fill=(30, 25, 20))
d.text((6, 500), "200 px tall: left old 1x (upscaled by the GPU), right new HD", fill=(30, 25, 20))
out.save("out_spr/hd_contact.jpg", quality=90); print(out.size)
