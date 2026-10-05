#!/usr/bin/env python3
"""Exercise drought forecasts against an isolated real server and rebuilt WebGL.

Requires cargo build, scripts/build_web.sh, aiohttp, Playwright and Chromium.
Never opens the production database. Output includes screenshots and observations.
"""
import argparse
import asyncio
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import tempfile

import aiohttp
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
URL = 'http://127.0.0.1:8000'


async def main(output):
    output.mkdir(parents=True, exist_ok=True)
    # Do not accidentally exercise an unrelated server already using this port.
    async with aiohttp.ClientSession() as http:
        try:
            async with http.get(URL + '/state'):
                raise RuntimeError('Port 8000 already serves a world; stop it first')
        except aiohttp.ClientConnectorError:
            pass
        with tempfile.TemporaryDirectory(prefix='aoa-drought-') as directory:
            database = Path(directory) / 'world.db'
            environment = dict(os.environ, AGE_OF_AGENTS_DB=str(database), AGE_OF_AGENTS_SEED='123')
            process = None

            async def start():
                nonlocal process
                process = subprocess.Popen([str(ROOT / 'target/debug/age-of-agents')],
                                           cwd=ROOT, env=environment, stdout=subprocess.DEVNULL,
                                           stderr=subprocess.DEVNULL)
                for _ in range(200):
                    if process.poll() is not None:
                        raise RuntimeError('Fixture server exited')
                    try:
                        async with http.get(URL + '/state') as response:
                            return await response.json()
                    except aiohttp.ClientConnectorError:
                        await asyncio.sleep(.1)
                raise TimeoutError('Fixture server did not start')

            def stop():
                if process and process.poll() is None:
                    process.terminate()
                    process.wait(timeout=10)

            async def state():
                async with http.get(URL + '/state') as response:
                    return await response.json()

            async def speed(multiplier):
                async with http.ws_connect(URL + '/ws') as socket:
                    await socket.send_json({'type': 'command', 'request_id': 'weather-speed',
                                            'command': {'type': 'set_simulation_speed', 'multiplier': multiplier}})
                    async for message in socket:
                        data = json.loads(message.data)
                        if data['type'] == 'command_result':
                            assert data['ok'], data
                            return

            observations = []
            try:
                await start()
                await speed(0)
                stop()
                async with async_playwright() as playwright:
                    browser = await playwright.chromium.launch(executable_path='/usr/bin/chromium',
                        args=['--no-sandbox', '--enable-unsafe-swiftshader'])
                    for label, elapsed, phase, size, dpr in [
                        ('desktop-warning', 359.0, 'drought_warning', (1280, 800), 1),
                        ('phone-drought', 390.0, 'drought', (390, 844), 2),
                        ('landscape-recovery', 420.0, 'calm', (844, 390), 1),
                    ]:
                        with sqlite3.connect(database) as connection:
                            world = json.loads(connection.execute('SELECT world_json FROM world_state WHERE id=1').fetchone()[0])
                            world['environment_seconds'] = elapsed
                            world['simulation_speed'] = 0.0
                            connection.execute('UPDATE world_state SET world_json=? WHERE id=1', (json.dumps(world),))
                        initial = await start()
                        assert initial['environment']['phase'] == phase
                        context = await browser.new_context(viewport={'width': size[0], 'height': size[1]},
                                                           device_scale_factor=dpr, has_touch=dpr == 2)
                        page = await context.new_page()
                        errors = []
                        page.on('pageerror', lambda error: errors.append(str(error)))
                        await page.goto(URL, wait_until='networkidle')
                        await page.locator('#loading').wait_for(state='detached', timeout=90000)
                        await page.wait_for_timeout(700)
                        await page.screenshot(path=str(output / (label + '.png')))
                        # Real mouse/touch on 2x, then pause, using the shared HUD geometry.
                        narrow = size[0] < 600 or size[1] < 500
                        if narrow:
                            gx, gy = size[0] - 92, size[1] - 92
                            fast, pause = (gx + 62, gy - 26), (gx - 26, gy - 26)
                        else:
                            import math
                            gx, gy, radius = size[0] - 154, size[1] - 154, 68
                            def center(index):
                                angle = math.pi * (1.16 + index * .17)
                                return gx + radius + math.cos(angle) * 90, gy + radius + math.sin(angle) * 90
                            fast, pause = center(2), center(0)
                        click = page.touchscreen.tap if dpr == 2 else page.mouse.click
                        await click(*fast)
                        await page.wait_for_timeout(1200)
                        running = await state()
                        assert running['simulation_speed'] == 2.0, running['simulation_speed']
                        if label == 'desktop-warning':
                            assert running['environment']['phase'] == 'drought'
                            await page.screenshot(path=str(output / 'desktop-onset.png'))
                        await click(*pause)
                        await page.wait_for_timeout(250)
                        paused = await state()
                        assert paused['simulation_speed'] == 0.0
                        await page.wait_for_timeout(400)
                        assert (await state())['environment'] == paused['environment']
                        assert not errors, errors
                        observations.append({'label': label, 'initial': initial['environment'],
                                             'running': running['environment'], 'paused': paused['environment'], 'errors': errors})
                        await context.close()
                        stop()
                    await browser.close()
            finally:
                stop()
            (output / 'results.json').write_text(json.dumps(observations, indent=2) + '\n')
            print(json.dumps(observations, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    asyncio.run(main(parser.parse_args().output))
