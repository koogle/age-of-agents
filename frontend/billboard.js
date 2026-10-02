// Camera-facing painted sprites (villagers, trees, resources) cut from
// generated sheets. Depth is alpha-tested to the figure so the ink pass
// outlines it, and the anchor follows the planet curve at far zoom.
import * as THREE from 'three';
import { CURVE_UNIFORMS, uniforms } from './materials.js';

export function loadSheet(url) {
  const texture = new THREE.TextureLoader().load(url);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.anisotropy = 4;
  return texture;
}

export function billboard(sheet, size) {
  const map = sheet.clone();
  const material = new THREE.SpriteMaterial({ map, alphaTest: 0.5 });
  material.onBeforeCompile = shader => {
    Object.assign(shader.uniforms, { uCurve: uniforms.uCurve, uCurveCenter: uniforms.uCurveCenter });
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', `#include <common>\n${CURVE_UNIFORMS}`)
      .replace('vec4 mvPosition = modelViewMatrix[ 3 ];', /* glsl */`
        vec4 anchorWorld = modelMatrix[ 3 ];
        vec2 away = anchorWorld.xz - uCurveCenter;
        anchorWorld.y -= uCurve * dot(away, away);
        vec4 mvPosition = viewMatrix * anchorWorld;`);
  };
  material.customProgramCacheKey = () => 'aoa-billboard';
  const sprite = new THREE.Sprite(material);
  sprite.userData.size = size;
  sprite.raycast = () => {};
  return sprite;
}

// Shows sheet cell `rect` ([x, y, w, h] in pixels, y down), optionally mirrored.
export function showFrame(sprite, [x, y, w, h], mirror = false) {
  const map = sprite.material.map;
  const [width, height] = sprite.userData.size;
  map.repeat.set((mirror ? -w : w) / width, h / height);
  map.offset.set((mirror ? x + w : x) / width, 1 - (y + h) / height);
}
