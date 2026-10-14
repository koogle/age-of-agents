#!/usr/bin/env python3
"""Generate ground-shadow masks for building and NPC sprite frames with fal-ai/nano-banana-pro/edit.

Each job pastes one sprite frame into a fixed 1024x1024 canvas (the frame cell is
scaled to 512 px at CELL_OFFSET) on flat ground, with a guide pole whose shadow
shows the sun direction, and asks the model for a black-and-white shadow mask of
the same framing. Request IDs are recorded before polling, so a rerun resumes
outstanding requests instead of paying for them again.

    python3 assets/shadows/tools/generate.py plan          # list jobs and cost
    python3 assets/shadows/tools/generate.py run [--only KEY_PREFIX] [--limit N]
"""
import argparse, base64, io, json, os, sys, time, urllib.error, urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[3]
SPRITES = ROOT / 'assets/sprites'
HERE = Path(__file__).resolve().parent
WORK = HERE.parent / 'sources'
LEDGER = HERE.parent / 'requests.jsonl'
ENDPOINT = 'fal-ai/nano-banana-pro/edit'
PRICE = 0.15
CANVAS = 1024
CELL_PX = 512
CELL_OFFSET = (128, 320)
GROUND = (236, 226, 196)
# Guide pole: base, height, and its drawn shadow (right and slightly away from the viewer).
POLE = ((60, 990), 100, (1.0, -0.2))

PROMPT = (
    "Produce a pure black-and-white cast-shadow mask of image 1, keeping exactly the same framing, size and position. "
    "Paint white (#FFFFFF) every patch of flat ground covered by the cast shadow of the object and of the small guide pole, "
    "Paint everything else pure black (#000000), including the whole object itself (it must appear as a black hole, "
    "never white), the ground and the pole. No grey shading, no outlines, no object details, no text. "
    "The sun is fixed: the guide pole in the bottom-left corner already shows its correct shadow as a thin grey line "
    "falling to the right and slightly up (away from the viewer), about as long as the pole is tall. "
    "The object's shadow must fall in exactly that direction with the same length-to-height ratio, "
    "start exactly where the object touches the ground, and follow the object's real silhouette "
    "(roofs, towers, legs, weapons, tails). "
    "Images 2 and 3 are an approved example: image 2 is an input and image 3 is its correct mask."
)


def load(name):
    return json.load(open(SPRITES / f'{name}.json'))


def jobs():
    """(key, sheet image, rect [x, y, w, h], mirror) for every shadow to author."""
    out = []
    for name in ['buildings_hd', 'buildings_economy', 'buildings_crafts', 'buildings_civic', 'buildings_sanctuary']:
        sheet = load(name)
        for row, stages in sheet['frames'].items():
            for stage, rect in enumerate(stages):
                out.append((f'building/{row}/{stage}', name, rect, False))
                if row in ('dock', 'dock_north'):
                    out.append((f'building/{row}/{stage}/mirror', name, rect, True))
    tc = load('towncenter')
    for frame, rect in tc['frames'].items():
        out.append((f'building/towncenter/{frame}', 'towncenter', rect, False))
    idle = load('villager_idle_hd')
    for person, views in idle['people'].items():
        for view in ('front', 'back'):
            for mirror in (False, True):
                out.append((f'npc/{person}/{view}' + ('/mirror' if mirror else ''), 'villager_idle_hd', views[view][0], mirror))
    units = load('units')
    for unit, spec in units['units'].items():
        for view in ('front', 'back'):
            for mirror in (False, True):
                out.append((f'npc/{unit}/{view}' + ('/mirror' if mirror else ''), 'units', spec['animations']['idle'][view][0], mirror))
    wild = load('wildlife')
    for frame, rect in wild['frames'].items():
        if frame.endswith('_idle'):
            species = frame[:-len('_idle')]
            for mirror in (False, True):
                out.append((f'npc/{species}' + ('/mirror' if mirror else ''), 'wildlife', rect, mirror))
    return out


def frame_image(sheet, rect, mirror):
    image = Image.open(SPRITES / f'{sheet}.png').convert('RGBA')
    x, y, w, h = rect
    cell = image.crop((x, y, x + w, y + h)).resize((CELL_PX, CELL_PX), Image.LANCZOS)
    return cell.transpose(Image.FLIP_LEFT_RIGHT) if mirror else cell


def canvas(sheet, rect, mirror):
    out = Image.new('RGBA', (CANVAS, CANVAS), GROUND + (255,))
    out.alpha_composite(frame_image(sheet, rect, mirror), CELL_OFFSET)
    d = ImageDraw.Draw(out)
    (bx, by), h, (dx, dy) = POLE
    d.line([(bx, by), (bx + dx * h, by + dy * h)], fill=(120, 116, 104, 255), width=9)
    d.line([(bx, by), (bx, by - h)], fill=(110, 70, 40, 255), width=8)
    return out.convert('RGB')


