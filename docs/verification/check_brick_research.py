#!/usr/bin/env python3
"""Actual mouse/touch farm upgrade -> research acceptance on disposable SQLite.

Generate fixture with cargo run -p aoa-game --example research_fixture. Requires
current web/server builds, aiohttp, Playwright and /usr/bin/chromium. Owns :8000.
"""
import argparse
import asyncio
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import tempfile

from aiohttp import ClientSession
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]

async def main(fixture, output):
    output.mkdir(parents=True, exist_ok=True)
    results = []
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        for phone in [False, True]:
            name = 'phone' if phone else 'desktop'
            with tempfile.TemporaryDirectory(prefix='brick-research-') as temp:
                db = Path(temp) / 'world.db'
                with sqlite3.connect(db) as connection:
                    connection.executescript('PRAGMA user_version=17; CREATE TABLE world_state(id INTEGER PRIMARY KEY, world_json TEXT NOT NULL);')
                    connection.execute('INSERT INTO world_state VALUES(1,?)', (fixture.read_text(),))
                server = subprocess.Popen([str(ROOT / 'target/debug/age-of-agents')], cwd=ROOT, env={**os.environ, 'AGE_OF_AGENTS_DB': str(db)}, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                try:
                    async with ClientSession() as http:
                        async with asyncio.timeout(20):
                            while True:
                                try:
                                    async with http.get('http://127.0.0.1:8000/state') as response:
                                        assert response.status == 200
                                    break
                                except OSError:
                                    await asyncio.sleep(.1)
                    width, height = (390, 844) if phone else (1440, 900)
                    page = await browser.new_page(viewport={'width': width, 'height': height}, device_scale_factor=2 if phone else 1, has_touch=phone, is_mobile=phone)
                    errors = []
                    page.on('pageerror', lambda e: errors.append(str(e)))
                    await page.add_init_script('window.sent=[];const send=WebSocket.prototype.send;WebSocket.prototype.send=function(d){window.sent.push(JSON.parse(d));return send.call(this,d)};')
                    await page.goto('http://127.0.0.1:8000', wait_until='networkidle')
                    await page.locator('#loading').wait_for(state='detached', timeout=90000)
                    async def state():
                        return await (await page.request.get('http://127.0.0.1:8000/state')).json()
                    async def wait_for(predicate):
                        async with asyncio.timeout(35):
                            while not predicate(await state()):
                                await asyncio.sleep(.1)
                    initial = await state()
                    origin = initial['buildings'][0]['origin']
                    x, z = (origin['column'] + 1.5) * .5, (origin['row'] + 1.5) * .5
                    xy = await page.evaluate('''async ([x,z])=>{const m=await import('/web/pkg/aoa_client.js');return Array.from(m.debug_screen_of(x,m.debug_height_at(x,z)+.3,z));}''',[x,z])
                    click = page.touchscreen.tap if phone else page.mouse.click
                    await click(xy[0],xy[1])
                    await click(324,726) if phone else await click(1308,736)
                    await wait_for(lambda s:s['simulation_speed']==1)
                    research_xy = (42,804) if phone else (720,850)
                    upgrade_xy = (228,740) if phone else (910,780)
                    await click(*research_xy)
                    await page.wait_for_timeout(300)
                    assert not any(m.get('command',{}).get('type')=='research' for m in await page.evaluate('window.sent'))
                    assert (await state())['inventories']==initial['inventories']
                    await page.screenshot(path=str(output/f'{name}-locked.png'))
                    if phone:
                        await click(80,740)
                    else:
                        await page.wait_for_timeout(4500)
                        await page.mouse.move(600,780)
                    await page.screenshot(path=str(output/f'{name}-description.png'))
                    assert (await state())['inventories']==initial['inventories']
                    if not phone:
                        await page.mouse.move(*upgrade_xy)
                        await page.screenshot(path=str(output/f'{name}-upgrade-hover.png'))
                    await click(*upgrade_xy)
                    await wait_for(lambda s:s['buildings'][0]['job'] is not None)
                    assert (await state())['buildings'][0]['job']['type']=='upgrade'
                    await click(*research_xy)
                    assert not any(m.get('command',{}).get('type')=='research' for m in await page.evaluate('window.sent'))
                    await click(360,726) if phone else await click(1354,724)
                    await wait_for(lambda s:s['buildings'][0]['masonry'])
                    upgraded = await state()
                    await page.screenshot(path=str(output/f'{name}-unlocked.png'))
                    await click(*research_xy)
                    await wait_for(lambda s:s['buildings'][0]['job'] is not None)
                    paid = await state()
                    assert paid['buildings'][0]['job']['type']=='research'
                    assert paid['inventories'][0]['food']==upgraded['inventories'][0]['food']-40
                    assert paid['inventories'][0]['wood']==upgraded['inventories'][0]['wood']-20
                    await wait_for(lambda s:'agriculture' in s['researched_technologies'])
                    await page.reload(wait_until='networkidle')
                    await page.locator('#loading').wait_for(state='detached', timeout=90000)
                    assert 'agriculture' in (await state())['researched_technologies']
                    assert not errors, errors
                    results.append({'mode':name,'errors':errors,'locked_click_sent_no_research':True,'research_paid_once':True,'reload_preserves_completion':True})
                    await page.close()
                    print(name,'passed',flush=True)
                finally:
                    server.terminate()
                    server.wait(timeout=10)
        await browser.close()
    (output/'result.json').write_text(json.dumps(results,indent=2)+'\n')

if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    asyncio.run(main(args.fixture,args.output))
