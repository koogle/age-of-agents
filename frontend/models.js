// Procedural low-poly placeholder models, cel-shaded; ink lines come from the finish pass.
// One cell is one world unit; every model stands on y = 0 at its anchor.
import * as THREE from 'three';
import { paint } from './materials.js';
import { random } from './terrain.js';

export const TEAM_COLOR = 0x2f6fe0;
const G = {
  box: new THREE.BoxGeometry(1, 1, 1),
  ball: new THREE.IcosahedronGeometry(1, 3),
  rock: new THREE.DodecahedronGeometry(1, 0),
  gem: new THREE.OctahedronGeometry(1, 0),
  cyl: new THREE.CylinderGeometry(1, 1, 1, 8),
  taper: new THREE.CylinderGeometry(0.78, 1, 1, 7),
  cone: new THREE.ConeGeometry(1, 1, 7),
  pyramid: new THREE.ConeGeometry(1, 1, 4),
  ring: new THREE.RingGeometry(0.8, 1, 40).rotateX(-Math.PI / 2)
};

function part(geometry, color, [x, y, z], [sx, sy, sz], options = {}) {
  const mesh = new THREE.Mesh(geometry, typeof color === 'number' ? paint(color, options) : color);
  mesh.position.set(x, y, z);
  mesh.scale.set(sx, sy, sz);
  if (options.rotation) mesh.rotation.set(...options.rotation);
  mesh.castShadow = options.shadow !== false;
  mesh.receiveShadow = true;
  // Linework on everything but tiny details, which would turn to dark specks.
  return mesh;
}
function group(...children) {
  const g = new THREE.Group();
  children.forEach(child => g.add(child));
  return g;
}

// ---------- Villagers ----------
const TUNICS = [0xc8553d, 0x5b8e7d, 0xd9a441, 0x7a6fb0, 0x6c9a3c, 0xb06a8a];
const SKINS = [0xf2c9a0, 0xd8a47a, 0xa8714d, 0xf0d3b5, 0x8a5a3c];
const HAIR = [0x3b2a1e, 0x7a4a26, 0xd9b26a, 0x1e1a18, 0xa0522d];

function seedOf(id) {
  let h = 0;
  for (const ch of id) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return h;
}

const TOOLS = {
  axe: () => group(
    part(G.cyl, 0x8a6036, [0, -0.08, 0], [0.012, 0.26, 0.012]),
    part(G.box, 0xc9ced4, [0.035, -0.2, 0], [0.06, 0.05, 0.012])
  ),
  pick: () => group(
    part(G.cyl, 0x8a6036, [0, -0.08, 0], [0.012, 0.26, 0.012]),
    part(G.box, 0x9aa1a8, [0, -0.2, 0], [0.16, 0.025, 0.02], { rotation: [0, 0, 0.12] })
  ),
  hammer: () => group(
    part(G.cyl, 0x8a6036, [0, -0.06, 0], [0.011, 0.18, 0.011]),
    part(G.box, 0x7d848c, [0, -0.15, 0], [0.07, 0.04, 0.04])
  ),
  shovel: () => group(
    part(G.cyl, 0x8a6036, [0, -0.08, 0], [0.011, 0.28, 0.011]),
    part(G.box, 0xa9b0b6, [0, -0.24, 0.0], [0.06, 0.08, 0.01])
  ),
  basket: () => group(part(G.taper, 0xb98a4e, [0, -0.06, 0.03], [0.06, 0.06, 0.06]))
};
const TOOL_FOR = { wood: 'axe', stone: 'pick', gold: 'pick', iron: 'pick', clay: 'shovel', food: 'basket', fiber: 'basket' };
export const ACTIVITY_FOR = { wood: 'chop', stone: 'mine', gold: 'mine', iron: 'mine', clay: 'dig', food: 'forage', fiber: 'forage' };

