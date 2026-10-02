"""Resource and tree billboard strips (one image per node so stages share scale, light and style)."""
import sys
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
REFS = [data_uri("spr/master_1.png"), data_uri("terrain/ref_med4.jpg")]
STYLE = ("The first attached image sets the drawing style (a thin-line European comic crossed with Studio Ghibli: fine one-pixel dark-brown ink "
  "linework inside the shapes, flat two-tone cel colour fills, warm sunlight from the upper left, cool teal-tinted shadows); the second sets "
  "the Mediterranean palette. Do NOT draw any heavy or thick outer outline around the objects, only fine internal pen lines. "
  "Draw a game sprite strip: {n} separate objects standing side by side in one row, evenly spaced with clear white space between them, all at the "
  "same scale, each standing on one shared ground line, three-quarter view seen from slightly above like an RTS camera. {what} "
  "No ground plane, no ground line, no cast shadows, no grass around them, plain pure white background, no text, no numbers.")
NODES = {
  "trees":  (3, "Left: a pair of tall slender dark-green Mediterranean cypress trees growing close together. Middle: a gnarled silver-green olive tree with a twisted grey trunk and a rounded airy crown. Right: a small cut tree stump with a pale cut top showing rings, and a few wood chips."),
  "berry":  (3, "Three depletion stages of the SAME low, wide, knee-high round wild berry shrub (a bush, NOT a tree: no trunk, leafy all the way down to the ground) with olive-green leaves: left laden with many bright red berries all over, middle the same leafy bush with clearly only about half as many berries, right the same leafy green bush (still full of leaves) with just two or three red berries left. Draw the ground line nowhere."),
  "stone":  (3, "Three depletion stages of the SAME pale cream limestone outcrop of chunky angular blocks: left a large full outcrop, middle about half of it quarried away into a smaller pile, right just a few small rubble blocks left."),
  "gold":   (3, "Three depletion stages of the SAME grey-ochre rock with glittering bright gold veins and nuggets: left a large rock rich in gold, middle about half quarried away, right a small remnant rock with a single gold glint."),
  "iron":   (3, "Three depletion stages of the SAME dark grey-brown iron ore rock with rusty red-orange streaks: left a large rock, middle about half quarried away, right a small remnant rock."),
  "clay":   (3, "Three depletion stages of the SAME shallow terracotta clay pit dug into the ground, seen from slightly above: left a full pit brimming with a wet reddish clay mound, middle the mound half dug out, right a nearly empty shallow pit with a little clay at the bottom."),
  "fiber":  (3, "Three depletion stages of the SAME clump of tall pale golden flax and reed stalks with a few small blue flowers: left a dense full clump, middle about half harvested and thinner, right only a few stalks and short cut stubble."),
}
def go(k):
    n, what = NODES[k]
    o = run("fal-ai/nano-banana/edit", {"prompt": STYLE.format(n=n, what=what), "image_urls": REFS, "num_images": 1, "output_format": "png", "aspect_ratio": "21:9"}, 0.0398, "res:" + k)
    download(o["images"][0]["url"], f"res/{k}.png"); return k
if __name__ == "__main__":
    with ThreadPoolExecutor(7) as ex:
        for r in ex.map(go, sys.argv[1:] or NODES): print(r, flush=True)
