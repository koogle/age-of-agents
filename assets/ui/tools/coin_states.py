"""Four coin button states from one generated blank medallion (assets/ui/buttons)."""
import numpy as np, colorsys, sys, os
from PIL import Image, ImageFilter
src = np.asarray(Image.open(sys.argv[1]).convert("RGB")).astype(np.float32)
out = sys.argv[2]; os.makedirs(out, exist_ok=True)
H, W, _ = src.shape
ink = (255 - src.min(2)) > 25                                    # anything not white paper
ys, xs = np.nonzero(ink)
cx, cy = (xs.min() + xs.max()) / 2, (ys.min() + ys.max()) / 2
R = ((xs.max() - xs.min()) + (ys.max() - ys.min())) / 4
yy, xx = np.mgrid[0:H, 0:W]; rr = np.hypot(xx - cx, yy - cy)
mx, mn = src.max(2), src.min(2); sat = (mx - mn) / np.maximum(mx, 1)
prof = [sat[(rr >= r) & (rr < r + 1)].mean() for r in range(int(R * 0.5), int(R))]
# face/rim boundary: first radius from the centre where saturation jumps above the face level
face_level = np.median(prof[: len(prof) // 3]); edge = next(i for i, v in enumerate(prof) if v > face_level + 0.12)
Rf = R * 0.5 + edge - 1
print(f"centre {cx:.1f},{cy:.1f} R {R:.1f} face R {Rf:.1f} ({Rf / R:.3f})")

SIZE, DIAM = 256, 224                                            # 16 px margin for the hover glow
s = DIAM / (2 * R); crop = int(round(SIZE / s))
x0, y0 = int(round(cx - crop / 2)), int(round(cy - crop / 2))
img = Image.fromarray(src.astype(np.uint8)).crop((x0, y0, x0 + crop, y0 + crop)).resize((SIZE, SIZE), Image.LANCZOS)
rgb = np.asarray(img).astype(np.float32)
g = np.mgrid[0:SIZE, 0:SIZE]; r = np.hypot(g[1] + 0.5 - SIZE / 2, g[0] + 0.5 - SIZE / 2)
Ro, Ri = DIAM / 2, Rf * s
alpha = np.clip(Ro - r + 0.5, 0, 1)                              # analytic antialiased edge
rim = np.clip(r - Ri + 0.5, 0, 1)[..., None]                     # 1 on the rim, 0 on the face

def save(name, rgb, a):
    Image.fromarray(np.dstack([np.clip(rgb, 0, 255), np.clip(a, 0, 1) * 255]).astype(np.uint8)).save(f"{out}/coin_{name}.png", optimize=True)

save("normal", rgb, alpha)

# hover: warmer, brighter rim and a soft warm glow outside the coin
warm = rgb * np.array([1.10, 1.07, 1.0]) + 10
hov = rgb * (1 - rim) + np.minimum(warm, 255) * rim
glow = Image.fromarray((alpha * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(7))
ga = np.asarray(glow).astype(np.float32) / 255 * 0.55
glow_rgb = np.array([255, 205, 120], np.float32)
a_h = alpha + ga * (1 - alpha)
hov_rgb = (hov * alpha[..., None] + glow_rgb * (ga * (1 - alpha))[..., None]) / np.maximum(a_h, 1e-4)[..., None]
save("hover", hov_rgb, a_h)

# pressed: darker rim, face shaded from the upper-left inner edge, whole coin 3% smaller
dark = rgb * (1 - rim) + rgb * 0.82 * rim
d = np.clip((Ri - r) / (Ri * 0.35), 0, 1)                        # 0 at the face edge, 1 inside
dirn = ((g[1] + 0.5 - SIZE / 2) + (g[0] + 0.5 - SIZE / 2)) / (np.sqrt(2) * Ri)   # -1 upper-left .. +1 lower-right
inset = (1 - d) * np.clip(0.55 - 0.45 * dirn, 0, 1) * 0.22 * (1 - rim[..., 0])
dark = dark * (1 - inset[..., None]) - 4 * (1 - rim)
p = Image.fromarray(np.dstack([np.clip(dark, 0, 255), alpha * 255]).astype(np.uint8)).resize((int(SIZE * 0.97),) * 2, Image.LANCZOS)
pc = Image.new("RGBA", (SIZE, SIZE)); pc.paste(p, ((SIZE - p.width) // 2,) * 2); pc.save(f"{out}/coin_pressed.png", optimize=True)

# disabled: mostly desaturated, lifted, faded
lum = (rgb @ np.array([0.299, 0.587, 0.114]))[..., None]
dis = (rgb * 0.15 + lum * 0.85) * 0.8 + 255 * 0.2
save("disabled", dis, alpha * 0.6)
