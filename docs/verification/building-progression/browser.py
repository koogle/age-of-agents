"""Menu acceptance with a domain-exported snapshot; no saved game is touched.

cargo run -p aoa-game --example island_preview -- 17 > /tmp/starter.json
python docs/verification/building-progression/browser.py /tmp/starter.json OUTPUT
"""
import asyncio
import hashlib
import json
import sys
from pathlib import Path

from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(sys.argv[2]).resolve()
OUT.mkdir(parents=True, exist_ok=True)
state = json.loads(Path(sys.argv[1]).read_text())
assert len(state['available_buildings']) == 10
assert 'barracks' not in state['available_buildings']
# Flatten the presentation fixture and center one worker for reproducible picking.
state['terrain']['cells'] = 'A' * (state['columns'] * state['rows'])
state['terrain']['heights'] = 'g' * (state['columns'] * state['rows'])
state['resources'] = []
state['buildings'] = []
state['animals'] = []
state['units'] = state['units'][:1]
state['units'][0].update(cell={'column': 30, 'row': 21},
                         position={'x': 30.0, 'y': 21.0}, step=None,
                         action={'type': 'idle'})

async def ws(request):
    sock = web.WebSocketResponse()
    await sock.prepare(request)
    sequence = 0
    try:
        while not sock.closed:
            sequence += 1
            await sock.send_json({'type': 'snapshot', 'sequence': sequence, 'world': state})
            await asyncio.sleep(.2)
    except (ConnectionError, asyncio.CancelledError):
        pass
    return sock

async def file(request):
    path = request.match_info['path']
    candidate = (ROOT / ('web/index.html' if path == 'play' else path)).resolve()
    if not any(candidate.is_relative_to(ROOT / folder) for folder in ['web', 'assets']):
        raise web.HTTPNotFound()
    return web.FileResponse(candidate)

async def main():
    app = web.Application()
    app.router.add_get('/ws', ws)
    app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner, '127.0.0.1', 8012).start()
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        for mode, w, h, dpr in [('desktop', 1280, 800, 1), ('phone', 390, 844, 2)]:
            page = await browser.new_page(viewport={'width': w, 'height': h}, device_scale_factor=dpr, is_mobile=dpr == 2, has_touch=dpr == 2)
            errors = []
            page.on('pageerror', lambda e: errors.append(str(e)))
            await page.goto('http://127.0.0.1:8012/play', wait_until='networkidle')
            await page.locator('#loading').wait_for(state='detached', timeout=120000)
            tap = page.touchscreen.tap if dpr == 2 else page.mouse.click
            await tap(w / 2, h / 2 - 20)
            await asyncio.sleep(1)
            await tap(w / 2 - 31 if dpr == 1 else 42, h - 50 if dpr == 1 else h - 40)
            await asyncio.sleep(1)
            await page.screenshot(path=str(OUT / f'{mode}-categories.png'))
            # Five categories/close actions: desktop single row, phone four columns.
            await tap(702 if dpr == 1 else 198, 724 if dpr == 1 else 700)
            await asyncio.sleep(1)
            await page.screenshot(path=str(OUT / f'{mode}-military.png'))
            await tap(702 if dpr == 1 else 146, 724 if dpr == 1 else 778)
            await asyncio.sleep(.5)
            await tap(640 if dpr == 1 else 146, 724 if dpr == 1 else 700)
            await asyncio.sleep(1)
            await page.screenshot(path=str(OUT / f'{mode}-production.png'))
            assert not errors, errors
            print(mode, 'menu exercised; no browser errors', flush=True)
            await page.close()
        await browser.close()
    await runner.cleanup()
    (OUT / 'result.json').write_text(json.dumps({'available_buildings': state['available_buildings'], 'wasm_sha256': hashlib.sha256((ROOT / 'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest(), 'errors': [], 'scope': 'Domain-exported availability; flattened presentation fixture; desktop mouse and emulated DPR2 phone touch. Inspect menu screenshots separately.'}, indent=2) + '\n')

asyncio.run(main())