const CARGO = {
  wood: () => group(
    part(G.cyl, 0x9a6a3c, [0, 0, -0.02], [0.03, 0.26, 0.03], { rotation: [0, 0, Math.PI / 2] }),
    part(G.cyl, 0x8a5a30, [0, 0.05, 0.02], [0.03, 0.24, 0.03], { rotation: [0, 0, Math.PI / 2] })
  ),
  food: () => group(part(G.taper, 0xb98a4e, [0, 0, 0], [0.07, 0.07, 0.07]), part(G.ball, 0xd1323a, [0, 0.05, 0], [0.05, 0.03, 0.05])),
  stone: () => group(part(G.rock, 0xb4b0a4, [0, 0, 0], [0.07, 0.06, 0.07])),
  gold: () => group(part(G.ball, 0xc9a66b, [0, 0, 0], [0.07, 0.07, 0.07]), part(G.gem, 0xffd54a, [0, 0.07, 0], [0.03, 0.03, 0.03], { emissive: 0x6a4a00 })),
  iron: () => group(part(G.rock, 0x6a5650, [0, 0, 0], [0.07, 0.06, 0.07]), part(G.box, 0xc4683a, [0.02, 0.03, 0.04], [0.04, 0.015, 0.015])),
  clay: () => group(part(G.ball, 0xc06c45, [0, 0, 0], [0.07, 0.05, 0.07])),
  fiber: () => group(part(G.cyl, 0xd8cf7a, [0, 0, 0], [0.04, 0.2, 0.04], { rotation: [0, 0, 1.3] }), part(G.cyl, 0x9a7a4a, [0, 0, 0], [0.042, 0.02, 0.042], { rotation: [0, 0, 1.3] }))
};

// Generated models (assets/models/README.md). Only the temple town center is on by
// default, because it is the only one that reads better than its procedural model
// at gameplay zoom; ?glb=villager,towncenter,cypress picks a set, ?glb=all or
// ?glb=none overrides. Procedural models are the fallback if loading fails.
const GLB_FILES = { villager: 'villager.glb', towncenter: 'towncenter.glb', cypress: 'cypress.glb' };
const GLB_DEFAULT = 'towncenter';
const VILLAGER_HEIGHT = 0.78;
const TOWN_CENTER_WIDTH = 1.8;
const CYPRESS_HEIGHT = 1.1;
const CLIP_FOR = { idle: 'idle', walk: 'walk', chop: 'hammer', mine: 'hammer', build: 'hammer', dig: 'dig', forage: 'forage' };
const glb = await loadModels(new URLSearchParams(location.search).get('glb') ?? GLB_DEFAULT)
  .catch(error => (console.warn('generated models unavailable, using procedural models', error), {}));
const villagerAsset = glb.villager;

// Loaders are fetched only when a generated model is requested.
async function loadModels(param) {
  const names = param === 'all' ? Object.keys(GLB_FILES) : (param || '').split(',').filter(name => GLB_FILES[name]);
  if (!names.length) return {};
  const [{ GLTFLoader }, { MeshoptDecoder }, { clone }] = await Promise.all([
    import('./vendor/GLTFLoader.js'), import('./vendor/meshopt_decoder.js'), import('./vendor/SkeletonUtils.js')
  ]);
  const loader = new GLTFLoader().setMeshoptDecoder(MeshoptDecoder);
  const loaded = await Promise.all(names.map(name => loader.loadAsync(`/assets/models/${GLB_FILES[name]}`)));
  const assets = {};
  names.forEach((name, i) => {
    const gltf = loaded[i];
    // Texture-mapped twin of the shared cel material.
    const meshes = [];
    gltf.scene.traverse(node => node.isMesh && meshes.push(node));
    meshes.forEach(node => {
      node.material = mapped(node.material.map);
      node.castShadow = true;
      node.receiveShadow = true;
      node.raycast = () => {};
    });
    gltf.scene.updateMatrixWorld(true);
    const size = new THREE.Box3().setFromObject(gltf.scene, true);
    const extent = size.getSize(new THREE.Vector3());
    const scale = name === 'towncenter' ? TOWN_CENTER_WIDTH / Math.max(extent.x, extent.z)
      : (name === 'villager' ? VILLAGER_HEIGHT : CYPRESS_HEIGHT) / extent.y;
    assets[name] = { scene: gltf.scene, scale, floor: size.min.y, clone, clips: Object.fromEntries(gltf.animations.map(clip => [clip.name, clip])) };
  });
  return assets;
}

