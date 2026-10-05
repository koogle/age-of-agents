#!/usr/bin/env python3
"""Inspect exported island snapshots in the current WebGL bundle, without a save.

Run preview.py first, then: python3 docs/verification/island-generation/browser.py /tmp/island-previews
This checks hosted snapshot playback and a fresh in-browser simulation.
"""
import asyncio
import json
import sys
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(sys.argv[1]).resolve()
state = json.loads((OUT / 'bay.json').read_text())

async def ws(request):
    sock = web.WebSocketResponse()
    await sock.prepare(request)
    sequence = 0
    try:
        while not sock.closed:
            sequence += 1
            await sock.send_json({'type': 'snapshot', 'sequence': sequence, 'world': state})
            await asyncio.sleep(0.1)
    except (ConnectionError, asyncio.CancelledError):
        pass
    return sock

async def file(request):
    path = request.match_info['path']
    path = 'web/index.html' if path == 'play' else path
    candidate = (ROOT / path).resolve()
    if not any(candidate.is_relative_to(ROOT / folder) for folder in ['web', 'assets']):
        raise web.HTTPNotFound()
    return web.FileResponse(candidate)

async def main():
    app = web.Application()
    app.router.add_get('/ws', ws)
    app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner, '127.0.0.1', 8001).start()
    try:
        async with async_playwright() as p:
            browser = await p.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
            for name, viewport, dpr, touch in [('desktop', {'width': 1280, 'height': 800}, 1, False), ('phone', {'width': 390, 'height': 844}, 2, True)]:
                page = await browser.new_page(viewport=viewport, device_scale_factor=dpr, is_mobile=touch, has_touch=touch)
                errors = []
                page.on('pageerror', lambda e: errors.append(str(e)))
                await page.goto('http://127.0.0.1:8001/play', wait_until='networkidle')
                await page.locator('#loading').wait_for(state='detached', timeout=90000)
                await page.wait_for_timeout(2500)
                await page.screenshot(path=str(OUT / f'{name}-bay.png'))
                if touch:
                    await page.touchscreen.tap(195, 400)
                else:
                    await page.mouse.move(640, 400)
                    await page.mouse.wheel(0, 550)
                await page.wait_for_timeout(1000)
                await page.screenshot(path=str(OUT / f'{name}-overview.png'))
                assert not errors, errors
                print(f'{name}: terrain displayed; input exercised; no JavaScript errors', flush=True)
                await page.goto('http://127.0.0.1:8001/play?local&seed=17', wait_until='networkidle')
                await page.locator('#loading').wait_for(state='detached', timeout=90000)
                await page.wait_for_timeout(2000)
                await page.screenshot(path=str(OUT / f'{name}-local-start.png'))
                assert not errors, errors
                print(f'{name}: fresh bay island generated in WASM; no JavaScript errors', flush=True)
                await page.close()
            await browser.close()
    finally:
        await runner.cleanup()

asyncio.run(main())
