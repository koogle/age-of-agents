#!/usr/bin/env python3
"""Controlled animal attack presentation, independent of live combat timing.
Uses loopback :8013 and no saved games. Domain tests and verify_wildlife.py cover
combat authority; this checks actual WebGL UVs, freeze/resume and screenshots.
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
HOOK = '''
window.animalSprites=[];
const proto=WebGL2RenderingContext.prototype, original=proto.bufferSubData;
proto.bufferSubData=function(target,offset,data,src=0,length){
 if(data && data.buffer){
 const bytes=(length===undefined?data.length-src:length)*data.BYTES_PER_ELEMENT;
 if(bytes%72===0 && bytes>0 && bytes<100000){
 const a=new Float32Array(data.buffer,data.byteOffset+src*data.BYTES_PER_ELEMENT,bytes/4);
 const animals=[];
 for(let i=0;i<a.length;i+=18){
  if((Math.abs(a[i+3]-.95)<.001 || Math.abs(a[i+3]-1.3)<.001) &&
     Math.abs(a[i+6]-(1-590/627))<.001 && a[i+12]===1)
   animals.push(Array.from(a.slice(i,i+18)));
 }
 if(animals.length===2)window.animalSprites=animals;

 }
 }
 return original.apply(this,arguments);
};
'''

async def main(out):
    out.mkdir(parents=True, exist_ok=True)
    base=json.loads((ROOT/'docs/verification/menu-icon-fixture.json').read_text())
    base['terrain']['cells']='A'*(base['columns']*base['rows'])
    base['terrain']['heights']='g'*(base['columns']*base['rows'])
    base.update(resources=[],buildings=[],ships=[],simulation_speed=0)
    unit=base['units'][0]
    unit.update(cell={'column':30,'row':21},position={'x':30.5,'y':21.5},step=None,
                action={'type':'idle'})
    guard=copy.deepcopy(unit)
    guard.update(id='guard',kind='guard',cell={'column':31,'row':21},position={'x':31.5,'y':21.5})
    base['units']=[unit,guard]
    base['animals']=[]
    for kind,at,hp in [('wolf',{'column':31,'row':24},40),('bear',{'column':34,'row':24},100)]:
        base['animals'].append(dict(id=kind,kind=kind,home=at,cell=at,step=None,
                                    health=hp,attack_seconds=0,heading=[1,0]))
    clients=[]
    sequence=0
    async def ws(request):
        sock=web.WebSocketResponse()
        await sock.prepare(request)
        clients.append(sock)
        async for _ in sock:
            pass
        return sock
    async def file(request):
        path=request.match_info['path']
        candidate=(ROOT/('web/index.html' if path=='play' else path)).resolve()
        if not any(candidate.is_relative_to(ROOT/f) for f in ['web','assets']):
            raise web.HTTPNotFound()
        return web.FileResponse(candidate)
    async def send():
        nonlocal sequence
        sequence+=1
        base['tick']=sequence
        for sock in clients:
            if not sock.closed:
                await sock.send_json({'type':'snapshot','sequence':sequence,'world':base})
    async def feed():
        while True:
            await send()
            await asyncio.sleep(.1)
    app=web.Application()
    app.router.add_get('/ws',ws)
    app.router.add_get('/{path:.*}',file)
    runner=web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner,'127.0.0.1',8013).start()
    feeding=asyncio.create_task(feed())
    results=[]
    try:
        async with async_playwright() as p:
            browser=await p.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--enable-unsafe-swiftshader'])
            for label,w,h,dpr in [('desktop',1280,800,1),('phone',390,844,2)]:
                for animal in base['animals']:
                    animal.update(attack_seconds=0,heading=[1,0])
                page=await browser.new_page(viewport={'width':w,'height':h},device_scale_factor=dpr,has_touch=dpr==2)
                errors=[]
                page.on('pageerror',lambda e:errors.append(str(e)))
                await page.add_init_script(HOOK)
                await page.goto('http://127.0.0.1:8013/play',wait_until='networkidle')
                await page.locator('#loading').wait_for(state='detached',timeout=120000)
                async def check(column,mirror=False):
                    expected=[(column+1)/4,column/4] if mirror else [column/4,(column+1)/4]
                    await page.wait_for_function('''([u0,u1])=>window.animalSprites.length===2 &&
                        window.animalSprites.every(a=>a[7]===u0 && a[9]===u1)''',arg=expected,timeout=30000)
                    return await page.evaluate('window.animalSprites')
                await check(0)
                # Both poses are held paused; the actual uploaded atlas UVs must match.
                samples={}
                for phase,name,column in [(.3,'windup',2),(.8,'strike',3),(0,'recovery',0)]:
                    for animal in base['animals']:
                        animal['attack_seconds']=phase
                    samples[name]=await check(column)
                    if phase:
                        await page.screenshot(path=str(out/f'{label}-{name}.png'))
                        await asyncio.sleep(.4)
                        assert await check(column)==samples[name], 'Paused pose/anchor changed'
                for animal in base['animals']:
                    animal.update(attack_seconds=.8,heading=[-1,0])
                samples['mirrored_strike']=await check(3,True)
                await page.screenshot(path=str(out/f'{label}-mirrored-strike.png'))
                for animal in base['animals']:
                    animal['heading']=[1,0]
                await check(3)
                for animal in base['animals']:
                    # Zoom out to frame the subject, then reach the true camera limit.
                    await page.mouse.move(w/2,h/2)
                    for _ in range(8):
                        await page.mouse.wheel(0,10000)
                        await page.wait_for_timeout(100)
                    at=animal['cell']
                    point=await page.evaluate('''async c=>{
                        const m=await import('/web/pkg/aoa_client.js');
                        const x=(c.column+.5)*.5,z=(c.row+.5)*.5;
                        return Array.from(m.debug_screen_of(x,m.debug_height_at(x,z)+.3,z)).slice(0,2);
                    }''',at)
                    await page.mouse.move(*point)
                    await page.mouse.down()
                    await page.mouse.move(w/2+80,h/2+60,steps=5)
                    await page.mouse.move(w/2,h/2,steps=5)
                    await page.mouse.up()
                    for _ in range(8):
                        await page.mouse.wheel(0,-10000)
                        await page.wait_for_timeout(100)
                    await page.screenshot(path=str(out/f'{label}-{animal["kind"]}-strike-maxzoom.png'))
                assert not errors,errors
                results.append(dict(viewport=label,dpr=dpr,samples=samples,errors=errors))
                print(label,'windup/strike/recovery, pause and mirrored UVs passed',flush=True)
                await page.close()
            await browser.close()
    finally:
        feeding.cancel()
        await asyncio.gather(feeding,return_exceptions=True)
        await runner.cleanup()
    (out/'results.json').write_text(json.dumps(dict(
        wasm_sha256=hashlib.sha256((ROOT/'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest(),
        results=results,limitation='Controlled paused snapshot presentation; not combat authority or physical-device evidence.'),indent=2)+'\n')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    asyncio.run(main(parser.parse_args().output))
