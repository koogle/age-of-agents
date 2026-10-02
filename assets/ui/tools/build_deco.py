from PIL import Image, ImageFilter
import numpy as np

# ---- paper: flatten, soften, make seamless, 512 ----
src = Image.open("deco/paper_c.png").convert("RGB").resize((512,512), Image.LANCZOS)
a = np.asarray(src).astype(np.float32)
low = np.asarray(src.filter(ImageFilter.GaussianBlur(48))).astype(np.float32)
mean = a.reshape(-1,3).mean(0)
target = np.array([240,229,206], np.float32)           # warm cream, matches reference UI panels
flat = target + 0.6*(a - low) * (target/mean)          # keep fibres, drop gradient, faint
h = flat.shape[0]
rolled = np.roll(np.roll(flat, h//2, 0), h//2, 1)
y = np.abs(np.linspace(-1,1,h)); w1 = np.clip((1-y)/0.35, 0, 1)
w = np.minimum.outer(w1, w1)[...,None]                 # 1 in the middle, 0 at the border
tile = np.clip(w*flat + (1-w)*rolled, 0, 255).astype(np.uint8)
Image.fromarray(tile).save("ui/paper_tile.png", optimize=True)

# ---- frame: ink-to-alpha un-composite over white ----
def uncomposite(img):
    c = np.asarray(img.convert("RGB")).astype(np.float32)
    lum = c.min(2)
    al = np.clip((246 - lum)/(246 - 70), 0, 1)
    rgb = np.where(al[...,None] > 0.01, (c - (1-al[...,None])*255)/np.maximum(al[...,None],1e-3), 0)
    return np.dstack([np.clip(rgb,0,255), al*255]).astype(np.uint8)

fr = Image.open("deco/frame_a.png")
O, H = 62, 84                                           # strip origin row and height (rules at 66..143)
P = 41.0
# measure the period precisely over a long span by autocorrelation
ink = (np.asarray(fr.convert("L")) < 140)[96:134, 160:880].mean(0); b = ink - ink.mean()
lags = range(300, 340); ac = [(b[:-k]*b[k:]).mean() for k in lags]
k8 = lags[int(np.argmax(ac))]; P = k8/8; print("period", P)
# find a period start: column inside the meander band with most ink (cell separator) near x=300
x0 = 300 + int(np.argmax((np.asarray(fr.convert("L"))[96:134, 300:342] < 140).mean(0)))
N = 8
strip = fr.crop((x0, O, int(round(x0 + N*P)), O+H))
corner = np.asarray(fr.crop((78, O, 78+H, O+H)).convert("RGB")).copy()
corner[32:74, 67:] = 255                                # drop the partial meander cell after the corner block
iu = np.tril_indices(H, -1)                             # below diagonal (x<y) <- transpose of above
ct = corner.transpose(1,0,2); corner[iu] = ct[iu]

SCALE = 32/H                                            # band 32 px high in the shipped assets
def fin(arr_rgba, w, h): return Image.fromarray(arr_rgba).resize((w,h), Image.LANCZOS)
B = 32; L = int(round(N*P*SCALE))
top = fin(uncomposite(strip), L, B)
cor = fin(uncomposite(Image.fromarray(corner)), B, B)
top.save("ui/border_strip.png", optimize=True)          # horizontal, repeat-x; rotate 90 for verticals

# 9-slice: corners B x B, edges hold exactly 2 strips (8*2 periods) so 'repeat'/'round' tiling stays on-pattern
M = 2*L; S = 2*B + M
nine = Image.new("RGBA", (S, S), (0,0,0,0))
t2 = Image.new("RGBA", (M, B)); t2.paste(top, (0,0)); t2.paste(top, (L,0))
left = t2.transpose(Image.TRANSPOSE)
nine.paste(t2, (B, 0)); nine.paste(t2.transpose(Image.FLIP_TOP_BOTTOM), (B, S-B))
nine.paste(left, (0, B)); nine.paste(left.transpose(Image.FLIP_LEFT_RIGHT), (S-B, B))
nine.paste(cor, (0,0)); nine.paste(cor.transpose(Image.FLIP_LEFT_RIGHT), (S-B,0))
nine.paste(cor.transpose(Image.FLIP_TOP_BOTTOM), (0,S-B)); nine.paste(cor.transpose(Image.ROTATE_180), (S-B,S-B))
nine.save("ui/border_frame.png", optimize=True)
print("strip", top.size, "frame", nine.size, "slice", B)
