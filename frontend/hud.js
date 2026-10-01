// The HUD, painted inside the WebGL canvas: a slim stockpile strip on top, a
// ledger column on the left (selection, commands, minimap), and short notes.
// A visually hidden mirror of every button keeps keyboard and screen-reader
// access; the pixels themselves all come from the canvas.
import { STYLE, createUiLayer, iconImage, wrapText } from './ui-layer.js';
import { ACTIVITY_TEXT, FRIENDLY_ERRORS, ICONS, PREREQUISITE, RESOURCES, TECHNOLOGIES } from './hud-content.js';
import { BIOME_COLORS } from './terrain.js';

const SIDE = 212;
const ROW = 34;

export function createHud(renderer, actions) {
  const layer = createUiLayer(renderer);
  const { panels, view } = layer;
  const state = {
    world: null, selection: { units: new Set(), building: null }, buildMode: false, online: false,
    hover: null, pressed: null, dragMinimap: false, toast: '', toastUntil: 0, bumps: {}, last: {},
    dirty: true, minimapDirty: true, corners: [], commands: [], a11yKey: ''
  };
  const icon = body => iconImage(body, () => { state.dirty = true; });
  document.fonts?.ready.then(() => { state.dirty = true; });

  function layout() {
    const mobile = view.width < 700;
    if (!mobile) {
      const mapH = (SIDE - 24) * 2 / 3;
      return {
        mobile, top: 30, mount: 6,
        side: { x: 0, y: 30, w: SIDE, h: view.height - 30 },
        picture: { x: SIDE, y: 30, w: view.width - SIDE - 6, h: view.height - 36 },
        minimap: { x: 12, y: view.height - 12 - mapH, w: SIDE - 24, h: mapH }
      };
    }
    const sheet = 62 + (state.commands.length ? 60 : 0);
    return {
      mobile, top: 54, mount: 3,
      side: { x: 0, y: view.height - sheet, w: view.width, h: sheet },
      picture: { x: 3, y: 54, w: view.width - 6, h: view.height - 54 - sheet },
      minimap: { x: view.width - 124, y: 62, w: 114, h: 76 }
    };
  }

  // ---------- what the HUD says ----------
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
        const cargo = unit.cargo ? ` · carrying ${Math.floor(unit.cargo.amount)} ${unit.cargo.kind}` : '';
        return { icon: 'villager', title: unit.id.replace('villager-', 'Villager '), detail: `${ACTIVITY_TEXT[phase] || phase}${cargo}`, commands };
      }
      const idle = units.filter(unit => unit.action.type === 'idle').length;
      return { icon: 'group', title: `${units.length} villagers`, detail: `${idle} awaiting orders; tap ground to move together`, commands };
    }
    if (building && building.construction !== null) {
      return {
        icon: 'townCenter', title: 'Town center foundation', detail: 'Select villagers and tap it to help build',
        job: { label: 'Construction', progress: building.construction / 4 }, commands: []
      };
    }
    if (building) {
      const job = building.job;
      const known = world.researched_technologies;
      return {
        icon: 'townCenter', title: 'Town center', detail: 'Trains villagers and studies new crafts',
        job: job && {
          label: job.type === 'produce' ? 'Training a villager' : `Researching ${TECHNOLOGIES[job.technology][0]}`,
          progress: job.elapsed_seconds / (job.type === 'produce' ? 6 : 8)
        },
        commands: [
          { id: 'train', icon: 'villager', label: 'Train villager', cost: '50 food', enabled: !job && world.stockpile.food >= 50, run: actions.onTrain },
          ...Object.entries(TECHNOLOGIES).map(([key, [label]]) => {
            const done = known.includes(key);
            const blocked = PREREQUISITE[key] && !known.includes(PREREQUISITE[key]);
            return {
              id: key, icon: key, label, done,
              cost: done ? 'known' : blocked ? `needs ${TECHNOLOGIES[PREREQUISITE[key]][0]}` : '40 food · 20 wood',
              enabled: !done && !blocked && !job && world.stockpile.food >= 40 && world.stockpile.wood >= 20,
              run: () => actions.onResearch(key)
            };
          })
        ]
      };
    }
    return { icon: 'scroll', title: 'Your village', detail: 'Tap a villager to give orders, or the town center to train and research', commands: [] };
  }

  // ---------- painting ----------
  function paintTop(L, now) {
    const c = panels.top.begin(0, 0, view.width, L.top, view.height, view.dpr);
    c.fillStyle = STYLE.ivory;
    c.fillRect(0, 0, view.width, L.top);
    c.fillStyle = STYLE.hair;
    c.fillRect(0, L.top - 1, view.width, 1);
    const row = L.mobile ? 16 : L.top / 2;
    const step = (view.width - 16) / RESOURCES.length;
    let x = 12;
    c.textBaseline = 'middle';
    RESOURCES.forEach(([key, , body], index) => {
      const value = Math.floor((state.world.stockpile[key] || 0) + 1e-6);
      if (state.last[key] !== undefined && value > state.last[key]) state.bumps[key] = now + 700;
      state.last[key] = value;
      if (L.mobile) x = 8 + index * step;
      c.drawImage(icon(body), x, row - 7, 14, 14);
      c.font = `500 13px ${STYLE.body}`;
      c.fillStyle = state.bumps[key] > now ? STYLE.accent : STYLE.ink;
      c.textAlign = 'left';
      c.fillText(String(value), x + 18, row + 1);
      if (!L.mobile) {
        x += 18 + Math.max(18, c.measureText(String(value)).width) + 10;
        c.fillStyle = STYLE.faint;
        c.fillRect(x, 8, 1, L.top - 16);
        x += 10;
      }
    });
    if (!L.mobile) {
      c.textAlign = 'center';
      c.fillStyle = STYLE.ink;
      c.font = `600 15px ${STYLE.caps}`;
      c.fillText('Age of Agents', view.width / 2, row + 1);
      c.textAlign = 'left';
      c.font = `italic 400 11px ${STYLE.body}`;
      c.fillStyle = STYLE.muted;
      c.fillText(`tick ${state.world.tick}`, view.width / 2 + 62, row + 1);
    }
    // Right-hand controls, laid out from the right edge.
    const controlsY = L.mobile ? 40 : row;
    let right = view.width - 12;
    c.fillStyle = state.online ? '#5f8f4a' : STYLE.accent;
    c.beginPath();
    c.arc(right - 4, controlsY, 3.5, 0, Math.PI * 2);
    c.fill();
    right -= 20;
    const items = [['reset', 'reset', actions.onReset], ['speed2', '2×', () => actions.onSpeed(2)], ['speed1', '1×', () => actions.onSpeed(1)], ['speed0', 'pause', () => actions.onSpeed(0)]];
    c.font = `500 14px ${STYLE.caps}`;
    for (const [id, label, run] of items) {
      const width = c.measureText(label).width + 14;
      const active = id === `speed${state.world.simulation_speed}`;
      c.globalAlpha = active || state.hover === id ? 1 : 0.7;
      c.fillStyle = active ? STYLE.accent : STYLE.ink;
      c.textAlign = 'center';
      c.fillText(label, right - width / 2, controlsY + 1);
      if (active) c.fillRect(right - width + 7, controlsY + 9, width - 14, 1);
      c.globalAlpha = 1;
      panels.top.regions.push({ id, x: right - width, y: controlsY - 12, w: width, h: 24, enabled: true, run });
      right -= width + (id === 'reset' ? 12 : 0);
    }
    panels.top.end();
  }

  function paintSide(L, model) {
    const s = L.side;
    const c = panels.side.begin(s.x, s.y, s.w, s.h, view.height, view.dpr);
    c.fillStyle = STYLE.ivory;
    c.fillRect(0, 0, s.w, s.h);
    c.fillStyle = STYLE.hair;
    if (L.mobile) c.fillRect(0, 0, s.w, 1);
    else c.fillRect(s.w - 1, 0, 1, s.h);
    // Selection header: framed portrait, title, and a line of detail.
    c.strokeStyle = STYLE.hair;
    c.lineWidth = 1;
    c.fillStyle = STYLE.ivoryLight;
    c.fillRect(13, 11, 33, 33);
    c.strokeRect(12.5, 10.5, 34, 34);
    c.drawImage(icon(ICONS[model.icon]), 17, 15, 25, 25);
    c.textAlign = 'left';
    c.textBaseline = 'alphabetic';
    c.fillStyle = STYLE.ink;
    c.font = `600 16px ${STYLE.caps}`;
    c.fillText(model.title, 56, 25, s.w - 68);
    c.font = `italic 400 12px ${STYLE.body}`;
    c.fillStyle = STYLE.muted;
    wrapText(c, model.detail, 56, 40, (L.mobile ? s.w - 150 : SIDE) - 68, 14, L.mobile ? 1 : 2);
    let y = 70;
    if (model.job) {
      const jobX = L.mobile ? s.w - 132 : 12;
      const jobY = L.mobile ? 22 : y;
      const jobW = L.mobile ? 120 : SIDE - 24;
      c.font = `italic 400 11px ${STYLE.body}`;
      c.fillStyle = STYLE.ink;
      c.fillText(model.job.label, jobX, jobY);
      c.fillStyle = STYLE.faint;
      c.fillRect(jobX, jobY + 5, jobW, 3);
      c.fillStyle = STYLE.accent;
      c.fillRect(jobX, jobY + 5, jobW * Math.min(1, model.job.progress), 3);
      if (!L.mobile) y += 20;
    }
    if (L.mobile) y = 61;
    c.fillStyle = STYLE.hair;
    c.fillRect(0, y, s.w, 1);
    // Command rows on desktop, a row of cells on phones.
    const cellW = L.mobile ? Math.max(104, s.w / Math.max(1, model.commands.length)) : SIDE;
    model.commands.forEach((command, index) => {
      const area = L.mobile ? { x: index * cellW, y: 62, w: cellW, h: 60 } : { x: 0, y: y + 1 + index * ROW, w: SIDE, h: ROW };
      const hot = state.hover === command.id && command.enabled;
      if (hot) {
        c.fillStyle = STYLE.ivoryLight;
        c.fillRect(area.x, area.y, area.w, area.h - 1);
      }
      c.globalAlpha = command.enabled || command.done ? 1 : 0.42;
      c.drawImage(icon(ICONS[command.icon]), area.x + 12, L.mobile ? area.y + 12 : area.y + 8, 18, 18);
      c.fillStyle = hot ? STYLE.accent : STYLE.ink;
      c.font = `400 13px ${STYLE.body}`;
      c.textAlign = 'left';
      c.fillText(command.label, area.x + (L.mobile ? 36 : 40), L.mobile ? area.y + 25 : area.y + 22, area.w - 50);
      c.font = `italic 400 11px ${STYLE.body}`;
      c.fillStyle = STYLE.muted;
      if (L.mobile) {
        c.fillText(command.cost, area.x + 36, area.y + 42, area.w - 44);
      } else {
        c.textAlign = 'right';
        c.fillText(command.cost, SIDE - 12, area.y + 22);
      }
      c.globalAlpha = 1;
      c.fillStyle = STYLE.faint;
      if (L.mobile) c.fillRect(area.x + area.w - 1, area.y + 6, 1, area.h - 12);
      else c.fillRect(0, area.y + area.h - 1, SIDE, 1);
      panels.side.regions.push({ id: command.id, ...area, enabled: command.enabled, run: command.run });
    });
    if (!model.commands.length && !L.mobile) {
      c.font = `italic 400 12px ${STYLE.body}`;
      c.fillStyle = STYLE.hair;
      c.textAlign = 'left';
      c.fillText('No orders available', 12, y + 22);
    }
    if (!L.mobile) {
      const footer = L.minimap.y - s.y - 12;
      c.font = `italic 400 11px ${STYLE.body}`;
      c.fillStyle = STYLE.muted;
      c.textAlign = 'left';
      wrapText(c, 'Drag to pan, wheel to zoom, Q/E or right-drag to rotate. Shift-drag selects a group. B builds, Esc cancels.', 12, footer - 34, SIDE - 24, 13, 3);
      c.fillStyle = STYLE.hair;
      c.fillRect(0, footer, SIDE, 1);
    }
    panels.side.end();
  }

  function paintNote(panel, text, y, L) {
    if (!text) {
      panel.mesh.visible = false;
      return;
    }
    const probe = panel.context;
    probe.font = `italic 400 13px ${STYLE.body}`;
    const width = Math.min(L.picture.w - 20, probe.measureText(text).width + 28);
    const c = panel.begin(L.picture.x + (L.picture.w - width) / 2, y, width, 28, view.height, view.dpr);
    c.fillStyle = STYLE.ivory;
    c.fillRect(0, 0, width, 28);
    c.strokeStyle = STYLE.hair;
    c.strokeRect(0.5, 0.5, width - 1, 27);
    c.font = `italic 400 13px ${STYLE.body}`;
    c.fillStyle = STYLE.ink;
    c.textAlign = 'center';
    c.textBaseline = 'middle';
    c.fillText(text, width / 2, 15, width - 16);
    panel.end();
  }

  function paintMinimap(L) {
    const m = L.minimap;
    const world = state.world;
    const c = panels.minimap.begin(m.x, m.y, m.w, m.h, view.height, view.dpr);
    c.fillStyle = STYLE.ivory;
    c.fillRect(0, 0, m.w, m.h);
    const inner = { x: 3, y: 3, w: m.w - 6, h: m.h - 6 };
    const sx = inner.w / world.columns;
    const sy = inner.h / world.rows;
    for (const cell of world.terrain) {
      const [r, g, b] = cell.biome ? BIOME_COLORS[cell.biome] : [226, 214, 186];
      const dim = cell.visibility === 'explored' ? 0.78 : 1;
      c.fillStyle = `rgb(${r * dim},${g * dim},${b * dim})`;
      c.fillRect(inner.x + cell.column * sx, inner.y + cell.row * sy, sx + 0.5, sy + 0.5);
    }
    c.fillStyle = STYLE.ink;
    for (const resource of world.resources) {
      if (resource.amount > 0) c.fillRect(inner.x + resource.cell.column * sx + 1, inner.y + resource.cell.row * sy + 1, sx - 2, sy - 2);
    }
    for (const building of world.buildings) {
      c.fillStyle = building.construction === null ? STYLE.accent : '#e8b49a';
      c.fillRect(inner.x + building.origin.column * sx, inner.y + building.origin.row * sy, building.columns * sx, building.rows * sy);
    }
    c.fillStyle = '#1f6fd0';
    for (const unit of world.units) c.fillRect(inner.x + unit.position.x * sx - 1.5, inner.y + unit.position.y * sy - 1.5, 3, 3);
    if (state.corners.length && state.corners.every(Boolean)) {
      c.strokeStyle = STYLE.ink;
      c.beginPath();
      state.corners.forEach((corner, index) => c[index ? 'lineTo' : 'moveTo'](inner.x + corner.x * sx, inner.y + corner.z * sy));
      c.closePath();
      c.stroke();
    }
    c.strokeStyle = STYLE.hair;
    c.strokeRect(0.5, 0.5, m.w - 1, m.h - 1);
    c.strokeRect(2.5, 2.5, m.w - 5, m.h - 5);
    panels.minimap.regions.push({ id: 'minimap', x: 0, y: 0, w: m.w, h: m.h, enabled: true });
    panels.minimap.end();
  }

  // ---------- accessible mirror ----------
  const mirror = document.getElementById('a11y');
  function syncMirror(model) {
    const key = model.commands.map(command => `${command.id}:${command.enabled}`).join('|');
    if (key === state.a11yKey) return;
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
      ...model.commands.map(command => button(`${command.label}, ${command.cost}`, command.run, !command.enabled))
    );
  }

  function regionAt(x, y) {
    return layer.hit(x, y)?.region || null;
  }
  function minimapPoint(x, y) {
    const m = layout().minimap;
    return {
      x: Math.min(1, Math.max(0, (x - m.x - 3) / (m.w - 6))) * state.world.columns,
      z: Math.min(1, Math.max(0, (y - m.y - 3) / (m.h - 6))) * state.world.rows
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
      document.getElementById('status').textContent = state.toast;
      state.toastUntil = performance.now() + 2400;
      state.dirty = true;
      setTimeout(() => { state.dirty = true; }, 2450);
    },
    minimap(corners) {
      state.corners = corners;
      state.minimapDirty = true;
    },
    box: area => layer.box(area),
    canBuild: () => state.commands.some(command => command.id === 'build' && command.enabled),
    pointer: {
      contains: (x, y) => Boolean(layer.hit(x, y)),
      down(x, y) {
        const region = regionAt(x, y);
        if (!region) return false;
        state.pressed = region.id;
        if (region.id === 'minimap' && state.world) {
          state.dragMinimap = true;
          actions.onMinimap(minimapPoint(x, y));
        }
        return true;
      },
      move(x, y) {
        if (state.dragMinimap) actions.onMinimap(minimapPoint(x, y));
      },
      up(x, y) {
        const region = regionAt(x, y);
        if (region && region.id === state.pressed && region.enabled && region.run) region.run();
        state.pressed = null;
        state.dragMinimap = false;
      },
      hover(x, y) {
        const region = regionAt(x, y);
        const id = (region?.run && region.enabled) || region?.id === 'minimap' ? region.id : null;
        if (id !== state.hover) {
          state.hover = id;
          state.dirty = true;
        }
        if (region) renderer.domElement.style.cursor = id ? 'pointer' : 'default';
        return Boolean(region);
      }
    },
    render(now) {
      if (!state.world) return;
      const bumping = Object.values(state.bumps).some(until => until > now - 50);
      if (state.dirty || bumping) {
        const model = describe();
        state.commands = model.commands;
        const L = layout();
        layer.frame(L.picture, L.mount);
        paintTop(L, now);
        paintSide(L, model);
        paintNote(panels.note, state.buildMode ? 'Choose ground for the new town center' : '', L.picture.y + 12, L);
        paintNote(panels.toast, now < state.toastUntil ? state.toast : '', L.picture.y + L.picture.h - 40, L);
        syncMirror(model);
        state.dirty = false;
        state.minimapDirty = true;
      }
      if (state.minimapDirty) {
        paintMinimap(layout());
        state.minimapDirty = false;
      }
      layer.render();
    }
  };
}
