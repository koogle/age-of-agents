// The island: a gently rolling playable rectangle that falls away to beaches
// and sea. Heights come from a fixed noise field (never from biome data) so the
// shape of the land reveals nothing about unexplored terrain.
import * as THREE from 'three';
import { GROUND_GLSL, BEACH_LAYER, flushGroundCells, groundUniforms, loadGroundTextures, writeGroundCell } from './ground-paint.js';
import { CURVE_GLSL, CURVE_UNIFORMS, MAP, celRamp, patchWorld, paint, uniforms, writeCell, cellTexture } from './materials.js';

export const SEA_LEVEL = -0.32;
const MARGIN = 9;

export const BIOME_COLORS = {
  meadow: [176, 204, 92],
  forest: [108, 156, 74],
  prairie: [230, 200, 104],
  highland: [214, 206, 182],
  wetland: [118, 184, 128],
  scrubland: [206, 180, 112],
  heath: [166, 176, 104],
  clayland: [214, 142, 92],
  beach: [236, 209, 153],
  water: [96, 172, 196]
};
const UNSEEN_COLOR = [233, 216, 176];

function hash(x, y) {
  const s = Math.sin(x * 127.1 + y * 311.7) * 43758.5453;
  return s - Math.floor(s);
}
function noise(x, y) {
  const ix = Math.floor(x), iy = Math.floor(y);
  const fx = x - ix, fy = y - iy;
  const ux = fx * fx * (3 - 2 * fx), uy = fy * fy * (3 - 2 * fy);
  const a = hash(ix, iy), b = hash(ix + 1, iy), c = hash(ix, iy + 1), d = hash(ix + 1, iy + 1);
  return a + (b - a) * ux + (c - a) * uy + (a - b - c + d) * ux * uy;
}
export function random(seed) {
  return hash(seed * 0.731, seed * 1.173);
}

function outsideDistance(x, z) {
  const dx = Math.max(-x, x - MAP.columns, 0);
  const dz = Math.max(-z, z - MAP.rows, 0);
  return Math.hypot(dx, dz);
}

export function heightAt(x, z) {
  const rolling = (noise(x * 0.18, z * 0.18) - 0.5) * 0.16 + (noise(x * 0.6 + 9, z * 0.6 + 3) - 0.5) * 0.04;
  const coast = outsideDistance(x, z) + (noise(x * 0.3 + 40, z * 0.3) - 0.5) * 1.6;
  const fall = THREE.MathUtils.smoothstep(coast, 0.8, 4.2);
  return rolling * (1 - fall) - fall * 0.9 + (fall > 0.6 ? (noise(x * 0.9, z * 0.9) - 0.5) * 0.2 : 0);
}

function buildGround() {
  const width = MAP.columns + MARGIN * 2;
  const depth = MAP.rows + MARGIN * 2;
  const geometry = new THREE.PlaneGeometry(width, depth, width * 3, depth * 3);
  geometry.rotateX(-Math.PI / 2);
  geometry.translate(MAP.columns / 2, 0, MAP.rows / 2);
  const position = geometry.attributes.position;
  const shade = new Float32Array(position.count);
  for (let i = 0; i < position.count; i += 1) {
    const x = position.getX(i);
    const z = position.getZ(i);
    position.setY(i, heightAt(x, z));
    shade[i] = 0.9 + noise(x * 1.7, z * 1.7) * 0.2;
  }
  geometry.setAttribute('aShade', new THREE.BufferAttribute(shade, 1));
  geometry.computeVertexNormals();

  const material = patchWorld(new THREE.MeshToonMaterial({ color: 0xffffff, gradientMap: celRamp }), shader => {
    Object.assign(shader.uniforms, groundUniforms);
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', '#include <common>\nattribute float aShade;\nvarying float vShade;')
      .replace('#include <begin_vertex>', '#include <begin_vertex>\nvShade = aShade;');
    shader.fragmentShader = shader.fragmentShader
      .replace('#include <common>', '#include <common>\nvarying float vShade;\nuniform float uGrid;')
      .replace('void main() {', `${GROUND_GLSL}\nvoid main() {`)
      .replace('vec4 diffuseColor = vec4( diffuse, opacity );', /* glsl */`
        vec2 xz = vAoaWorld.xz;
        vec2 outside = max(max(-xz, xz - uMapSize), 0.0);
        float wild = smoothstep(0.0, 1.6, length(outside));
        // Cell colors are stored as sRGB bytes; light in linear space.
        vec3 biome = pow(texture2D(uCells, clamp(xz / uMapSize, 0.0, 1.0)).rgb, vec3(2.2));
        vec3 land = mix(biome, vec3(0.38, 0.55, 0.16), wild);
        land *= vShade * (0.94 + 0.1 * aoaNoise(xz * 3.1));
        // Painted grass: broad warm and cool patches, then short directional
        // brush strokes and a few light flecks, like a Ghibli background.
        float patchTone = aoaFbm(xz * 0.32);
        land = mix(land * vec3(0.9, 1.0, 0.97), land * vec3(1.1, 1.05, 0.84), patchTone);
        float stroke = aoaFbm(vec2(xz.x * 7.0 + xz.y * 2.5, xz.y * 1.6 - xz.x * 0.6));
        land *= 0.9 + 0.2 * stroke;
        float fleck = step(0.86, aoaNoise(xz * 23.0)) * smoothstep(0.4, 0.8, stroke);
        land = mix(land, land * vec3(1.18, 1.16, 1.0), fleck * 0.6);
        // Generated painted ground replaces the procedural strokes where loaded.
        vec4 painted = groundPaint(xz) * (1.0 - wild);
        land = land * (1.0 - painted.a) + painted.rgb * vShade * mix(0.94, 1.06, patchTone);
        float h = vAoaWorld.y;
        vec3 sand = vec3(0.93, 0.82, 0.6) * (0.96 + 0.08 * aoaNoise(xz * 6.0));
        vec4 beach = groundSample(xz, ${BEACH_LAYER}.0);
        sand = sand * (1.0 - beach.a) + beach.rgb;
        vec3 color = mix(sand, land, smoothstep(-0.2, -0.08, h));
        color = mix(color * vec3(0.35, 0.8, 0.85), color, smoothstep(${SEA_LEVEL - 0.25}, ${SEA_LEVEL}, h));
        float foam = smoothstep(0.03, 0.0, abs(h - ${SEA_LEVEL} - 0.012 * sin(uTime * 1.4 + xz.x * 2.0 + xz.y)));
        color = mix(color, vec3(1.0), foam * 0.85);
        vec2 cell = fract(xz);
        float line = (1.0 - wild) * uGrid * (1.0 - smoothstep(0.0, 0.025, min(min(cell.x, 1.0 - cell.x), min(cell.y, 1.0 - cell.y))));
        color = mix(color, vec3(1.0, 0.92, 0.6), line * 0.32);
        vec4 diffuseColor = vec4(color, opacity);`);
  });
  const mesh = new THREE.Mesh(geometry, material);
  mesh.receiveShadow = true;
  mesh.name = 'ground';
  return mesh;
}

