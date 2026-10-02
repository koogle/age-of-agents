// Trees and resource nodes are generated illustrated billboards
// (assets/sprites/resources.*). Woods are a small grove whose trees turn to
// stumps as they are cut; other nodes show one sprite stepping through its
// drawn depletion stages.
import * as THREE from 'three';
import { billboard, loadSheet, showFrame } from './billboard.js';
import { random } from './terrain.js';

const sheet = await fetch('/assets/sprites/resources.json').then(response => response.json());
const texture = loadSheet(`/assets/sprites/${sheet.image}`);
const [CELL_W, CELL_H] = sheet.cell;
const NODE_FOR = { food: 'berry', stone: 'stone', gold: 'gold', iron: 'iron', clay: 'clay', fiber: 'fiber' };
const GROVE = [[-0.2, -0.14], [0.2, -0.04], [-0.02, 0.22]];

function seedOf(id) {
  let h = 0;
  for (const ch of id) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return h;
}

function sprite(node, scale = 1) {
  const s = billboard(texture, sheet.size);
  const size = CELL_W * sheet.nodes[node].unitsPerPixel * scale;
  s.center.set(sheet.anchor[0] / CELL_W, 1 - sheet.anchor[1] / CELL_H);
  s.scale.set(size, size * CELL_H / CELL_W, 1);
  showFrame(s, sheet.nodes[node].stages[0]);
  return s;
}

export function createResource(resource, biome) {
  const root = new THREE.Group();
  const seed = seedOf(resource.id);
  const parts = [];
  if (resource.kind === 'wood') {
    // Dark cypress pairs on most land, gnarled olives on the heath.
    const tree = biome === 'heath' ? 'olive' : 'cypress';
    // Three spreading olives share one cell, so each is drawn smaller.
    const base = tree === 'olive' ? 0.62 : 0.85;
    GROVE.forEach(([dx, dz], i) => {
      const scale = base * (0.9 + random(seed + i * 7) * 0.25);
      const standing = sprite(tree, scale);
      const stump = sprite('stump', scale);
      stump.visible = false;
      const spot = new THREE.Group();
      spot.position.set(dx + (random(seed + i) - 0.5) * 0.08, 0, dz + (random(seed - i) - 0.5) * 0.08);
      spot.add(standing, stump);
      root.add(spot);
      parts.push({ standing, stump });
    });
  } else {
    const node = NODE_FOR[resource.kind] || 'stone';
    const s = sprite(node, 0.95 + random(seed) * 0.1);
    s.userData.node = node;
    root.add(s);
    parts.push(s);
  }
  root.userData.parts = parts;
  return root;
}

export function setResourceAmount(root, fraction) {
  const parts = root.userData.parts;
  if (parts[0].standing) {
    const left = fraction <= 0 ? 0 : Math.max(1, Math.ceil(fraction * parts.length));
    parts.forEach(({ standing, stump }, index) => {
      standing.visible = index < left;
      stump.visible = index >= left;
    });
    return;
  }
  const s = parts[0];
  const stages = sheet.nodes[s.userData.node].stages;
  s.visible = fraction > 0;
  if (s.visible) showFrame(s, stages[Math.min(stages.length - 1, Math.floor((1 - fraction) * stages.length))]);
}
