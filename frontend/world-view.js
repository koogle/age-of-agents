// Turns authoritative snapshots into animated scene objects. Nothing here
// decides game outcomes; it only interpolates and dresses what the server says.
import * as THREE from 'three';
import { heightAt } from './terrain.js';
import {
  ACTIVITY_FOR, TEAM_COLOR, createRing, createResource, createTownCenter, createVillager,
  poseTownCenter, poseVillager, setResourceAmount
} from './models.js';

const TICK_MS = 100;
// Units are drawn this many ticks behind the newest snapshot, so jittery
// arrivals still always have a sample to interpolate toward.
const PLAYOUT_TICKS = 1.6;
const HISTORY = 6;
const proxyMaterial = new THREE.MeshBasicMaterial({ visible: false });
const unitProxy = new THREE.CylinderGeometry(0.28, 0.28, 0.75, 8).translate(0, 0.37, 0);
const resourceProxy = new THREE.BoxGeometry(0.9, 0.7, 0.9).translate(0, 0.35, 0);
const FX_FOR = { chop: ['chips', 1.05], mine: ['sparks', 1.05], dig: ['dirt', 1.25], forage: ['leaves', 1.0], build: ['dust', 0.7] };
const LABEL = { wood: 'wood', food: 'food', stone: 'stone', gold: 'gold', iron: 'iron', clay: 'clay', fiber: 'fiber' };

