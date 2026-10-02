// The HUD, painted inside the WebGL canvas and mostly out of the way, after
// assets/reference/diorama_primary.webp: a round globe minimap, a small speed
// pill, a resource pill that lists only what you have, and glossy medallion
// buttons that appear only when something is selected. A visually hidden DOM
// mirror of every button keeps keyboard and screen-reader access.
import { STYLE, createUiLayer, iconImage } from './ui-layer.js';
import { ACTIVITY_TEXT, ART, FRIENDLY_ERRORS, ICONS, PREREQUISITE, RESOURCES, TECHNOLOGIES } from './hud-content.js';
import { BIOME_COLORS } from './terrain.js';

const ALWAYS_SHOWN = new Set(['wood', 'food']);

export function createHud(renderer, actions) {
  const layer = createUiLayer(renderer);
  const { view } = layer;
  const state = {
    world: null, selection: { units: new Set(), building: null }, buildMode: false, online: false,
    hover: null, pressed: null, dragGlobe: false, toast: '', toastUntil: 0, bumps: {}, last: {},
    dirty: true, globeDirty: true, corners: [], commands: [], a11yKey: ''
  };
  const icon = body => iconImage(body, () => { state.dirty = true; });
  document.fonts?.ready.then(() => { state.dirty = true; });

  // Generated illustrations (assets/ui) replace the vector placeholders as
  // soon as they load; until then, or if one is missing, the placeholder draws.
  const art = new Map();
  function picture(name, fallback) {
    if (!art.has(name)) {
      const image = new Image();
      image.onload = () => { state.dirty = true; };
      image.src = `/assets/ui/${ART[name]}`;
      art.set(name, image);
    }
    const image = art.get(name);
    return image.complete && image.naturalWidth ? image : icon(fallback);
  }

  // Layout after Jakob's second diorama reference (see assets/reference/README.md): a menu coin top-left, a
  // row of resource coins top-right, a parchment globe bottom-right with speed
  // coins on its shoulder, and one slim command bar at the bottom centre.
  function layout() {
    const mobile = view.width < 700;
    const r = mobile ? 46 : 76;
    const globe = mobile
      ? { cx: view.width - 14 - r, cy: view.height - 14 - r, r }
      : { cx: view.width - 22 - r, cy: view.height - 22 - r, r };
    return {
      mobile, globe,
      menu: { x: 14, y: 14, d: mobile ? 36 : 42 },
      resourceCoin: mobile ? 30 : 36,
      button: mobile ? 42 : 48,
      speedCoin: mobile ? 28 : 32,
      dockBottom: view.height - (mobile ? 14 : 22),
      // Keep the bar centred on the picture but clear of the globe.
      dockCenter: mobile ? (view.width - (2 * r + 28)) / 2 : view.width / 2
    };
  }

  // ---------- what the HUD offers ----------
  function describe() {
    const { world, selection, buildMode } = state;
    const units = world.units.filter(unit => selection.units.has(unit.id));
    const building = world.buildings.find(b => b.id === selection.building);
    if (units.length) {
      const commands = buildMode
        ? [{ id: 'cancel', icon: 'cancel', label: 'Cancel placement', cost: 'Esc', enabled: true, run: actions.onCancel }]
        : [{ id: 'build', icon: 'build', label: 'Build town center', cost: '20 wood', enabled: world.stockpile.wood >= 20, run: actions.onBuild }];
      if (units.length === 1) {
        const unit = units[0];
        const phase = unit.action.type === 'gather' ? unit.action.phase : unit.action.type;
        const cargo = unit.cargo ? `, carrying ${Math.floor(unit.cargo.amount)} ${unit.cargo.kind}` : '';
        return { portrait: 'villager', title: unit.id.replace('villager-', 'Villager '), detail: `${ACTIVITY_TEXT[phase] || phase}${cargo}`, commands };
      }
      const idle = units.filter(unit => unit.action.type === 'idle').length;
      return { portrait: 'group', title: `${units.length} villagers`, detail: `${idle} awaiting orders`, commands };
    }
    if (building && building.construction !== null) {
      return { portrait: 'townCenter', title: 'Town center foundation', detail: 'Villagers can help build it', job: building.construction / 4, commands: [] };
    }
    if (building) {
      const job = building.job;
      const known = world.researched_technologies;
      return {
        portrait: 'townCenter',
        title: 'Town center',
        detail: job ? (job.type === 'produce' ? 'Training a villager' : `Researching ${TECHNOLOGIES[job.technology][0]}`) : 'Ready',
        job: job && job.elapsed_seconds / (job.type === 'produce' ? 6 : 8),
        commands: [
          { id: 'train', icon: 'train', label: 'Train villager', cost: '50 food', enabled: !job && world.stockpile.food >= 50, run: actions.onTrain },
          ...Object.entries(TECHNOLOGIES).map(([key, [label, effect]]) => {
            const done = known.includes(key);
            const blocked = PREREQUISITE[key] && !known.includes(PREREQUISITE[key]);
            return {
              id: key, icon: key, done,
              label: `${label}: ${effect}`,
              cost: done ? 'researched' : blocked ? `needs ${TECHNOLOGIES[PREREQUISITE[key]][0]}` : '40 food, 20 wood',
              enabled: !done && !blocked && !job && world.stockpile.food >= 40 && world.stockpile.wood >= 20,
              run: () => actions.onResearch(key)
            };
          })
        ]
      };
    }
    return null;
  }

  // ---------- drawing helpers ----------
  function pill(c, x, y, w, h) {
    c.save();
    c.shadowColor = STYLE.shadow;
    c.shadowBlur = 8;
    c.shadowOffsetY = 2;
    c.fillStyle = STYLE.glass;
    c.beginPath();
    c.roundRect(x, y, w, h, h / 2);
    c.fill();
    c.restore();
    c.strokeStyle = 'rgba(255,255,255,0.9)';
    c.lineWidth = 1;
    c.stroke();
  }
  // A coin button in the style of the generated portrait coins: bronze-gold
  // rim, ivory face, and the illustrated icon in the middle.
  function coin(c, x, y, d, image, { enabled = true, hot = false, done = false, label = '', active = false } = {}) {
    const r = d / 2;
    const cx = x + r;
    const cy = y + r - (hot ? 3 : 0);
    c.save();
    if (!enabled && !done) {
      c.globalAlpha = 0.62;
      c.filter = 'grayscale(0.85)';
    }
    c.shadowColor = hot ? 'rgba(255,196,96,0.85)' : STYLE.shadow;
    c.shadowBlur = hot ? 16 : 7;
    c.shadowOffsetY = hot ? 2 : 3;
    const rim = c.createLinearGradient(cx - r, cy - r, cx + r, cy + r);
    rim.addColorStop(0, '#f6dc9a');
    rim.addColorStop(0.45, done ? '#e2b04a' : '#c9963f');
    rim.addColorStop(1, '#7d5420');
    c.fillStyle = rim;
    c.beginPath();
    c.arc(cx, cy, r, 0, Math.PI * 2);
    c.fill();
    c.shadowColor = 'transparent';
    const face = c.createRadialGradient(cx - r * 0.25, cy - r * 0.3, r * 0.1, cx, cy, r * 0.82);
    face.addColorStop(0, '#fffaf0');
    face.addColorStop(1, '#efe3c8');
    c.fillStyle = face;
    c.beginPath();
    c.arc(cx, cy, r * 0.8, 0, Math.PI * 2);
    c.fill();
    c.strokeStyle = 'rgba(110,74,30,0.55)';
    c.lineWidth = 1;
    c.stroke();
    c.beginPath();
    c.arc(cx, cy, r - 1, 0, Math.PI * 2);
    c.strokeStyle = 'rgba(90,60,20,0.6)';
    c.stroke();
    if (image) c.drawImage(image, cx - r * 0.66, cy - r * 0.66, r * 1.32, r * 1.32);
    if (label) {
      if (active) {
        c.fillStyle = STYLE.accent;
        c.beginPath();
        c.arc(cx, cy, r * 0.8, 0, Math.PI * 2);
        c.fill();
      }
      c.fillStyle = active ? '#fffaf0' : STYLE.ink;
      c.font = `800 ${Math.round(d * 0.36)}px ${STYLE.body}`;
      c.textAlign = 'center';
      c.textBaseline = 'middle';
      c.fillText(label, cx, cy + 1);
    }
    c.restore();
    if (done) {
      c.fillStyle = '#4f9a5a';
      c.beginPath();
      c.arc(cx + r * 0.68, cy + r * 0.68, r * 0.2, 0, Math.PI * 2);
      c.fill();
      c.strokeStyle = '#fffaf0';
      c.lineWidth = 1.5;
      c.stroke();
    }
  }

  // ---------- panels ----------
  function paintMenu(L) {
    const panel = layer.panel('menu');
    const { x, y, d } = L.menu;
    const c = panel.begin(x - 8, y - 8, d + 16, d + 16, view.height, view.dpr);
    const hot = state.hover === 'reset';
    coin(c, 8, 8, d, null, { hot, label: '\u21BA' });
    c.fillStyle = state.online ? '#5f9f5a' : STYLE.accent;
    c.beginPath();
    c.arc(8 + d * 0.86, 8 + d * 0.86, 4, 0, Math.PI * 2);
    c.fill();
    panel.regions.push({ id: 'reset', x: 8, y: 8, w: d, h: d, round: true, enabled: true, run: actions.onReset, label: 'Reset the world' });
    panel.end();
  }

  function paintResources(L, now) {
    const panel = layer.panel('resources');
    const shown = RESOURCES.filter(([key]) => ALWAYS_SHOWN.has(key) || (state.world.stockpile[key] || 0) >= 1);
    const d = L.resourceCoin;
    const step = d + (L.mobile ? 14 : 22);
    const width = shown.length * step + 8;
    const height = d + 26;
    const c = panel.begin(view.width - width - (L.mobile ? 8 : 16), 10, width, height, view.height, view.dpr);
    c.textAlign = 'center';
    c.textBaseline = 'alphabetic';
    shown.forEach(([key, label, body], index) => {
      const value = Math.floor((state.world.stockpile[key] || 0) + 1e-6);
      if (state.last[key] !== undefined && value > state.last[key]) state.bumps[key] = now + 700;
      state.last[key] = value;
      const x = 4 + index * step + (step - d) / 2;
      const bump = state.bumps[key] > now;
      coin(c, x, 4, d, picture(key, body), { hot: bump });
      // Count on a small glass tab under each coin.
      const text = String(value);
      c.font = `800 ${L.mobile ? 11 : 12}px ${STYLE.body}`;
      const tab = Math.max(22, c.measureText(text).width + 12);
      c.fillStyle = STYLE.glass;
      c.beginPath();
      c.roundRect(x + d / 2 - tab / 2, d + 6, tab, 16, 8);
      c.fill();
      c.fillStyle = bump ? STYLE.accent : STYLE.ink;
      c.fillText(text, x + d / 2, d + 18);
      panel.regions.push({ id: `resource-${key}`, x, y: 4, w: d, h: d, round: true, label });
    });
    panel.end();
  }

  function paintSpeed(L) {
    const panel = layer.panel('speed');
    const { cx, cy, r } = L.globe;
    const d = L.speedCoin;
    const box = r + d + 12;
    const c = panel.begin(cx - box, cy - box, box, box, view.height, view.dpr);
    // Three coins on the globe's upper-left shoulder, like the reference.
    [['speed0', 'II', 0], ['speed1', '1\u00D7', 1], ['speed2', '2\u00D7', 2]].forEach(([id, label, speed], index) => {
      const angle = Math.PI * (1.08 + index * 0.17);
      const x = box + Math.cos(angle) * (r + d / 2 + 6) - d / 2;
      const y = box + Math.sin(angle) * (r + d / 2 + 6) - d / 2;
      const active = state.world.simulation_speed === speed;
      coin(c, x, y, d, null, { hot: state.hover === id, done: false, label, active });
      panel.regions.push({ id, x, y, w: d, h: d, round: true, enabled: true, run: () => actions.onSpeed(speed), label: `Speed ${label}` });
    });
    panel.end();
  }

  function paintDock(L, model) {
    const panel = layer.panel('dock');
    const notes = layer.panel('notes');
    if (!model) {
      panel.hide();
    } else {
      const m = L.button;
      const gap = L.mobile ? 6 : 10;
      const count = model.commands.length;
      const barWidth = count ? count * m + (count - 1) * gap + 28 : 0;
      const hovered = model.commands.find(command => command.id === state.hover);
      const title = hovered ? hovered.label : model.title;
      const detail = hovered ? hovered.cost : model.detail;
      const probe = panel.context;
      probe.font = `800 14px ${STYLE.body}`;
      const titleWidth = probe.measureText(title).width;
      probe.font = `600 12px ${STYLE.body}`;
      const infoWidth = Math.min(view.width - 24, Math.max(titleWidth, probe.measureText(detail).width) + 84);
      const width = Math.max(barWidth, infoWidth) + 24;
      const height = (count ? m + 26 : 0) + 64;
      const left = Math.max(8, L.dockCenter - width / 2);
      const c = panel.begin(left, L.dockBottom - height, width, height, view.height, view.dpr);
      const infoX = (width - infoWidth) / 2;
      const hasJob = model.job !== undefined && model.job !== null;
      pill(c, infoX, 8, infoWidth, hasJob ? 46 : 40);
      // The selection's coin portrait sits on the pill's left end like a badge.
      const portrait = picture(`portrait_${model.portrait}`, ICONS[model.portrait]);
      c.save();
      c.shadowColor = STYLE.shadow;
      c.shadowBlur = 5;
      c.shadowOffsetY = 2;
      c.drawImage(portrait, infoX - 4, 2, 52, 52);
      c.restore();
      const textX = infoX + 50 + (infoWidth - 64) / 2;
      c.textAlign = 'center';
      c.textBaseline = 'alphabetic';
      c.fillStyle = STYLE.ink;
      c.font = `800 14px ${STYLE.body}`;
      c.fillText(title, textX, 25, infoWidth - 70);
      c.fillStyle = STYLE.muted;
      c.font = `600 12px ${STYLE.body}`;
      c.fillText(detail, textX, 40, infoWidth - 70);
      if (hasJob) {
        c.fillStyle = 'rgba(61,51,40,0.15)';
        c.fillRect(infoX + 56, 46, infoWidth - 76, 3);
        c.fillStyle = STYLE.accent;
        c.fillRect(infoX + 56, 46, (infoWidth - 76) * Math.min(1, model.job), 3);
      }
      if (count) {
        // One slim glass bar holds the command coins.
        const barX = (width - barWidth) / 2;
        const barY = height - m - 18;
        pill(c, barX, barY, barWidth, m + 12);
        model.commands.forEach((command, index) => {
          const x = barX + 14 + index * (m + gap);
          const y = barY + 6;
          const hot = state.hover === command.id && command.enabled;
          coin(c, x, y, m, picture(command.icon, ICONS[command.icon] || ICONS.villager), { enabled: command.enabled, hot, done: command.done });
          panel.regions.push({ id: command.id, x, y, w: m, h: m, round: true, enabled: command.enabled, run: command.run });
        });
      }
      panel.end();
    }
    // Placement hint and toast share one pill near the top centre.
    const text = performance.now() < state.toastUntil ? state.toast : state.buildMode ? 'Choose a clear spot for the new town center' : '';
    if (!text) {
      notes.hide();
      return;
    }
    const probe = notes.context;
    probe.font = `700 13px ${STYLE.body}`;
    const width = Math.min(view.width - 24, probe.measureText(text).width + 36);
    const c = notes.begin((view.width - width) / 2 - 6, (L.mobile ? 70 : 18) - 6, width + 12, 44, view.height, view.dpr);
    pill(c, 6, 6, width, 32);
    c.fillStyle = STYLE.ink;
    c.font = `700 13px ${STYLE.body}`;
    c.textAlign = 'center';
    c.textBaseline = 'middle';
    c.fillText(text, (width + 12) / 2, 23, width - 20);
    notes.end();
  }

  // The minimap as an old parchment map set in a gold coin rim.
  function paintGlobe(L) {
    const panel = layer.panel('globe');
    const { cx, cy, r } = L.globe;
    const world = state.world;
    const pad = 12;
    const size = r * 2 + pad * 2;
    const c = panel.begin(cx - r - pad, cy - r - pad, size, size, view.height, view.dpr);
    const ox = pad + r;
    const oy = pad + r;
    coin(c, pad, pad, r * 2, null, {});
    c.save();
    c.beginPath();
    c.arc(ox, oy, r * 0.84, 0, Math.PI * 2);
    c.clip();
    const sea = c.createRadialGradient(ox - r * 0.3, oy - r * 0.3, r * 0.1, ox, oy, r);
    sea.addColorStop(0, '#d6ebe8');
    sea.addColorStop(1, '#9ec8cf');
    c.fillStyle = sea;
    c.fillRect(ox - r, oy - r, r * 2, r * 2);
    const mapW = r * 1.4;
    const mapH = mapW * world.rows / world.columns;
    const mx = ox - mapW / 2;
    const my = oy - mapH / 2;
    const sx = mapW / world.columns;
    const sy = mapH / world.rows;
    for (const cell of world.terrain) {
      const [red, green, blue] = cell.biome ? BIOME_COLORS[cell.biome] : [236, 226, 200];
      // Parchment wash: biome colours blended toward old paper.
      const mix = cell.biome ? (cell.visibility === 'explored' ? 0.55 : 0.35) : 0;
      c.fillStyle = `rgb(${red * (1 - mix) + 236 * mix},${green * (1 - mix) + 224 * mix},${blue * (1 - mix) + 196 * mix})`;
      c.fillRect(mx + cell.column * sx, my + cell.row * sy, sx + 0.5, sy + 0.5);
    }
    c.strokeStyle = 'rgba(110,80,40,0.55)';
    c.lineWidth = 1;
    c.strokeRect(mx, my, mapW, mapH);
    for (const building of world.buildings) {
      c.fillStyle = building.construction === null ? STYLE.accent : '#e8b49a';
      c.fillRect(mx + building.origin.column * sx, my + building.origin.row * sy, building.columns * sx, building.rows * sy);
    }
    c.fillStyle = '#2f5f8f';
    for (const unit of world.units) {
      c.beginPath();
      c.arc(mx + unit.position.x * sx, my + unit.position.y * sy, 1.8, 0, Math.PI * 2);
      c.fill();
    }
    if (state.corners.length && state.corners.every(Boolean)) {
      c.strokeStyle = 'rgba(61,51,40,0.85)';
      c.lineWidth = 1.2;
      c.beginPath();
      state.corners.forEach((corner, index) => c[index ? 'lineTo' : 'moveTo'](mx + corner.x * sx, my + corner.z * sy));
      c.closePath();
      c.stroke();
    }
    const dome = c.createRadialGradient(ox - r * 0.35, oy - r * 0.45, r * 0.05, ox, oy, r);
    dome.addColorStop(0, 'rgba(255,255,255,0.32)');
    dome.addColorStop(0.55, 'rgba(255,255,255,0)');
    dome.addColorStop(1, 'rgba(80,50,20,0.25)');
    c.fillStyle = dome;
    c.fillRect(ox - r, oy - r, r * 2, r * 2);
    c.restore();
    panel.regions.push({ id: 'globe', x: pad, y: pad, w: r * 2, h: r * 2, round: true, enabled: true, map: { mx, my, mapW, mapH } });
    panel.end();
  }

  // ---------- accessible mirror ----------
  const mirror = document.getElementById('a11y');
  function syncMirror(model) {
    const commands = model?.commands || [];
    const key = commands.map(command => `${command.id}:${command.enabled}`).join('|');
    if (key === state.a11yKey && mirror.childElementCount) return;
    state.a11yKey = key;
    const button = (label, run, disabled = false) => {
      const element = document.createElement('button');
      element.type = 'button';
      element.textContent = label;
      element.disabled = disabled;
      element.addEventListener('click', run);
      return element;
    };
    mirror.replaceChildren(
      button('Pause simulation', () => actions.onSpeed(0)),
      button('Normal speed', () => actions.onSpeed(1)),
      button('Double speed', () => actions.onSpeed(2)),
      button('Reset the world', actions.onReset),
      ...commands.map(command => button(`${command.label}, ${command.cost}`, command.run, !command.enabled))
    );
  }

  function globePoint(x, y) {
    const panel = layer.panel('globe');
    const map = panel.regions[0]?.map;
    if (!map) return null;
    return {
      x: Math.min(1, Math.max(0, (x - panel.x - map.mx) / map.mapW)) * state.world.columns,
      z: Math.min(1, Math.max(0, (y - panel.y - map.my) / map.mapH)) * state.world.rows
    };
  }

  return {
    resize(width, height, dpr) {
      layer.resize(width, height, dpr);
      state.dirty = true;
    },
    update(world, selection, buildMode) {
      Object.assign(state, { world, selection, buildMode, dirty: true });
    },
    setConnection(online) {
      state.online = online;
      state.dirty = true;
    },
    toast(message) {
      state.toast = FRIENDLY_ERRORS[message] || message;
      state.toastUntil = performance.now() + 2400;
      document.getElementById('status').textContent = state.toast;
      state.dirty = true;
      setTimeout(() => { state.dirty = true; }, 2450);
    },
    minimap(corners) {
      state.corners = corners;
      state.globeDirty = true;
    },
    box: area => layer.box(area),
    canBuild: () => state.commands.some(command => command.id === 'build' && command.enabled),
    pointer: {
      contains: (x, y) => Boolean(layer.hit(x, y)),
      down(x, y) {
        const region = layer.hit(x, y);
        if (!region) return false;
        state.pressed = region.id;
        if (region.id === 'globe' && state.world) {
          state.dragGlobe = true;
          const point = globePoint(x, y);
          if (point) actions.onMinimap(point);
        }
        return true;
      },
      move(x, y) {
        const point = state.dragGlobe && globePoint(x, y);
        if (point) actions.onMinimap(point);
      },
      up(x, y) {
        const region = layer.hit(x, y);
        if (region && region.id === state.pressed && region.enabled && region.run) region.run();
        state.pressed = null;
        state.dragGlobe = false;
      },
      hover(x, y) {
        const region = layer.hit(x, y);
        const id = region && (region.run || region.id === 'globe') ? region.id : null;
        if (id !== state.hover) {
          state.hover = id;
          state.dirty = true;
        }
        if (region) renderer.domElement.style.cursor = id && region.enabled !== false ? 'pointer' : 'default';
        return Boolean(region);
      }
    },
    render(now) {
      if (!state.world) return;
      const bumping = Object.values(state.bumps).some(until => until > now - 50);
      const L = layout();
      if (state.dirty || bumping) {
        const model = describe();
        state.commands = model?.commands || [];
        paintMenu(L);
        paintResources(L, now);
        paintSpeed(L);
        paintDock(L, model);
        syncMirror(model);
        state.dirty = false;
        state.globeDirty = true;
      }
      if (state.globeDirty) {
        paintGlobe(L);
        state.globeDirty = false;
      }
      layer.render();
    }
  };
}
