// DOM overlay drawn as inked paper: coin medallions for the stockpile on top and
// a console of illustrated cards at the bottom (commands, info, minimap). Real
// buttons only; the canvas never draws UI.
import { BIOME_COLORS } from './terrain.js';

const RESOURCES = [
  ['wood', 'Wood', '<rect x="2" y="8" width="18" height="8" rx="4" fill="#b07a42" stroke="#3a2210"/><ellipse cx="19" cy="12" rx="3" ry="4" fill="#f0cf96" stroke="#3a2210"/><circle cx="19" cy="12" r="1.3" fill="#b07a42"/>'],
  ['food', 'Food', '<circle cx="8.5" cy="14" r="4.2" fill="#e0303f" stroke="#4a0c12"/><circle cx="15.5" cy="15" r="3.8" fill="#c21f30" stroke="#4a0c12"/><circle cx="12" cy="8.5" r="3.6" fill="#f0485a" stroke="#4a0c12"/><path d="M12 5c1-2 3-3 5-3" stroke="#3f7a2a" stroke-width="1.8" fill="none"/>'],
  ['stone', 'Stone', '<path d="M3 18 6 8l8-4 7 6 1 8z" fill="#d6d0c4" stroke="#3a3228"/><path d="M6 8l8-4 1 7z" fill="#f2eee6"/>'],
  ['gold', 'Gold', '<path d="M12 2 20 12 12 22 4 12z" fill="#ffcf2e" stroke="#6a4a00"/><path d="M12 2 20 12h-8z" fill="#fff2a0"/>'],
  ['iron', 'Iron', '<path d="M4 17h16l-2.5-7h-11z" fill="#8a8fa0" stroke="#2a2e38"/><path d="M6.5 10h11L16 7.5H8z" fill="#c4c9d6" stroke="#2a2e38"/>'],
  ['clay', 'Clay', '<path d="M8 4h8l-1 3c3 1 4.5 4 3.5 7-1 3.5-4 6-6.5 6S7 17.5 6 14c-1-3 .5-6 3.5-7z" fill="#e2784c" stroke="#5a1e0c"/>'],
  ['fiber', 'Fiber', '<path d="M12 22V6" stroke="#6f8a2a" stroke-width="1.8"/><ellipse cx="12" cy="5" rx="2.2" ry="3.2" fill="#f2df6a" stroke="#6a5a10"/><ellipse cx="8.5" cy="10.5" rx="1.8" ry="2.8" fill="#f2df6a" stroke="#6a5a10" transform="rotate(-30 8.5 10.5)"/><ellipse cx="15.5" cy="10.5" rx="1.8" ry="2.8" fill="#f2df6a" stroke="#6a5a10" transform="rotate(30 15.5 10.5)"/>']
];
const ICONS = {
  build: '<path d="M4 20V11l8-6 8 6v9z" fill="#e0452e" stroke="#2c1b0e" stroke-width="1.5"/><path d="M7 20v-7h10v7" fill="#fbefd2" stroke="#2c1b0e" stroke-width="1.5"/><path d="M10.5 20v-4h3v4" fill="#6b4528"/>',
  cancel: '<path d="M6 6l12 12M18 6 6 18" stroke="#7d1f17" stroke-width="3.5" stroke-linecap="round"/>',
  villager: '<circle cx="12" cy="7" r="3.6" fill="#f2c9a0" stroke="#2c1b0e" stroke-width="1.3"/><path d="M5 21v-4.5C5 13 8 11.5 12 11.5s7 1.5 7 5V21z" fill="#5b8e7d" stroke="#2c1b0e" stroke-width="1.3"/><path d="M8.5 12.2h7l-1 1.8h-5z" fill="#2f6fe0"/>',
  forestry: '<path d="M12 3 6 13h3l-3 5h12l-3-5h3z" fill="#4f9a5a" stroke="#1e3a20" stroke-width="1.3"/><rect x="11" y="18" width="2" height="4" fill="#6b4528"/>',
  agriculture: '<path d="M12 22V8" stroke="#6f8a2a" stroke-width="1.8"/><ellipse cx="12" cy="6" rx="2.4" ry="3.6" fill="#f2c84a" stroke="#6a4a00"/><ellipse cx="8.5" cy="12" rx="2" ry="3" fill="#f2c84a" stroke="#6a4a00" transform="rotate(-35 8.5 12)"/><ellipse cx="15.5" cy="12" rx="2" ry="3" fill="#f2c84a" stroke="#6a4a00" transform="rotate(35 15.5 12)"/>',
  masonry: '<rect x="3" y="14" width="8" height="5" fill="#d6d0c4" stroke="#3a3228"/><rect x="13" y="14" width="8" height="5" fill="#d6d0c4" stroke="#3a3228"/><rect x="8" y="8" width="8" height="5" fill="#e2784c" stroke="#5a1e0c"/>',
  mining: '<path d="M4 9c4-5 12-5 16 0" stroke="#5a5f6e" stroke-width="2.6" fill="none" stroke-linecap="round"/><path d="M12 6v15" stroke="#8a5a30" stroke-width="2.4" stroke-linecap="round"/><path d="M16 17l3 3-3 2-2-3z" fill="#ffcf2e" stroke="#6a4a00"/>',
  textiles: '<ellipse cx="12" cy="12" rx="6" ry="8" fill="#f2df6a" stroke="#6a5a10" stroke-width="1.3"/><path d="M6.5 9h11M6 12h12M6.5 15h11" stroke="#b8a03a"/><path d="M18 18l3 3" stroke="#7a7f8c" stroke-width="1.6"/>',
  townCenter: '<path d="M3 21V12l9-7 9 7v9z" fill="#e0452e" stroke="#2c1b0e" stroke-width="1.3"/><rect x="6" y="13" width="12" height="8" fill="#fbefd2" stroke="#2c1b0e" stroke-width="1.3"/><rect x="10.5" y="16" width="3" height="5" fill="#6b4528"/><path d="M12 5V1.5l4 1.5-4 1.5" fill="#2f6fe0" stroke="#2c1b0e" stroke-width=".8"/>',
  group: '<circle cx="8" cy="8" r="3" fill="#f2c9a0" stroke="#2c1b0e"/><circle cx="16" cy="8" r="3" fill="#d8a47a" stroke="#2c1b0e"/><path d="M2 20v-3.5c0-3 2.5-4.5 6-4.5s6 1.5 6 4.5V20z" fill="#5b8e7d" stroke="#2c1b0e"/><path d="M10 20v-3.5c0-3 2.5-4.5 6-4.5s6 1.5 6 4.5V20z" fill="#c8553d" stroke="#2c1b0e"/>',
  scroll: '<path d="M6 4h11a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H7" fill="#fbefd2" stroke="#6b4528" stroke-width="1.4"/><path d="M6 4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h1V6a2 2 0 0 0-1-2z" fill="#e2c992" stroke="#6b4528" stroke-width="1.4"/><path d="M10 9h6M10 12h6M10 15h4" stroke="#8a6524"/>'
};
const TECHNOLOGIES = {
  forestry: ['Forestry', 'Wood +20%'],
  agriculture: ['Agriculture', 'Food +20%'],
  masonry: ['Masonry', 'Stone and clay +20%'],
  mining: ['Mining', 'Gold and iron +20%'],
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
  idle: 'Awaiting orders', move: 'Walking', build: 'Building',
  to_resource: 'Heading out to gather', gathering: 'Gathering', returning: 'Carrying goods home', depositing: 'Unloading'
};
const svg = (body, size = 24) => `<svg viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true">${body}</svg>`;

