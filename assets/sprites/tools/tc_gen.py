"""Town center billboard frames: complete temple first, then the other stages as edits of it."""
import sys, os
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
STYLE = ("Drawing style as in the attached references: a thin-line European comic crossed with Studio Ghibli, fine one-pixel dark-brown ink "
  "linework inside the shapes, flat two-tone cel colour fills, warm sunlight from the upper left, cool teal-tinted shadows, no heavy or thick outer "
  "outline. ")
VIEW = ("Three-quarter view from the front-left, seen from above at about 45 degrees like an RTS game camera, the whole building visible and centred, "
  "square footprint. Plain pure white background, no ground plane, no grass, no cast shadow on the ground, no people, no text.")
COMPLETE = (STYLE + "Draw ONE building as a game sprite: a small ancient Greek temple used as a town center, white marble, a stepped marble base "
  "(three low steps all around), a ring of fluted white columns, a warm terracotta tiled gable roof with triangular pediments, and a small blue team "
  "pennant flying on a short pole at the front end of the roof ridge. Simple, readable, sturdy proportions, about as wide as it is tall. " + VIEW)
EDIT = ("The attached image is the finished town-center temple. Redraw EXACTLY the same building footprint, camera angle, scale, position in the frame, "
  "drawing style and colours, but show it {state}. Keep the stepped marble base footprint identical in size and position. Plain pure white "
  "background, no ground plane, no cast shadow, no people, no text.")
STATES = {
  "foundation": "as a fresh building site: only the lowest marble step of the base laid out as a square stone outline, with a few loose cut marble blocks and a couple of wooden planks scattered on it; no columns, no roof, no pennant",
  "build33": "about one third built: the full stepped marble base complete; on it ONLY short column stumps, each just knee-high (about one third of the final column height), with open sky above them: NO beams, NO entablature, NO ceiling, NO roof, NO pennant; a few marble column drums and blocks lie on the base and a single low wooden scaffold platform stands at one side",
  "build66": "about two thirds built: the full stepped base, all columns standing at full height, the beams on top partly placed, wooden scaffolding with ladders around it, the roof not yet built; no pennant",
  "working": "complete and busy: pixel-for-pixel the same finished temple at the same width and size, INCLUDING the small blue team pennant on its pole at the roof ridge, the only differences being a warm orange-gold glow of firelight in the doorway between the front columns and a thin wisp of pale grey smoke rising from the roof",
}
def complete():
    refs = [data_uri("spr/master_1.png"), data_uri("res/stone.png")]
    o = run("fal-ai/nano-banana/edit", {"prompt": COMPLETE, "image_urls": refs, "num_images": 1, "output_format": "png", "aspect_ratio": "1:1"}, 0.0398, "tc:complete")
    download(o["images"][0]["url"], "tc/complete.png")
def stage(k):
    o = run("fal-ai/nano-banana/edit", {"prompt": EDIT.format(state=STATES[k]), "image_urls": [data_uri("tc/complete.png")], "num_images": 1, "output_format": "png", "aspect_ratio": "1:1"}, 0.0398, "tc:" + k)
    download(o["images"][0]["url"], f"tc/{k}.png"); return k
if __name__ == "__main__":
    args = sys.argv[1:]
    if not args or "complete" in args:
        complete(); args = [a for a in args if a != "complete"] or (list(STATES) if not sys.argv[1:] else [])
    with ThreadPoolExecutor(4) as ex: print(list(ex.map(stage, args)))