function mapped(map) {
  const base = paint(0xffffff);
  const material = base.clone();
  material.map = map;
  material.onBeforeCompile = base.onBeforeCompile;
  material.customProgramCacheKey = () => 'aoa-world-map';
  return material;
}

// A scaled, grounded copy of a static generated model; geometry and materials are shared.
function placeModel(asset) {
  const model = asset.scene.clone();
  model.scale.multiplyScalar(asset.scale);
  model.position.y = -asset.floor * asset.scale;
  return model;
}

// Holder under `bone` whose bind-pose world transform is `desired` (in villager-root space).
function boneHolder(bone, desired) {
  const holder = new THREE.Group();
  bone.matrixWorld.clone().invert().multiply(desired).decompose(holder.position, holder.quaternion, holder.scale);
  bone.add(holder);
  return holder;
}

function createModelVillager(seed) {
  const root = new THREE.Group();
  const model = villagerAsset.clone(villagerAsset.scene);
  model.scale.multiplyScalar(villagerAsset.scale);
  model.position.y = -villagerAsset.floor * villagerAsset.scale;
  root.add(model);
  model.traverse(node => { if (node.isSkinnedMesh) node.frustumCulled = false; });
  root.updateMatrixWorld(true);
  // Tools and cargo are the procedural props, sized for the procedural villager's 1.3 scale.
  const at = (bone, offset) => new THREE.Matrix4().compose(
    model.getObjectByName(bone).getWorldPosition(new THREE.Vector3()).add(offset),
    new THREE.Quaternion(), new THREE.Vector3(1.3, 1.3, 1.3));
  const hand = boneHolder(model.getObjectByName('RightHand'), at('RightHand', new THREE.Vector3()));
  const back = boneHolder(model.getObjectByName('Spine'), at('Spine', new THREE.Vector3(0, 0, -0.12)));
  const tools = {};
  for (const [name, make] of Object.entries(TOOLS)) {
    tools[name] = make();
    tools[name].visible = false;
    hand.add(tools[name]);
  }
  const cargo = {};
  for (const [name, make] of Object.entries(CARGO)) {
    cargo[name] = make();
    cargo[name].visible = false;
    back.add(cargo[name]);
  }
  const mixer = new THREE.AnimationMixer(model);
  const actions = Object.fromEntries(Object.entries(villagerAsset.clips).map(([name, clip]) => [name, mixer.clipAction(clip)]));
  const phase = random(seed) * 6;
  root.userData.rig = { model: true, mixer, actions, clip: null, tools, cargo, phase, last: null };
  return root;
}

function poseModelVillager(rig, activity, time) {
  const clip = CLIP_FOR[activity] || 'idle';
  if (clip !== rig.clip) {
    const next = rig.actions[clip].reset().fadeIn(rig.clip ? 0.2 : 0).play();
    if (rig.clip) rig.actions[rig.clip].fadeOut(0.2);
    else next.time = rig.phase % next.getClip().duration;
    rig.clip = clip;
  }
  rig.mixer.update(rig.last === null ? 0 : Math.min(0.1, Math.max(0, time - rig.last)));
  rig.last = time;
}

