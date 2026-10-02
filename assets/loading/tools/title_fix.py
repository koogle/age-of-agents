import numpy as np
from PIL import Image
from scipy import ndimage
raw = np.asarray(Image.open("load/title_ideo1.png").convert("RGB")).astype(np.float32)
cut = np.asarray(Image.open("load/title_cut.png").convert("RGBA")).astype(np.float32)
al = cut[..., 3] / 255
solid = al > 0.5
# Holes in the mask smaller than a letter counter are highlight drop-outs: fill them.
holes = ndimage.binary_fill_holes(solid) & ~solid
lab, n = ndimage.label(holes)
bg0 = np.median(raw[~ndimage.binary_dilation(solid, iterations=6)], axis=0)
# A counter shows flat paper background; a highlight drop-out shows marble. Classify each hole by its colour.
is_counter = [np.abs(raw[lab == i] - bg0).sum(1).mean() < 14 and al[lab == i].mean() < 0.05 for i in range(1, n + 1)]
small = np.isin(lab, [i + 1 for i, c in enumerate(is_counter) if not c])
# Also close thin partial-alpha dents inside letters.
closed = ndimage.binary_closing(solid, structure=np.ones((5, 5)), iterations=2) & ndimage.binary_fill_holes(solid)
print('holes', n, 'counters', sum(is_counter))
counters = holes & ~small
fill = (small | closed) & ~counters
al2 = np.where(fill, 1.0, al)
# Colour: original render (no refine-foreground smearing); un-composite edges against the paper background.
bgc = np.median(raw[~ndimage.binary_dilation(solid, iterations=6)], axis=0)
a3 = np.clip(al2, 1e-3, 1)[..., None]
rgb = np.where(al2[..., None] > 0.98, raw, (raw - (1 - a3) * bgc) / a3)
out = np.dstack([np.clip(rgb, 0, 255), al2 * 255]).astype(np.uint8)
im = Image.fromarray(out, "RGBA")
ys, xs = np.nonzero(al2 > 0.08); pad = 16
im = im.crop((xs.min() - pad, ys.min() - pad, xs.max() + pad, ys.max() + pad))
print("filled px", int(fill.sum() - (solid & fill).sum()), "counters kept", int(counters.sum()), "bg", bgc.round(), im.size)
im.save("load/title_fixed.png")
for name, col in {"dark": (40, 52, 44, 255), "grad": (226, 234, 235, 255)}.items():
    b = Image.new("RGBA", im.size, col); b.alpha_composite(im); b.convert("RGB").save(f"load/title_{name}.png")
