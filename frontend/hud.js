// DOM overlay: stockpile, selection card, anchored building popover, toasts,
// and the minimap. Real buttons only; the canvas never draws UI text.
import * as THREE from 'three';
import { BIOME_COLORS } from './terrain.js';

const RESOURCES = [
  ['wood', 'Wood', '#e7b56a', '<rect x="3" y="8" width="18" height="8" rx="4" fill="#a8743f"/><ellipse cx="19" cy="12" rx="2.6" ry="4" fill="#e7c08a"/><circle cx="19" cy="12" r="1.2" fill="#a8743f"/>'],
  ['food', 'Food', '#ef8a80', '<circle cx="9" cy="14" r="4" fill="#d1323a"/><circle cx="15" cy="15" r="3.6" fill="#b8262e"/><circle cx="12" cy="9" r="3.4" fill="#e04a4f"/><path d="M12 6c1-2 3-3 5-3" stroke="#5a8a3a" stroke-width="1.6" fill="none"/>'],
  ['stone', 'Stone', '#c9c6bb', '<path d="M4 17 7 8l7-3 6 5 1 7z" fill="#b7b2a5"/><path d="M7 8l7-3 1 6z" fill="#d6d2c6"/>'],
  ['gold', 'Gold', '#f2d25a', '<path d="M12 3 19 12 12 21 5 12z" fill="#ffd23f"/><path d="M12 3 19 12h-7z" fill="#fff0a0"/>'],
  ['iron', 'Iron', '#b9b2b0', '<path d="M5 16h14l-2-6H7z" fill="#7d7470"/><path d="M7 10h10l-1-2H8z" fill="#a49a96"/>'],
  ['clay', 'Clay', '#e0956a', '<path d="M8 5h8l-1 3c3 1 4 4 3 7-1 3-4 5-6 5s-5-2-6-5c-1-3 0-6 3-7z" fill="#c06c45"/>'],
  ['fiber', 'Fiber', '#c9d98a', '<path d="M12 21V6" stroke="#9aa84a" stroke-width="1.6"/><ellipse cx="12" cy="5" rx="2" ry="3" fill="#e6d77a"/><ellipse cx="9" cy="10" rx="1.6" ry="2.6" fill="#e6d77a" transform="rotate(-30 9 10)"/><ellipse cx="15" cy="10" rx="1.6" ry="2.6" fill="#e6d77a" transform="rotate(30 15 10)"/>']
];
const TECHNOLOGIES = {
  forestry: ['Forestry', 'Wood +20%'],
  agriculture: ['Agriculture', 'Food +20%'],
  masonry: ['Masonry', 'Stone, clay +20%'],
  mining: ['Mining', 'Gold, iron +20%'],
  textiles: ['Textiles', 'Fiber +20%']
};
const PREREQUISITE = { mining: 'masonry', textiles: 'agriculture' };
const FRIENDLY_ERRORS = {
  'unit is busy': 'That villager is busy with its current task.',
  'destination cell is occupied': 'Something already stands there.',
  'target is unreachable': 'No path leads there.',
  'build site is blocked or outside the world': 'The town center needs a clear 2×2 site.',
  'insufficient wood': 'You need 20 wood to build.',
  'insufficient food': 'You need 50 food to train a villager.'
};
const ACTIVITY_TEXT = {
  idle: 'Idle', move: 'Walking', build: 'Building',
  to_resource: 'Heading to gather', gathering: 'Gathering', returning: 'Carrying to town center', depositing: 'Depositing'
};

