// Shared look: cel-shaded toon materials, inked outlines, and one fog-of-war
// field sampled by every world material so terrain, props, and buildings fade
// into the unknown together.
import * as THREE from 'three';

export const MAP = { columns: 30, rows: 20 };

// Per-cell texture: rgb = known biome color, a = visibility (0 unseen, .5 explored, 1 visible).
const cellData = new Uint8Array(MAP.columns * MAP.rows * 4);
export const cellTexture = new THREE.DataTexture(cellData, MAP.columns, MAP.rows);
cellTexture.magFilter = THREE.LinearFilter;
cellTexture.minFilter = THREE.LinearFilter;
cellTexture.colorSpace = THREE.NoColorSpace;
cellTexture.needsUpdate = true;

export const uniforms = {
  uTime: { value: 0 },
  uCells: { value: cellTexture },
  uMapSize: { value: new THREE.Vector2(MAP.columns, MAP.rows) },
  uGrid: { value: 0 }
};

export function writeCell(column, row, rgb, visibility) {
  const offset = (row * MAP.columns + column) * 4;
  cellData[offset] = rgb[0];
  cellData[offset + 1] = rgb[1];
  cellData[offset + 2] = rgb[2];
  cellData[offset + 3] = visibility;
}

const NOISE_GLSL = /* glsl */`
float aoaHash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float aoaNoise(vec2 p) {
  vec2 i = floor(p); vec2 f = fract(p); vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(mix(aoaHash(i), aoaHash(i + vec2(1, 0)), u.x), mix(aoaHash(i + vec2(0, 1)), aoaHash(i + vec2(1, 1)), u.x), u.y);
}
float aoaFbm(vec2 p) { return aoaNoise(p) * .55 + aoaNoise(p * 2.1 + 7.3) * .3 + aoaNoise(p * 4.3 + 1.7) * .15; }
`;

// Visibility outside the playable rectangle fades to "scenery": always shown.
const FOG_GLSL = /* glsl */`
uniform sampler2D uCells; uniform vec2 uMapSize; uniform float uTime;
varying vec3 vAoaWorld;
${NOISE_GLSL}
float aoaVisibility(vec2 xz) {
  vec2 outside = max(max(-xz, xz - uMapSize), 0.0);
  float scenery = smoothstep(0.3, 1.8, length(outside));
  return max(texture2D(uCells, clamp(xz / uMapSize, 0.0, 1.0)).a, scenery);
}
vec3 aoaApplyWorldLight(vec3 color) {
  // Slow cloud shadows drifting across the land.
  float cloud = aoaFbm(vAoaWorld.xz * 0.07 + vec2(uTime * 0.018, uTime * 0.011));
  color *= mix(1.0, 0.8, smoothstep(0.52, 0.72, cloud));
  float vis = aoaVisibility(vAoaWorld.xz);
  float seen = smoothstep(0.08, 0.45, vis);
  float lit = smoothstep(0.55, 0.95, vis);
  float grey = dot(color, vec3(0.299, 0.587, 0.114));
  vec3 remembered = mix(vec3(grey), color, 0.4) * vec3(0.62, 0.63, 0.7);
  color = mix(remembered, color, lit);
  float mist = aoaFbm(vAoaWorld.xz * 0.35 + vec2(uTime * 0.05, -uTime * 0.03));
  vec3 unknown = mix(vec3(0.07, 0.085, 0.12), vec3(0.2, 0.22, 0.28), mist);
  return mix(unknown, color, seen);
}
`;

const WORLD_POSITION_GLSL = /* glsl */`
vec4 aoaWorld = vec4(transformed, 1.0);
#ifdef USE_INSTANCING
aoaWorld = instanceMatrix * aoaWorld;
#endif
vAoaWorld = (modelMatrix * aoaWorld).xyz;
`;

// Injects fog of war and cloud shadows. `extra(shader)` may patch further.
export function patchWorld(material, extra) {
  material.onBeforeCompile = shader => {
    Object.assign(shader.uniforms, uniforms);
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', '#include <common>\nvarying vec3 vAoaWorld;')
      .replace('#include <project_vertex>', `#include <project_vertex>\n${WORLD_POSITION_GLSL}`);
    shader.fragmentShader = shader.fragmentShader
      .replace('#include <common>', `#include <common>\n${FOG_GLSL}`)
      .replace('#include <tonemapping_fragment>', 'gl_FragColor.rgb = aoaApplyWorldLight(gl_FragColor.rgb);\n#include <tonemapping_fragment>');
    if (extra) extra(shader);
  };
  material.customProgramCacheKey = () => (extra ? 'aoa-world-extra' : 'aoa-world');
  return material;
}

const gradient = new THREE.DataTexture(new Uint8Array([90, 90, 90, 255, 165, 165, 165, 255, 225, 225, 225, 255, 255, 255, 255, 255]), 4, 1);
gradient.magFilter = THREE.NearestFilter;
gradient.minFilter = THREE.NearestFilter;
gradient.needsUpdate = true;

const toonCache = new Map();
export function toon(color, options = {}) {
  const key = `${color}|${options.emissive || 0}|${options.transparent ? options.opacity : 1}`;
  if (!toonCache.has(key)) {
    toonCache.set(key, patchWorld(new THREE.MeshToonMaterial({
      color,
      gradientMap: gradient,
      emissive: options.emissive || 0x000000,
      transparent: Boolean(options.transparent),
      opacity: options.opacity ?? 1
    })));
  }
  return toonCache.get(key);
}

// Inverted-hull ink line: back faces pushed out along view-space normals so
// the line keeps a constant world width regardless of a part's scale.
export const outlineMaterial = new THREE.MeshBasicMaterial({ color: 0x2b1d14, side: THREE.BackSide });
outlineMaterial.onBeforeCompile = shader => {
  shader.vertexShader = shader.vertexShader.replace('#include <project_vertex>', `
    vec4 mvPosition = vec4(transformed, 1.0);
    vec3 inkNormal = normal;
    #ifdef USE_INSTANCING
    mvPosition = instanceMatrix * mvPosition;
    inkNormal = mat3(instanceMatrix) * inkNormal;
    #endif
    mvPosition = modelViewMatrix * mvPosition;
    mvPosition.xyz += normalize(normalMatrix * inkNormal) * 0.011;
    gl_Position = projectionMatrix * mvPosition;`);
};

export function inked(mesh) {
  const line = new THREE.Mesh(mesh.geometry, outlineMaterial);
  line.raycast = () => {};
  mesh.add(line);
  return mesh;
}
