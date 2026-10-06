#!/usr/bin/env python3
"""Inspect house/granary linework at gameplay and maximum zoom through WebGL.

Presentation fixture only; does not validate upgrade payments. Uses an
isolated loopback WebSocket, no saved world or production connection.
"""
import argparse
import asyncio
import json
from pathlib import Path

from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]


def fixture(kind):
    state = json.loads((Path(__file__).parent / 'presentation-fixture.json').read_text())
    state['units'] = []
    state['resources'] = []
    state['buildings'] = [dict(id='style-test', kind=kind, origin={'column': 28, 'row': 19},
                               masonry=False, construction=None, produces=[], researches=[], job=None,
                               queue=[], next_queue_id=0, columns=3, rows=3)]
    state['terrain'] = dict(columns=120, rows=80, cells='I' * 9600, heights='g' * 9600)
    return state


async def main(output):
    output.mkdir(parents=True, exist_ok=True)
    clients = []

    async def ws(request):
        sock = web.WebSocketResponse()
        await sock.prepare(request)
        clients.append(sock)
        async for _ in sock:
            pass
        return sock

    async def file(request):
        path = request.match_info['path']
        path = 'web/index.html' if path == 'play' else path
        candidate = (ROOT / path).resolve()
        if not any(candidate.is_relative_to(ROOT / folder) for folder in ['web', 'assets']):
            raise web.HTTPNotFound()
        return web.FileResponse(candidate)

    app = web.Application()
    app.router.add_get('/ws', ws)
    app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner, '127.0.0.1', 8001).start()
    errors = []
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        for name, viewport, dpr in [('desktop', {'width': 1200, 'height': 900}, 1),
                                    ('phone', {'width': 390, 'height': 844}, 2)]:
            page = await browser.new_page(viewport=viewport, device_scale_factor=dpr, has_touch=dpr == 2)
            page.on('pageerror', lambda e: errors.append(str(e)))
            page.on('console', lambda m: print(m.type, m.text, flush=True) if m.type in ['warning', 'error'] else None)
            await page.goto('http://127.0.0.1:8001/play', wait_until='networkidle')
            async with asyncio.timeout(90):
                while len(clients) < (1 if name == 'desktop' else 2):
                    await asyncio.sleep(0.1)
            print(name, 'connected', flush=True)
            sock = clients[-1]
            await page.clock.install()
            await page.clock.pause_at(await page.evaluate('Date.now() + 100'))
            tick = 0

            async def send(state):
                nonlocal tick
                for _ in range(4):
                    tick += 1
                    state['tick'] = tick
                    await sock.send_json(dict(type='snapshot', sequence=tick, world=state))
                    await asyncio.sleep(0.05)
                    await page.clock.run_for(17)

            for _ in range(2):
                await send(fixture('house'))
            await page.clock.fast_forward(1000)
            await page.locator('#loading').wait_for(state='detached', timeout=90000)
            for zoom in ['gameplay', 'maximum']:
                if zoom == 'maximum':
                    await page.mouse.move(viewport['width'] / 2, viewport['height'] / 2)
                    for _ in range(3):
                        await page.mouse.wheel(0, -10000)
                        await page.clock.run_for(34)
                for kind in ['house', 'granary']:
                    state = fixture(kind)
                    for stage, work in [('complete', None), ('roof', 3.1 if kind == 'house' else 5.1)]:
                        state['buildings'][0]['construction'] = work
                        await send(state)
                        await page.screenshot(path=str(output / f'{name}-{kind}-{zoom}-{stage}.png'))
                    print(name, kind, zoom, 'captured', flush=True)
            await page.close()
        await browser.close()
    await runner.cleanup()
    assert not errors, errors
    (output / 'result.json').write_text(json.dumps({'browser_errors': errors, 'captures': 16, 'phone': 'emulated DPR2; wheel-controlled zoom', 'fixture': 'presentation only'}, indent=2) + '\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    asyncio.run(main(args.output))
