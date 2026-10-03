"""Unit sprites (guard, archer, healer, siege cart): front + back masters, then animation strips."""
import sys, os
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
VIL = data_uri("spr/master_1.png")
STYLE = ("The attached image shows the DRAWING STYLE and costume family of this game (fine thin dark-brown ink linework in the style of Moebius and "
  "Studio Ghibli, restrained flat watercolour wash, cream linen, brown leather, bright royal-blue team colour). Draw ONE full-body game sprite in "
  "exactly that style and at the same scale: ")
TAIL = (" The royal-blue team colour must be strong and clearly visible at small size. Three-quarter view from the front, seen from slightly above like "
  "an RTS camera, standing relaxed. Refined, not cute. Whole figure visible, centered, plain pure white background, no shadow, no ground, no text.")
WHO = {
 "guard": "a sturdy Greek hoplite guard in a cream linen cuirass over a short tunic, a bronze crested helmet, greaves and sandals, a royal-blue cloak, holding a tall spear upright in one hand and a round bronze-rimmed shield painted royal blue in the other.",
 "archer": "a lean young Greek archer in a short cream tunic with a royal-blue scarf and chest band, a leather cap, a quiver of arrows on the back, holding a curved wooden bow in one hand, sandals.",
 "healer": "a calm older Greek healer woman in a long cream robe with a royal-blue mantle over the shoulders, a leather satchel of herbs at her hip, holding a wooden staff with a small bundle of green herbs tied to it, grey hair in a bun, sandals.",
 "siege_cart": "a small wooden siege cart: a light ballista (a large crossbow-like bolt thrower) mounted on a two-wheeled wooden cart with big spoked wheels, iron fittings, rope, a small royal-blue pennant on a pole and royal-blue painted panels. It is a vehicle, not a person.",
}
BACK = ("The attached image shows a game sprite. Draw EXACTLY THE SAME {what}, same clothes or parts, colours, drawing style and scale, but seen from "
  "BEHIND in a three-quarter BACK view (the viewer sees its back{face}), seen from slightly above like an RTS camera. Whole figure, centered, plain "
  "pure white background, no shadow, no text.")
FRONT = "Three-quarter FRONT view (facing the viewer, turned slightly to the viewer's left), seen from slightly above like an RTS camera."
BACKV = "Three-quarter BACK view: moving AWAY from the viewer into the scene, the BACK of the body and cloak facing the viewer, turned only slightly to the viewer's right (NOT a side profile){face}, seen from slightly above like an RTS camera."
GAIT = ("a calm, natural walking stride in four frames: frame 1 left foot forward, frame 2 legs closer together mid-step, frame 3 right foot forward, "
  "frame 4 legs closer together; knees low, no hopping, feet near the ground line")
ACT = {
 "guard": "thrusting the spear forward: frame 1 spear drawn back behind the shield, frame 2 lunging forward, frame 3 spear fully extended",
 "archer": "shooting the bow: frame 1 nocking an arrow, frame 2 drawing the string back to the cheek with the bow raised, frame 3 just released with the bow arm still raised",
 "healer": "healing: frame 1 kneeling and opening the satchel, frame 2 holding out a bundle of herbs with a soft warm golden glow around the hands, frame 3 raising the staff with a soft golden glow",
 "siege_cart": "firing the ballista: frame 1 arms cranked back and a bolt loaded, frame 2 the bow arms snapping forward as the bolt flies off, frame 3 arms forward and the cart rocked back slightly",
}
STRIPS = {"idle_front": (2, FRONT, "standing idle, relaxed; frame 1 neutral, frame 2 a slight breath and weight shift"),
          "idle_back": (2, BACKV, "standing idle, relaxed; frame 1 neutral, frame 2 a slight breath and weight shift"),
          "walk_front": (4, FRONT, GAIT), "walk_back": (4, BACKV, GAIT), "action": (3, FRONT, None)}
CART = {"idle_front": "parked still; frame 1 and frame 2 identical except the small blue pennant fluttering",
        "idle_back": "parked still; frame 1 and frame 2 identical except the small blue pennant fluttering",
        "walk_front": "rolling forward: four frames of the cart moving with the spoked wheels turning a quarter spoke each frame",
        "walk_back": "rolling away: four frames of the cart moving with the spoked wheels turning a quarter spoke each frame"}
AR = {2: "4:3", 3: "16:9", 4: "21:9"}
def master(job):
    who, back = job
    if back:
        face = "" if who == "siege_cart" else ", the face is not visible"
        what = "siege cart" if who == "siege_cart" else "person"
        o = run("fal-ai/nano-banana/edit", {"prompt": BACK.format(what=what, face=face), "image_urls": [data_uri(f"units/{who}_master.png")], "num_images": 1, "output_format": "png", "aspect_ratio": "3:4"}, 0.0398, f"unit:{who}_master_back")
        download(o["images"][0]["url"], f"units/{who}_master_back.png")
    else:
        o = run("fal-ai/nano-banana/edit", {"prompt": STYLE + WHO[who] + TAIL, "image_urls": [VIL], "num_images": 1, "output_format": "png", "aspect_ratio": "3:4"}, 0.0398, f"unit:{who}_master")
        download(o["images"][0]["url"], f"units/{who}_master.png")
    return who, back
def strip(job):
    who, k = job
    n, view, action = STRIPS[k]
    if who == "siege_cart":
        action = CART.get(k, ACT[who])
        view = view.replace("{face}", "")
        subject = "siege cart: identical wooden two-wheeled cart, ballista, iron fittings, ropes, blue pennant and blue panels"
    else:
        action = action or ACT[who]
        view = view.replace("{face}", ", face hidden")
        subject = "person: identical face, hair, clothes, equipment and royal-blue team colour, identical proportions"
    ref = f"units/{who}_master_back.png" if k.endswith("_back") else f"units/{who}_master.png"
    p = (f"The attached image is the IDENTITY and STYLE reference. Draw a horizontal sprite-sheet strip of exactly {n} animation frames of this SAME "
         f"{subject}, the same fine thin dark-brown ink linework and flat watercolour wash. {view} Action, one pose per frame: {action}. The {n} figures "
         "stand side by side in a single row, evenly spaced, all the same height and scale, on one shared ground line, with clear white space between them "
         "so they do not touch or overlap. Every weapon, tool, projectile or effect stays inside its own frame. Plain pure white background, no shadows, "
         "no ground, no frame borders, no vertical dividing lines between frames, no numbers, no text. Every frame shows the complete subject with ALL of its equipment (weapon, shield, bow, staff, satchel, pennant).")
    o = run("fal-ai/nano-banana/edit", {"prompt": p, "image_urls": [data_uri(ref)], "num_images": 1, "output_format": "png", "aspect_ratio": AR[n]}, 0.0398, f"unit:{who}_{k}")
    download(o["images"][0]["url"], f"units/{who}_{k}.png"); return f"{who}_{k}"
if __name__ == "__main__":
    mode = sys.argv[1]; whos = sys.argv[2:] or list(WHO)
    with ThreadPoolExecutor(8) as ex:
        if mode == "front": print(list(ex.map(master, [(w, False) for w in whos])))
        elif mode == "back": print(list(ex.map(master, [(w, True) for w in whos])))
        else:
            ks = [a for a in os.environ.get("ONLY", "").split(",") if a] or list(STRIPS)
            print(list(ex.map(strip, [(w, k) for w in whos for k in ks])))