def data_uri(image):
    buf = io.BytesIO()
    image.save(buf, 'PNG')
    return 'data:image/png;base64,' + base64.b64encode(buf.getvalue()).decode()


def request(method, url, body=None, tries=8):
    data = json.dumps(body).encode() if body is not None else None
    for attempt in range(tries):
        r = urllib.request.Request(url, data=data, method=method)
        r.add_header('Authorization', f"Key {os.environ['FAL_KEY']}")
        if data is not None:
            r.add_header('Content-Type', 'application/json')
        try:
            with urllib.request.urlopen(r, timeout=120) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            if e.code < 500 and e.code != 429:
                raise RuntimeError(f'{method} {url} -> {e.code}: {e.read()[:400]!r}')
        except (urllib.error.URLError, ConnectionError, TimeoutError):
            pass
        if method == 'POST' and attempt == 0 and data is not None:
            # A submit that may have reached the server is never retried blindly.
            raise RuntimeError(f'submit to {url} failed; check the request history before retrying')
        time.sleep(min(2 ** attempt, 30))
    raise RuntimeError(f'{method} {url} kept failing')


def ledger():
    entries = {}
    if LEDGER.exists():
        for line in LEDGER.read_text().splitlines():
            e = json.loads(line)
            entries.setdefault(e['key'], {}).update(e)
    return entries


def record(entry):
    with open(LEDGER, 'a') as f:
        f.write(json.dumps(entry) + '\n')


def run_job(job, example):
    key, sheet, rect, mirror = job
    out = WORK / (key.replace('/', '__') + '.png')
    done = ledger().get(key, {})
    if out.exists():
        return key, 'cached'
    rid = done.get('request_id')
    if not rid:
        ref = canvas(sheet, rect, mirror)
        ref.save(WORK / (key.replace('/', '__') + '_input.png'))
        sub = request('POST', f'https://queue.fal.run/{ENDPOINT}', {
            'prompt': PROMPT if example else PROMPT.rsplit(' Images 2', 1)[0],
            'image_urls': [data_uri(ref)] + example,
            'output_format': 'png',
        })
        rid = sub['request_id']
        record({'key': key, 'sheet': sheet, 'rect': rect, 'mirror': mirror, 'endpoint': ENDPOINT,
                'request_id': rid, 'est_usd': PRICE, 'submitted': time.time()})
    app = '/'.join(ENDPOINT.split('/')[:2])
    while True:
        st = request('GET', f'https://queue.fal.run/{app}/requests/{rid}/status')
        if st.get('status') == 'COMPLETED':
            break
        time.sleep(3)
    result = request('GET', f'https://queue.fal.run/{app}/requests/{rid}')
    url = result['images'][0]['url']
    for attempt in range(6):
        try:
            urllib.request.urlretrieve(url, out)
            break
        except Exception:
            time.sleep(2 ** attempt)
    record({'key': key, 'completed': time.time()})
    return key, 'ok'


def reject(keys, reason):
    for key in keys:
        (WORK / (key.replace('/', '__') + '.png')).unlink(missing_ok=True)
        record({'key': key, 'request_id': None, 'rejected': reason, 'at': time.time()})


def main():
    p = argparse.ArgumentParser()
    p.add_argument('mode', choices=['plan', 'run', 'reject'])
    p.add_argument('--keys', nargs='*', default=[])
    p.add_argument('--reason', default='')
    p.add_argument('--only', default='')
    p.add_argument('--limit', type=int, default=0)
    p.add_argument('--workers', type=int, default=6)
    a = p.parse_args()
    if a.mode == 'reject':
        reject(a.keys, a.reason)
        return
    todo = [j for j in jobs() if j[0].startswith(a.only)]
    if a.limit:
        todo = todo[:a.limit]
    pending = [j for j in todo if not (WORK / (j[0].replace('/', '__') + '.png')).exists()]
    print(f'{len(todo)} jobs, {len(pending)} without a mask, est ${len(pending) * PRICE:.2f}')
    if a.mode == 'plan':
        for j in pending:
            print(' ', j[0])
        return
    WORK.mkdir(parents=True, exist_ok=True)
    # The approved example pair is bootstrapped from the first accepted mask.
    # Buildings and NPCs each get an example of their own family.
    examples = {}
    for family, suffix in (('building', ''), ('npc', '_npc')):
        pair = [HERE / f'example{suffix}_input.png', HERE / f'example{suffix}_mask.png']
        examples[family] = [data_uri(Image.open(f)) for f in pair] if all(f.exists() for f in pair) else []
    with ThreadPoolExecutor(a.workers) as pool:
        for key, status in pool.map(lambda j: run_job(j, examples[j[0].split('/')[0]]), pending):
            print(status, key, flush=True)


if __name__ == '__main__':
    main()
