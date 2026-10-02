// Short-lived particles (wood chips, sparks, dust, chimney smoke) and floating
// gain labels. Everything here is presentation only.
import * as THREE from 'three';

const geometry = new THREE.IcosahedronGeometry(1, 0);
const materials = new Map();
function material(color, opacity) {
  const key = `${color}|${opacity}`;
  if (!materials.has(key)) {
    materials.set(key, new THREE.MeshBasicMaterial({ color, transparent: true, opacity, depthWrite: false }));
  }
  return materials.get(key);
}

const STYLES = {
  chips: { color: 0xc8955a, size: 0.025, count: 4, speed: 1.1, gravity: 5, life: 0.6, opacity: 1 },
  sparks: { color: 0xffd36a, size: 0.018, count: 5, speed: 1.3, gravity: 4, life: 0.45, opacity: 1 },
  dust: { color: 0xd8c6a0, size: 0.05, count: 3, speed: 0.4, gravity: -0.2, life: 0.9, opacity: 0.55, grow: 2 },
  leaves: { color: 0x6fae4a, size: 0.025, count: 3, speed: 0.6, gravity: 1.2, life: 0.9, opacity: 1 },
  smoke: { color: 0xe6e0d8, size: 0.035, count: 1, speed: 0.22, gravity: -0.3, life: 2.6, opacity: 0.3, grow: 2.5 },
  dirt: { color: 0x9a5a3a, size: 0.03, count: 4, speed: 0.9, gravity: 5, life: 0.5, opacity: 1 }
};

export function createEffects(scene, camera) {
  const live = [];

  function burst(style, position) {
    const s = STYLES[style];
    for (let i = 0; i < s.count; i += 1) {
      const mesh = new THREE.Mesh(geometry, material(s.color, s.opacity));
      mesh.position.copy(position);
      mesh.scale.setScalar(s.size);
      mesh.raycast = () => {};
      const angle = Math.random() * Math.PI * 2;
      const velocity = new THREE.Vector3(Math.cos(angle) * s.speed * 0.5, s.speed * (0.6 + Math.random() * 0.6), Math.sin(angle) * s.speed * 0.5);
      scene.add(mesh);
      live.push({ mesh, velocity, age: 0, style: s });
    }
  }

  // Gain labels are sprites in the world, painted once into a small texture.
  const labels = [];
  function floatText(text, position) {
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    const font = 'italic 500 34px Alegreya, Georgia, serif';
    context.font = font;
    canvas.width = Math.ceil(context.measureText(text).width) + 16;
    canvas.height = 48;
    context.font = font;
    context.textBaseline = 'middle';
    context.lineWidth = 5;
    context.strokeStyle = '#4a3a2a';
    context.strokeText(text, 8, 25);
    context.fillStyle = '#f6f0e1';
    context.fillText(text, 8, 25);
    const texture = new THREE.CanvasTexture(canvas);
    texture.colorSpace = THREE.SRGBColorSpace;
    const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: texture, transparent: true, depthTest: false, toneMapped: false }));
    sprite.center.set(0.5, 0);
    sprite.scale.set(canvas.width / 110, canvas.height / 110, 1);
    sprite.position.copy(position);
    sprite.renderOrder = 5;
    scene.add(sprite);
    labels.push({ sprite, age: 0 });
  }

  function update(dt) {
    for (let i = labels.length - 1; i >= 0; i -= 1) {
      const label = labels[i];
      label.age += dt;
      label.sprite.position.y += dt * 0.45;
      label.sprite.material.opacity = Math.min(1, label.age * 6) * (1 - Math.max(0, label.age - 0.9) / 0.5);
      if (label.age > 1.4) {
        scene.remove(label.sprite);
        label.sprite.material.map.dispose();
        label.sprite.material.dispose();
        labels.splice(i, 1);
      }
    }
    for (let i = live.length - 1; i >= 0; i -= 1) {
      const p = live[i];
      p.age += dt;
      if (p.age >= p.style.life) {
        scene.remove(p.mesh);
        live.splice(i, 1);
        continue;
      }
      p.velocity.y -= p.style.gravity * dt;
      p.mesh.position.addScaledVector(p.velocity, dt);
      const k = p.age / p.style.life;
      p.mesh.scale.setScalar(p.style.size * (1 + (p.style.grow || 0) * k) * (p.style.grow ? 1 : 1 - k * 0.6));
    }
  }

  return { burst, floatText, update };
}
