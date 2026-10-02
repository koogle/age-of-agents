// Entry point: renderer, lighting, and the mapping from player intent to typed
// server commands. The server decides every outcome; this file only asks.
import * as THREE from 'three';
import { MAP, uniforms } from './materials.js';
import { createTerrain } from './terrain.js';
import { createSky } from './sky.js';
import { createTiltShift } from './tilt-shift.js';
import { createBuildGhost, createWorldView } from './world-view.js';
import { setVillagerCamera } from './villager-sprite.js';
import { createEffects } from './effects.js';
import { bindPointer, createCameraRig } from './controls.js';
import { createHud } from './hud.js';
import { connect } from './net.js';

const canvas = document.getElementById('world');
const mobile = matchMedia('(pointer: coarse)').matches;
const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
renderer.setPixelRatio(Math.min(devicePixelRatio || 1, mobile ? 1.75 : 2));
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.VSMShadowMap;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.0;

const scene = new THREE.Scene();
scene.background = skyTexture();
scene.fog = new THREE.Fog(0xcfe5f2, 26, 62);
const camera = new THREE.PerspectiveCamera(36, 1, 0.1, 200);

// Warm late-afternoon sun with a soft sky fill, like a lit tabletop model.
scene.add(new THREE.HemisphereLight(0x9fc8ff, 0x8c7f6a, 1.25));
const sun = new THREE.DirectionalLight(0xfff0d4, 3.0);
sun.castShadow = true;
sun.shadow.mapSize.setScalar(mobile ? 1024 : 2048);
Object.assign(sun.shadow.camera, { left: -16, right: 16, top: 16, bottom: -16, near: 1, far: 60 });
sun.shadow.bias = -0.0008;
sun.shadow.radius = 4;
sun.shadow.intensity = 0.8;
sun.shadow.blurSamples = 12;
sun.shadow.normalBias = 0.02;
scene.add(sun, sun.target);

const terrain = createTerrain(scene);
const effects = createEffects(scene, camera);
const sky = createSky(scene);
const tiltShift = createTiltShift(renderer);
setVillagerCamera(camera);
const view = createWorldView(scene, effects);
const ghost = createBuildGhost(scene);
const rig = createCameraRig(camera);
const raycaster = new THREE.Raycaster();
const pointerNdc = new THREE.Vector2();

const selection = { units: new Set(), building: null };
let world = null;
let buildMode = false;
let cameraPlaced = false;

const hud = createHud(renderer, {
  onSpeed: multiplier => order({ type: 'set_simulation_speed', multiplier }),
  onReset: async () => {
    if (!confirm('Reset the world? All progress will be lost.')) return;
    const response = await fetch('/reset', { method: 'POST' }).catch(() => null);
    if (!response?.ok) hud.toast('The world could not be reset.');
    selection.units.clear();
    selection.building = null;
  },
  onTrain: () => order({ type: 'produce', building_id: selection.building, product: 'villager' }),
  onResearch: technology => order({ type: 'research', building_id: selection.building, technology }),
  onBuild: () => setBuildMode(true),
  onCancel: () => setBuildMode(false),
  onMinimap: ({ x, z }) => rig.lookAt(x, z)
});

const net = connect({
  onStatus: online => hud.setConnection(online),
  onSnapshot: next => {
    world = next;
    view.sync(world, performance.now());
    terrain.update(world);
    const present = new Set(world.units.map(unit => unit.id));
    for (const id of selection.units) if (!present.has(id)) selection.units.delete(id);
    if (selection.building && !world.buildings.some(b => b.id === selection.building)) selection.building = null;
    if (!cameraPlaced && world.units.length) {
      const sum = world.units.reduce((acc, unit) => ({ x: acc.x + unit.position.x, z: acc.z + unit.position.y }), { x: 0, z: 0 });
      rig.lookAt(sum.x / world.units.length, sum.z / world.units.length - 1);
      if (innerWidth < innerHeight) rig.zoom(1.4);
      cameraPlaced = true;
    }
    hud.update(world, selection, buildMode);
  }
});

async function order(command) {
  const result = await net.send(command);
  if (!result.ok) hud.toast(result.error);
  return result.ok;
}

function setBuildMode(enabled) {
  buildMode = enabled && selection.units.size > 0;
  uniforms.uGrid.value = buildMode ? 1 : 0;
  if (!buildMode) ghost.hide();
  if (world) hud.update(world, selection, buildMode);
}

function cellKey(column, row) {
  return `${column},${row}`;
}

// Cells the client knows are taken; used only to preview placement and to keep
// decorations off resources and buildings. The server re-validates everything.
function blockedCells(snapshot, includeUnits) {
  const blocked = new Set();
  for (const resource of snapshot.resources) if (resource.amount > 0) blocked.add(cellKey(resource.cell.column, resource.cell.row));
  for (const building of snapshot.buildings) {
    for (let dz = 0; dz < building.rows; dz += 1) {
      for (let dx = 0; dx < building.columns; dx += 1) blocked.add(cellKey(building.origin.column + dx, building.origin.row + dz));
    }
  }
  if (includeUnits) {
    for (const unit of snapshot.units) {
      blocked.add(cellKey(unit.cell.column, unit.cell.row));
      if (unit.step) blocked.add(cellKey(unit.step.to.column, unit.step.to.row));
      if (unit.action.type === 'move') blocked.add(cellKey(unit.action.to.column, unit.action.to.row));
    }
  }
  return blocked;
}

