// Camera rig plus one pointer path for mouse and touch. The canvas-drawn UI
// gets first refusal on every press; the rest becomes tap, box select, hover,
// or camera motion.
import * as THREE from 'three';
import { MAP } from './materials.js';

const DRAG_THRESHOLD = 8;
const LONG_PRESS_MS = 450;
const MIN_DISTANCE = 5;
const MAX_DISTANCE = 34;

export function createCameraRig(camera) {
  const target = new THREE.Vector3(15, 0, 10.5);
  const state = { distance: 11, yaw: Math.PI / 4, goalYaw: Math.PI / 4, pitch: 0.92 };
  const plane = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0);
  const raycaster = new THREE.Raycaster();
  const ndc = new THREE.Vector2();

  function place() {
    const horizontal = Math.cos(state.pitch) * state.distance;
    camera.position.set(
      target.x + Math.sin(state.yaw) * horizontal,
      target.y + Math.sin(state.pitch) * state.distance,
      target.z + Math.cos(state.yaw) * horizontal
    );
    camera.lookAt(target);
  }
  function clamp() {
    target.x = THREE.MathUtils.clamp(target.x, -1, MAP.columns + 1);
    target.z = THREE.MathUtils.clamp(target.z, -1, MAP.rows + 1);
    state.distance = THREE.MathUtils.clamp(state.distance, MIN_DISTANCE, MAX_DISTANCE);
  }
  function groundAt(x, y) {
    ndc.set(x / window.innerWidth * 2 - 1, -(y / window.innerHeight) * 2 + 1);
    raycaster.setFromCamera(ndc, camera);
    return raycaster.ray.intersectPlane(plane, new THREE.Vector3());
  }
  return {
    target,
    state,
    groundAt,
    update(dt) {
      const turn = state.goalYaw - state.yaw;
      state.yaw += turn * Math.min(1, dt * 8);
      // Closer views tilt toward the horizon; distant views look down like a map.
      state.pitch = THREE.MathUtils.lerp(0.72, 1.12, (state.distance - MIN_DISTANCE) / (MAX_DISTANCE - MIN_DISTANCE));
      place();
    },
    // Keep the ground point grabbed at `from` under the pointer at `to`.
    drag(from, to) {
      target.add(from.clone().sub(to).setY(0));
      clamp();
      place();
    },
    zoom(factor) {
      state.distance *= factor;
      clamp();
    },
    rotate(radians, smooth = false) {
      state.goalYaw += radians;
      if (!smooth) state.yaw = state.goalYaw;
    },
    nudge(dx, dz) {
      const right = new THREE.Vector3(Math.cos(state.yaw), 0, -Math.sin(state.yaw));
      const forward = new THREE.Vector3(-Math.sin(state.yaw), 0, -Math.cos(state.yaw));
      target.addScaledVector(right, dx * state.distance * 0.05).addScaledVector(forward, dz * state.distance * 0.05);
      clamp();
    },
    lookAt(x, z) {
      target.set(x, 0, z);
      clamp();
    }
  };
}

function boxRect(start, event) {
  return {
    left: Math.min(start.x, event.clientX), right: Math.max(start.x, event.clientX),
    top: Math.min(start.y, event.clientY), bottom: Math.max(start.y, event.clientY)
  };
}

