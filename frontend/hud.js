// The HUD, painted inside the WebGL canvas and mostly out of the way, after
// assets/reference/diorama_primary.webp: a round globe minimap, a small speed
// pill, a resource pill that lists only what you have, and glossy medallion
// buttons that appear only when something is selected. A visually hidden DOM
// mirror of every button keeps keyboard and screen-reader access.
import { STYLE, createUiLayer, iconImage } from './ui-layer.js';
import { ACTIVITY_TEXT, FRIENDLY_ERRORS, ICONS, PREREQUISITE, RESOURCES, TECHNOLOGIES } from './hud-content.js';
import { BIOME_COLORS } from './terrain.js';

const TONES = { terracotta: ['#e6876a', '#c8553a', '#8e3220'], blue: ['#7fb2dc', '#3f78b5', '#264d7a'], teal: ['#7cc6c9', '#2f8f95', '#1c5a5e'], gold: ['#f2d58a', '#d1a33c', '#8a6618'], stone: ['#f2eee6', '#c9c1b2', '#8a8274'] };
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

  // White silhouettes of the placeholder icons, for glyphs on coloured medallions.
  const glyphs = new Map();
  function glyph(name) {
    const image = icon(ICONS[name]);
    if (!image.complete || !image.naturalWidth) return null;
    if (!glyphs.has(name)) {
      const canvas = document.createElement('canvas');
      canvas.width = canvas.height = 96;
      const c = canvas.getContext('2d');
      c.drawImage(image, 0, 0, 96, 96);
      c.globalCompositeOperation = 'source-in';
      c.fillStyle = '#fffaf0';
      c.fillRect(0, 0, 96, 96);
      glyphs.set(name, canvas);
    }
    return glyphs.get(name);
  }

  function layout() {
    const mobile = view.width < 700;
    const radius = mobile ? 44 : 70;
    const globe = mobile
      ? { cx: view.width - 12 - radius, cy: 12 + radius, r: radius }
      : { cx: 18 + radius, cy: view.height - 18 - radius, r: radius };
    return {
      mobile, globe,
      speed: mobile ? { x: 12, y: 50 } : { x: 18, y: globe.cy - radius - 46 },
      resources: { x: 12, y: 12 },
      medallion: mobile ? 50 : 58,
      dockBottom: view.height - (mobile ? 16 : 22)
    };
  }

  // ---------- what the HUD offers ----------
  function describe() {
    const { world, selection, buildMode } = state;
    const units = world.units.filter(unit => selection.units.has(unit.id));
    const building = world.buildings.find(b => b.id === selection.building);
    if (units.length) {
      const commands = buildMode
        ? [{ id: 'cancel', icon: 'cancel', tone: 'stone', label: 'Cancel placement', cost: 'Esc', enabled: true, run: actions.onCancel }]
        : [{ id: 'build', icon: 'build', tone: 'terracotta', label: 'Build town center', cost: '20 wood', enabled: world.stockpile.wood >= 20, run: actions.onBuild }];
      if (units.length === 1) {
        const unit = units[0];
        const phase = unit.action.type === 'gather' ? unit.action.phase : unit.action.type;
        const cargo = unit.cargo ? `, carrying ${Math.floor(unit.cargo.amount)} ${unit.cargo.kind}` : '';
        return { title: unit.id.replace('villager-', 'Villager '), detail: `${ACTIVITY_TEXT[phase] || phase}${cargo}`, commands };
      }
      const idle = units.filter(unit => unit.action.type === 'idle').length;
      return { title: `${units.length} villagers`, detail: `${idle} awaiting orders`, commands };
    }
    if (building && building.construction !== null) {
      return { title: 'Town center foundation', detail: 'Villagers can help build it', job: building.construction / 4, commands: [] };
    }
    if (building) {
      const job = building.job;
      const known = world.researched_technologies;
      return {
        title: 'Town center',
        detail: job ? (job.type === 'produce' ? 'Training a villager' : `Researching ${TECHNOLOGIES[job.technology][0]}`) : 'Ready',
        job: job && job.elapsed_seconds / (job.type === 'produce' ? 6 : 8),
        commands: [
          { id: 'train', icon: 'villager', tone: 'blue', label: 'Train villager', cost: '50 food', enabled: !job && world.stockpile.food >= 50, run: actions.onTrain },
          ...Object.entries(TECHNOLOGIES).map(([key, [label, effect]]) => {
            const done = known.includes(key);
            const blocked = PREREQUISITE[key] && !known.includes(PREREQUISITE[key]);
            return {
              id: key, icon: key, tone: done ? 'gold' : 'teal', done,
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
  function medallion(c, x, y, d, tone, art, { enabled = true, hot = false, glyphScale = 0.5 } = {}) {
    const [light, base, dark] = TONES[tone];
    const r = d / 2;
    const cx = x + r;
    const cy = y + r - (hot ? 2 : 0);
    c.save();
    c.globalAlpha = enabled ? 1 : 0.5;
    c.shadowColor = STYLE.shadow;
    c.shadowBlur = hot ? 12 : 7;
    c.shadowOffsetY = hot ? 5 : 3;
    const fill = c.createRadialGradient(cx - r * 0.35, cy - r * 0.45, r * 0.1, cx, cy, r);
    fill.addColorStop(0, light);
    fill.addColorStop(0.55, base);
    fill.addColorStop(1, dark);
    c.fillStyle = fill;
    c.beginPath();
    c.arc(cx, cy, r, 0, Math.PI * 2);
    c.fill();
    c.restore();
    c.save();
    c.globalAlpha = enabled ? 1 : 0.5;
    c.lineWidth = 2;
    c.strokeStyle = 'rgba(255,255,255,0.75)';
    c.beginPath();
    c.arc(cx, cy, r - 1.5, 0, Math.PI * 2);
    c.stroke();
    if (art) c.drawImage(art, cx - d * glyphScale / 2, cy - d * glyphScale / 2, d * glyphScale, d * glyphScale);
    const shine = c.createLinearGradient(0, cy - r, 0, cy);
    shine.addColorStop(0, 'rgba(255,255,255,0.45)');
    shine.addColorStop(1, 'rgba(255,255,255,0)');
    c.fillStyle = shine;
    c.beginPath();
    c.ellipse(cx, cy - r * 0.45, r * 0.62, r * 0.38, 0, 0, Math.PI * 2);
    c.fill();
    c.restore();
  }

  // ---------- panels ----------
  function paintResources(L, now) {
    const panel = layer.panel('resources');
    const shown = RESOURCES.filter(([key]) => ALWAYS_SHOWN.has(key) || (state.world.stockpile[key] || 0) >= 1);
    const width = 20 + shown.length * 58;
    const c = panel.begin(L.resources.x - 6, L.resources.y - 6, width + 12, 44, view.height, view.dpr);
    pill(c, 6, 6, width, 32);
    c.textBaseline = 'middle';
    c.font = `800 14px ${STYLE.body}`;
    shown.forEach(([key, label, body], index) => {
      const value = Math.floor((state.world.stockpile[key] || 0) + 1e-6);
      if (state.last[key] !== undefined && value > state.last[key]) state.bumps[key] = now + 700;
      state.last[key] = value;
      const x = 18 + index * 58;
      c.drawImage(icon(body), x, 13, 18, 18);
      c.fillStyle = state.bumps[key] > now ? STYLE.accent : STYLE.ink;
      c.fillText(String(value), x + 22, 23);
      panel.regions.push({ id: `resource-${key}`, x: x - 6, y: 6, w: 56, h: 32, label });
    });
    panel.end();
  }

  function paintSpeed(L) {
    const panel = layer.panel('speed');
    const items = [['speed0', 'pause'], ['speed1', '1×'], ['speed2', '2×'], ['reset', 'reset']];
    const widths = [46, 30, 30, 44];
    const width = widths.reduce((a, b) => a + b, 0) + 40;
    const c = panel.begin(L.speed.x - 6, L.speed.y - 6, width + 12, 40, view.height, view.dpr);
    pill(c, 6, 6, width, 28);
    let x = 14;
    c.textBaseline = 'middle';
    c.textAlign = 'center';
    items.forEach(([id, label], index) => {
      const active = id === `speed${state.world.simulation_speed}`;
      if (active) {
        c.fillStyle = STYLE.accent;
        c.beginPath();
        c.roundRect(x, 10, widths[index], 20, 10);
        c.fill();
      }
      c.font = `${active ? 800 : 600} 12px ${STYLE.body}`;
      c.fillStyle = active ? '#fffaf0' : state.hover === id ? STYLE.accent : STYLE.muted;
      c.fillText(label, x + widths[index] / 2, 21);
      const run = id === 'reset' ? actions.onReset : () => actions.onSpeed(Number(id.slice(5)));
      panel.regions.push({ id, x, y: 8, w: widths[index], h: 24, enabled: true, run });
      x += widths[index] + (index === 2 ? 10 : 0);
    });
    c.fillStyle = state.online ? '#5f9f5a' : STYLE.accent;
    c.beginPath();
    c.arc(width - 4, 20, 3, 0, Math.PI * 2);
    c.fill();
    panel.end();
  }

  function paintDock(L, model) {
    const panel = layer.panel('dock');
    const notes = layer.panel('notes');
    if (!model) {
      panel.hide();
    } else {
      const m = L.medallion;
      const gap = L.mobile ? 8 : 14;
      const count = model.commands.length;
      const rowWidth = count * m + Math.max(0, count - 1) * gap;
      const hovered = model.commands.find(command => command.id === state.hover);
      const title = hovered ? hovered.label : model.title;
      const detail = hovered ? hovered.cost : model.detail;
      const probe = panel.context;
      probe.font = `800 14px ${STYLE.body}`;
      const titleWidth = probe.measureText(title).width;
      probe.font = `600 12px ${STYLE.body}`;
      const infoWidth = Math.min(view.width - 24, Math.max(titleWidth, probe.measureText(detail).width) + 40);
      const width = Math.max(rowWidth, infoWidth) + 24;
      const height = (count ? m + 14 : 0) + 64;
      const c = panel.begin((view.width - width) / 2, L.dockBottom - height, width, height, view.height, view.dpr);
      const infoX = (width - infoWidth) / 2;
      pill(c, infoX, 8, infoWidth, model.job !== undefined && model.job !== null ? 46 : 40);
      c.textAlign = 'center';
      c.textBaseline = 'alphabetic';
      c.fillStyle = STYLE.ink;
      c.font = `800 14px ${STYLE.body}`;
      c.fillText(title, width / 2, 25, infoWidth - 24);
      c.fillStyle = STYLE.muted;
      c.font = `600 12px ${STYLE.body}`;
      c.fillText(detail, width / 2, 40, infoWidth - 24);
      if (model.job !== undefined && model.job !== null) {
        c.fillStyle = 'rgba(61,51,40,0.15)';
        c.fillRect(infoX + 20, 46, infoWidth - 40, 3);
        c.fillStyle = STYLE.accent;
        c.fillRect(infoX + 20, 46, (infoWidth - 40) * Math.min(1, model.job), 3);
      }
      const rowX = (width - rowWidth) / 2;
      model.commands.forEach((command, index) => {
        const x = rowX + index * (m + gap);
        const y = height - m - 8;
        const hot = state.hover === command.id && command.enabled;
        medallion(c, x, y, m, command.tone, glyph(command.icon), { enabled: command.enabled || command.done, hot });
        panel.regions.push({ id: command.id, x, y, w: m, h: m, round: true, enabled: command.enabled, run: command.run });
      });
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
    const c = notes.begin((view.width - width) / 2 - 6, (L.mobile ? 100 : 18) - 6, width + 12, 44, view.height, view.dpr);
    pill(c, 6, 6, width, 32);
    c.fillStyle = STYLE.ink;
    c.font = `700 13px ${STYLE.body}`;
    c.textAlign = 'center';
    c.textBaseline = 'middle';
    c.fillText(text, (width + 12) / 2, 23, width - 20);
    notes.end();
  }

  function paintGlobe(L) {
    const panel = layer.panel('globe');
    const { cx, cy, r } = L.globe;
    const world = state.world;
    const pad = 10;
    const size = r * 2 + pad * 2;
    const c = panel.begin(cx - r - pad, cy - r - pad, size, size, view.height, view.dpr);
    const ox = pad;
    const oy = pad;
    c.save();
    c.shadowColor = STYLE.shadow;
    c.shadowBlur = 10;
    c.shadowOffsetY = 4;
    c.fillStyle = '#2c6f95';
    c.beginPath();
    c.arc(ox + r, oy + r, r, 0, Math.PI * 2);
    c.fill();
    c.restore();
    c.save();
    c.beginPath();
    c.arc(ox + r, oy + r, r - 4, 0, Math.PI * 2);
    c.clip();
    const sea = c.createRadialGradient(ox + r * 0.7, oy + r * 0.6, r * 0.2, ox + r, oy + r, r);
    sea.addColorStop(0, '#6fb8d6');
    sea.addColorStop(1, '#2c6f95');
    c.fillStyle = sea;
    c.fillRect(ox, oy, r * 2, r * 2);
    // The 30 by 20 map fills the middle of the globe.
    const mapW = r * 1.5;
    const mapH = mapW * world.rows / world.columns;
    const mx = ox + r - mapW / 2;
    const my = oy + r - mapH / 2;
    const sx = mapW / world.columns;
    const sy = mapH / world.rows;
    for (const cell of world.terrain) {
      const [red, green, blue] = cell.biome ? BIOME_COLORS[cell.biome] : [196, 206, 218];
      const dim = cell.visibility === 'explored' ? 0.8 : 1;
      c.fillStyle = `rgb(${red * dim},${green * dim},${blue * dim})`;
      c.fillRect(mx + cell.column * sx, my + cell.row * sy, sx + 0.5, sy + 0.5);
    }
    for (const building of world.buildings) {
      c.fillStyle = building.construction === null ? STYLE.accent : '#e8b49a';
      c.fillRect(mx + building.origin.column * sx, my + building.origin.row * sy, building.columns * sx, building.rows * sy);
    }
    c.fillStyle = '#ffffff';
    for (const unit of world.units) {
      c.beginPath();
      c.arc(mx + unit.position.x * sx, my + unit.position.y * sy, 1.8, 0, Math.PI * 2);
      c.fill();
    }
    if (state.corners.length && state.corners.every(Boolean)) {
      c.strokeStyle = 'rgba(255,255,255,0.9)';
      c.lineWidth = 1.2;
      c.beginPath();
      state.corners.forEach((corner, index) => c[index ? 'lineTo' : 'moveTo'](mx + corner.x * sx, my + corner.z * sy));
      c.closePath();
      c.stroke();
    }
    // Glassy dome: shading at the rim and a highlight top-left.
    const dome = c.createRadialGradient(ox + r * 0.65, oy + r * 0.55, r * 0.1, ox + r, oy + r, r);
    dome.addColorStop(0, 'rgba(255,255,255,0.28)');
    dome.addColorStop(0.6, 'rgba(255,255,255,0)');
    dome.addColorStop(1, 'rgba(10,30,50,0.35)');
    c.fillStyle = dome;
    c.fillRect(ox, oy, r * 2, r * 2);
    c.restore();
    c.lineWidth = 4;
    c.strokeStyle = '#f2ede3';
    c.beginPath();
    c.arc(ox + r, oy + r, r - 2, 0, Math.PI * 2);
    c.stroke();
    c.lineWidth = 1;
    c.strokeStyle = 'rgba(61,51,40,0.35)';
    c.beginPath();
    c.arc(ox + r, oy + r, r, 0, Math.PI * 2);
    c.stroke();
    panel.regions.push({ id: 'globe', x: pad, y: pad, w: r * 2, h: r * 2, round: true, enabled: true, map: { mx: mx - ox + pad, my: my - oy + pad, mapW, mapH } });
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