export function createVillager(id) {
  const seed = seedOf(id);
  if (villagerAsset) return createModelVillager(seed);
  const tunic = TUNICS[seed % TUNICS.length];
  const skin = SKINS[(seed >> 3) % SKINS.length];
  const hair = HAIR[(seed >> 6) % HAIR.length];
  const root = new THREE.Group();
  const body = new THREE.Group();
  root.add(body);

  const leg = side => {
    const pivot = new THREE.Group();
    pivot.position.set(side * 0.045, 0.19, 0);
    pivot.add(part(G.box, 0x6b4a2f, [0, -0.08, 0], [0.065, 0.17, 0.075]));
    pivot.add(part(G.box, 0x3a2a1e, [0, -0.175, 0.015], [0.07, 0.04, 0.1]));
    body.add(pivot);
    return pivot;
  };
  const arm = side => {
    const pivot = new THREE.Group();
    pivot.position.set(side * 0.125, 0.39, 0);
    pivot.add(part(G.box, tunic, [0, -0.08, 0], [0.055, 0.17, 0.06]));
    pivot.add(part(G.ball, skin, [0, -0.18, 0], [0.032, 0.032, 0.032]));
    const hand = new THREE.Group();
    hand.position.set(0, -0.18, 0);
    pivot.add(hand);
    body.add(pivot);
    return { pivot, hand };
  };

  const legs = [leg(-1), leg(1)];
  body.add(part(G.taper, tunic, [0, 0.3, 0], [0.11, 0.22, 0.09]));
  body.add(part(G.cyl, 0x5a3b22, [0, 0.215, 0], [0.105, 0.025, 0.085]));
  body.add(part(G.cyl, TEAM_COLOR, [0, 0.415, 0], [0.07, 0.03, 0.06]));
  const head = group(
    part(G.ball, skin, [0, 0, 0], [0.08, 0.085, 0.08]),
    part(G.ball, hair, [0, 0.03, -0.012], [0.084, 0.06, 0.084]),
    part(G.ball, 0x2b1d14, [-0.028, 0.0, 0.072], [0.011, 0.014, 0.008], { shadow: false }),
    part(G.ball, 0x2b1d14, [0.028, 0.0, 0.072], [0.011, 0.014, 0.008], { shadow: false })
  );
  if (seed % 3 === 0) head.add(part(G.cone, 0xe0c071, [0, 0.07, 0], [0.13, 0.06, 0.13]));
  head.position.y = 0.51;
  body.add(head);
  const arms = [arm(-1), arm(1)];

  const back = new THREE.Group();
  back.position.set(0, 0.36, -0.1);
  body.add(back);

  const tools = {};
  for (const [name, make] of Object.entries(TOOLS)) {
    tools[name] = make();
    tools[name].visible = false;
    arms[1].hand.add(tools[name]);
  }
  const cargo = {};
  for (const [name, make] of Object.entries(CARGO)) {
    cargo[name] = make();
    cargo[name].visible = false;
    back.add(cargo[name]);
  }
  root.scale.setScalar(1.3);
  root.userData.rig = { body, legs, arms, head, tools, cargo, phase: random(seed) * 6 };
  return root;
}

// Poses the rig for an activity: idle, walk, chop, mine, dig, forage, build.
export function poseVillager(root, activity, resourceKind, carrying, time) {
  const rig = root.userData.rig;
  const t = time + rig.phase;
  const tool = activity === 'build' ? 'hammer' : activity === 'walk' || activity === 'idle' ? null : TOOL_FOR[resourceKind];
  for (const [name, mesh] of Object.entries(rig.tools)) mesh.visible = name === tool;
  for (const [name, mesh] of Object.entries(rig.cargo)) mesh.visible = name === carrying;
  if (rig.model) return poseModelVillager(rig, activity, time);

  let legSwing = 0, armL = 0, armR = 0, lean = 0, bob = 0, headTilt = 0;
  if (activity === 'walk') {
    const s = Math.sin(t * 11);
    legSwing = s * 0.65;
    armL = -s * 0.5;
    armR = carrying ? -0.3 : s * 0.5;
    bob = Math.abs(Math.cos(t * 11)) * 0.025;
  } else if (activity === 'chop' || activity === 'mine' || activity === 'build') {
    const speed = activity === 'build' ? 9 : 6;
    const cycle = (t * speed) % (Math.PI * 2);
    // Slow wind-up overhead, then a fast strike forward and down.
    const strike = cycle < 4.2 ? -0.5 - 2.1 * Math.sin(cycle / 4.2 * Math.PI / 2) : -2.6 + 2.1 * ((cycle - 4.2) / 2.08);
    armR = activity === 'build' ? -1.2 + Math.max(0, Math.sin(cycle)) * -1.0 : strike;
    armL = activity === 'build' ? -0.6 : strike * 0.6;
    lean = activity === 'build' ? 0.15 : 0.12 + Math.max(0, -Math.sin(cycle)) * 0.12;
  } else if (activity === 'dig') {
    const s = Math.sin(t * 5);
    armR = -0.9 + s * 0.5;
    armL = -0.7 + s * 0.4;
    lean = 0.25 + s * 0.1;
  } else if (activity === 'forage') {
    const s = Math.sin(t * 6);
    lean = 0.45;
    armR = -0.9 + s * 0.4;
    armL = -0.5 - s * 0.3;
    headTilt = 0.2;
  } else {
    bob = Math.sin(t * 2) * 0.006;
    armL = Math.sin(t * 2) * 0.04;
    armR = carrying ? -0.3 : -armL;
    headTilt = Math.sin(t * 0.7) * 0.15;
  }
  rig.legs[0].rotation.x = legSwing;
  rig.legs[1].rotation.x = -legSwing;
  rig.arms[0].pivot.rotation.x = armL;
  rig.arms[1].pivot.rotation.x = armR;
  rig.body.rotation.x = lean;
  rig.body.position.y = bob;
  rig.head.rotation.y = headTilt;
}