function buildOrigin(point) {
  return { column: Math.round(point.x) - 1, row: Math.round(point.z) - 1 };
}

function siteLooksFree(origin) {
  const blocked = blockedCells(world, true);
  for (let dz = 0; dz < 2; dz += 1) {
    for (let dx = 0; dx < 2; dx += 1) {
      const column = origin.column + dx;
      const row = origin.row + dz;
      if (column < 0 || row < 0 || column >= MAP.columns || row >= MAP.rows) return false;
      if (blocked.has(cellKey(column, row)) || world.terrain[row * MAP.columns + column].visibility === 'unseen') return false;
    }
  }
  return true;
}

function pick(x, y) {
  pointerNdc.set(x / innerWidth * 2 - 1, -(y / innerHeight) * 2 + 1);
  raycaster.setFromCamera(pointerNdc, camera);
  // Villagers win over the buildings and props they stand beside.
  const hits = raycaster.intersectObjects(view.pickables(), false);
  const hit = hits.find(h => h.object.userData.pick.type === 'unit') || hits[0];
  if (hit) return hit.object.userData.pick;
  const point = rig.groundAt(x, y);
  if (!point || point.x < 0 || point.z < 0 || point.x >= MAP.columns || point.z >= MAP.rows) return null;
  // A tap on an occupied cell means its occupant, even if the model was missed.
  const cell = { column: Math.floor(point.x), row: Math.floor(point.z) };
  const resource = world.resources.find(r => r.amount > 0 && r.cell.column === cell.column && r.cell.row === cell.row);
  if (resource) return { type: 'resource', id: resource.id };
  const building = world.buildings.find(b => cell.column >= b.origin.column && cell.column < b.origin.column + b.columns &&
    cell.row >= b.origin.row && cell.row < b.origin.row + b.rows);
  if (building) return { type: 'building', id: building.id };
  return { type: 'ground', point, cell };
}

function idleSelected() {
  return world.units.filter(unit => selection.units.has(unit.id) && unit.action.type === 'idle').map(unit => unit.id);
}

async function orderEach(unitIds, makeCommand, marker) {
  const results = await Promise.all(unitIds.map(id => net.send(makeCommand(id))));
  const failure = results.find(result => !result.ok);
  if (failure) hud.toast(failure.error);
  if (results.some(result => result.ok) && marker) view.markTarget(marker.x, marker.z, marker.color, uniforms.uTime.value);
}

function commandSelection(hit) {
  const idle = idleSelected();
  if (idle.length === 0) {
    hud.toast('unit is busy');
    return;
  }
  if (idle.length < selection.units.size) hud.toast(`${selection.units.size - idle.length} busy villager(s) kept working.`);
  if (hit.type === 'resource') {
    const resource = world.resources.find(r => r.id === hit.id);
    orderEach(idle, unit_id => ({ type: 'gather', unit_id, resource_id: hit.id }),
      { x: resource.cell.column + 0.5, z: resource.cell.row + 0.5, color: 0xffd36a });
  } else if (hit.type === 'building') {
    const building = world.buildings.find(b => b.id === hit.id);
    orderEach(idle, unit_id => ({ type: 'construct', unit_id, building_id: hit.id }),
      { x: building.origin.column + 1, z: building.origin.row + 1, color: 0xffd36a });
  } else {
    const command = idle.length === 1
      ? { type: 'move', unit_id: idle[0], to: hit.cell }
      : { type: 'group_move', unit_ids: idle, to: hit.cell };
    order(command).then(ok => {
      view.markTarget(hit.cell.column + 0.5, hit.cell.row + 0.5, ok ? 0x9fe07a : 0xe0604a, uniforms.uTime.value);
    });
  }
}

function tap(x, y, additive) {
  if (!world) return;
  if (buildMode) {
    // Placement aims at the ground under the pointer; the server judges the site.
    const point = rig.groundAt(x, y);
    if (!point) return;
    const [builder] = idleSelected();
    if (!builder) {
      hud.toast('unit is busy');
      return;
    }
    const origin = buildOrigin(point);
    order({ type: 'build', unit_id: builder, origin }).then(ok => { if (ok) setBuildMode(false); });
    return;
  }
  const hit = pick(x, y);
  if (!hit) return;
  if (hit.type === 'unit') {
    selection.building = null;
    if (additive) {
      if (selection.units.has(hit.id)) selection.units.delete(hit.id);
      else selection.units.add(hit.id);
    } else {
      selection.units = new Set([hit.id]);
    }
  } else if (selection.units.size && (hit.type === 'resource' || hit.type === 'ground' ||
    (hit.type === 'building' && world.buildings.find(b => b.id === hit.id)?.construction !== null))) {
    commandSelection(hit);
  } else if (hit.type === 'building') {
    selection.units.clear();
    selection.building = hit.id;
  } else if (hit.type === 'ground') {
    selection.building = null;
  }
  hud.update(world, selection, buildMode);
}

