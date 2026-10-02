// Villagers are generated illustrated billboards (assets/sprites/villager*.*):
// camera-facing sprites anchored at the feet, picking the front or back
// three-quarter frame from the walking direction relative to the camera and
// mirroring for the other two diagonals.
import * as THREE from 'three';
import { billboard, loadSheet, showFrame } from './billboard.js';
import { patchWorld } from './materials.js';

// Three people share one sheet layout; each villager keeps one by id.
const SHEETS = ['villager', 'villager_woman', 'villager_elder'];
const sheet = await fetch('/assets/sprites/villager.json').then(response => response.json());
const textures = SHEETS.map(name => loadSheet(`/assets/sprites/${name}.png`));
const SHEET_SIZE = [2048, 1280];
const [CELL_W, CELL_H] = sheet.cell;
// The figure is drawn figureHeight px tall inside its cell; in the world it is this tall.
const FIGURE_HEIGHT = 0.78;
const WORLD_CELL = FIGURE_HEIGHT * CELL_H / sheet.figureHeight;
const shadowGeometry = new THREE.CircleGeometry(0.17, 20).rotateX(-Math.PI / 2);
const shadowMaterial = patchWorld(new THREE.MeshBasicMaterial({ color: 0x2a2a1e, transparent: true, opacity: 0.28, depthWrite: false }));

let camera = null;
export function setVillagerCamera(next) {
  camera = next;
}

function seedOf(id) {
  let h = 0;
  for (const ch of id) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return h;
}

export function createVillager(id) {
  const root = new THREE.Group();
  const sprite = billboard(textures[seedOf(id) % textures.length], SHEET_SIZE);
  sprite.center.set(sheet.anchor[0] / CELL_W, 1 - sheet.anchor[1] / CELL_H);
  sprite.scale.set(WORLD_CELL, WORLD_CELL, 1);
  const shadow = new THREE.Mesh(shadowGeometry, shadowMaterial);
  shadow.position.y = 0.015;
  shadow.raycast = () => {};
  root.add(shadow, sprite);
  root.userData.sprite = sprite;
  return root;
}

const forward = new THREE.Vector3();
const right = new THREE.Vector3();
// `facing` is the world yaw the villager looks along (atan2(dx, dz)).
export function poseVillager(root, activity, carrying, facing, time) {
  const sprite = root.userData.sprite;
  let name = activity === 'walk' && carrying ? 'carry' : activity;
  if (!sheet.animations[name]) name = 'idle';
  const direction = new THREE.Vector3(Math.sin(facing), 0, Math.cos(facing));
  camera.getWorldDirection(forward).setY(0).normalize();
  right.crossVectors(forward, camera.up).normalize();
  const towardViewer = direction.dot(forward) < 0;
  const screenRight = direction.dot(right) > 0;
  const facings = sheet.animations[name];
  const view = towardViewer || !facings.back ? 'front' : 'back';
  // Front frames look toward viewer-left, back frames toward viewer-right.
  const mirror = view === 'front' ? screenRight : !screenRight;
  const frames = facings[view];
  showFrame(sprite, frames[Math.floor(time * sheet.fps[name]) % frames.length], mirror);
}