// ---------- Resources ----------
// Each resource cell holds a few pieces; pieces disappear as the node depletes.
function tree(conifer, seed) {
  const g = new THREE.Group();
  // Mediterranean pair: tall dark cypress spires, or a gnarled silver-green olive.
  if (conifer && glb.cypress) {
    g.add(placeModel(glb.cypress));
  } else if (conifer) {
    const green = random(seed) > 0.5 ? 0x2f6b3c : 0x3a7a44;
    g.add(part(G.cyl, 0x6b4a2e, [0, 0.06, 0], [0.035, 0.12, 0.035]));
    g.add(part(G.ball, green, [0, 0.5, 0], [0.13, 0.42, 0.13]));
    g.add(part(G.cone, green, [0, 0.95, 0], [0.09, 0.22, 0.09]));
  } else {
    g.add(part(G.cyl, 0x8a7458, [0, 0.14, 0], [0.04, 0.28, 0.04], { rotation: [0, 0, 0.18] }));
    g.add(part(G.ball, 0x8fa860, [0.04, 0.42, 0], [0.22, 0.15, 0.2]));
    g.add(part(G.ball, 0xa9bd78, [-0.07, 0.5, 0.04], [0.14, 0.1, 0.13]));
  }
  g.userData.stump = part(G.cyl, 0x7a5634, [0, 0.03, 0], [0.05, 0.06, 0.05]);
  return g;
}

const PIECE_OFFSETS = [[-0.22, -0.18], [0.2, -0.1], [-0.05, 0.2], [0.24, 0.24]];

