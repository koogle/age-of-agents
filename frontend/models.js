// Procedural low-poly placeholder models, cel-shaded; ink lines come from the finish pass.
// One cell is one world unit; every model stands on y = 0 at its anchor.
import * as THREE from 'three';
import { paint } from './materials.js';

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

export const ACTIVITY_FOR = { wood: 'chop', stone: 'mine', gold: 'mine', iron: 'mine', clay: 'dig', food: 'forage', fiber: 'forage' };

// Generated models (assets/models/README.md). Only the temple town center is on by
// default, because it is the only one that reads better than its procedural model
// at gameplay zoom; ?glb=towncenter, ?glb=all or
// ?glb=none overrides. Procedural models are the fallback if loading fails.
const GLB_FILES = { towncenter: 'towncenter.glb' };
const GLB_DEFAULT = 'towncenter';
const TOWN_CENTER_WIDTH = 1.8;
const glb = await loadModels(new URLSearchParams(location.search).get('glb') ?? GLB_DEFAULT)
  .catch(error => (console.warn('generated models unavailable, using procedural models', error), {}));

// Loaders are fetched only when a generated model is requested.
async function loadModels(param) {
  const names = param === 'all' ? Object.keys(GLB_FILES) : (param || '').split(',').filter(name => GLB_FILES[name]);
  if (!names.length) return {};
  const [{ GLTFLoader }, { MeshoptDecoder }] = await Promise.all([
    import('./vendor/GLTFLoader.js'), import('./vendor/meshopt_decoder.js')
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
    const scale = TOWN_CENTER_WIDTH / Math.max(extent.x, extent.z);
    assets[name] = { scene: gltf.scene, scale, floor: size.min.y };
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