export function createHud({ onSpeed, onReset, onTrain, onResearch, onBuild, onCancel, onMinimap }) {
  const $ = id => document.getElementById(id);
  const list = $('stockpiles');
  const values = {};
  for (const [key, label, color, icon] of RESOURCES) {
    const item = document.createElement('li');
    item.title = label;
    item.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true">${icon}</svg><span class="sr">${label}</span><strong style="color:${color}">0</strong>`;
    item.querySelector('.sr').style.cssText = 'position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)';
    list.append(item);
    values[key] = { item, strong: item.querySelector('strong'), last: 0 };
  }
  const popover = $('building-popover');
  const research = $('research-actions');
  const researchButtons = {};
  for (const [key, [label, effect]] of Object.entries(TECHNOLOGIES)) {
    const button = document.createElement('button');
    button.type = 'button';
    button.innerHTML = `${label}<small>${effect} · 40 food, 20 wood</small>`;
    button.addEventListener('click', () => onResearch(key));
    research.append(button);
    researchButtons[key] = button;
  }
  document.querySelectorAll('#speed-controls button').forEach(button => {
    button.addEventListener('click', () => onSpeed(Number(button.dataset.speed)));
  });
  $('reset-world').addEventListener('click', () => {
    if (confirm('Reset the world? All progress will be lost.')) onReset();
  });
  $('train-villager').addEventListener('click', onTrain);
  $('build').addEventListener('click', onBuild);
  $('cancel').addEventListener('click', onCancel);

  let toastTimer = 0;
  function toast(message) {
    const element = $('toast');
    element.textContent = FRIENDLY_ERRORS[message] || message;
    element.classList.add('visible');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => element.classList.remove('visible'), 2400);
  }

  const minimap = $('minimap');
  const mini = minimap.getContext('2d');
  function minimapCell(event) {
    const rect = minimap.getBoundingClientRect();
    return { x: (event.clientX - rect.left) / rect.width * 30, z: (event.clientY - rect.top) / rect.height * 20 };
  }
  minimap.addEventListener('pointerdown', event => {
    minimap.setPointerCapture(event.pointerId);
    onMinimap(minimapCell(event));
  });
  minimap.addEventListener('pointermove', event => {
    if (event.buttons) onMinimap(minimapCell(event));
  });

  function drawMinimap(world, viewCorners) {
    const sx = minimap.width / world.columns;
    const sy = minimap.height / world.rows;
    for (const cell of world.terrain) {
      const [r, g, b] = cell.biome ? BIOME_COLORS[cell.biome] : [14, 18, 26];
      const dim = cell.visibility === 'visible' ? 1 : 0.5;
      mini.fillStyle = `rgb(${r * dim},${g * dim},${b * dim})`;
      mini.fillRect(cell.column * sx, cell.row * sy, sx + 0.5, sy + 0.5);
    }
    mini.fillStyle = '#e8dcc0';
    for (const resource of world.resources) {
      if (resource.amount > 0) mini.fillRect(resource.cell.column * sx + 1, resource.cell.row * sy + 1, sx - 2, sy - 2);
    }
    for (const building of world.buildings) {
      mini.fillStyle = building.construction === null ? '#3f8fd8' : '#9cc4ea';
      mini.fillRect(building.origin.column * sx, building.origin.row * sy, building.columns * sx, building.rows * sy);
    }
    mini.fillStyle = '#ffffff';
    for (const unit of world.units) mini.fillRect(unit.position.x * sx - 1.5, unit.position.y * sy - 1.5, 3, 3);
    if (viewCorners.every(Boolean)) {
      mini.strokeStyle = 'rgba(255,240,190,.9)';
      mini.lineWidth = 1;
      mini.beginPath();
      viewCorners.forEach((corner, index) => mini[index ? 'lineTo' : 'moveTo'](corner.x * sx, corner.z * sy));
      mini.closePath();
      mini.stroke();
    }
  }

  const projected = new THREE.Vector3();
  return {
    toast,
    setConnection(online) {
      $('connection').textContent = online ? 'Online' : 'Reconnecting';
      $('connection').classList.toggle('online', online);
    },
    update(world, selection, buildMode) {
      for (const [key] of RESOURCES) {
        const value = Math.floor(world.stockpile[key] + 1e-6);
        const entry = values[key];
        if (value !== entry.last) {
          entry.strong.textContent = value;
          if (value > entry.last) {
            entry.item.classList.remove('bump');
            void entry.item.offsetWidth;
            entry.item.classList.add('bump');
          }
          entry.last = value;
        }
      }
      $('clock').textContent = `Tick ${world.tick}`;
      document.querySelectorAll('#speed-controls button').forEach(button => {
        button.classList.toggle('active', Number(button.dataset.speed) === world.simulation_speed);
      });

      const selectedUnits = world.units.filter(unit => selection.units.has(unit.id));
      const building = world.buildings.find(b => b.id === selection.building);
      let title = 'Tap a villager';
      let detail = 'Drag to pan · pinch or scroll to zoom';
      if (selectedUnits.length === 1) {
        const unit = selectedUnits[0];
        const state = unit.action.type === 'gather' ? unit.action.phase : unit.action.type;
        title = unit.id.replace('villager-', 'Villager ');
        detail = `${ACTIVITY_TEXT[state] || state}${unit.cargo ? ` · carrying ${Math.floor(unit.cargo.amount)} ${unit.cargo.kind}` : ''}`;
      } else if (selectedUnits.length > 1) {
        title = `${selectedUnits.length} villagers`;
        const idle = selectedUnits.filter(unit => unit.action.type === 'idle').length;
        detail = `${idle} idle · tap ground to move together`;
      } else if (building) {
        title = building.construction === null ? 'Town Center' : 'Town Center foundation';
        detail = building.construction === null ? 'Trains villagers and researches' : `Under construction · ${Math.round(building.construction / 4 * 100)}%`;
      }
      $('selection-title').textContent = title;
      $('selection-detail').textContent = detail;
      $('build').disabled = selectedUnits.length === 0 || world.stockpile.wood < 20;
      $('build').classList.toggle('active', buildMode);
      $('build').hidden = buildMode;
      $('cancel').hidden = !buildMode;
      $('placement').hidden = !buildMode;

      const complete = building && building.construction === null;
      popover.hidden = !complete;
      if (complete) {
        const job = building.job;
        $('building-job').hidden = !job;
        if (job) {
          const total = job.type === 'produce' ? 6 : 8;
          $('building-job-label').textContent = job.type === 'produce' ? 'Training villager' : `Researching ${TECHNOLOGIES[job.technology][0]}`;
          $('building-progress').value = Math.min(1, job.elapsed_seconds / total);
        }
        $('train-villager').disabled = Boolean(job) || world.stockpile.food < 50;
        for (const [key, button] of Object.entries(researchButtons)) {
          const done = world.researched_technologies.includes(key);
          const blocked = PREREQUISITE[key] && !world.researched_technologies.includes(PREREQUISITE[key]);
          button.classList.toggle('done', done);
          button.disabled = done || blocked || Boolean(job) || world.stockpile.food < 40 || world.stockpile.wood < 20;
          button.title = done ? 'Researched' : blocked ? `Requires ${TECHNOLOGIES[PREREQUISITE[key]][0]}` : '';
        }
      }
    },
    positionPopover(anchor, camera) {
      if (popover.hidden || !anchor) return;
      projected.copy(anchor).setY(anchor.y + 1.6).project(camera);
      const x = THREE.MathUtils.clamp((projected.x + 1) / 2 * window.innerWidth, 130, window.innerWidth - 130);
      const y = THREE.MathUtils.clamp((1 - projected.y) / 2 * window.innerHeight, popover.offsetHeight + 70, window.innerHeight);
      popover.style.left = `${x}px`;
      popover.style.top = `${y}px`;
    },
    drawMinimap
  };
}
