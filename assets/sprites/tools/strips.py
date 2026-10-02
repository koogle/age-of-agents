import sys
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
MASTER = data_uri("spr/master_1.png")
FRONT = "Three-quarter FRONT view (facing the viewer, turned slightly to the viewer's left), seen from slightly above like an RTS camera."
BACK = "Three-quarter BACK view (walking away from the viewer, turned slightly to the viewer's right, back of the head visible, face hidden), seen from slightly above like an RTS camera."
STRIPS = {
  "idle_front":  (2, FRONT, "standing idle, relaxed; frame 1 neutral, frame 2 a slight breath and weight shift"),
  "idle_back":   (2, BACK,  "standing idle, relaxed; frame 1 neutral, frame 2 a slight breath and weight shift"),
  "walk_front":  (4, FRONT, "a walk cycle: contact, passing, opposite contact, passing; arms swinging naturally"),
  "walk_back":   (4, BACK,  "a walk cycle: contact, passing, opposite contact, passing; arms swinging naturally"),
  "carry_front": (4, FRONT, "a walk cycle while carrying a bulging brown cloth sack of goods over one shoulder, held with both hands; the sack is on the shoulder in EVERY frame"),
  "carry_back":  (4, BACK,  "a walk cycle while carrying a bulging brown cloth sack of goods over one shoulder, held with both hands"),
  "chop":        (3, FRONT, "chopping wood with a wooden-handled axe held in both hands: axe raised high behind the head, mid swing, axe down at knee height"),
  "mine":        (3, FRONT, "mining with a pickaxe held in both hands: pickaxe raised overhead, mid swing, pickaxe striking the ground in front"),
  "forage":      (2, FRONT, "crouched low, reaching forward and down with one hand to pick berries, a small woven basket in the other hand; frame 2 reaching slightly further. Draw ONLY the villager and the basket, do NOT draw any bush, plant or berries on the ground"),
  "dig":         (3, FRONT, "digging with a wooden shovel: pushing the blade into the ground with one foot, lifting a scoop of earth, tossing it to the side"),
  "build":       (3, FRONT, "building with a wooden mallet-hammer held in one hand: hammer raised, mid swing, hammer striking down at waist height"),
}
AR = {2: "4:3", 3: "16:9", 4: "21:9"}
def prompt(n, view, action):
    return (f"The attached image is the IDENTITY and STYLE reference. Draw a horizontal sprite-sheet strip of exactly {n} animation frames of this SAME "
            "villager: identical face, short dark-brown curly hair, warm olive skin, cream knee-length linen tunic, the same royal-blue scarf and "
            "chest band, brown belt, brown sandals, identical proportions, the same fine thin dark-brown ink linework and flat watercolour wash. "
            f"{view} Action, one pose per frame: {action}. The {n} figures stand side by side in a single row, evenly spaced, all the same height "
            "and scale, feet on one shared ground line, with clear white space between them so they do not touch or overlap. Any tool or carried "
            "object stays inside its own frame. Plain pure white background, no shadows, no ground, no frame borders, no numbers, no text.")
def go(k):
    n, view, action = STRIPS[k]
    o = run("fal-ai/nano-banana/edit", {"prompt": prompt(n, view, action), "image_urls": [MASTER], "num_images": 1, "output_format": "png", "aspect_ratio": AR[n]}, 0.0398, "strip:" + k)
    download(o["images"][0]["url"], f"spr/{k}.png"); return k
if __name__ == "__main__":
    with ThreadPoolExecutor(6) as ex:
        for r in ex.map(go, sys.argv[1:] or STRIPS): print(r, flush=True)
