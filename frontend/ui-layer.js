// The interface is drawn inside the WebGL canvas: each panel is painted into a
// texture and composited on an orthographic layer above the world. Painting
// records hit regions, so pointer input is resolved against what was drawn.
import * as THREE from 'three';

export const STYLE = {
  ivory: '#efe6d0', ivoryLight: '#f6f0e1', ink: '#4a3a2a',
  hair: 'rgba(74,58,42,0.38)', faint: 'rgba(74,58,42,0.16)', muted: 'rgba(74,58,42,0.72)', accent: '#b5502e',
  caps: '"Cormorant SC", Georgia, serif', body: 'Alegreya, Georgia, serif'
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
  panel.hit = (px, py) => {
    if (!panel.mesh.visible || px < panel.x || py < panel.y || px > panel.x + panel.w || py > panel.y + panel.h) return null;
    const lx = px - panel.x;
    const ly = py - panel.y;
    return panel.regions.find(r => lx >= r.x && ly >= r.y && lx <= r.x + r.w && ly <= r.y + r.h) || { id: 'panel' };
  };
  return panel;
}

export function createUiLayer(renderer) {
  const scene = new THREE.Scene();
  const camera = new THREE.OrthographicCamera(0, 1, 1, 0, -1, 1);
  const panels = {
    top: createPanel(scene, 10), side: createPanel(scene, 10), minimap: createPanel(scene, 11),
    note: createPanel(scene, 12), toast: createPanel(scene, 12)
  };
  // The ivory mount on the remaining edges plus one hairline around the picture.
  const flat = color => new THREE.MeshBasicMaterial({ color, toneMapped: false, depthTest: false, transparent: true });
  const mount = [flat(STYLE.ivory), flat(STYLE.ivory), flat('#9a8a74'), flat('#9a8a74'), flat('#9a8a74'), flat('#9a8a74')]
    .map((material, index) => {
      const mesh = new THREE.Mesh(new THREE.PlaneGeometry(1, 1), material);
      mesh.renderOrder = index < 2 ? 9 : 13;
      scene.add(mesh);
      return mesh;
    });
  const boxFill = new THREE.Mesh(new THREE.PlaneGeometry(1, 1), new THREE.MeshBasicMaterial({ color: STYLE.ivory, opacity: 0.18, transparent: true, depthTest: false, toneMapped: false }));
  const boxLine = new THREE.LineSegments(new THREE.EdgesGeometry(new THREE.PlaneGeometry(1, 1)), new THREE.LineBasicMaterial({ color: STYLE.ink, depthTest: false, toneMapped: false }));
  boxFill.visible = boxLine.visible = false;
  boxFill.renderOrder = boxLine.renderOrder = 14;
  scene.add(boxFill, boxLine);

  const view = { width: 1, height: 1, dpr: 1 };
  function place(mesh, x, y, w, h) {
    mesh.position.set(x + w / 2, view.height - (y + h / 2), 0);
    mesh.scale.set(Math.max(w, 0.001), Math.max(h, 0.001), 1);
  }
  function rect(x, y, w, h) {
    return { x, y, w, h };
  }

  return {
    panels,
    view,
    resize(width, height, dpr) {
      Object.assign(view, { width, height, dpr });
      Object.assign(camera, { left: 0, right: width, top: height, bottom: 0 });
      camera.updateProjectionMatrix();
    },
    // Mount strips: right and bottom ivory edges, then a hairline frame around `picture`.
    frame(picture, mountWidth) {
      const { x, y, w, h } = picture;
      place(mount[0], x + w, y, mountWidth, h + mountWidth);
      place(mount[1], x, y + h, w, mountWidth);
      place(mount[2], x, y, w, 1);
      place(mount[3], x, y + h - 1, w, 1);
      place(mount[4], x, y, 1, h);
      place(mount[5], x + w - 1, y, 1, h);
    },
    box(area) {
      boxFill.visible = boxLine.visible = Boolean(area);
      if (!area) return;
      place(boxFill, area.left, area.top, area.right - area.left, area.bottom - area.top);
      place(boxLine, area.left, area.top, area.right - area.left, area.bottom - area.top);
    },
    hit(x, y) {
      for (const panel of [panels.toast, panels.note, panels.minimap, panels.side, panels.top]) {
        const region = panel.hit(x, y);
        if (region) return { panel, region };
      }
      return null;
    },
    render() {
      renderer.autoClear = false;
      renderer.clearDepth();
      renderer.render(scene, camera);
      renderer.autoClear = true;
    },
    rect
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
