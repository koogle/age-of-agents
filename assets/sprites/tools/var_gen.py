"""Variant strips for resource nodes: same style, stages and scale as the existing strip, different individual shapes."""
import sys
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
STAGED = {
  "berry": "a low leafy wild berry bush with large scarlet berries (exactly THREE bushes: left many berries, middle exactly three berries, right no berries at all)",
  "stone": "a pale cream limestone outcrop of angular blocks (left full, middle about half quarried away, right a little rubble)",
  "gold":  "an irregular lumpy grey-ochre boulder with bright gold veins (left full, middle about half quarried away, right a small remnant)",
  "iron":  "a dark grey-brown iron ore rock with rusty red-orange streaks (left full, middle about half quarried away, right a small remnant)",
  "clay":  "a shallow terracotta clay pit (left brimming with a wet clay mound, middle the mound half dug out, right nearly empty)",
  "fiber": "a clump of tall pale golden flax and reed stalks with a few blue flowers (left dense, middle about half harvested, right a few stalks and stubble)",
}
SHAPE = {1: "Give it a clearly different individual silhouette and arrangement: a little wider and lower, leaning slightly to the left.",
         2: "Give it a clearly different individual silhouette and arrangement: a little narrower and taller, leaning slightly to the right."}
STAGED_P = ("The attached image is a sprite strip showing three depletion stages of one game resource node. Draw a NEW sprite strip of the "
  "SAME kind of node, {what}, as a different individual of the same species/material, so a cluster of them does not look stamped. {shape} "
  "Keep EXACTLY the same drawing style (fine internal dark-brown pen lines, flat two-tone fills, warm light from the upper left, cool teal "
  "shadows, no heavy outer outline), the same overall size, scale and three-quarter view, and the same three stages left to right. Three "
  "separate objects in one row with white space between them, no ground line, no cast shadows, plain pure white background, no text.")
TREES = {
  "cypress": ("res/trees.png", "THREE different single tall slender dark-green Mediterranean cypress trees (one tree each, NOT pairs): one very slim and tall, one slightly fuller with a gently curved tip, one a bit shorter and wider"),
  "olive": ("res/trees.png", "THREE different gnarled silver-green olive trees: one with a wide umbrella crown, one with a split double trunk leaning left, one younger and rounder"),
}
TREE_P = ("The attached strip shows trees and a stump in a game's drawing style. Draw a NEW sprite strip of {what}, side by side in one row with "
  "white space between them, each standing on its own on the same ground line, same scale as the trees in the reference. Keep EXACTLY the same "
  "drawing style (fine internal dark-brown pen lines, flat two-tone fills, warm light from the upper left, cool teal shadows, no heavy outer "
  "outline). No ground line, no cast shadows, plain pure white background, no text.")
def staged(job):
    node, v = job
    o = run("fal-ai/nano-banana/edit", {"prompt": STAGED_P.format(what=STAGED[node], shape=SHAPE[v]), "image_urls": [data_uri(f"res/{node}.png")],
            "num_images": 1, "output_format": "png", "aspect_ratio": "21:9"}, 0.0398, f"var:{node}{v}")
    download(o["images"][0]["url"], f"var/{node}{v}.png"); return f"{node}{v}"
def tree(node):
    ref, what = TREES[node]
    o = run("fal-ai/nano-banana/edit", {"prompt": TREE_P.format(what=what), "image_urls": [data_uri(ref)], "num_images": 1, "output_format": "png", "aspect_ratio": "21:9"}, 0.0398, f"var:{node}")
    download(o["images"][0]["url"], f"var/{node}.png"); return node
if __name__ == "__main__":
    jobs = [a for a in sys.argv[1:]] or [f"{n}{v}" for n in STAGED for v in (1, 2)] + list(TREES)
    st = [(j[:-1], int(j[-1])) for j in jobs if j[:-1] in STAGED]; tr = [j for j in jobs if j in TREES]
    with ThreadPoolExecutor(8) as ex:
        print(list(ex.map(staged, st)), list(ex.map(tree, tr)))
