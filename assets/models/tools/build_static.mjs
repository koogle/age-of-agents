// Optimize a static Tripo GLB: face +Z, smaller texture, simplify, quantize, meshopt.
import { NodeIO } from '@gltf-transform/core';
import { ALL_EXTENSIONS } from '@gltf-transform/extensions';
import { weld, simplify, prune, dedup, quantize, meshopt, transformMesh } from '@gltf-transform/functions';
import { MeshoptSimplifier, MeshoptEncoder } from 'meshoptimizer';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';

const [input, out, ratio = '1', texSize = '512', turnDeg = '-90', remap = ''] = process.argv.slice(2);
await MeshoptSimplifier.ready; await MeshoptEncoder.ready;
const io = new NodeIO().registerExtensions(ALL_EXTENSIONS).registerDependencies({ 'meshopt.encoder': MeshoptEncoder });
const doc = await io.read(input);
const root = doc.getRoot();
// Tripo exports characters facing +X; the game's models face +Z. turnDeg is the Y rotation to bake in.
const c = Math.cos(Number(turnDeg) * Math.PI / 180), s = Math.sin(Number(turnDeg) * Math.PI / 180);
for (const mesh of root.listMeshes()) transformMesh(mesh, [c, 0, -s, 0, 0, 1, 0, 0, s, 0, c, 0, 0, 0, 0, 1]);
for (const tex of root.listTextures()) {
  fs.writeFileSync('/tmp/_tex_in.img', tex.getImage());
  // Optional remap onto the procedural palette, keeping each texel's relative shading (Tripo bakes dull colours):
  // "marble": neutral (stone) texels become marble white, saturated ones (terracotta) get a milder lift;
  // "foliage": green texels become cypress green, the brown trunk is left alone.
  execFileSync('python3', ['-c', `
import numpy as np
from PIL import Image
a = np.asarray(Image.open('/tmp/_tex_in.img').convert('RGB').resize((${texSize},${texSize}), Image.LANCZOS)).astype(np.float32)
if '${remap}' == 'marble':
    mx, mn = a.max(2), a.min(2); sat = (mx - mn) / np.maximum(mx, 1)
    k = np.clip((sat - 0.12) / 0.25, 0, 1)[..., None]
    lum = a.mean(2, keepdims=True); ref = np.median(lum[k[..., 0] < 0.5])
    stone = np.array([246, 241, 228], np.float32) * np.clip(lum / ref, 0, 1.15)
    a = stone * (1 - k) + a * 1.15 * k
if '${remap}' == 'foliage':
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    k = np.clip((g - np.maximum(r * 0.8, b) ) / 12, 0, 1)[..., None]
    lum = a.mean(2, keepdims=True); ref = np.median(lum[k[..., 0] > 0.5])
    leaf = np.array([52, 116, 62], np.float32) * np.clip(lum / ref, 0.4, 1.6)
    a = leaf * k + a * (1 - k)
Image.fromarray(np.clip(a, 0, 255).astype(np.uint8)).save('/tmp/_tex_out.jpg', quality=86, optimize=True)`]);
  tex.setImage(fs.readFileSync('/tmp/_tex_out.jpg')).setMimeType('image/jpeg');
}
for (const m of root.listMaterials()) m.setDoubleSided(false);
await doc.transform(
  dedup(), prune(), weld(),
  simplify({ simplifier: MeshoptSimplifier, ratio: Number(ratio), error: 0.002 }),
  prune({ keepAttributes: false }),
  quantize({ quantizePosition: 14, quantizeNormal: 8, quantizeTexcoord: 12 }),
  meshopt({ encoder: MeshoptEncoder, level: 'high' })
);
await io.write(out, doc);
const tris = root.listMeshes().flatMap(m => m.listPrimitives()).reduce((n, p) => n + p.getIndices().getCount() / 3, 0);
console.log(out, fs.statSync(out).size, 'bytes,', tris, 'tris');
