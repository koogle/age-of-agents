// Shared look: soft matte "painted miniature" materials and one fog-of-war
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
  color *= mix(1.0, 0.86, smoothstep(0.52, 0.72, cloud));
  float vis = aoaVisibility(vAoaWorld.xz);
  float seen = smoothstep(0.08, 0.45, vis);
  float lit = smoothstep(0.55, 0.95, vis);
  float grey = dot(color, vec3(0.299, 0.587, 0.114));
  vec3 remembered = mix(mix(vec3(grey), color, 0.5) * 0.85, vec3(0.92, 0.93, 0.95), 0.25);
  color = mix(remembered, color, lit);
  // Unexplored land lies under a bank of soft, slowly churning cloud.
  float mist = aoaFbm(vAoaWorld.xz * 0.28 + vec2(uTime * 0.03, -uTime * 0.02));
  float billow = aoaFbm(vAoaWorld.xz * 0.9 - vec2(uTime * 0.05, uTime * 0.04));
  // Shaded like cumulus: cool grey in the folds, warm white on the tops.
  vec3 unknown = mix(vec3(0.6, 0.66, 0.74), vec3(0.98, 0.96, 0.92), smoothstep(0.3, 0.72, mist * 0.65 + billow * 0.35));
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

// Soft matte "painted miniature" surfaces: the diorama reference has no ink
// lines and no hard cel bands, just gentle light falloff on rounded shapes.
const paintCache = new Map();
export function paint(color, options = {}) {
  const key = `${color}|${options.emissive || 0}|${options.transparent ? options.opacity : 1}`;
  if (!paintCache.has(key)) {
    paintCache.set(key, patchWorld(new THREE.MeshStandardMaterial({
      color,
      roughness: 0.82,
      metalness: 0,
      emissive: options.emissive || 0x000000,
      transparent: Boolean(options.transparent),
      opacity: options.opacity ?? 1
    })));
  }
  return paintCache.get(key);
}
