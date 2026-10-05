#!/usr/bin/env python3
"""Real-server wildlife mouse/touch acceptance using a temporary SQLite fixture.
Requires rebuilt debug server, WebAssembly, aiohttp, Playwright and Chromium.
No production resources are accessed. Fixtures put two guards beside wild animals.
"""
import argparse
import asyncio
import copy
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

async def main(out, only):
    out.mkdir(parents=True, exist_ok=True)
    async with aiohttp.ClientSession() as http:
        try:
            async with http.get(URL + '/state'):
                raise RuntimeError('Port 8000 already in use')
        except aiohttp.ClientConnectorError:
            pass
        with tempfile.TemporaryDirectory(prefix='aoa-wildlife-') as directory:
            db = Path(directory) / 'world.db'
            env = dict(os.environ, AGE_OF_AGENTS_DB=str(db), AGE_OF_AGENTS_SEED='123')
            process = None
            async def start():
                nonlocal process
                process = subprocess.Popen([str(ROOT/'target/debug/age-of-agents')],cwd=ROOT,env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
                for _ in range(200):
                    if process.poll() is not None: raise RuntimeError('Server exited')
                    try:
                        async with http.get(URL+'/state') as r: return await r.json()
                    except aiohttp.ClientConnectorError: await asyncio.sleep(.1)
                raise TimeoutError('Server startup')
            def stop():
                if process and process.poll() is None:
                    process.terminate(); process.wait(timeout=10)
            async def state():
                async with http.get(URL+'/state') as r: return await r.json()
            async def command(c):
                async with http.ws_connect(URL+'/ws') as ws:
                    await ws.send_json({'type':'command','request_id':'fixture','command':c})
                    async for m in ws:
                        d=json.loads(m.data)
                        if d['type']=='command_result':
                            assert d['ok'],d
                            return
            def cell(x,y):return {'column':x,'row':y}
            results=[]
            try:
                await start();await command({'type':'set_simulation_speed','multiplier':0.0});stop()
                with sqlite3.connect(db) as con:
                    fixture=json.loads(con.execute('SELECT world_json FROM world_state WHERE id=1').fetchone()[0])
                assert [a['kind'] for a in fixture['animals']] == ['wolf'], fixture['animals']
                x=fixture['units'][0]['cell']['column'];y=fixture['units'][0]['cell']['row']+3
                fixture['resources']=[r for r in fixture['resources'] if not (x-9<=r['cell']['column']<=x+9 and y-2<=r['cell']['row']<=y+7)]
                for t in fixture['terrain']:
                    if x-9<=t['column']<=x+9 and y-2<=t['row']<=y+7:
                        t['biome']='meadow';t['elevation']=0.3
                for i,u in enumerate(fixture['units']):
                    u.update(kind='guard',health=100.0,cell=cell(x-i,y),step=None,action={'type':'idle'},cargo=None)
                fixture['animals']=[]
                for name,kind,at,hp in [('bear','bear',cell(x+4,y+1),100.0),('wolf','wolf',cell(x-4,y+2),40.0)]:
                    fixture['animals'].append({'id':name,'kind':kind,'home':at,'cell':at,'step':None,'health':hp,'attack_seconds':0.0,'heading':[1,0]})
                async with async_playwright() as p:
                    browser=await p.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--enable-unsafe-swiftshader'])
                    for label,size,dpr in [('desktop',(1280,800),1),('phone',(390,844),2)]:
                        if only and label != only: continue
                        with sqlite3.connect(db) as con:
                            con.execute('UPDATE world_state SET world_json=? WHERE id=1',(json.dumps(fixture),))
                        await start()
                        context=await browser.new_context(viewport={'width':size[0],'height':size[1]},device_scale_factor=dpr,has_touch=dpr==2)
                        page=await context.new_page();errors=[]
                        page.on('pageerror',lambda e:errors.append(str(e)))
                        await page.goto(URL,wait_until='networkidle')
                        await page.locator('#loading').wait_for(state='detached',timeout=90000)
                        await page.wait_for_timeout(700)
                        if dpr == 2:
                            await page.screenshot(path=str(out/(label+"-closeup.png")))
                            touch = await context.new_cdp_session(page)
                            await touch.send('Input.dispatchTouchEvent', {'type':'touchStart','touchPoints':[{'x':60,'y':420},{'x':330,'y':420}]})
                            for step in range(1,11):
                                await touch.send('Input.dispatchTouchEvent', {'type':'touchMove','touchPoints':[{'x':60+step*9.5,'y':420},{'x':330-step*9.5,'y':420}]})
                            await touch.send('Input.dispatchTouchEvent', {'type':'touchEnd','touchPoints':[]})
                            await page.wait_for_timeout(700)
                        async def screen(at,lift):
                            return await page.evaluate('''async ([c,lift])=>{const m=await import('/web/pkg/aoa_client.js');const x=(c.column+.5)*.5,z=(c.row+.5)*.5;return Array.from(m.debug_screen_of(x,m.debug_height_at(x,z)+lift,z)).slice(0,2)}''',[at,lift])
                        click=page.touchscreen.tap if dpr==2 else page.mouse.click
                        await page.screenshot(path=str(out/(label+'-wildlife.png')))
                        await click(*(await screen(cell(x,y),.3)))
                        await page.wait_for_timeout(250)
                        await click(*(await screen(cell(x+4,y+1),.35)))
                        await page.wait_for_timeout(400)
                        ordered=await state()
                        assert ordered['units'][0]['action']['type']=='attack_animal', ordered['units'][0]['action']
                        await command({'type':'set_simulation_speed','multiplier':2.0})
                        injured=None
                        for _ in range(120):
                            await asyncio.sleep(.15)
                            live=await state()
                            bear=next((a for a in live['animals'] if a['id']=='bear'),None)
                            if bear and bear['health']<100 and injured is None:
                                injured=copy.deepcopy(live)
                                await page.screenshot(path=str(out/(label+'-combat.png')))
                            if not bear:break
                        else:raise AssertionError('Bear was not defeated')
                        assert injured is not None
                        assert any(u['health']<100 for u in live['units']), 'Animal must fight back'
                        await command({'type':'set_simulation_speed','multiplier':0.0})
                        await page.wait_for_timeout(300)
                        await page.screenshot(path=str(out/(label+'-after.png')))
                        assert not errors,errors
                        results.append({'viewport':label,'ordered':ordered['units'][0]['action'],
                                        'injured_animals':injured['animals'],'survivors':live['units'],'errors':errors})
                        await context.close();stop()
                    await browser.close()
            finally:stop()
            result_path=out/('results-'+only+'.json' if only else 'results.json')
            result_path.write_text(json.dumps(results,indent=2)+'\n')
            print(f'{only or "Mouse and touch"} hunting, damage, defeat, and screenshots passed')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--only',choices=['desktop','phone'])
    args=parser.parse_args()
    asyncio.run(main(args.output,args.only))