function buildSea() {
  const geometry = new THREE.PlaneGeometry(400, 400, 80, 80);
  geometry.rotateX(-Math.PI / 2);
  geometry.translate(MAP.columns / 2, SEA_LEVEL, MAP.rows / 2);
  const material = new THREE.MeshPhongMaterial({ color: 0x2c8db2, transparent: true, opacity: 0.9, shininess: 70, specular: 0xcfe8f0 });
  material.onBeforeCompile = shader => {
    Object.assign(shader.uniforms, { uTime: uniforms.uTime, uCurve: uniforms.uCurve, uCurveCenter: uniforms.uCurveCenter });
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', `#include <common>\nuniform float uTime;\n${CURVE_UNIFORMS}`)
      .replace('#include <begin_vertex>', `#include <begin_vertex>
        transformed.y += sin(position.x * 0.9 + uTime * 1.1) * 0.025 + cos(position.z * 0.7 + uTime * 0.8) * 0.025;`)
      .replace('#include <project_vertex>', `#include <project_vertex>\n${CURVE_GLSL}`);
  };
  const mesh = new THREE.Mesh(geometry, material);
  mesh.receiveShadow = true;
  mesh.raycast = () => {};
  return mesh;
}

export function createTerrain(scene) {
  loadGroundTextures();
  scene.add(buildGround());
  scene.add(buildSea());
  scene.add(buildScenery());

  function update(world) {
    for (const cell of world.terrain) {
      const visibility = cell.visibility === 'visible' ? 255 : cell.visibility === 'explored' ? 128 : 0;
      writeCell(cell.column, cell.row, cell.biome ? BIOME_COLORS[cell.biome] : UNSEEN_COLOR, visibility);
      writeGroundCell(cell.column, cell.row, cell.biome);
    }
    cellTexture.needsUpdate = true;
    flushGroundCells();
  }
  return { update };
}

// Decorative trees and boulders on the island rim, outside the playable grid.
function buildScenery() {
  const group = new THREE.Group();
  const trunk = new THREE.CylinderGeometry(0.05, 0.07, 0.3, 5);
  const crown = new THREE.ConeGeometry(0.17, 1.1, 6);
  const boulder = new THREE.DodecahedronGeometry(0.35, 0);
  for (let i = 0; i < 260; i += 1) {
    const x = -MARGIN + random(i * 3 + 1) * (MAP.columns + MARGIN * 2);
    const z = -MARGIN + random(i * 3 + 2) * (MAP.rows + MARGIN * 2);
    const out = outsideDistance(x, z);
    const y = heightAt(x, z);
    if (out < 0.9 || y < SEA_LEVEL + 0.12) continue;
    const scale = 0.8 + random(i * 5) * 0.8;
    if (random(i * 7) < 0.78) {
      const tree = new THREE.Group();
      const t = new THREE.Mesh(trunk, paint(0x6b4a2e));
      t.position.y = 0.15;
      const c = new THREE.Mesh(crown, paint(random(i) > 0.5 ? 0x2f6b3c : 0x3f7f44));
      c.position.y = 0.75;
      c.castShadow = true;
      tree.add(t, c);
      tree.position.set(x, y, z);
      tree.scale.setScalar(scale);
      group.add(tree);
    } else {
      const rock = new THREE.Mesh(boulder, paint(0xe6dfca));
      rock.position.set(x, y + 0.08, z);
      rock.scale.set(scale, scale * 0.7, scale);
      rock.rotation.y = random(i) * 6;
      rock.castShadow = true;
      group.add(rock);
    }
  }
  group.traverse(object => { object.raycast = () => {}; });
  return group;
}
