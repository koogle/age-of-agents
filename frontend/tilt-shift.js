// Finish passes: fine ink linework found from the depth buffer (silhouettes
// and creases, one pixel wide like a pen drawing), then a tilt-shift blur that
// grows away from a horizontal focus band so the island reads as a miniature
// diorama. Tone mapping and sRGB output happen in the final pass.
import * as THREE from 'three';

const VERTEX = /* glsl */`
varying vec2 vUv;
void main() { vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }`;

const BLUR = /* glsl */`
uniform sampler2D uImage;
uniform vec2 uStep;
uniform float uFocus;
uniform float uBand;
uniform float uStrength;
varying vec2 vUv;
vec4 blurred() {
  float away = smoothstep(uBand * 0.35, uBand, abs(vUv.y - uFocus));
  vec2 stepUv = uStep * away * uStrength;
  vec4 sum = texture2D(uImage, vUv) * 0.227;
  sum += (texture2D(uImage, vUv + stepUv * 1.385) + texture2D(uImage, vUv - stepUv * 1.385)) * 0.316;
  sum += (texture2D(uImage, vUv + stepUv * 3.231) + texture2D(uImage, vUv - stepUv * 3.231)) * 0.07;
  return sum;
}`;

// Creases and silhouettes are where the screen-space Laplacian of inverse
// depth is non-zero: inverse depth is linear across any flat surface.
const INK = /* glsl */`
uniform sampler2D uImage;
uniform sampler2D uDepth;
uniform vec2 uTexel;
uniform float uNear;
uniform float uFar;
uniform float uInk;
varying vec2 vUv;
float inverseDepth(vec2 uv) {
  float z = texture2D(uDepth, uv).x * 2.0 - 1.0;
  float viewZ = 2.0 * uNear * uFar / (uFar + uNear - z * (uFar - uNear));
  return 1.0 / viewZ;
}
float crease(vec2 step) {
  float c = inverseDepth(vUv);
  float a = inverseDepth(vUv + step);
  float b = inverseDepth(vUv - step);
  return abs(a + b - 2.0 * c) / c;
}
void main() {
  vec4 color = texture2D(uImage, vUv);
  float edge = max(max(crease(vec2(uTexel.x, 0.0)), crease(vec2(0.0, uTexel.y))),
                   max(crease(uTexel), crease(vec2(uTexel.x, -uTexel.y))));
  float ink = smoothstep(0.0025, 0.012, edge) * uInk;
  // Lines thin out over far, dense detail rather than turning it to soot.
  ink *= mix(1.0, 0.55, smoothstep(30.0, 90.0, 1.0 / inverseDepth(vUv)));
  color.rgb = mix(color.rgb, color.rgb * vec3(0.16, 0.11, 0.08), ink);
  gl_FragColor = color;
}`;

function pass(fragment, toneMapped) {
  return new THREE.ShaderMaterial({
    uniforms: {
      uImage: { value: null }, uStep: { value: new THREE.Vector2() },
      uFocus: { value: 0.46 }, uBand: { value: 0.6 }, uStrength: { value: 1.1 }
    },
    vertexShader: VERTEX,
    fragmentShader: fragment,
    depthTest: false,
    depthWrite: false,
    toneMapped
  });
}

export function createTiltShift(renderer) {
  const options = { type: THREE.HalfFloatType, samples: 4 };
  const scene = new THREE.WebGLRenderTarget(1, 1, options);
  scene.depthTexture = new THREE.DepthTexture(1, 1, THREE.FloatType);
  const inked = new THREE.WebGLRenderTarget(1, 1, { type: THREE.HalfFloatType });
  const half = new THREE.WebGLRenderTarget(1, 1, { type: THREE.HalfFloatType });
  const ink = new THREE.ShaderMaterial({
    uniforms: {
      uImage: { value: scene.texture }, uDepth: { value: scene.depthTexture }, uTexel: { value: new THREE.Vector2() },
      uNear: { value: 0.1 }, uFar: { value: 1000 }, uInk: { value: 0.9 }
    },
    vertexShader: VERTEX,
    fragmentShader: INK,
    depthTest: false,
    depthWrite: false,
    toneMapped: false
  });
  const horizontal = pass(`${BLUR}\nvoid main() { gl_FragColor = blurred(); }`, false);
  const vertical = pass(`${BLUR}
    void main() {
      vec4 color = blurred();
      // A touch of extra saturation and a soft warm vignette, like a photographed model.
      float grey = dot(color.rgb, vec3(0.299, 0.587, 0.114));
      color.rgb = mix(vec3(grey), color.rgb, 1.12);
      float edge = smoothstep(0.95, 0.35, length(vUv - vec2(0.5, 0.48)));
      color.rgb *= mix(vec3(0.86, 0.84, 0.8), vec3(1.0), edge);
      gl_FragColor = color;
      #include <tonemapping_fragment>
      #include <colorspace_fragment>
    }`, true);
  const quad = new THREE.Mesh(new THREE.PlaneGeometry(2, 2), horizontal);
  quad.frustumCulled = false;
  const quadScene = new THREE.Scene();
  quadScene.add(quad);
  const quadCamera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);

  return {
    setSize(width, height) {
      const dpr = renderer.getPixelRatio();
      scene.setSize(Math.round(width * dpr), Math.round(height * dpr));
      inked.setSize(Math.round(width * dpr), Math.round(height * dpr));
      ink.uniforms.uTexel.value.set(1 / (width * dpr), 1 / (height * dpr));
      half.setSize(Math.round(width * dpr), Math.round(height * dpr));
      horizontal.uniforms.uStep.value.set(1 / (width * dpr), 0);
      vertical.uniforms.uStep.value.set(0, 1 / (height * dpr));
    },
    render(world, camera) {
      renderer.setRenderTarget(scene);
      renderer.render(world, camera);
      quad.material = ink;
      ink.uniforms.uNear.value = camera.near;
      ink.uniforms.uFar.value = camera.far;
      renderer.setRenderTarget(inked);
      renderer.render(quadScene, quadCamera);
      quad.material = horizontal;
      horizontal.uniforms.uImage.value = inked.texture;
      renderer.setRenderTarget(half);
      renderer.render(quadScene, quadCamera);
      quad.material = vertical;
      vertical.uniforms.uImage.value = half.texture;
      renderer.setRenderTarget(null);
      renderer.render(quadScene, quadCamera);
    }
  };
}
