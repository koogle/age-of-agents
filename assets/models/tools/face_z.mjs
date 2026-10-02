import { NodeIO } from '@gltf-transform/core';
import { transformMesh } from '@gltf-transform/functions';
const io = new NodeIO(); const doc = await io.read(process.argv[2]);
const c = Math.cos(-Math.PI / 2), s = Math.sin(-Math.PI / 2);
const m = [c, 0, -s, 0,  0, 1, 0, 0,  s, 0, c, 0,  0, 0, 0, 1]; // column-major rotation about Y by -90deg: +X -> +Z
for (const mesh of doc.getRoot().listMeshes()) transformMesh(mesh, m);
await io.write(process.argv[3], doc); console.log('ok');
