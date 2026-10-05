#!/usr/bin/env python3
"""Render four controlled shore orientations through current WebGL assets.

Presentation fixture only: domain tests cover shore choice/placement. Uses an
isolated loopback WebSocket, no saved world or production connection.
"""
import argparse
import asyncio
import copy
import json
from pathlib import Path

from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]


def fixture(direction):
    state = json.loads((Path(__file__).parent / 'presentation-fixture.json').read_text())
    stock = state.pop('stockpile')
    state.update(island_id=0, island_count=1, island_origins=[{'column': 0, 'row': 0}],
                 inventories=[stock], stored_inventories=[stock], ship_connections=[], ships=[])
    state['units'] = []
    state['resources'] = []
    state['buildings'] = [dict(id='dock-test', kind='dock', origin={'column': 28, 'row': 19},
                               construction=None, produces=['transport_ship'], researches=[], job=None,
                               queue=[], next_queue_id=0, columns=4, rows=4)]
    water = {'south': lambda x, y: y >= 23, 'east': lambda x, y: x >= 32,
             'north': lambda x, y: y < 19, 'west': lambda x, y: x < 28}[direction]
    state['terrain'] = dict(columns=120, rows=80,
                            cells=''.join('J' if water(x, y) else 'I' for y in range(80) for x in range(120)),
                            heights='g' * 9600)
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
                await send(fixture('south'))
            await page.clock.fast_forward(1000)
            await page.locator('#loading').wait_for(state='detached', timeout=90000)
            await page.mouse.move(viewport['width'] / 2, viewport['height'] / 2)
            await page.mouse.wheel(0, -1200)
            await page.clock.run_for(50)
            for direction in ['south', 'east', 'north', 'west']:
                state = fixture(direction)
                await send(state)
                await page.screenshot(path=str(output / f'{name}-{direction}.png'))
                print(name, direction, 'captured', flush=True)
                # Exercise sprite picking with the same projected anchor on mouse/touch.
                xy = await page.evaluate("""async () => {
                    const m = await import('/web/pkg/aoa_client.js');
                    return Array.from(m.debug_screen_of(15, m.debug_height_at(15,10.5)+0.25, 10.5));
                }""")
                if dpr == 2:
                    await page.touchscreen.tap(xy[0], xy[1])
                else:
                    await page.mouse.click(xy[0], xy[1])
                await page.clock.run_for(34)
                selected = await page.evaluate("async () => Array.from((await import('/web/pkg/aoa_client.js')).debug_screen_of(15,0,10.5))[3]")
                assert selected == 1, (name, direction, 'dock picking failed')
                for work, stage in [(0, 'foundation'), (3, 'walls'), (6, 'roof')]:
                    building = copy.deepcopy(state)
                    building['buildings'][0]['construction'] = work
                    await send(building)
                    await page.screenshot(path=str(output / f'{name}-{direction}-{stage}.png'))
            await page.close()
        await browser.close()
    await runner.cleanup()
    assert not errors, errors
    print(json.dumps({'browser_errors': errors, 'captures': 32}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    asyncio.run(main(parser.parse_args().output))