export function createHud({ onSpeed, onReset, onTrain, onResearch, onBuild, onCancel, onMinimap }) {
  const $ = id => document.getElementById(id);
  const values = {};
  for (const [key, label, icon] of RESOURCES) {
    const item = document.createElement('li');
    item.title = label;
    item.innerHTML = `<span class="coin">${svg(icon)}</span><span class="sr">${label}</span><strong>0</strong>`;
    $('stockpiles').append(item);
    values[key] = { item, strong: item.querySelector('strong'), last: 0 };
  }
  document.querySelectorAll('#speed-controls button').forEach(button => {
    button.addEventListener('click', () => onSpeed(Number(button.dataset.speed)));
  });
  $('reset-world').addEventListener('click', () => {
    if (confirm('Reset the world? All progress will be lost.')) onReset();
  });

  // The command grid is rebuilt only when its context changes, so a button is
  // never replaced between press and release; enabled states update in place.
  const commands = $('commands');
  let commandContext = '';
  let buttons = [];
  function command(icon, label, cost, onClick, key) {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'command';
    button.title = label;
    button.setAttribute('aria-label', cost ? `${label}, ${cost}` : label);
    button.innerHTML = `${svg(ICONS[icon], 34)}${cost ? `<span class="cost">${cost}</span>` : ''}${key ? `<span class="key">${key}</span>` : ''}`;
    button.addEventListener('click', onClick);
    return button;
  }
  function renderCommands(context, specs) {
    if (context === commandContext) return;
    commandContext = context;
    buttons = specs.map(spec => command(...spec));
    commands.replaceChildren(...buttons);
    const slots = Math.max(0, (buttons.length <= 4 ? 4 : 8) - buttons.length);
    for (let i = 0; i < slots; i += 1) commands.append(Object.assign(document.createElement('span'), { className: 'slot' }));
  }

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
      const [r, g, b] = cell.biome ? BIOME_COLORS[cell.biome] : [233, 216, 176];
      const dim = cell.visibility === 'visible' ? 1 : cell.visibility === 'explored' ? 0.7 : 1;
      mini.fillStyle = `rgb(${r * dim},${g * dim},${b * dim})`;
      mini.fillRect(cell.column * sx, cell.row * sy, sx + 0.5, sy + 0.5);
    }
    mini.fillStyle = '#3b2614';
    for (const resource of world.resources) {
      if (resource.amount > 0) mini.fillRect(resource.cell.column * sx + 1, resource.cell.row * sy + 1, sx - 2, sy - 2);
    }
    for (const building of world.buildings) {
      mini.fillStyle = building.construction === null ? '#d0532e' : '#f0a888';
      mini.fillRect(building.origin.column * sx, building.origin.row * sy, building.columns * sx, building.rows * sy);
    }
    mini.fillStyle = '#1f6fd0';
    for (const unit of world.units) mini.fillRect(unit.position.x * sx - 2, unit.position.y * sy - 2, 4, 4);
    if (viewCorners.every(Boolean)) {
      mini.strokeStyle = '#3b2614';
      mini.lineWidth = 1.5;
      mini.beginPath();
      viewCorners.forEach((corner, index) => mini[index ? 'lineTo' : 'moveTo'](corner.x * sx, corner.z * sy));
      mini.closePath();
      mini.stroke();
    }
  }

  function setInfo(icon, title, detail, job) {
    if ($('portrait').dataset.icon !== icon) {
      $('portrait').innerHTML = svg(ICONS[icon], 44);
      $('portrait').dataset.icon = icon;
    }
    $('selection-title').textContent = title;
    $('selection-detail').textContent = detail;
    $('job').hidden = !job;
    if (job) {
      $('job-label').textContent = job.label;
      $('job-fill').style.width = `${Math.round(Math.min(1, job.progress) * 100)}%`;
    }
  }

  return {
    toast,
    drawMinimap,
    canBuild: () => buttons[0]?.dataset.role === 'build' && !buttons[0].disabled,
    setConnection(online) {
      $('connection').classList.toggle('online', online);
      $('connection').title = online ? 'Connected' : 'Reconnecting';
    },
    update(world, selection, buildMode) {
      for (const [key] of RESOURCES) {
        const value = Math.floor(world.stockpile[key] + 1e-6);
        const entry = values[key];
        if (value === entry.last) continue;
        entry.strong.textContent = value;
        if (value > entry.last) {
          entry.item.classList.remove('bump');
          void entry.item.offsetWidth;
          entry.item.classList.add('bump');
        }
        entry.last = value;
      }
      $('clock').textContent = `Tick ${world.tick}`;
      document.querySelectorAll('#speed-controls button').forEach(button => {
        button.classList.toggle('active', Number(button.dataset.speed) === world.simulation_speed);
      });
      $('placement').hidden = !buildMode;

      const selectedUnits = world.units.filter(unit => selection.units.has(unit.id));
      const building = world.buildings.find(b => b.id === selection.building);
      if (selectedUnits.length) {
        if (selectedUnits.length === 1) {
          const unit = selectedUnits[0];
          const state = unit.action.type === 'gather' ? unit.action.phase : unit.action.type;
          const cargo = unit.cargo ? ` · carrying ${Math.floor(unit.cargo.amount)} ${unit.cargo.kind}` : '';
          setInfo('villager', unit.id.replace('villager-', 'Villager '), `${ACTIVITY_TEXT[state] || state}${cargo}`);
        } else {
          const idle = selectedUnits.filter(unit => unit.action.type === 'idle').length;
          setInfo('group', `${selectedUnits.length} villagers`, `${idle} awaiting orders · tap ground to move together`);
        }
        if (buildMode) {
          renderCommands('placing', [['cancel', 'Cancel placement', '', onCancel, 'Esc']]);
        } else {
          renderCommands('villager', [['build', 'Build town center', '20 wood', onBuild, 'B']]);
          buttons[0].dataset.role = 'build';
          buttons[0].disabled = world.stockpile.wood < 20;
        }
      } else if (building && building.construction !== null) {
        setInfo('townCenter', 'Town Center foundation', 'Villagers raise the walls; tap it with villagers selected to help',
          { label: 'Construction', progress: building.construction / 4 });
        renderCommands('foundation', []);
      } else if (building) {
        const job = building.job;
        setInfo('townCenter', 'Town Center', 'Trains villagers and studies new crafts', job && {
          label: job.type === 'produce' ? 'Training a villager' : `Researching ${TECHNOLOGIES[job.technology][0]}`,
          progress: job.elapsed_seconds / (job.type === 'produce' ? 6 : 8)
        });
        renderCommands('town-center', [
          ['villager', 'Train villager', '50 food', onTrain],
          ...Object.entries(TECHNOLOGIES).map(([key, [label, effect]]) => [key, `${label}: ${effect}`, '40F 20W', () => onResearch(key)])
        ]);
        buttons[0].disabled = Boolean(job) || world.stockpile.food < 50;
        Object.keys(TECHNOLOGIES).forEach((key, index) => {
          const button = buttons[index + 1];
          const done = world.researched_technologies.includes(key);
          const blocked = PREREQUISITE[key] && !world.researched_technologies.includes(PREREQUISITE[key]);
          button.classList.toggle('done', done);
          button.disabled = done || blocked || Boolean(job) || world.stockpile.food < 40 || world.stockpile.wood < 20;
          button.title = `${TECHNOLOGIES[key][0]}: ${TECHNOLOGIES[key][1]}${done ? ' (researched)' : blocked ? ` (requires ${TECHNOLOGIES[PREREQUISITE[key]][0]})` : ''}`;
        });
      } else {
        setInfo('scroll', 'Your village awaits', 'Tap a villager to give orders, or the town center to train and research');
        renderCommands('none', []);
      }
    }
  };
}
