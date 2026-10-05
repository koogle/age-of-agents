import sys, json
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
REF = data_uri("bake/nano.png")
PREFIX = ("The attached image is a STYLE REFERENCE only. Draw a NEW, different single game UI icon of: {s}. "
  "Match the reference exactly: the same fine thin dark-brown ink contour lines in the style of Moebius and Studio Ghibli, "
  "the same sparse hatching, the same restrained flat watercolor wash, the same warm Mediterranean palette (ochre, olive, "
  "terracotta, limestone cream, muted turquoise accents), the same three-quarter viewpoint from the upper left and similar size in frame. "
  "Refined and subtle, not cute, not chunky. One isolated object, centered, on a plain pure white background, "
  "no cast shadow, no ground, no frame, no border, no text, no letters.")
MEDAL = ("The attached image is a STYLE REFERENCE only. Draw a NEW image: a single round antique coin medallion seen straight on, "
  "with a thin double ink rim and a warm aged bronze-gold disc, showing {s}. "
  "Use the same fine thin dark-brown ink linework in the style of Moebius and Studio Ghibli, the same restrained flat watercolor wash "
  "and warm Mediterranean palette as the reference. Refined and dignified, not cute. The coin is perfectly circular, centered, filling most of the frame, "
  "on a plain pure white background, no shadow, no text, no letters, no inscription.")
SUBJ = {
 "resource_food": (PREFIX, "a generous cluster of plump glossy red wild berries on a short twig with three broad olive-green leaves, berries large and prominent"),
 "resource_stone": (PREFIX, "a small pile of three rough-cut pale limestone blocks"),
 "resource_gold": (PREFIX, "a small heap of three raw gold nuggets"),
 "resource_iron": (PREFIX, "a few chunks of dark grey iron ore with rusty red-brown veins"),
 "resource_clay": (PREFIX, "a lump of wet reddish terracotta clay with a small unfired clay brick beside it"),
 "resource_fiber": (PREFIX, "a coiled skein of pale golden plant fiber cord, loosely twisted, with a few thin flax stalks with small blue flowers beside it"),
 "command_build": (PREFIX, "a small Mediterranean town hall with whitewashed walls and a terracotta tile roof, with a wooden scaffold and a mason's hammer leaning against it"),
 "command_cancel": (PREFIX, "a simple hand-drawn ink saltire cross mark made of two crossed brush strokes in muted terracotta red"),
 "command_train": (PREFIX, "a single small full-body villager figure in a cream linen tunic and sandals, walking forward with a wooden staff"),
 "tech_forestry": (PREFIX, "a felling axe embedded in a tree stump with a fresh sapling sprouting beside it"),
 "tech_agriculture": (PREFIX, "a small diamond-shaped patch of freshly tilled terracotta earth with three neat parallel furrows and three prominent green two-leaf seedlings, with one simple wooden-handled bronze hoe laid diagonally beside the furrows; a clear symbol of improved field cultivation, no wheat, no grain sheaf, no corn, no sickle"),
 "tech_masonry": (PREFIX, "a mason's iron chisel and wooden mallet resting on a carved limestone block"),
 "tech_mining": (PREFIX, "a pickaxe resting against a chunk of rock with a glint of ore"),
 "tech_textiles": (PREFIX, "a wooden drop spindle wound with cream wool thread and a small folded piece of woven cloth"),
 "portrait_villager": (MEDAL, "the head and shoulders of a calm young Mediterranean villager in profile, short dark hair, cream linen tunic"),
 "portrait_group": (MEDAL, "three villagers' heads in profile, overlapping in a row, in cream linen tunics"),
 "portrait_towncenter": (MEDAL, "a small Mediterranean town hall with whitewashed walls, columns and a terracotta tile roof"),
}
def go(k):
    tmpl, s = SUBJ[k]
    try:
        o = run("fal-ai/nano-banana/edit", {"prompt": tmpl.format(s=s), "image_urls":[REF], "num_images":1, "output_format":"png", "aspect_ratio":"1:1"}, 0.0398, "icon:"+k)
        download(o["images"][0]["url"], f"icons/{k}.png"); return k, "ok"
    except Exception as e: return k, str(e)[:300]
keys = sys.argv[1:] or list(SUBJ)
with ThreadPoolExecutor(6) as ex:
    for r in ex.map(go, keys): print(r)
