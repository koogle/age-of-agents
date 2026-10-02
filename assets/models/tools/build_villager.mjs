// Merge Meshy clips (same rig) into one GLB and optimize it for the web client.
import { NodeIO } from '@gltf-transform/core';
import { ALL_EXTENSIONS } from '@gltf-transform/extensions';
import { weld, simplify, prune, dedup, resample, quantize, meshopt, sparse } from '@gltf-transform/functions';
import { MeshoptSimplifier, MeshoptEncoder } from 'meshoptimizer';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';

const [dir, out, ratio = '0.7', texSize = '512'] = process.argv.slice(2);
await MeshoptSimplifier.ready; await MeshoptEncoder.ready;
const io = new NodeIO().registerExtensions(ALL_EXTENSIONS).registerDependencies({ 'meshopt.encoder': MeshoptEncoder });
const doc = await io.read(`${dir}/anim_0.glb`);
const root = doc.getRoot();
const buffer = root.listBuffers()[0];
const byName = new Map(root.listNodes().map(n => [n.getName(), n]));
root.listAnimations()[0].setName('idle');

const CLIPS = [['anim_613.glb', 'walk'], ['anim_128.glb', 'hammer'], ['anim_278.glb', 'forage'], ['anim_283.glb', 'dig']];
for (const [file, name] of CLIPS) {
  const src = (await io.read(`${dir}/${file}`)).getRoot().listAnimations()[0];
  const anim = doc.createAnimation(name);
  for (const ch of src.listChannels()) {
    const target = byName.get(ch.getTargetNode().getName());
    if (!target) throw new Error(`missing node ${ch.getTargetNode().getName()}`);
    const s = ch.getSampler();
    const copy = a => doc.createAccessor().setType(a.getType()).setArray(a.getArray().slice()).setBuffer(buffer);
    const sampler = doc.createAnimationSampler().setInput(copy(s.getInput())).setOutput(copy(s.getOutput())).setInterpolation(s.getInterpolation());
    anim.addSampler(sampler).addChannel(doc.createAnimationChannel().setTargetNode(target).setTargetPath(ch.getTargetPath()).setSampler(sampler));
  }
}

// Texture: downsize with Pillow (JPEG q86), keeps the dependency list short.
for (const tex of root.listTextures()) {
  fs.writeFileSync('/tmp/_tex_in.jpg', tex.getImage());
  execFileSync('python3', ['-c', `from PIL import Image; Image.open('/tmp/_tex_in.jpg').convert('RGB').resize((${texSize},${texSize}), Image.LANCZOS).save('/tmp/_tex_out.jpg', quality=86, optimize=True)`]);
  tex.setImage(fs.readFileSync('/tmp/_tex_out.jpg')).setMimeType('image/jpeg');
}
for (const m of root.listMaterials()) m.setDoubleSided(false);

// Clips are played in place: pin the hips' horizontal position to the idle rest pose.
const hipsTrack = a => a.listChannels().find(c => c.getTargetNode().getName() === 'Hips' && c.getTargetPath() === 'translation').getSampler().getOutput();
const rest = hipsTrack(root.listAnimations()[0]).getArray();
for (const anim of root.listAnimations()) {
  const out = hipsTrack(anim); const v = out.getArray().slice();
  for (let i = 0; i < v.length; i += 3) { v[i] = rest[0]; v[i + 2] = rest[2]; }
  out.setArray(v);
}

// Drop channels that hold the node's rest value for the whole clip (scale, most bone translations).
let dropped = 0;
for (const anim of root.listAnimations()) {
  for (const ch of anim.listChannels()) {
    const node = ch.getTargetNode(); const path = ch.getTargetPath();
    const restValue = path === 'translation' ? node.getTranslation() : path === 'rotation' ? node.getRotation() : node.getScale();
    const v = ch.getSampler().getOutput().getArray(); const n = restValue.length;
    let same = true;
    for (let i = 0; i < v.length && same; i++) same = Math.abs(v[i] - restValue[i % n]) < 1e-3 * Math.max(1, Math.abs(restValue[i % n]));
    if (same) { const s = ch.getSampler(); ch.dispose(); s.dispose(); dropped++; }
  }
}
console.log('dropped constant channels:', dropped);

await doc.transform(
  dedup(), prune(),
  weld(),
  simplify({ simplifier: MeshoptSimplifier, ratio: Number(ratio), error: 0.002 }),
  resample({ tolerance: Number(process.env.RESAMPLE_TOL || 1e-4) }),
  prune({ keepAttributes: false }),
  sparse(),
  quantize({ quantizePosition: 14, quantizeNormal: 8, quantizeTexcoord: 12, quantizeWeight: 8 }),
  meshopt({ encoder: MeshoptEncoder, level: 'high' })
);
await io.write(out, doc);
const tris = root.listMeshes()[0].listPrimitives()[0].getIndices().getCount() / 3;
console.log(out, fs.statSync(out).size, 'bytes,', tris, 'tris,', root.listAnimations().map(a => a.getName()).join('/'));