export function bindPointer(canvas, rig, handlers) {
  const pointers = new Map();
  let gesture = null;
  let longPress = 0;

  function cancelLongPress() {
    clearTimeout(longPress);
    longPress = 0;
  }
  function pinchState() {
    const [a, b] = [...pointers.values()];
    return {
      distance: Math.hypot(a.x - b.x, a.y - b.y),
      angle: Math.atan2(b.y - a.y, b.x - a.x),
      mid: { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }
    };
  }

  canvas.addEventListener('contextmenu', event => event.preventDefault());
  canvas.addEventListener('pointerdown', event => {
    canvas.setPointerCapture(event.pointerId);
    if (pointers.size === 0 && handlers.ui.down(event.clientX, event.clientY)) {
      pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
      gesture = { type: 'ui' };
      return;
    }
    if (gesture?.type === 'ui') return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    cancelLongPress();
    if (pointers.size === 2) {
      gesture = { type: 'pinch', last: pinchState(), anchor: rig.groundAt(pinchState().mid.x, pinchState().mid.y) };
      return;
    }
    const rotate = event.pointerType === 'mouse' && event.button !== 0;
    gesture = {
      type: 'pending', start: { x: event.clientX, y: event.clientY }, rotate,
      boxSelect: event.pointerType === 'mouse' && event.shiftKey && !rotate,
      additive: event.shiftKey, anchor: rig.groundAt(event.clientX, event.clientY), lastX: event.clientX
    };
    if (event.pointerType === 'touch') {
      longPress = setTimeout(() => {
        if (gesture?.type === 'pending') {
          gesture = { type: 'done' };
          handlers.tap(event.clientX, event.clientY, true);
          navigator.vibrate?.(15);
        }
      }, LONG_PRESS_MS);
    }
  });
  canvas.addEventListener('pointermove', event => {
    if (!pointers.has(event.pointerId)) {
      if (event.pointerType === 'mouse' && !handlers.ui.hover(event.clientX, event.clientY)) handlers.hover(event.clientX, event.clientY);
      return;
    }
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (!gesture) return;
    if (gesture.type === 'ui') {
      handlers.ui.move(event.clientX, event.clientY);
      return;
    }
    if (gesture.type === 'pinch' && pointers.size === 2) {
      const now = pinchState();
      rig.zoom(gesture.last.distance / Math.max(1, now.distance));
      rig.rotate(gesture.last.angle - now.angle);
      const ground = rig.groundAt(now.mid.x, now.mid.y);
      if (gesture.anchor && ground) rig.drag(gesture.anchor, ground);
      gesture.last = now;
      return;
    }
    if (gesture.type === 'pending' && Math.hypot(event.clientX - gesture.start.x, event.clientY - gesture.start.y) > DRAG_THRESHOLD) {
      cancelLongPress();
      gesture.type = gesture.rotate ? 'rotate' : gesture.boxSelect ? 'box' : 'pan';
    }
    if (gesture.type === 'pan' && gesture.anchor) {
      const ground = rig.groundAt(event.clientX, event.clientY);
      if (ground) rig.drag(gesture.anchor, ground);
    } else if (gesture.type === 'rotate') {
      rig.rotate((gesture.lastX - event.clientX) * 0.008);
      gesture.lastX = event.clientX;
    } else if (gesture.type === 'box') {
      handlers.boxDraw(boxRect(gesture.start, event));
    }
  });
  function end(event) {
    if (!pointers.has(event.pointerId)) return;
    pointers.delete(event.pointerId);
    cancelLongPress();
    if (gesture?.type === 'ui') {
      handlers.ui.up(event.clientX, event.clientY);
      gesture = null;
      return;
    }
    if (gesture?.type === 'pending' && pointers.size === 0) {
      handlers.tap(event.clientX, event.clientY, gesture.additive);
    } else if (gesture?.type === 'box') {
      handlers.boxDraw(null);
      handlers.boxSelect(boxRect(gesture.start, event));
    }
    if (pointers.size === 1 && gesture?.type === 'pinch') {
      const [remaining] = pointers.values();
      gesture = { type: 'pan', anchor: rig.groundAt(remaining.x, remaining.y) };
    } else if (pointers.size === 0) {
      gesture = null;
    }
  }
  canvas.addEventListener('pointerup', end);
  canvas.addEventListener('pointercancel', event => {
    handlers.boxDraw(null);
    gesture = { type: 'done' };
    end(event);
  });
  canvas.addEventListener('wheel', event => {
    event.preventDefault();
    if (handlers.ui.contains(event.clientX, event.clientY)) return;
    rig.zoom(Math.exp(event.deltaY * 0.0012));
  }, { passive: false });

  const held = new Set();
  window.addEventListener('keydown', event => {
    if (event.target instanceof HTMLInputElement) return;
    held.add(event.code);
    if (event.code === 'KeyQ') rig.rotate(-Math.PI / 4, true);
    if (event.code === 'KeyE') rig.rotate(Math.PI / 4, true);
    handlers.key(event);
  });
  window.addEventListener('keyup', event => held.delete(event.code));
  window.addEventListener('blur', () => held.clear());
  return {
    update(dt) {
      const x = (held.has('KeyD') || held.has('ArrowRight') ? 1 : 0) - (held.has('KeyA') || held.has('ArrowLeft') ? 1 : 0);
      const z = (held.has('KeyW') || held.has('ArrowUp') ? 1 : 0) - (held.has('KeyS') || held.has('ArrowDown') ? 1 : 0);
      if (x || z) rig.nudge(x * dt * 14, z * dt * 14);
    }
  };
}
