// The interface is drawn inside the WebGL canvas: each panel is painted into a
// texture and composited on an orthographic layer above the world. Painting
// records hit regions, so pointer input is resolved against what was drawn.
import * as THREE from 'three';

export const STYLE = {
  ivory: '#f4efe4', ink: '#3d3328', muted: 'rgba(61,51,40,0.7)', accent: '#c8553a',
  glass: 'rgba(250,247,240,0.86)', shadow: 'rgba(40,32,24,0.35)',
  body: '"Nunito", "Trebuchet MS", system-ui, sans-serif'
};

// One painted rectangle on the overlay, backed by its own canvas texture.
function createPanel(scene, order) {
  const canvas = document.createElement('canvas');
  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.generateMipmaps = false;
  texture.minFilter = THREE.LinearFilter;
  const material = new THREE.MeshBasicMaterial({ map: texture, transparent: true, toneMapped: false, depthTest: false, depthWrite: false });
  const mesh = new THREE.Mesh(new THREE.PlaneGeometry(1, 1), material);
  mesh.renderOrder = order;
  mesh.visible = false;
  scene.add(mesh);
  const panel = { x: 0, y: 0, w: 0, h: 0, regions: [], mesh, canvas, context: canvas.getContext('2d') };
  // Places the panel in CSS pixels and returns a context scaled for the device.
  panel.begin = (x, y, w, h, viewHeight, dpr) => {
    Object.assign(panel, { x, y, w, h, regions: [] });
    const width = Math.max(1, Math.round(w * dpr));
    const height = Math.max(1, Math.round(h * dpr));
    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
      texture.dispose();
    }
    mesh.position.set(x + w / 2, viewHeight - (y + h / 2), 0);
    mesh.scale.set(w, h, 1);
    mesh.visible = w > 0 && h > 0;
    const context = panel.context;
    context.setTransform(dpr, 0, 0, dpr, 0, 0);
    context.clearRect(0, 0, w, h);
    return context;
  };
  panel.end = () => { texture.needsUpdate = true; };
  // Only painted regions catch the pointer; the transparent rest of a panel
  // lets taps through to the world.
  panel.hit = (px, py) => {
    if (!panel.mesh.visible) return null;
    const lx = px - panel.x;
    const ly = py - panel.y;
    return panel.regions.find(r => r.round
      ? Math.hypot(lx - (r.x + r.w / 2), ly - (r.y + r.h / 2)) <= r.w / 2
      : lx >= r.x && ly >= r.y && lx <= r.x + r.w && ly <= r.y + r.h) || null;
  };
  panel.hide = () => { mesh.visible = false; panel.regions = []; };
  return panel;
}

export function createUiLayer(renderer) {
  const scene = new THREE.Scene();
  const camera = new THREE.OrthographicCamera(0, 1, 1, 0, -1, 1);
  const panels = new Map();
  const boxFill = new THREE.Mesh(new THREE.PlaneGeometry(1, 1), new THREE.MeshBasicMaterial({ color: STYLE.ivory, opacity: 0.18, transparent: true, depthTest: false, toneMapped: false }));
  const boxLine = new THREE.LineSegments(new THREE.EdgesGeometry(new THREE.PlaneGeometry(1, 1)), new THREE.LineBasicMaterial({ color: '#ffffff', depthTest: false, toneMapped: false }));
  boxFill.visible = boxLine.visible = false;
  boxFill.renderOrder = boxLine.renderOrder = 14;
  scene.add(boxFill, boxLine);

  const view = { width: 1, height: 1, dpr: 1 };
  function place(mesh, x, y, w, h) {
    mesh.position.set(x + w / 2, view.height - (y + h / 2), 0);
    mesh.scale.set(Math.max(w, 0.001), Math.max(h, 0.001), 1);
  }

  return {
    view,
    // Panels are created on first use and drawn in creation order.
    panel(name) {
      if (!panels.has(name)) panels.set(name, createPanel(scene, 10 + panels.size));
      return panels.get(name);
    },
    resize(width, height, dpr) {
      Object.assign(view, { width, height, dpr });
      Object.assign(camera, { left: 0, right: width, top: height, bottom: 0 });
      camera.updateProjectionMatrix();
    },
    box(area) {
      boxFill.visible = boxLine.visible = Boolean(area);
      if (!area) return;
      place(boxFill, area.left, area.top, area.right - area.left, area.bottom - area.top);
      place(boxLine, area.left, area.top, area.right - area.left, area.bottom - area.top);
    },
    // Topmost painted region under a point, or null when the world is there.
    hit(x, y) {
      for (const panel of [...panels.values()].reverse()) {
        const region = panel.hit(x, y);
        if (region) return region;
      }
      return null;
    },
    render() {
      renderer.autoClear = false;
      renderer.clearDepth();
      renderer.render(scene, camera);
      renderer.autoClear = true;
    }
  };
}

const iconCache = new Map();
// Vector placeholder icons become images once; generated art can replace them.
export function iconImage(svgBody, onLoad) {
  if (!iconCache.has(svgBody)) {
    const image = new Image();
    image.onload = onLoad;
    image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="48" height="48">${svgBody}</svg>`)}`;
    iconCache.set(svgBody, image);
  }
  return iconCache.get(svgBody);
}

// Draws `text` wrapped to `maxWidth`, up to `maxLines`, ending in an ellipsis if cut.
export function wrapText(context, text, x, y, maxWidth, lineHeight, maxLines) {
  const words = String(text).split(' ');
  let line = '';
  let lines = 0;
  for (let i = 0; i < words.length; i += 1) {
    const candidate = line ? `${line} ${words[i]}` : words[i];
    if (context.measureText(candidate).width <= maxWidth || !line) {
      line = candidate;
      continue;
    }
    if (lines === maxLines - 1) {
      while (line && context.measureText(`${line}…`).width > maxWidth) line = line.slice(0, -1);
      context.fillText(`${line}…`, x, y + lines * lineHeight);
      return lines + 1;
    }
    context.fillText(line, x, y + lines * lineHeight);
    lines += 1;
    line = words[i];
  }
  if (line) context.fillText(line, x, y + lines * lineHeight);
  return lines + (line ? 1 : 0);
}