const projected = new THREE.Vector3();
const controls = bindPointer(canvas, rig, {
  tap,
  ui: hud.pointer,
  boxDraw: area => hud.box(area),
  boxSelect(rect) {
    if (!world) return;
    const chosen = view.unitPositions().filter(({ position }) => {
      projected.copy(position).setY(position.y + 0.3).project(camera);
      const sx = (projected.x + 1) / 2 * innerWidth;
      const sy = (1 - projected.y) / 2 * innerHeight;
      return sx >= rect.left && sx <= rect.right && sy >= rect.top && sy <= rect.bottom;
    });
    selection.units = new Set(chosen.map(entry => entry.id));
    selection.building = null;
    hud.update(world, selection, buildMode);
  },
  hover(x, y) {
    if (!world) return;
    if (buildMode) {
      const point = rig.groundAt(x, y);
      if (point) {
        const origin = buildOrigin(point);
        ghost.show(origin, siteLooksFree(origin));
      }
      canvas.style.cursor = 'crosshair';
      return;
    }
    const hit = pick(x, y);
    canvas.style.cursor = hit && hit.type !== 'ground' ? 'pointer' : 'default';
  },
  key(event) {
    if (event.code === 'Escape') {
      if (buildMode) setBuildMode(false);
      else {
        selection.units.clear();
        selection.building = null;
        if (world) hud.update(world, selection, buildMode);
      }
    }
    if (event.code === 'KeyB' && (buildMode || hud.canBuild())) setBuildMode(!buildMode);
  }
});

function resize() {
  renderer.setSize(innerWidth, innerHeight, false);
  hud.resize(innerWidth, innerHeight, renderer.getPixelRatio());
  tiltShift.setSize(innerWidth, innerHeight);
  camera.aspect = innerWidth / innerHeight;
  // Portrait phones need a wider lens to see a useful slice of the island.
  camera.fov = camera.aspect < 1 ? 52 : 36;
  camera.updateProjectionMatrix();
}
addEventListener('resize', resize);
resize();

function skyTexture() {
  const sky = document.createElement('canvas');
  sky.width = 2;
  sky.height = 256;
  const context = sky.getContext('2d');
  const gradient = context.createLinearGradient(0, 0, 0, 256);
  gradient.addColorStop(0, '#3a8ad4');
  gradient.addColorStop(0.45, '#79bde9');
  gradient.addColorStop(0.8, '#cfe8f4');
  gradient.addColorStop(1, '#f4ecd6');
  context.fillStyle = gradient;
  context.fillRect(0, 0, 2, 256);
  const texture = new THREE.CanvasTexture(sky);
  texture.colorSpace = THREE.SRGBColorSpace;
  return texture;
}

// Read-only hook for automated browser checks: where an entity is on screen.
window.ageOfAgents = {
  get world() { return world; },
  screenOf(x, z, lift = 0.3) {
    projected.set(x, lift, z).project(camera);
    return { x: (projected.x + 1) / 2 * innerWidth, y: (1 - projected.y) / 2 * innerHeight };
  },
  lookAt: (x, z) => rig.lookAt(x, z),
  get target() { return { x: rig.target.x, z: rig.target.z }; },
  unitPositions: () => view.unitPositions().map(u => ({ id: u.id, x: u.position.x, z: u.position.z }))
};

const viewCorners = [[0, 0], [1, 0], [1, 1], [0, 1]];
let previous = performance.now();
let minimapAt = 0;
renderer.setAnimationLoop(now => {
  const dt = Math.min(0.1, (now - previous) / 1000);
  previous = now;
  const time = now / 1000;
  uniforms.uTime.value = time;
  controls.update(dt);
  rig.update(dt);
  scene.fog.near = rig.state.distance + 8;
  scene.fog.far = rig.state.distance * 2 + 60;
  uniforms.uCurve.value = THREE.MathUtils.smoothstep(rig.state.distance, 24, 70) * 0.014;
  uniforms.uCurveCenter.value.set(rig.target.x, rig.target.z);
  // Shadow maps are not bent with the world, so the planet view goes without.
  sun.castShadow = uniforms.uCurve.value < 0.0005;
  sun.position.set(rig.target.x - 11, 15, rig.target.z + 7);
  sun.target.position.copy(rig.target);
  view.frame(now, time, dt, selection);
  effects.update(dt);
  sky.update(time);
  if (world && now > minimapAt) {
    minimapAt = now + 200;
    hud.minimap(viewCorners.map(([u, v]) => rig.groundAt(u * innerWidth, v * innerHeight)));
  }
  tiltShift.render(scene, camera);
  hud.render(now);
});
