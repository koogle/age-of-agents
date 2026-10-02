// Painted ground: one generated seamless texture per biome (assets/terrain/),
// stacked into a texture array and blended across cell borders with a ragged,
// noise-shifted edge. Missing textures fall back to the procedural brush look.
import * as THREE from 'three';
import { MAP } from './materials.js';

export const GROUND_LAYERS = ['meadow', 'forest', 'prairie', 'highland', 'wetland', 'scrubland', 'heath', 'clayland', 'beach', 'shallows'];
export const BEACH_LAYER = GROUND_LAYERS.indexOf('beach');
export const SHALLOWS_LAYER = GROUND_LAYERS.indexOf('shallows');
const SIZE = 512;
// One texture repeat spans this many cells.
const REPEAT = 2.5;

// Per-cell layer index; 255 means unknown biome.
const indexData = new Uint8Array(MAP.columns * MAP.rows).fill(255);
const indexTexture = new THREE.DataTexture(indexData, MAP.columns, MAP.rows, THREE.RedFormat);
indexTexture.magFilter = THREE.NearestFilter;
indexTexture.minFilter = THREE.NearestFilter;
indexTexture.needsUpdate = true;

const layerData = new Uint8Array(SIZE * SIZE * 4 * GROUND_LAYERS.length);
const layers = new THREE.DataArrayTexture(layerData, SIZE, SIZE, GROUND_LAYERS.length);
layers.wrapS = layers.wrapT = THREE.RepeatWrapping;
layers.minFilter = THREE.LinearMipmapLinearFilter;
layers.magFilter = THREE.LinearFilter;
layers.generateMipmaps = true;
layers.colorSpace = THREE.SRGBColorSpace;
layers.anisotropy = 4;

export const groundUniforms = {
  uGroundLayers: { value: layers },
  uGroundIndex: { value: indexTexture },
  uGroundLoaded: { value: new Array(GROUND_LAYERS.length).fill(0) }
};

export function writeGroundCell(column, row, biome) {
  const layer = GROUND_LAYERS.indexOf(biome);
  indexData[row * MAP.columns + column] = layer < 0 ? 255 : layer;
}
export function flushGroundCells() {
  indexTexture.needsUpdate = true;
}

export function loadGroundTextures() {
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = SIZE;
  const context = canvas.getContext('2d', { willReadFrequently: true });
  GROUND_LAYERS.forEach((name, layer) => {
    const image = new Image();
    image.onload = () => {
      context.clearRect(0, 0, SIZE, SIZE);
      context.drawImage(image, 0, 0, SIZE, SIZE);
      layerData.set(context.getImageData(0, 0, SIZE, SIZE).data, layer * SIZE * SIZE * 4);
      layers.needsUpdate = true;
      groundUniforms.uGroundLoaded.value[layer] = 1;
    };
    // Generated textures ship as .webp, or .png when lossless is needed.
    image.onerror = () => { if (image.src.endsWith('.webp')) image.src = `/assets/terrain/${name}.png`; };
    image.src = `/assets/terrain/${name}.webp`;
  });
}

export const GROUND_GLSL = /* glsl */`
uniform highp sampler2DArray uGroundLayers;
uniform sampler2D uGroundIndex;
uniform float uGroundLoaded[${GROUND_LAYERS.length}];
float groundLayerAt(vec2 cell) {
  vec2 clamped = clamp(cell, vec2(0.0), uMapSize - 1.0);
  return floor(texture2D(uGroundIndex, (clamped + 0.5) / uMapSize).r * 255.0 + 0.5);
}
// Returns rgb painted color (linear) and a = how much of it is real paint.
vec4 groundSample(vec2 xz, float layer) {
  if (layer > ${GROUND_LAYERS.length - 0.5}) return vec4(0.0);
  int index = int(layer);
  float loaded = 0.0;
  for (int i = 0; i < ${GROUND_LAYERS.length}; i++) if (i == index) loaded = uGroundLoaded[i];
  if (loaded < 0.5) return vec4(0.0);
  // Two scales, slightly rotated, hide the repeat.
  vec2 uv = xz / ${REPEAT.toFixed(1)};
  vec3 a = texture(uGroundLayers, vec3(uv, layer)).rgb;
  vec3 b = texture(uGroundLayers, vec3(mat2(0.8, -0.6, 0.6, 0.8) * uv * 0.43 + 0.37, layer)).rgb;
  vec3 paint = mix(a, b, 0.3 * aoaFbm(xz * 0.21));
  // Lift the brushwork that mipmapping flattens at gameplay distance.
  vec3 mean = texture(uGroundLayers, vec3(uv, layer), 9.0).rgb;
  return vec4(max(mean + (paint - mean) * 1.8, 0.0), 1.0);
}
// Blends the four nearest cells' paint with a ragged, painterly border.
vec4 groundPaint(vec2 xz) {
  vec2 g = xz - 0.5;
  vec2 base = floor(g);
  vec2 f = fract(g) + (vec2(aoaFbm(xz * 1.7), aoaFbm(xz * 1.7 + 11.3)) - 0.5) * 0.7;
  f = smoothstep(0.3, 0.7, clamp(f, 0.0, 1.0));
  vec4 c00 = groundSample(xz, groundLayerAt(base));
  vec4 c10 = groundSample(xz, groundLayerAt(base + vec2(1.0, 0.0)));
  vec4 c01 = groundSample(xz, groundLayerAt(base + vec2(0.0, 1.0)));
  vec4 c11 = groundSample(xz, groundLayerAt(base + vec2(1.0, 1.0)));
  return mix(mix(c00, c10, f.x), mix(c01, c11, f.x), f.y);
}
`;
