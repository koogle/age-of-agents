"""Cast a rough mesh's shadow on the ground, matched to the painted sprite view.

Finds the camera azimuth (pitch fixed at the game's 35.26 degrees) whose
orthographic silhouette best matches the sprite alpha, then projects the mesh
along the game's shadow sun onto the ground plane and composites the result
under the sprite on a flat green ground.
"""
import sys, math, numpy as np, trimesh
from PIL import Image, ImageDraw
S = '/tmp/claude-0/-home-user-age-of-agents/16660f84-9487-5e5a-b78c-8d6f01e25d5e/scratchpad/spike/'
name = sys.argv[1]
PITCH = 0.6154797
sprite = Image.open(S + f'{name}.png').convert('RGBA')
alpha = np.array(sprite)[:, :, 3] > 127
scene = trimesh.load(S + f'{name}.glb', force='mesh')
V = np.asarray(scene.vertices, dtype=np.float64); F = np.asarray(scene.faces)
V = V - V.mean(0)

def basis(az):
    # Camera direction (toward camera) and screen axes for yaw az, pitch PITCH; y is up.
    toward = np.array([math.sin(az) * math.cos(PITCH), math.sin(PITCH), math.cos(az) * math.cos(PITCH)])
    right = np.array([math.cos(az), 0.0, -math.sin(az)])
    up = np.cross(toward, right); up /= np.linalg.norm(up)
    return toward, right, up

def raster(P, size, fill=255):
    im = Image.new('L', size, 0); d = ImageDraw.Draw(im)
    for f in F:
        d.polygon([tuple(P[i]) for i in f], fill=fill)
    return np.array(im) > 0

def screen(pts, right, up, scale, offset):
    x = pts @ right; y = pts @ up
    return np.stack([x * scale + offset[0], -y * scale + offset[1]], 1)

def fit(az):
    toward, right, up = basis(az)
    x = V @ right; y = V @ up
    # Fit the mesh's screen bounds to the sprite's opaque bounds.
    ys, xs = np.where(alpha); bx0, bx1, by0, by1 = xs.min(), xs.max(), ys.min(), ys.max()
    scale = min((bx1 - bx0) / (x.max() - x.min()), (by1 - by0) / (y.max() - y.min()))
    offset = ((bx0 + bx1) / 2 - (x.max() + x.min()) / 2 * scale, (by0 + by1) / 2 + (y.max() + y.min()) / 2 * scale)
    sil = raster(screen(V, right, up, scale, offset), sprite.size)
    iou = (sil & alpha).sum() / (sil | alpha).sum()
    return iou, scale, offset

best = max(((fit(math.radians(a))[0], a) for a in range(0, 360, 10)))
az = math.radians(best[1]); iou, scale, offset = fit(az)
print(name, 'best azimuth', best[1], 'IoU %.2f' % iou)
toward, right, up = basis(az)
# Game sun: 0.15 toward camera + 0.9 up - 1.0 right.
sun = 0.15 * toward + np.array([0, 0.9, 0]) - 1.0 * right; sun /= np.linalg.norm(sun)
ground = V[:, 1].min()
t = (V[:, 1] - ground) / sun[1]
shadow = V - sun * t[:, None]
# Render on a bigger canvas so the shadow fits.
W, H = sprite.size[0] * 2, sprite.size[1] * 2
off = (offset[0] + sprite.size[0] // 2, offset[1] + sprite.size[1] // 2)
mask = raster(screen(shadow, right, up, scale, off), (W, H))
sil = raster(screen(V, right, up, scale, off), (W, H))
canvas = Image.new('RGBA', (W, H), (168, 196, 96, 255))
shade = Image.new('RGBA', (W, H), (70, 80, 110, 0)); sa = np.zeros((H, W), np.uint8); sa[mask] = 110
shade.putalpha(Image.fromarray(sa)); canvas.alpha_composite(shade)
canvas.alpha_composite(sprite, (sprite.size[0] // 2, sprite.size[1] // 2))
canvas.save(S + f'{name}.mesh_shadow.png')
# Silhouette match debug: sprite alpha (red) vs mesh silhouette (blue).
dbg = np.zeros((H, W, 3), np.uint8); dbg[..., 0] = np.pad(alpha, ((sprite.size[1]//2,)*2, (sprite.size[0]//2,)*2)) * 255; dbg[..., 2] = sil * 255
Image.fromarray(dbg).save(S + f'{name}.mesh_fit.png')
