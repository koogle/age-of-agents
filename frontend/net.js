// WebSocket link to the authoritative server: sequenced snapshots in, typed
// commands out, each command resolved by its acknowledgement.
const RETRY_MS = [500, 1000, 2000, 4000];
const BIOMES = ['meadow', 'forest', 'prairie', 'highland', 'wetland', 'scrubland', 'heath', 'clayland', 'beach', 'water', 'mountain', 'river'];

// Snapshots carry terrain as one character per cell (crates/game terrain_codec);
// this frozen client still reads the per-cell objects it was written for.
function decodeTerrain({ columns, cells }) {
  return [...cells].map((code, index) => {
    const cell = { column: index % columns, row: Math.floor(index / columns), visibility: 'unseen' };
    if (code !== '.') {
      const visible = code < 'a';
      cell.visibility = visible ? 'visible' : 'explored';
      cell.biome = BIOMES[code.charCodeAt(0) - (visible ? 65 : 97)];
    }
    return cell;
  });
}

export function connect({ onSnapshot, onStatus }) {
  const pending = new Map();
  let socket = null;
  let attempt = 0;
  let lastSequence = 0;
  let requestNumber = 0;

  function open() {
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    socket = new WebSocket(`${protocol}//${location.host}/ws`);
    socket.addEventListener('open', () => {
      attempt = 0;
      lastSequence = 0;
      onStatus(true);
    });
    socket.addEventListener('message', event => {
      const message = JSON.parse(event.data);
      if (message.type === 'snapshot') {
        if (message.sequence <= lastSequence) return;
        lastSequence = message.sequence;
        onSnapshot({ ...message.world, terrain: decodeTerrain(message.world.terrain) });
      } else if (message.type === 'command_result') {
        pending.get(message.request_id)?.(message);
        pending.delete(message.request_id);
      }
    });
    socket.addEventListener('close', () => {
      onStatus(false);
      for (const resolve of pending.values()) resolve({ ok: false, error: 'connection lost' });
      pending.clear();
      setTimeout(open, RETRY_MS[Math.min(attempt, RETRY_MS.length - 1)]);
      attempt += 1;
    });
  }
  open();

  return {
    send(command) {
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        return Promise.resolve({ ok: false, error: 'not connected' });
      }
      requestNumber += 1;
      const requestId = `r${requestNumber}`;
      socket.send(JSON.stringify({ type: 'command', request_id: requestId, command }));
      return new Promise(resolve => pending.set(requestId, resolve));
    }
  };
}
