"""Loading-screen buildings: complete building first (temple as style/camera reference), then 3 earlier stages as edits."""
import sys, os
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
STYLE = ("Match the attached reference exactly in drawing style, camera and light: a thin-line European comic crossed with Studio Ghibli, fine "
  "one-pixel dark-brown ink linework inside the shapes, flat two-tone cel fills, warm sunlight from the upper left, cool teal-tinted shadows, no "
  "heavy outer outline; the same three-quarter view from the front-left seen from above at about 45 degrees, at a similar scale in the frame. ")
TAIL = " One building only, centred, the whole building visible. Plain pure white background, no ground plane, no grass, no cast shadow, no people, no text, no signs."
B = {
  "house": "Draw a small Greek island house: whitewashed cube walls, a low terracotta tile roof, a blue-painted wooden door and one small window with blue shutters, a stone step, a potted plant by the door. Absolutely NO shadow cast on the ground anywhere around the house: the white background touches the walls directly.",
  "granary": "Draw a small Mediterranean granary storehouse: rough cream limestone walls on a raised stone base, a terracotta tiled gable roof, a wide wooden door, a few clay amphorae and grain sacks leaning against the front.",
  "watchtower": "Draw a slender round stone watchtower: whitewashed and limestone masonry, a wooden balcony gallery near the top, a small conical terracotta roof, a narrow wooden door at the base, a tiny blue pennant on the roof tip.",
  "dock": "Draw a small wooden fishing dock: a short plank jetty on wooden posts with a little wooden boathouse shed with a terracotta roof at its land end, coils of rope and a fish basket, and a small wooden fishing boat with a cream sail moored alongside. Draw no water surface; the posts simply end at the bottom.",
}
STAGES = {
  "foundation": "only just started: the stone footing outline of the building laid on bare ground with a few loose cut stones, wooden stakes and planks; no walls, no roof",
  "walls": "half built: the walls standing about two thirds high with open tops, simple wooden scaffolding poles and a ladder against them; no roof",
  "roof": "nearly finished: the walls complete, the roof timbers in place with only some of the terracotta tiles laid, a little scaffolding still standing",
}
DOCK_STAGES = {
  "foundation": "only just started: a few wooden posts driven in a row with a couple of planks and a pile of timber beside them; no deck, no shed, no boat",
  "walls": "half built: the posts with about half of the plank deck laid and the shed's timber frame standing; no shed roof, no boat",
  "roof": "nearly finished: the deck complete and the shed with its roof timbers and only part of the terracotta tiles; no boat yet",
}
EDIT = ("The attached image is a finished building. Redraw EXACTLY the same building footprint, camera angle, scale, position in the frame, drawing "
  "style and colours, but show it {state}. Keep the footprint identical in size and position. Plain pure white background, no cast shadow, no people, no text.")
def complete(k):
    o = run("fal-ai/nano-banana/edit", {"prompt": STYLE + B[k] + TAIL, "image_urls": [data_uri("tc/complete.png")], "num_images": 1, "output_format": "png", "aspect_ratio": "1:1"}, 0.0398, f"bld:{k}_complete")
    download(o["images"][0]["url"], f"bld/{k}_complete.png"); return k
def stage(job):
    k, st = job
    states = DOCK_STAGES if k == "dock" else STAGES
    o = run("fal-ai/nano-banana/edit", {"prompt": EDIT.format(state=states[st]), "image_urls": [data_uri(f"bld/{k}_complete.png")], "num_images": 1, "output_format": "png", "aspect_ratio": "1:1"}, 0.0398, f"bld:{k}_{st}")
    download(o["images"][0]["url"], f"bld/{k}_{st}.png"); return f"{k}_{st}"
if __name__ == "__main__":
    mode, names = sys.argv[1], sys.argv[2:] or list(B)
    with ThreadPoolExecutor(8) as ex:
        if mode == "complete": print(list(ex.map(complete, names)))
        else: print(list(ex.map(stage, [(k, s) for k in names for s in STAGES])))
