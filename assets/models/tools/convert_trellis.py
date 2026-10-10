#!/usr/bin/env python3
"""Convert Trellis GLB meshes into the client's model blobs, textured by
projecting the building's own sprite frame onto the mesh.

The camera is fixed, so every vertex takes its UV from where it lands in the
sprite at the game view (yaw 45 degrees, pitch 35.26 degrees). Visible
surfaces then show the painted sprite exactly; the mesh supplies depth,
occlusion and the cast shadow. Usage:

    python3 convert_trellis.py <dir with name.glb and name.png sprite frames>
"""
import json, math, struct, sys, warnings
import numpy as np, trimesh
from PIL import Image
warnings.filterwarnings('ignore')
SRC = sys.argv[1].rstrip('/') + '/'
OUT = 'assets/models/'
PITCH = 0.6154797
YAW = math.pi / 4
# Mesh yaw (degrees) at which the Trellis mesh matches the sprite view; chosen
# from docs/verification/shadow-spike azimuth sheets.
AZIMUTH = {'towncenter': 130, 'house': 320, 'watchtower': 40, 'monument': 30}
manifest = {}
for name, az in AZIMUTH.items():
    g = list(trimesh.load(SRC + name + '.glb').geometry.values())[0]
    V = np.asarray(g.vertices, float); F = np.asarray(g.faces, np.uint32)
    th = math.radians(45.0 - az); c, s = math.cos(th), math.sin(th)
    V = V @ np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]]).T
    V[:, 1] -= V[:, 1].min()
    V[:, 0] -= (V[:, 0].min() + V[:, 0].max()) / 2
    V[:, 2] -= (V[:, 2].min() + V[:, 2].max()) / 2
    N = np.asarray(trimesh.Trimesh(V, F, process=False).vertex_normals, np.float32)
    # Project onto the sprite at the game view and fit to its opaque bounds.
    sprite = Image.open(SRC + name + '.png').convert('RGBA')
    alpha = np.array(sprite)[:, :, 3] > 127
    right = np.array([math.cos(YAW), 0, -math.sin(YAW)])
    up = np.array([-math.sin(PITCH) * math.sin(YAW), math.cos(PITCH), -math.sin(PITCH) * math.cos(YAW)])
    x = V @ right; y = V @ up
    ys, xs = np.where(alpha); bx0, bx1, by0, by1 = xs.min(), xs.max(), ys.min(), ys.max()
    scale = min((bx1 - bx0) / (x.max() - x.min()), (by1 - by0) / (y.max() - y.min()))
    ox = (bx0 + bx1) / 2 - (x.max() + x.min()) / 2 * scale
    oy = (by0 + by1) / 2 + (y.max() + y.min()) / 2 * scale
    UV = np.stack([(x * scale + ox) / sprite.width, (-y * scale + oy) / sprite.height], 1).astype(np.float32)
    sprite.save(OUT + name + '.png')
    with open(OUT + name + '.bin', 'wb') as f:
        f.write(struct.pack('<II', len(V), len(F) * 3))
        f.write(np.concatenate([V.astype(np.float32), N, UV], 1).astype(np.float32).tobytes())
        f.write(F.astype(np.uint32).tobytes())
    manifest[name] = {'extent': [round(float(e), 4) for e in V.max(0) - V.min(0)], 'vertices': int(len(V)),
                      'triangles': int(len(F)), 'azimuth_fit': az,
                      'texture': 'sprite frame projected at the game view',
                      'source': 'fal-ai/trellis, see docs/verification/shadow-spike/README.md'}
    print(name, len(V), len(F))
json.dump(manifest, open(OUT + 'models.json', 'w'), indent=1)