export function createResource(resource, biome) {
  const root = new THREE.Group();
  const seed = seedOf(resource.id);
  const pieces = [];
  const add = (object, index, scale = 1) => {
    const [dx, dz] = PIECE_OFFSETS[index];
    object.position.set(dx + (random(seed + index) - 0.5) * 0.08, 0, dz + (random(seed - index) - 0.5) * 0.08);
    object.rotation.y = random(seed * 3 + index) * Math.PI * 2;
    object.scale.multiplyScalar(scale * (0.85 + random(seed + index * 7) * 0.3));
    root.add(object);
    pieces.push(object);
  };
  switch (resource.kind) {
    case 'wood':
      for (let i = 0; i < 3; i += 1) add(tree(biome !== 'heath', seed + i), i, 1 + (i === 0 ? 0.15 : 0));
      break;
    case 'food':
      for (let i = 0; i < 3; i += 1) {
        const bush = group(part(G.ball, 0x4f8f3c, [0, 0.12, 0], [0.17, 0.13, 0.17]));
        for (let b = 0; b < 5; b += 1) {
          const a = b * 1.26 + i;
          bush.add(part(G.ball, 0xe8203a, [Math.cos(a) * 0.13, 0.13 + (b % 2) * 0.06, Math.sin(a) * 0.13], [0.035, 0.035, 0.035]));
        }
        add(bush, i);
      }
      break;
    case 'fiber':
      for (let i = 0; i < 4; i += 1) {
        const tuft = new THREE.Group();
        for (let r = 0; r < 5; r += 1) {
          const a = r * 1.3;
          tuft.add(part(G.cone, 0xb7c46a, [Math.cos(a) * 0.05, 0.15, Math.sin(a) * 0.05], [0.025, 0.32, 0.025], { rotation: [Math.sin(a) * 0.2, 0, Math.cos(a) * 0.2] }));
          tuft.add(part(G.ball, 0xf2e6b0, [Math.cos(a) * 0.07, 0.32, Math.sin(a) * 0.07], [0.025, 0.04, 0.025]));
        }
        add(tuft, i);
      }
      break;
    case 'clay':
      root.add(part(G.cyl, 0x8a4f33, [0, 0.005, 0], [0.4, 0.01, 0.4], { shadow: false }));
      for (let i = 0; i < 3; i += 1) add(group(part(G.ball, 0xc06c45, [0, 0.04, 0], [0.16, 0.09, 0.14])), i);
      break;
    default: {
      const rockColor = { stone: 0xe6e0cc, gold: 0xd8c8a4, iron: 0x8a7a72 }[resource.kind] || 0xaaaaaa;
      for (let i = 0; i < 4; i += 1) {
        const piece = group(part(G.rock, rockColor, [0, 0.1, 0], [0.17, 0.14, 0.15]));
        if (resource.kind === 'gold') piece.add(part(G.gem, 0xffc21a, [0.05, 0.2, 0.03], [0.05, 0.08, 0.05], { emissive: 0x7a5200 }));
        if (resource.kind === 'iron') piece.add(part(G.box, 0xd0703a, [0, 0.16, 0.1], [0.12, 0.03, 0.03], { rotation: [0.3, 0.4, 0] }));
        add(piece, i, i === 0 ? 1.2 : 0.9);
      }
    }
  }
  root.userData.pieces = pieces;
  return root;
}

export function setResourceAmount(root, fraction) {
  const pieces = root.userData.pieces;
  const shown = fraction <= 0 ? 0 : Math.max(1, Math.ceil(fraction * pieces.length));
  pieces.forEach((piece, index) => {
    const alive = index < shown;
    const stump = piece.userData.stump;
    if (stump) {
      piece.children.forEach(child => { if (child !== stump) child.visible = alive; });
      if (!alive && !stump.parent) piece.add(stump);
    } else {
      piece.visible = alive;
    }
  });
}

