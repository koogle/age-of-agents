import sys
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
STYLE = ("Drawing style from the FIRST attached image (fine one-pixel dark-brown ink linework inside the shapes, flat two-tone cel fills, warm "
  "sunlight from the upper left, cool teal shadows, no heavy outer outline); palette and subject feel from the SECOND (a sunlit Greek island "
  "diorama). Plain pure white background, no ground, no sea surface, no text. ")
JOBS = {
  "volcano": ("16:9", "Draw ONE distant snow-capped volcano as a game background billboard: a broad, gently sloped cone with a soft snow cap and "
              "a thin wisp of pale smoke at the summit, hazy blue-violet distant colours with a warm lit left flank, isolated, centred, the whole "
              "mountain visible, its base cut off flat and straight at the bottom edge of the shape."),
  "ships":   ("21:9", "Draw THREE different small ancient Mediterranean sailing ships side by side in one row with white space between them, "
              "three-quarter view: a small single-mast boat with a cream square sail, a slightly larger merchant ship with a terracotta sail, "
              "and a slim boat with a striped blue-and-cream sail. Hulls cut off flat at the waterline (no water drawn)."),
  "clouds":  ("21:9", "Draw FOUR different soft puffy white cumulus clouds side by side in one row with white space between them: rounded "
              "cauliflower tops lit warm cream from the upper left, cool pale blue-grey shaded undersides, fine light ink lines, flat bottoms; "
              "two large, two small. Draw them on a plain light-grey background so the white clouds stand out."),
  "fx":      ("21:9", "Draw FOUR small game effect sprites side by side in one row with white space between them, each a small tight cluster: "
              "1) a burst of a few flying light-brown wood chips and splinters, 2) a small puff of pale grey-cream stone dust with a few tiny "
              "pebbles, 3) a few small fluttering olive-green leaves with one red berry, 4) a single soft round puff of pale grey chimney smoke."),
}
REFS = [data_uri("res/trees.png"), data_uri("scen/dio.jpg")]
def go(k):
    ar, what = JOBS[k]
    o = run("fal-ai/nano-banana/edit", {"prompt": STYLE + what, "image_urls": REFS, "num_images": 1, "output_format": "png", "aspect_ratio": ar}, 0.0398, "scen:" + k)
    download(o["images"][0]["url"], f"scen/{k}.png"); return k
with ThreadPoolExecutor(4) as ex: print(list(ex.map(go, sys.argv[1:] or JOBS)))
