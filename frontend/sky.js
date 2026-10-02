// Diorama scenery beyond the playable island: puffy cumulus clouds drifting
// over the sea, a distant snow-capped volcano, and a few sailing ships.
// Decoration only; none of it can be picked or affects play.
import * as THREE from 'three';
import { MAP, paint, patchWorld } from './materials.js';
import { SEA_LEVEL, random } from './terrain.js';

const puff = new THREE.IcosahedronGeometry(1, 3);

function cloud(seed) {
  const group = new THREE.Group();
  const material = patchWorld(new THREE.MeshStandardMaterial({ color: 0xfff6ea, roughness: 1, emissive: 0x3a3028, emissiveIntensity: 0.35 }));
  const count = 6 + Math.floor(random(seed) * 5);
  for (let i = 0; i < count; i += 1) {
    const mesh = new THREE.Mesh(puff, material);
    const r = 0.9 + random(seed + i * 3) * 1.1;
    mesh.position.set((i - count / 2) * 0.9 + random(seed + i) * 0.6, random(seed - i) * 0.9 + r * 0.4, (random(seed * 2 + i) - 0.5) * 1.6);
    mesh.scale.setScalar(r);
    mesh.castShadow = true;
    group.add(mesh);
  }
  return group;
}

function volcano() {
  const group = new THREE.Group();
  const slope = new THREE.Mesh(new THREE.ConeGeometry(26, 20, 48, 6), paint(0x9aa49a));
  slope.position.y = 10 + SEA_LEVEL;
  const snow = new THREE.Mesh(new THREE.ConeGeometry(8.4, 6.6, 48), paint(0xf4f2ee));
  snow.position.y = 16.8 + SEA_LEVEL;
  const foothills = new THREE.Mesh(new THREE.ConeGeometry(34, 6, 40), paint(0x7e9a6a));
  foothills.position.y = 3 + SEA_LEVEL;
  group.add(foothills, slope, snow);
  return group;
}

function ship(seed) {
  const group = new THREE.Group();
  const hull = new THREE.Mesh(new THREE.SphereGeometry(1, 16, 8, 0, Math.PI * 2, Math.PI / 2, Math.PI / 2), paint(0x8a4a2a));
  hull.scale.set(0.28, 0.22, 0.9);
  hull.position.y = 0.16;
  const deck = new THREE.Mesh(new THREE.BoxGeometry(0.5, 0.04, 1.6), paint(0xb27a48));
  deck.position.y = 0.16;
  const mast = new THREE.Mesh(new THREE.CylinderGeometry(0.02, 0.025, 1.3, 6), paint(0x5a3a22));
  mast.position.y = 0.8;
  const sail = new THREE.Mesh(new THREE.CylinderGeometry(0.6, 0.6, 0.8, 12, 1, true, -0.5, 1), paint(random(seed) > 0.5 ? 0xf3ead6 : 0xe8c8a0));
  sail.material.side = THREE.DoubleSide;
  sail.position.set(0, 0.85, -0.55);
  group.add(hull, deck, mast, sail);
  group.traverse(object => { object.castShadow = object.isMesh; });
  return group;
}

export function createSky(scene) {
  const group = new THREE.Group();
  const center = new THREE.Vector3(MAP.columns / 2, 0, MAP.rows / 2);
  // Clouds ring the island over open water so they never hide the play area.
  const clouds = [];
  for (let i = 0; i < 9; i += 1) {
    const c = cloud(i * 17 + 3);
    const angle = (i / 9) * Math.PI * 2 + random(i) * 0.4;
    const radius = 24 + random(i * 5) * 10;
    c.position.set(center.x + Math.cos(angle) * radius, 5.5 + random(i * 7) * 3, center.z + Math.sin(angle) * radius * 0.8);
    c.scale.setScalar(0.9 + random(i * 11) * 0.8);
    c.userData.speed = 0.15 + random(i * 13) * 0.2;
    clouds.push(c);
    group.add(c);
  }
  const mountain = volcano();
  mountain.position.set(center.x - 48, 0, center.z - 52);
  group.add(mountain);
  const ships = [];
  for (let i = 0; i < 4; i += 1) {
    const s = ship(i * 29 + 1);
    s.userData.phase = (i / 4) * Math.PI * 2;
    s.userData.radius = 21 + i * 2.5;
    ships.push(s);
    group.add(s);
  }
  group.traverse(object => { object.raycast = () => {}; });
  scene.add(group);

  return {
    update(time) {
      for (const c of clouds) {
        c.position.x += c.userData.speed * 0.016;
        if (c.position.x > center.x + 40) c.position.x = center.x - 40;
      }
      for (const s of ships) {
        const angle = s.userData.phase + time * 0.012;
        const x = center.x + Math.cos(angle) * s.userData.radius;
        const z = center.z + Math.sin(angle) * s.userData.radius * 0.75;
        s.position.set(x, SEA_LEVEL - 0.05 + Math.sin(time * 1.3 + s.userData.phase) * 0.03, z);
        s.rotation.y = -angle;
        s.rotation.z = Math.sin(time * 0.9 + s.userData.phase) * 0.04;
      }
    }
  };
}
