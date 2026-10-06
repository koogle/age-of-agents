#!/usr/bin/env python3
"""Check building selection wire behavior with mouse and DPR2 touch.

Requires rebuilt web/pkg, aiohttp, Playwright and /usr/bin/chromium.
Uses a controlled snapshot on loopback :8017, never a saved/production world.
Checks every gather phase, building switching and stopped-carrier deposits.
"""
import argparse
import asyncio
import copy
import hashlib
import json
from pathlib import Path

from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--mode', choices=['desktop', 'phone'])
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
state = json.loads((ROOT / 'docs/verification/menu-icon-fixture.json').read_text())
state['terrain']['cells'] = 'A' * (state['columns'] * state['rows'])
state['terrain']['heights'] = 'g' * (state['columns'] * state['rows'])
state['resources'] = []
state['ships'] = []
state['animals'] = []
state['simulation_speed'] = 1
building = state['buildings'][0]
building.update(origin={'column': 29, 'row': 20}, columns=5, rows=5)
second = copy.deepcopy(building)
second.update(id='lumber-mill', kind='lumber_mill', origin={'column': 24, 'row': 26}, columns=3, rows=3, produces=[], researches=[])
state['buildings'] = [building, second]
state['units'] = state['units'][:1]
unit = state['units'][0]
unit.update(cell={'column': 35, 'row': 26}, position={'x': 35.5, 'y': 26.5}, step=None, cargo={'kind': 'wood', 'amount': 7.0})
clients = []
commands = []


async def websocket(request):
    socket = web.WebSocketResponse()
    await socket.prepare(request)
    clients.append(socket)
    for _ in range(3):
        await broadcast()
        await asyncio.sleep(0.1)
    async for msg in socket:
        if msg.type == web.WSMsgType.TEXT:
            commands.append(json.loads(msg.data)['command'])
    return socket


async def file(request):
    path = request.match_info['path'] or 'web/index.html'
    candidate = (ROOT / path).resolve()
    if not any(candidate.is_relative_to(ROOT / x) for x in ['assets', 'web']):
        raise web.HTTPNotFound()
    return web.FileResponse(candidate)


async def broadcast():
    state['tick'] += 1
    for socket in clients:
        if not socket.closed:
            await socket.send_json({'type': 'snapshot', 'sequence': state['tick'], 'world': state})


async def main():
    app = web.Application()
    app.router.add_get('/ws', websocket)
    app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner, '127.0.0.1', 8017).start()
    results = []
    previous = args.output / 'results.json'
    if args.mode and previous.exists():
        saved = json.loads(previous.read_text())
        assert saved['bundle_sha256'] == hashlib.sha256((ROOT / 'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest()
        results = [check for check in saved['checks'] if check['mode'] != args.mode]
    async with async_playwright() as playwright:
        browser = await playwright.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        for mode, width, height, dpr in [('desktop', 1280, 800, 1), ('phone', 390, 844, 2)]:
            if args.mode and mode != args.mode:
                continue
            page = await browser.new_page(viewport={'width': width, 'height': height}, device_scale_factor=dpr, is_mobile=mode == 'phone', has_touch=mode == 'phone')
            errors = []
            page.on('pageerror', lambda error: errors.append(str(error)))
            page.on('console', lambda message: print(message.type, message.text, flush=True) if message.type in ('warning', 'error') else None)
            unit['action'] = {'type': 'idle'}
            await page.goto('http://127.0.0.1:8017/', wait_until='networkidle')
            await page.locator('#loading').wait_for(state='detached', timeout=120000)
            if mode == 'phone':
                # Camera setup keeps both storage sprites within the touch viewport.
                await page.mouse.move(width / 2, height / 2)
                for _ in range(2):
                    await page.mouse.wheel(0, 500)
                    await asyncio.sleep(1.0)
            tap = page.touchscreen.tap if mode == 'phone' else page.mouse.click

            async def project(x, z):
                return await page.evaluate('''async ([x,z]) => {
                    const m = await import('/web/pkg/aoa_client.js');
                    return Array.from(m.debug_screen_of(x,m.debug_height_at(x,z),z));
                }''', [x, z])

            async def click_at(x, z, lift=12):
                point = await project(x, z)
                assert 0 <= point[0] < width and 0 <= point[1] - lift < height, (mode, 'target outside viewport', point)
                await tap(point[0], point[1] - lift)
                await asyncio.sleep(1.0)
                return await project(x, z)

            for phase in ['idle', 'to_resource', 'gathering', 'returning', 'depositing']:
                unit['action'] = {'type': 'idle'} if phase == 'idle' else {'type': 'gather', 'resource_id': 'tree-test', 'phase': phase}
                await broadcast()
                await asyncio.sleep(0.5)
                await broadcast()
                await asyncio.sleep(1.0)
                selection = await click_at(17.75, 13.25)
                assert selection[2:4] == [1, 0], (mode, phase, 'unit selection', selection)
                commands.clear()
                selection = await click_at(15.75, 11.25, 35)
                assert selection[2:4] == [0, 1], (mode, phase, 'building selection', selection)
                expected = [{'type': 'deposit', 'unit_id': unit['id'], 'storage_id': building['id']}] if phase == 'idle' else []
                assert commands == expected, (mode, phase, commands)
                commands.clear()
                selection = await click_at(12.75, 13.75, 25)
                assert selection[2:4] == [0, 1], (mode, phase, 'building switch', selection)
                assert not commands, (mode, phase, 'building switch issued order', commands)
                if phase == 'idle':
                    await click_at(17.75, 13.25)
                    await click_at(12.75, 13.75, 25)
                    assert commands == [{'type': 'deposit', 'unit_id': unit['id'], 'storage_id': second['id']}], (mode, 'lumber mill target', commands)
                    commands.clear()
                results.append({'mode': mode, 'phase': phase, 'building_selected': True, 'switch_issued_orders': False, 'deposit_orders': len(expected)})
                if phase == 'gathering':
                    await page.screenshot(path=str(args.output / f'{mode}-building-selected.png'))
                print(mode, phase, 'PASS', flush=True)
            assert not errors, errors
            await page.close()
        await browser.close()
    await runner.cleanup()
    (args.output / 'results.json').write_text(json.dumps({'bundle_sha256': hashlib.sha256((ROOT / 'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest(), 'checks': results}, indent=2) + '\n')


asyncio.run(main())
