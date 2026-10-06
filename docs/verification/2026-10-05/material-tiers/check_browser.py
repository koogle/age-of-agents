#!/usr/bin/env python3
"""Check mouse/touch masonry upgrades against an isolated funded local server.

Run after starting the current server at :8000 on a disposable seed-123 database,
paused, with unrestricted economy rules and 100 bricks / 100 timber. Restore the
same fixture before each --phone run. This never resets or funds a live game.
"""
import argparse
import asyncio
import json
from pathlib import Path
from playwright.async_api import async_playwright

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--phone', action='store_true')
parser.add_argument('--functional-only', action='store_true', help='Retest commands/reload while retaining existing art captures')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        width, height = (390, 844) if args.phone else (1440, 900)
        page = await browser.new_page(viewport={'width': width, 'height': height}, device_scale_factor=2 if args.phone else 1, has_touch=args.phone, is_mobile=args.phone)
        errors = []
        page.on('pageerror', lambda e: errors.append(str(e)))
        await page.add_init_script('''window.sentCommands=[];const send=WebSocket.prototype.send;WebSocket.prototype.send=function(data){try{window.sentCommands.push(JSON.parse(data));}catch(e){}return send.call(this,data);};''')
        await page.goto('http://127.0.0.1:8000', wait_until='networkidle')
        await page.locator('#loading').wait_for(state='detached', timeout=60000)
        async def state():
            return await (await page.request.get('http://127.0.0.1:8000/state')).json()
        async def capture(filename):
            if not args.functional_only or filename == 'masonry-complete.png':
                await page.screenshot(path=str(args.output / filename))
        async def zoom_capture(filename):
            if args.functional_only:
                return
            if args.phone:
                cdp = await page.context.new_cdp_session(page)
                for _ in range(4):
                    await cdp.send('Input.dispatchTouchEvent', {'type': 'touchStart', 'touchPoints': [{'x': 175, 'y': 350, 'id': 1}, {'x': 215, 'y': 350, 'id': 2}]})
                    for spread in [35, 55, 80, 110, 140]:
                        await cdp.send('Input.dispatchTouchEvent', {'type': 'touchMove', 'touchPoints': [{'x': 195-spread, 'y': 350, 'id': 1}, {'x': 195+spread, 'y': 350, 'id': 2}]})
                    await cdp.send('Input.dispatchTouchEvent', {'type': 'touchEnd', 'touchPoints': []})
            else:
                await page.mouse.move(width/2, height/2-80)
                for _ in range(3):
                    await page.mouse.wheel(0, -10000)
                    await page.wait_for_timeout(100)
            await page.wait_for_timeout(500)
            await capture(filename)
        initial = await state()
        assert initial['simulation_speed'] == 0
        assert initial['masonry_upgrades_available']
        assert not initial['buildings'][0]['masonry']
        click = page.touchscreen.tap if args.phone else page.mouse.click
        await click(width / 2, height / 2 - 120)
        await page.wait_for_timeout(500)
        await capture('starter-selected.png')
        await zoom_capture('starter-maximum-zoom.png')
        if not args.functional_only:
            await page.reload(wait_until='networkidle')
            await page.locator('#loading').wait_for(state='detached', timeout=60000)
            await click(width / 2, height / 2 - 120)
        # Upstream pause contract rejects gameplay orders until resumed.
        await click(324, 726) if args.phone else await click(1308, 736)
        async with asyncio.timeout(10):
            while (await state())['simulation_speed'] != 1:
                await asyncio.sleep(.1)
        await click(146, 804) if args.phone else await click(906, 850)
        async with asyncio.timeout(10):
            while not any(m.get('command', {}).get('type') == 'upgrade_building' for m in await page.evaluate('window.sentCommands')):
                await asyncio.sleep(.1)
        paid = await state()
        assert paid['inventories'][0]['bricks'] == initial['inventories'][0]['bricks'] - 30
        assert paid['inventories'][0]['timber'] == initial['inventories'][0]['timber'] - 15
        assert paid['buildings'][0]['job']['type'] == 'upgrade'
        await capture('upgrade-active.png')
        # The actual speed coin advances the real domain/server, not a mocked snapshot.
        await click(360, 726) if args.phone else await click(1354, 724)
        async with asyncio.timeout(30):
            while not (await state())['buildings'][0]['masonry']:
                await asyncio.sleep(.25)
        done = await state()
        assert done['buildings'][0]['origin'] == initial['buildings'][0]['origin']
        assert done['buildings'][0]['id'] == initial['buildings'][0]['id']
        assert done['inventories'][0] == paid['inventories'][0]
        await capture('masonry-complete.png')
        commands = await page.evaluate('window.sentCommands')
        await page.reload(wait_until='networkidle')
        await page.locator('#loading').wait_for(state='detached', timeout=60000)
        assert (await state())['buildings'][0]['masonry']
        await zoom_capture('maximum-zoom.png')
        assert not errors, errors
        result = {'mode': 'DPR2 phone touch' if args.phone else 'desktop mouse', 'viewport': [width, height], 'browser': browser.version, 'errors': errors, 'commands': commands, 'bricks_after': done['inventories'][0]['bricks'], 'timber_after': done['inventories'][0]['timber'], 'masonry': True, 'full_art_capture': not args.functional_only, 'completed_capture': True}
        (args.output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))
        await browser.close()

asyncio.run(main())