export function createWorldView(scene, effects) {
  const units = new Map();
  const resources = new Map();
  const buildings = new Map();
  const markers = [];
  let world = null;
  // Presentation clock in server ticks; advanced smoothly every frame and
  // nudged toward the newest tick, never reset by an individual snapshot.
  let renderTick = null;
  let latestTick = 0;

  function proxy(geometry, type, id) {
    const mesh = new THREE.Mesh(geometry, proxyMaterial);
    mesh.userData.pick = { type, id };
    return mesh;
  }
  function ground(x, z) {
    return new THREE.Vector3(x, heightAt(x, z), z);
  }
  function center(building) {
    return { x: building.origin.column + building.columns / 2, z: building.origin.row + building.rows / 2 };
  }

  function sync(next, now) {
    const firstSync = world === null;
    world = next;
    latestTick = next.tick;
    if (renderTick === null || Math.abs(latestTick - PLAYOUT_TICKS - renderTick) > 8) renderTick = latestTick - PLAYOUT_TICKS;
    const terrainBiome = cell => next.terrain[cell.row * next.columns + cell.column]?.biome;

    const seenResources = new Set();
    for (const resource of next.resources) {
      seenResources.add(resource.id);
      let entry = resources.get(resource.id);
      if (!entry) {
        const root = createResource(resource, terrainBiome(resource.cell));
        root.position.copy(ground(resource.cell.column + 0.5, resource.cell.row + 0.5));
        root.add(proxy(resourceProxy, 'resource', resource.id));
        scene.add(root);
        entry = { root, amount: -1 };
        resources.set(resource.id, entry);
      }
      if (entry.amount !== resource.amount) {
        setResourceAmount(entry.root, resource.amount / resource.capacity);
        entry.amount = resource.amount;
      }
      entry.data = resource;
    }
    removeMissing(resources, seenResources);

    const seenBuildings = new Set();
    for (const building of next.buildings) {
      seenBuildings.add(building.id);
      let entry = buildings.get(building.id);
      const { x, z } = center(building);
      if (!entry) {
        const root = createTownCenter();
        root.position.copy(ground(x, z));
        const box = new THREE.BoxGeometry(building.columns, 1.3, building.rows).translate(0, 0.65, 0);
        root.add(proxy(box, 'building', building.id));
        const ring = createRing(1.45, TEAM_COLOR, 0.85);
        ring.position.y = 0.03;
        ring.visible = false;
        root.add(ring);
        scene.add(root);
        entry = { root, ring, smokeAt: 0 };
        buildings.set(building.id, entry);
      } else if (entry.data.construction !== null && building.construction === null) {
        for (let i = 0; i < 4; i += 1) effects.burst('dust', ground(x + (i % 2 - 0.5), z + (i < 2 ? -0.5 : 0.5)));
        effects.floatText('Town Center complete', ground(x, z).setY(1.6));
      }
      entry.data = building;
    }
    removeMissing(buildings, seenBuildings);

    const seenUnits = new Set();
    for (const unit of next.units) {
      seenUnits.add(unit.id);
      const target = ground(unit.position.x, unit.position.y);
      let entry = units.get(unit.id);
      if (!entry) {
        const root = createVillager(unit.id);
        root.position.copy(target);
        root.add(proxy(unitProxy, 'unit', unit.id));
        const ring = createRing(0.3, TEAM_COLOR);
        ring.position.y = 0.02;
        ring.visible = false;
        root.add(ring);
        scene.add(root);
        entry = { root, ring, samples: [], velocity: new THREE.Vector3(), facing: 0, nextFx: 0 };
        units.set(unit.id, entry);
        // A villager appearing after the first snapshot was just trained.
        if (!firstSync) effects.burst('dust', target);
      } else {
        const delivered = entry.data.cargo && !unit.cargo;
        if (delivered) {
          const cargo = entry.data.cargo;
          effects.floatText(`+${Math.round(cargo.amount)} ${LABEL[cargo.kind] || cargo.kind}`, target.clone().setY(target.y + 0.8));
        }
      }
      // A command snapshot repeats the current tick: replace that sample.
      const last = entry.samples[entry.samples.length - 1];
      if (last && last.tick >= next.tick) last.position.copy(target);
      else entry.samples.push({ tick: next.tick, position: target });
      if (entry.samples.length > HISTORY) entry.samples.shift();
      entry.data = unit;
    }
    removeMissing(units, seenUnits);
  }

  function removeMissing(map, seen) {
    for (const [id, entry] of map) {
      if (!seen.has(id)) {
        scene.remove(entry.root);
        map.delete(id);
      }
    }
  }

  function workTarget(unit) {
    const action = unit.action;
    if (action.type === 'gather' && action.phase === 'gathering') {
      const resource = resources.get(action.resource_id)?.data;
      return resource && { x: resource.cell.column + 0.5, z: resource.cell.row + 0.5, kind: resource.kind };
    }
    if (action.type === 'build') {
      const building = buildings.get(action.building_id)?.data;
      return building && { ...center(building), kind: null };
    }
    return null;
  }

  function sampleAt(samples, tick, out) {
    let i = samples.length - 1;
    while (i > 0 && samples[i - 1].tick > tick) i -= 1;
    const b = samples[i];
    const a = samples[i - 1];
    if (!a || tick >= b.tick) return out.copy(b.position);
    if (tick <= a.tick) return out.copy(a.position);
    return out.lerpVectors(a.position, b.position, (tick - a.tick) / (b.tick - a.tick));
  }

  const previous = new THREE.Vector3();
  function frame(now, time, dt, selection) {
    if (renderTick !== null) {
      // Run slightly fast or slow to hold the playout buffer; stop at the
      // newest tick when the simulation pauses.
      const lag = latestTick - PLAYOUT_TICKS - renderTick;
      const rate = THREE.MathUtils.clamp(1 + lag * 0.35, 0.6, 1.6);
      renderTick = Math.min(latestTick, renderTick + (dt * 1000 / TICK_MS) * rate);
    }
    for (const entry of units.values()) {
      const unit = entry.data;
      previous.copy(entry.root.position);
      sampleAt(entry.samples, renderTick, entry.root.position);
      if (dt > 0) entry.velocity.lerp(previous.sub(entry.root.position).multiplyScalar(-1 / dt), Math.min(1, dt * 10));
      const dx = entry.velocity.x;
      const dz = entry.velocity.z;
      const moving = Math.hypot(dx, dz) > 0.05;
      const work = moving ? null : workTarget(unit);
      let activity = moving ? 'walk' : 'idle';
      let desired = entry.facing;
      if (moving) desired = Math.atan2(dx, dz);
      if (work) {
        activity = work.kind ? ACTIVITY_FOR[work.kind] || 'forage' : 'build';
        desired = Math.atan2(work.x - entry.root.position.x, work.z - entry.root.position.z);
        if (time >= entry.nextFx) {
          const [style, period] = FX_FOR[activity];
          entry.nextFx = time + period;
          const at = new THREE.Vector3((entry.root.position.x + work.x) / 2, entry.root.position.y + 0.12, (entry.root.position.z + work.z) / 2);
          effects.burst(style, at);
        }
      }
      const turn = Math.atan2(Math.sin(desired - entry.facing), Math.cos(desired - entry.facing));
      entry.facing += turn * Math.min(1, dt * 12);
      entry.root.rotation.y = entry.facing;
      poseVillager(entry.root, activity, work?.kind, unit.cargo?.kind || null, time);
      const selected = selection.units.has(unit.id);
      entry.ring.visible = selected;
      if (selected) entry.ring.scale.setScalar(0.3 + Math.sin(time * 5) * 0.015);
    }
    for (const entry of buildings.values()) {
      const building = entry.data;
      const complete = building.construction === null;
      const working = complete && building.job !== null;
      poseTownCenter(entry.root, complete ? null : building.construction / 4, working, time);
      entry.ring.visible = selection.building === building.id;
      if (complete && time >= entry.smokeAt) {
        entry.smokeAt = time + (working ? 0.35 : 0.9);
        const chimney = entry.root.userData.parts.chimney.clone().add(entry.root.position);
        effects.burst('smoke', chimney);
      }
    }
    for (let i = markers.length - 1; i >= 0; i -= 1) {
      const marker = markers[i];
      const age = (time - marker.userData.born) / 0.8;
      if (age >= 1) {
        scene.remove(marker);
        markers.splice(i, 1);
        continue;
      }
      marker.scale.setScalar(0.15 + age * 0.35);
      marker.material.opacity = 0.9 * (1 - age);
    }
  }

  function markTarget(x, z, color, time) {
    const marker = createRing(0.2, color);
    marker.material = marker.material.clone();
    marker.position.copy(ground(x, z)).add(new THREE.Vector3(0, 0.03, 0));
    marker.userData.born = time;
    scene.add(marker);
    markers.push(marker);
  }

  function pickables() {
    const list = [];
    for (const map of [units, resources, buildings]) {
      for (const entry of map.values()) {
        // Stumps and stripped bushes are scenery, not targets.
        if (map === resources && entry.data.amount <= 0) continue;
        entry.root.traverse(object => { if (object.userData.pick) list.push(object); });
      }
    }
    return list;
  }

  return {
    sync, frame, markTarget, pickables,
    unitPositions: () => [...units.values()].map(entry => ({ id: entry.data.id, position: entry.root.position })),
    get world() { return world; }
  };
}

// A translucent town center that follows the pointer during placement.
export function createBuildGhost(scene) {
  const root = createTownCenter();
  const valid = new THREE.MeshBasicMaterial({ color: 0x9fe07a, transparent: true, opacity: 0.45, depthWrite: false });
  const invalid = new THREE.MeshBasicMaterial({ color: 0xe0604a, transparent: true, opacity: 0.45, depthWrite: false });
  root.traverse(object => {
    object.raycast = () => {};
    if (object.isMesh) object.castShadow = false;
  });
  root.userData.parts.scaffold.visible = false;
  root.visible = false;
  scene.add(root);
  return {
    show(origin, ok) {
      root.visible = true;
      root.position.set(origin.column + 1, heightAt(origin.column + 1, origin.row + 1) + 0.02, origin.row + 1);
      const material = ok ? valid : invalid;
      root.traverse(object => { if (object.isMesh) object.material = material; });
    },
    hide() { root.visible = false; }
  };
}
