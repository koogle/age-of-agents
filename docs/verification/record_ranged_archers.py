#!/usr/bin/env python3
"""Record isolated real-server ranged fights (one and four archers).
Requires built server/WASM, Playwright, Chromium and ffmpeg. No live save access.
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
import time
import aiohttp
from playwright.async_api import async_playwright
ROOT = Path(__file__).resolve().parents[2]
URL = 'http://127.0.0.1:8000'

async def main(out, only=None):
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
                template=copy.deepcopy(fixture['units'][0])
                fixture['units']=[copy.deepcopy(template) for _ in range(4)]
                for i,u in enumerate(fixture['units']):
                    u.update(id=f'archer-{i}',kind='archer',health=100.0,cell=cell(x-i,y),step=None,action={'type':'idle'},cargo=None)
                fixture['animals']=[]
                for name,kind,at,hp in [('wolf','wolf',cell(x+7,y+1),300.0),('bear','bear',cell(x-8,y+6),600.0)]:
                    fixture['animals'].append({'id':name,'kind':kind,'home':at,'cell':at,'step':None,'health':hp,'attack_seconds':0.0,'heading':[1,0]})
                async with async_playwright() as p:
                    browser=await p.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--enable-unsafe-swiftshader'])
                    for count,label in [(1,'one-archer'),(4,'four-archers')]:
                        if only and label != only: continue
                        scene=copy.deepcopy(fixture)
                        scene['units']=scene['units'][:count]
                        scene['animals']=scene['animals'][:1]
                        with sqlite3.connect(db) as con:
                            con.execute('UPDATE world_state SET world_json=? WHERE id=1',(json.dumps(scene),))
                        await start()
                        context=await browser.new_context(viewport={'width':960,'height':600},record_video_dir=str(out/'raw'),record_video_size={'width':960,'height':600})
                        video_start=time.monotonic()
                        page=await context.new_page();errors=[]
                        page.on('pageerror',lambda e:errors.append(str(e)))
                        await page.goto(URL,wait_until='networkidle')
                        await page.locator('#loading').wait_for(state='detached',timeout=90000)
                        await page.wait_for_timeout(500)
                        async def screen(at,lift=0):
                            return await page.evaluate("""async ([c,lift])=>{const m=await import('/web/pkg/aoa_client.js');const x=(c.column+.5)*.5,z=(c.row+.5)*.5;return Array.from(m.debug_screen_of(x,m.debug_height_at(x,z)+lift,z)).slice(0,2)}""",[at,lift])
                        point=await screen(cell(x+2,y))
                        await page.mouse.move(*point);await page.mouse.down()
                        await page.mouse.move(480,260,steps=12);await page.mouse.up()
                        await page.mouse.wheel(0,-180)
                        await page.wait_for_timeout(1000)
                        await page.screenshot(path=str(out/(label+'-preview.png')))
                        clip_start=time.monotonic()-video_start
                        await command({'type':'set_simulation_speed','multiplier':1.0})
                        await command({'type':'attack_animal','unit_ids':[u['id'] for u in scene['units']], 'animal_id':'wolf'})
                        frames=[]
                        fight_start=time.monotonic()
                        while time.monotonic()-fight_start<25:
                            await asyncio.sleep(.2)
                            live=await state()
                            frames.append({'seconds':round(time.monotonic()-fight_start,2),'animals':live['animals'],'units':live['units']})
                            if not live['animals'] or not live['units']:
                                await command({'type':'set_simulation_speed','multiplier':0.0})
                                await page.wait_for_timeout(2500)
                                await page.screenshot(path=str(out/(label+'-outcome.png')))
                                await page.wait_for_timeout(1000)
                                break
                        else:raise AssertionError('Fight did not finish')
                        duration=time.monotonic()-video_start-clip_start
                        await command({'type':'set_simulation_speed','multiplier':0.0})
                        assert not errors,errors
                        assert (not live['animals']) == (count==4)
                        video=page.video
                        await context.close();stop()
                        path=await video.path()
                        results.append({'example':label,'source':str(path),'start':clip_start,'duration':duration,'states':frames,'errors':errors})
                        (out/'recording.json').write_text(json.dumps(results,indent=2)+'\n')
                        title='6-cell range - 1 archer' if count==1 else '6-cell range - 4 archers'
                        subprocess.run(['ffmpeg','-y','-loglevel','error','-ss',str(clip_start),'-i',str(path),'-vf',f"fps=10,drawbox=x=0:y=0:w=iw:h=42:color=black@0.65:t=fill,drawtext=text='{title}':fontcolor=white:fontsize=22:x=20:y=10",'-c:v','libx264','-pix_fmt','yuv420p',str(out/(label+'.mp4'))],check=True)
                        print(label+' recorded',flush=True)
                    await browser.close()
            finally:stop()
            (out/'recording.json').write_text(json.dumps(results,indent=2)+'\n')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--only',choices=['one-archer','four-archers'])
    args=parser.parse_args()
    asyncio.run(main(args.output,args.only))
