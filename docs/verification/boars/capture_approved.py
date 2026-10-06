#!/usr/bin/env python3
"""Capture the selected boar art at normal/max zoom on desktop and DPR-2 phone.
Uses a paused loopback snapshot on :8013; no saved games or combat assertions.
"""
import argparse
import asyncio
import copy
import hashlib
import json
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[3]

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
    for kind,at,hp in [('boar',{'column':31,'row':24},60)]:
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
                await page.goto('http://127.0.0.1:8013/play',wait_until='networkidle')
                await page.locator('#loading').wait_for(state='detached',timeout=120000)
                await page.screenshot(path=str(out/f'{label}-approved-normal.png'))
                point=await page.evaluate("""async()=>{
                    const m=await import('/web/pkg/aoa_client.js');
                    return Array.from(m.debug_screen_of(15.75,m.debug_height_at(15.75,12.25)+.25,12.25)).slice(0,2);
                }""")
                await page.mouse.move(*point)
                await page.mouse.down()
                await page.mouse.move(w/2+20,h/2+20)
                await page.mouse.move(w/2,h/2)
                await page.mouse.up()
                await page.locator('canvas').evaluate("""(canvas,[x,y])=>{
                    for(let i=0;i<8;i++)canvas.dispatchEvent(new WheelEvent('wheel',{
                        clientX:x,clientY:y,deltaY:-10000,bubbles:true,cancelable:true}));
                }""", [w/2,h/2])
                await page.wait_for_timeout(1000)
                await page.screenshot(path=str(out/f'{label}-approved-maxzoom.png'))
                assert not errors, errors
                results.append(dict(viewport=label,dpr=dpr,errors=errors))
                print(label,'approved boar normal/maxzoom captured',flush=True)
                await page.close()
            await browser.close()
    finally:
        feeding.cancel()
        await asyncio.gather(feeding,return_exceptions=True)
        await runner.cleanup()
    (out/'results.json').write_text(json.dumps(dict(
        wasm_sha256=hashlib.sha256((ROOT/'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest(),
        results=results,limitation='Paused art preview; synthetic wheel input for zoom. Not combat authority or physical-device evidence.'),indent=2)+'\n')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    asyncio.run(main(parser.parse_args().output))
