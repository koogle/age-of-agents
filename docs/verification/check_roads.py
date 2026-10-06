#!/usr/bin/env python3
"""Desktop/touch road placement against an isolated authoritative server.

Prerequisites: cargo build -p age-of-agents; cargo build -p aoa-game --example
roads_fixture; scripts/build_web.sh; Python aiohttp/playwright and Chromium.
Does not touch the configured save. Port 8000 must be free.
"""
import asyncio
import hashlib
import json
import math
import os
from pathlib import Path
import re
import socket
import sqlite3
import subprocess
import tempfile
import argparse

import aiohttp
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
URL = 'http://127.0.0.1:8000'


def button(width, height, count, index, labels=False):
    narrow = width < 600
    size, gap, padding = (44, 8, 8) if narrow else (52, 10, 14)
    per_row = max(1, math.floor((width - 12 - 80 - 28 - 12 - 12 - 16 + gap) / (size + gap))) if narrow else count
    rows = math.ceil(count / per_row)
    step = size + gap + (26 if labels else 0)
    bar_width = min(count, per_row) * (size + gap) - gap + 2 * padding
    bar_height = rows * step - gap + 12
    left = 12 if narrow else (width - bar_width) / 2
    top = height - (12 if narrow else 18) - bar_height
    return left + padding + (index % per_row) * (size + gap) + size / 2, top + 6 + (index // per_row) * step + size / 2


async def main(output, mode_filter, screenshots, resume_only=False):
    with socket.socket() as probe:
        if probe.connect_ex(("127.0.0.1", 8000)) == 0:
            raise RuntimeError("Port 8000 is already in use; stop the task-owned server first")
    output.mkdir(parents=True, exist_ok=True)
    world = subprocess.check_output([ROOT / 'target/debug/examples/roads_fixture'], cwd=ROOT).decode()
    version = int(re.search(r'const STORE_VERSION: u32 = (\d+)', (ROOT/'src/store.rs').read_text())[1])
    results = []
    async with aiohttp.ClientSession() as client, async_playwright() as pw:
        browser = await pw.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        for mode, width, height, dpr in [('desktop',1280,800,1),('phone',390,844,2)]:
            if mode_filter != 'all' and mode != mode_filter: continue
            with tempfile.TemporaryDirectory(prefix='aoa-roads-') as temp:
                fixture = json.loads(world)
                if resume_only:
                    start = 31 if mode == 'phone' else 27
                    fixture['roads'] = [
                        {'cell':{'column':start+n,'row':row}, 'kind':kind,
                         'work':0.5 if n==0 else 0.0}
                        for kind,row in [('dirt',24),('stone',26)] for n in range(7)
                    ]
                    fixture['inventories'][0]['stone'] -= 7
                db=Path(temp)/'roads.db'
                with sqlite3.connect(db) as conn:
                    conn.execute(f'PRAGMA user_version={version}')
                    conn.execute('CREATE TABLE world_state(id INTEGER PRIMARY KEY, world_json TEXT NOT NULL)')
                    conn.execute('INSERT INTO world_state VALUES (1,?)',(json.dumps(fixture),))
                with (output/f'{mode}-server.log').open('w') as log:
                    process=subprocess.Popen([ROOT/'target/debug/age-of-agents'],cwd=ROOT,env={**os.environ,'AGE_OF_AGENTS_DB':str(db)},stdout=log,stderr=log)
                    try:
                        async def state():
                            async with client.get(URL+'/state') as response: return await response.json()
                        for _ in range(100):
                            try:
                                await state(); break
                            except aiohttp.ClientError: await asyncio.sleep(.1)
                        assert process.poll() is None, 'isolated server failed to start'
                        page=await browser.new_page(viewport={'width':width,'height':height},device_scale_factor=dpr,is_mobile=mode=='phone',has_touch=mode=='phone')
                        errors=[];commands=[]
                        async def capture(name):
                            if screenshots: await page.screenshot(path=str(output/name))
                        page.on('pageerror',lambda e:errors.append(str(e)))
                        def capture_socket(ws):
                            ws.on('framesent',lambda data:commands.append(json.loads(data)))
                        page.on('websocket',capture_socket)
                        await page.add_init_script('const raf=window.requestAnimationFrame.bind(window); window.roadFrames=0; window.requestAnimationFrame=cb=>setTimeout(()=>raf(t=>{cb(t);window.roadFrames++;}),90);')
                        await page.goto(URL+'/play',wait_until='networkidle')
                        await page.locator('#loading').wait_for(state='detached',timeout=180000)
                        print(mode,'loaded',flush=True)
                        await page.evaluate("async () => window.debug=await import('/web/pkg/aoa_client.js')")
                        async def settle(frames=3):
                            before=await page.evaluate('window.roadFrames')
                            await page.wait_for_function('(v)=>window.roadFrames>=v[0]+v[1]',arg=[before,frames],polling=100,timeout=120000)
                        async def ground(x,y):
                            return await page.evaluate('([x,z])=>Array.from(debug.debug_screen_of(x,debug.debug_height_at(x,z),z))',[(x+.5)*.5,(y+.5)*.5])
                        tap=page.touchscreen.tap if mode=='phone' else page.mouse.click
                        async def select():
                            s=await state();u=s['units'][0]['cell'];p=await ground(u['column'],u['row']);await tap(p[0],p[1]-12);await settle()
                        async def menu(count,index,labels=False):
                            await tap(*button(width,height,count,index,labels));await settle()
                        await select()
                        await capture(f'{mode}-before.png')
                        start_column = 31 if mode == 'phone' else 27
                        for kind,row in [('dirt',24),('stone',26)]:
                            if not resume_only:
                                await menu(1,0) # Build
                                await menu(6,4,True) # Roads
                                if kind == 'dirt': await capture(f'{mode}-{kind}-menu.png')
                                await menu(3,0 if kind=='dirt' else 1,True)
                                p=await ground(start_column,row);await tap(*p[:2]);await asyncio.sleep(.3)
                                p=await ground(start_column+6,row+1) # dominant-axis snapping
                                if mode=='desktop': await page.mouse.move(*p[:2])
                                await asyncio.sleep(.3)
                                if kind == 'dirt': await capture(f'{mode}-{kind}-preview.png')
                                await tap(*p[:2])
                            # Interrupt paid work, then resume by clicking one unfinished piece.
                            async with client.ws_connect(URL+'/ws') as control:
                                unit_id = (await state())['units'][0]['id']
                                await control.send_json({'type':'command','request_id':'road-stop',
                                    'command':{'type':'stop','unit_id':unit_id}})
                                async for message in control:
                                    reply=json.loads(message.data)
                                    if reply.get('request_id')=='road-stop':
                                        assert reply['ok'],reply
                                        break
                            stopped=await state()
                            unfinished=[r for r in stopped['roads'] if r['kind']==kind and r['work'] is not None]
                            assert len(unfinished)>1, stopped
                            await asyncio.sleep(.3)
                            assert (await state())['roads']==stopped['roads']
                            await settle(6)
                            target=unfinished[len(unfinished)//2]['cell']
                            p=await ground(target['column'],target['row']);await tap(*p[:2])
                            for _ in range(350):
                                s=await state()
                                roads=[r for r in s['roads'] if r['kind']==kind]
                                if len(roads)==7 and all(r['work'] is None for r in roads): break
                                await asyncio.sleep(.1)
                            assert len(roads)==7 and all(r['work'] is None for r in roads), (mode,kind,s['units'],roads,commands)
                            assert {r['cell']['row'] for r in roads}=={row}
                            assert s['inventories'][0]['stone']==(30 if kind=='dirt' and not resume_only else 23)
                            await settle(6)
                            if kind == 'stone': await capture(f'{mode}-{kind}-complete.png')
                        if screenshots:
                            # Anchor maximum zoom on both road strips, after input checks.
                            p=await ground(start_column+3,25)
                            await page.mouse.move(*p[:2])
                            for _ in range(3):
                                await page.mouse.wheel(0,-5000)
                                await settle()
                            await capture(f'{mode}-maximum-zoom.png')
                        assert len([c for c in commands if c.get('command',{}).get('type')=='build_road'])==(2 if resume_only else 4),commands
                        assert not errors,errors
                        results.append({'mode':mode,'viewport':[width,height,dpr],'road_cells':14,'resumed_by_single_cell':True,'resume_only':resume_only,'stone_remaining':23,'errors':errors,'server_sha256':hashlib.sha256((ROOT/'target/debug/age-of-agents').read_bytes()).hexdigest()})
                        (output/f'{mode}-state.json').write_text(json.dumps(await state()))
                        print(mode,'passed',flush=True)
                        await page.close()
                    finally:
                        process.terminate();process.wait(timeout=10)
        await browser.close()
    (output/'results.json').write_text(json.dumps({'wasm_sha256':hashlib.sha256((ROOT/'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest(),'results':results},indent=2)+'\n')

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--mode',choices=['all','desktop','phone'],default='all')
    parser.add_argument('--no-screenshots',action='store_true')
    parser.add_argument('--resume-only',action='store_true',help='Resume already paid interrupted roads without replaying placement menus')
    args=parser.parse_args()
    asyncio.run(main(args.output,args.mode,not args.no_screenshots,args.resume_only))