// ---------- Town center ----------
export function createTownCenter() {
  const root = new THREE.Group();
  const plinth = part(G.box, 0xe8e0cc, [0, 0.06, 0], [1.75, 0.12, 1.75]);
  const walls = new THREE.Group();
  // A small marble hall fronted by a colonnade.
  walls.add(part(G.box, 0xf6f1e4, [0, 0.36, -0.1], [1.15, 0.5, 0.85]));
  walls.add(part(G.box, 0xf6f1e4, [0, 0.64, 0.12], [1.3, 0.07, 1.25]));
  for (const x of [-0.5, -0.17, 0.17, 0.5]) walls.add(part(G.cyl, 0xfbf7ee, [x, 0.36, 0.66], [0.055, 0.5, 0.055]));
  walls.add(part(G.box, 0x4a2f1c, [0, 0.25, 0.331], [0.2, 0.3, 0.03]));
  const windows = [];
  for (const x of [-0.32, 0.32]) {
    const glass = part(G.box, 0x3a3a48, [x, 0.42, 0.331], [0.14, 0.12, 0.02]);
    windows.push(glass);
    walls.add(glass);
  }
  const roof = new THREE.Group();
  roof.add(part(G.pyramid, 0xd0532e, [0, 0.86, 0.12], [1.05, 0.42, 0.98], { rotation: [0, Math.PI / 4, 0] }));
  roof.add(part(G.box, 0xe8e0cc, [0.3, 0.98, -0.2], [0.12, 0.3, 0.12]));
  const tower = new THREE.Group();
  tower.add(part(G.box, 0xefe6d2, [-0.62, 0.5, -0.55], [0.36, 0.95, 0.36]));
  tower.add(part(G.pyramid, 0xd0532e, [-0.62, 1.15, -0.55], [0.32, 0.36, 0.32], { rotation: [0, Math.PI / 4, 0] }));
  tower.add(part(G.cyl, 0x5a3d26, [-0.62, 1.5, -0.55], [0.012, 0.45, 0.012]));
  const flag = part(G.box, TEAM_COLOR, [-0.5, 1.62, -0.55], [0.22, 0.13, 0.01]);
  tower.add(flag);
  const props = group(
    part(G.cyl, 0x8a5a30, [0.72, 0.18, 0.62], [0.08, 0.14, 0.08]),
    part(G.cyl, 0x8a5a30, [0.6, 0.18, 0.72], [0.08, 0.14, 0.08]),
    part(G.box, 0xa87a46, [-0.7, 0.17, 0.66], [0.15, 0.15, 0.15], { rotation: [0, 0.4, 0] }),
    part(G.ball, 0xd8c79a, [0.75, 0.15, -0.2], [0.09, 0.07, 0.09])
  );
  const scaffold = new THREE.Group();
  for (const x of [-0.7, 0, 0.7]) for (const z of [-0.65, 0.7]) scaffold.add(part(G.cyl, 0xb08a58, [x, 0.5, z], [0.02, 0.95, 0.02]));
  for (const y of [0.35, 0.75]) for (const z of [-0.65, 0.7]) scaffold.add(part(G.cyl, 0xb08a58, [0, y, z], [0.015, 1.45, 0.015], { rotation: [0, 0, Math.PI / 2] }));
  scaffold.add(part(G.box, 0xc9a26a, [0.55, 0.16, 0.75], [0.4, 0.05, 0.12]), part(G.box, 0xc9a26a, [0.58, 0.21, 0.73], [0.36, 0.05, 0.12]));
  root.add(plinth, walls, roof, tower, props, scaffold);
  // The generated temple stands in for the finished hall; the procedural plinth, walls and roof
  // still show the construction stages. Yard props are dropped (they would sink into its steps).
  const temple = glb.towncenter ? placeModel(glb.towncenter) : null;
  if (temple) {
    root.add(temple);
    props.clear();
    plinth.visible = walls.visible = roof.visible = false;
  }
  root.userData.parts = { plinth, walls, roof, tower, props, scaffold, windows, flag, temple, chimney: new THREE.Vector3(0.3, 1.18, -0.2) };
  return root;
}

const WINDOW_DARK = paint(0x3a3a48);
const WINDOW_LIT = paint(0xffd27a, { emissive: 0xc07a20 });

export function poseTownCenter(root, construction, working, time) {
  const p = root.userData.parts;
  const progress = construction === null ? 1 : Math.min(1, construction);
  p.walls.scale.y = Math.max(0.05, progress);
  p.tower.scale.y = Math.max(0.05, Math.min(1, progress * 1.2));
  p.roof.visible = progress > 0.7;
  p.roof.scale.setScalar(THREE.MathUtils.smoothstep(progress, 0.7, 1));
  p.props.visible = progress >= 1;
  p.scaffold.visible = progress < 1;
  p.windows.forEach(w => { w.material = working ? WINDOW_LIT : WINDOW_DARK; });
  if (p.temple) {
    p.temple.visible = progress >= 1;
    p.plinth.visible = p.walls.visible = progress < 1;
    p.roof.visible &&= progress < 1;
  }
  p.flag.rotation.y = Math.sin(time * 3) * 0.25;
  p.flag.scale.x = 0.2 + Math.sin(time * 5) * 0.015;
}

// ---------- Markers ----------
export function createRing(radius, color, opacity = 0.9) {
  const material = new THREE.MeshBasicMaterial({ color, transparent: true, opacity, depthWrite: false });
  const ring = new THREE.Mesh(G.ring, material);
  ring.scale.setScalar(radius);
  ring.renderOrder = 2;
  ring.raycast = () => {};
  return ring;
}
