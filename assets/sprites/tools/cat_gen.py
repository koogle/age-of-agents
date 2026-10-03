"""Catalog buildings: complete first (house + temple as style/camera refs), then 3 earlier stages as edits."""
import sys
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
SRC = "/home/user/age-of-agents/assets/sprites/building_sources/"
REFS = [data_uri(SRC + "clean_roofs/house_complete.png"), data_uri("tc/complete.png")]
STYLE = ("Match the attached references exactly in drawing style, camera and light: a thin-line European comic crossed with Studio Ghibli, fine "
  "one-pixel dark-brown ink linework, flat two-tone cel fills, warm sunlight from the upper left, cool teal-tinted shadows, no heavy outer outline; "
  "the same three-quarter view from the front-left seen from above at about 45 degrees, a square footprint, at a similar scale in the frame as the "
  "house. Mediterranean Greek island architecture: whitewash, cream limestone, terracotta roof tiles, weathered wood, small blue accents. ")
TAIL = (" One building only, centred, the whole building visible. Plain pure white background, no ground plane, no grass, absolutely no shadow cast "
  "on the ground, no people, no animals, no text, no letters, no signs.")
B = {
  "mining_camp": "Draw a small mining camp: a low timber-framed shed with a terracotta tile roof built against a small grey rock outcrop, a rack of pickaxes and shovels, a wooden mine cart full of grey ore, a stack of cut stone and a lantern on a post.",
  "farm": "Draw a small farm plot: a rectangular field of ripe golden wheat in neat rows bordered by a low dry-stone wall, a tiny whitewashed tool shed with a terracotta roof at one corner, a wooden plough and a straw scarecrow.",
  "lumber_mill": "Draw a small lumber mill: an open-sided timber shed with a terracotta tile roof sheltering a big two-man saw bench, a pile of round logs on one side and a neat stack of sawn planks on the other, a sawhorse.",
  "smelter": "Draw a small smelter: a sturdy limestone workshop with a terracotta roof and a tall square stone chimney with a little smoke, an open front with a glowing orange furnace mouth, a leather bellows, an anvil and a stack of dark metal ingots.",
  "kiln": "Draw a small pottery kiln workshop: a domed beehive kiln of terracotta bricks with a small chimney and a glowing arched firing door, next to a small whitewashed shed with a terracotta roof, stacks of fired bricks and amphorae.",
  "weaver": "Draw a small weaver's workshop: a whitewashed house with a terracotta roof and a wide open front showing a wooden upright loom inside, a pergola beside it with dyed cloth drying on a line in blue, ochre and terracotta, baskets of wool.",
  "kitchen": "Draw a small cookhouse kitchen: a whitewashed building with a terracotta roof, a domed clay bread oven attached at the side with a little chimney, a vine-covered pergola over a long wooden table, baskets of bread, fruit and amphorae.",
  "barracks": "Draw a small military barracks: a sturdy rectangular limestone hall with a terracotta gable roof and a columned porch, round wooden shields painted with a blue emblem hanging on the wall, a rack of spears, a small fenced training yard with a straw training dummy.",
  "range": "Draw a small archery range: a fenced sandy training yard with three round straw target butts on wooden stands, a small open shelter with a terracotta roof sheltering a rack of bows and quivers of arrows.",
  "workshop": "Draw a small siege workshop: a large open timber-framed workshop hall with a terracotta roof, a wooden crane arm, a half-built wooden cart with big spoked wheels inside, stacks of timber, ropes and tools.",
  "infirmary": "Draw a small infirmary: a whitewashed building with a terracotta roof and a shaded columned porch, a blue-painted door, clay planters of green healing herbs along the front, a bench with folded white linens and a water jar.",
  "monument": "Draw a civic monument: a tall white marble statue of a robed figure holding up a laurel wreath, standing on a stepped square marble plinth, with two small bronze braziers and a little blue pennant on a pole beside it.",
}
STAGES = {
  "foundation": "only just started: the stone footing outline of the building's footprint laid on bare ground, with a few loose cut stones, wooden stakes and planks; nothing standing yet",
  "walls": "half built: the main walls or frame standing about two thirds high with open tops, simple wooden scaffolding poles and a ladder; no roof, none of the finishing details",
  "roof": "nearly finished: the walls or frame complete and the roof timbers in place with only some of the terracotta tiles laid (or, without a roof, the main structure finished but the details still missing), a little scaffolding still standing",
}
SPECIAL = {
  ("farm", "foundation"): "only just started: the field marked out with wooden stakes and string on bare brown earth, a few stones for the wall and a pile of planks; no crops, no shed",
  ("farm", "walls"): "half built: the low dry-stone wall half finished around a ploughed brown field with fresh furrows, the shed's timber frame standing; no crops, no roof",
  ("farm", "roof"): "nearly finished: the wall complete, small green sprouts in the furrows, the shed roof half tiled; no scarecrow yet",
  ("range", "walls"): "half built: the fence posts standing with only some rails, the shelter's posts up without a roof, one target stand without its straw butt",
  ("monument", "walls"): "half built: the stepped plinth complete and the lower half of the statue roughly carved from a marble block, wooden scaffolding around it",
  ("monument", "roof"): "nearly finished: the full statue standing but the wreath and face still rough, scaffolding on one side, no braziers or pennant yet",
}
EDIT = ("The attached image is a finished building. Redraw EXACTLY the same building footprint, camera angle, scale, position in the frame, drawing "
  "style and colours, but show it {state}. Keep the footprint identical in size and position. Plain pure white background, no cast shadow, no "
  "people, no text.")
def complete(k):
    o = run("fal-ai/nano-banana/edit", {"prompt": STYLE + B[k] + TAIL, "image_urls": REFS, "num_images": 1, "output_format": "png", "aspect_ratio": "1:1"}, 0.0398, f"cat:{k}_complete")
    download(o["images"][0]["url"], f"cat/{k}_complete.png"); return k
def stage(job):
    k, st = job
    o = run("fal-ai/nano-banana/edit", {"prompt": EDIT.format(state=SPECIAL.get((k, st), STAGES[st])), "image_urls": [data_uri(f"cat/{k}_complete.png")], "num_images": 1, "output_format": "png", "aspect_ratio": "1:1"}, 0.0398, f"cat:{k}_{st}")
    download(o["images"][0]["url"], f"cat/{k}_{st}.png"); return f"{k}_{st}"
if __name__ == "__main__":
    mode, names = sys.argv[1], sys.argv[2:] or list(B)
    with ThreadPoolExecutor(8) as ex:
        if mode == "complete": print(list(ex.map(complete, names)))
        else: print(list(ex.map(stage, [(k, s) for k in names for s in STAGES])))
