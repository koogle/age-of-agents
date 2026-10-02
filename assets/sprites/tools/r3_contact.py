import json, random
from PIL import Image, ImageDraw
R = "/home/user/age-of-agents/assets"
PXU = 120                                 # screen px per world unit (cell = 0.5 units = 60 px)
base = json.load(open(f"{R}/sprites/resources.json")); bs = Image.open(f"{R}/sprites/resources.png")
var = json.load(open("out_spr/resources_variants.json")); vs = Image.open("out_spr/resources_variants.png")
sc = json.load(open("out_spr/scenery.json")); ss = Image.open("out_spr/scenery.png")
vm = json.load(open(f"{R}/sprites/villager.json")); vls = Image.open(f"{R}/sprites/villager.png")
crop = lambda img, r: img.crop((r[0], r[1], r[0] + r[2], r[1] + r[3]))
NODE_UNITS = 0.45                          # each node drawn about this big (client draws ~0.4-0.5)
def node_sprite(node, k, stage=0):
    opts = [(bs, base["nodes"][node]["stages"], base["nodes"][node]["unitsPerPixel"])] + [(vs, v["stages"], v["unitsPerPixel"]) for v in var["nodes"].get(node, [])]
    sheet, stages, upp = opts[k % len(opts)]
    img = crop(sheet, stages[min(stage, len(stages) - 1)])
    # normalise so the base node's full stage is NODE_UNITS along its measured axis, keeping each variant's relative size
    ref_upp = base["nodes"][node]["unitsPerPixel"]; ref_world = base["nodes"][node]["worldSize"]
    f = NODE_UNITS / ref_world * (upp / ref_upp) * ref_upp * PXU
    return img.resize((max(1, round(256 * f)),) * 2, Image.LANCZOS), (128 * f, 248 * f)
W, H = 1400, 900
out = Image.new("RGB", (W, H))
tex = Image.open(f"{R}/terrain/meadow.webp").convert("RGB").resize((240, 240))
for y in range(300, H, 240):
    for x in range(0, W, 240): out.paste(tex, (x, y))
# sea band with volcano, ships and clouds
d = ImageDraw.Draw(out); d.rectangle([0, 0, W, 300], fill=(150, 196, 214)); d.rectangle([0, 170, W, 300], fill=(46, 120, 140))
def put(name, x, y, world):
    s = sc["sprites"][name]; img = crop(ss, s["rect"]); f = world / s["worldWidth"] * s["worldWidth"] / s["rect"][2] * PXU * 0.35
    img = img.resize((max(1, round(img.width * f)), max(1, round(img.height * f))), Image.LANCZOS)
    ax, ay = s["anchor"][0] * f, s["anchor"][1] * f; out.paste(img, (round(x - ax), round(y - ay)), img)
put("volcano", 1050, 172, 24); put("cloud_1", 300, 60, 5); put("cloud_4", 700, 95, 2.5); put("cloud_3", 1250, 50, 5.5)
put("ship_small", 200, 235, 1.0 * 4); put("ship_merchant", 560, 255, 1.5 * 4); put("ship_striped", 880, 228, 1.3 * 4)
rng = random.Random(3)
def cluster(node, cells, ox, oy, label):
    for i, (cx, cz) in enumerate(cells):
        img, (ax, ay) = node_sprite(node, rng.randrange(8))
        x = ox + cx * 0.5 * PXU + cz * 0.12 * PXU; y = oy + cz * 0.5 * PXU * 0.55
        out.paste(img, (round(x - ax), round(y - ay)), img)
    d.text((ox - 10, oy + 70), label, fill=(30, 25, 20))
cluster("cypress", [(i % 5, i // 5) for i in range(10)], 60, 380, "woodline: cypress (base + 4 variants)")
cluster("olive", [(i % 5, i // 5) for i in range(10)], 480, 380, "woodline: olive (base + 3 variants)")
cluster("berry", [(0, 0), (1, 0), (2, 0), (0.5, 1), (1.5, 1)], 940, 400, "berry patch")
for j, n in enumerate(["stone", "gold", "iron"]):
    cluster(n, [(0, 0), (1, 0), (0, 1), (1, 1)], 80 + j * 260, 640, f"{n} 2x2")
cluster("clay", [(0, 0), (1, 0), (2, 0)], 860, 650, "clay patch"); cluster("fiber", [(0, 0), (1, 0), (2, 0)], 1120, 650, "fiber patch")
# villager for scale and the effects row
vsc = 0.78 / vm["figureHeight"] * PXU; vil = crop(vls, vm["animations"]["idle"]["front"][0]).resize((round(256 * vsc),) * 2, Image.LANCZOS)
out.paste(vil, (round(1300 - 128 * vsc), round(470 - 240 * vsc)), vil)
for i, n in enumerate(["fx_wood_chips", "fx_stone_dust", "fx_berry_leaves", "fx_smoke_puff"]):
    s = sc["sprites"][n]; img = crop(ss, s["rect"]); f = s["unitsPerPixel"] * PXU * 1.5
    img = img.resize((max(1, round(img.width * f)), max(1, round(img.height * f))), Image.LANCZOS)
    out.paste(img, (100 + i * 120 - img.width // 2, 830 - img.height // 2), img)
d.text((60, 862), "effects (1.5x): wood chips, stone dust, berry leaves, smoke puff", fill=(30, 25, 20))
out.save("out_spr/r3_contact.jpg", quality=88); print(out.size)
