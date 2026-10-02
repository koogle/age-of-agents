"""Final title: ESRGAN 2x colour, alpha = 2x BiRefNet bounded by the (dilated) repaired 1x alpha."""
import numpy as np, os
from PIL import Image
from scipy import ndimage
exec(open("title_fix.py").read().split("ys, xs = np.nonzero(al2")[0]); a1 = al2
exec(open("title_fix_x2.py").read().split("ys, xs = np.nonzero(al2")[0]); a2, rgb2 = al2, rgb
up = np.asarray(Image.fromarray((a1 * 255).astype(np.uint8)).resize((a2.shape[1], a2.shape[0]), Image.BICUBIC)).astype(np.float32) / 255
bound = ndimage.maximum_filter(up, size=5)            # 1x shape, grown 2 px so 2x edges stay crisp
a = np.minimum(a2, bound)
print("alpha removed (sum)", round(float((a2 - a).sum() / 255 * 255)))
out = np.dstack([np.clip(rgb2, 0, 255), a * 255]).astype(np.uint8)
im = Image.fromarray(out, "RGBA"); ys, xs = np.nonzero(a > 0.08)
im = im.crop((xs.min() - 32, ys.min() - 32, xs.max() + 32, ys.max() + 32))
im = im.resize((1600, round(im.height * 1600 / im.width)), Image.LANCZOS)
im.save("load/title_final.png"); im.save("load/title.webp", quality=88, method=6)
for n, c in {"dark": (40, 52, 44, 255), "grad": (226, 234, 235, 255)}.items():
    b = Image.new("RGBA", im.size, c); b.alpha_composite(im); b.convert("RGB").save(f"load/title_final_{n}.png")
print(im.size, os.path.getsize("load/title.webp"))
