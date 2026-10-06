#!/usr/bin/env python3
"""Controlled health-bar presentation on villagers, military and all three animals.
Uses loopback :8018 and no saved games. Domain tests and verify_wildlife.py cover
combat authority; this checks uploaded health-bar widths/colors and screenshots.
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
HOOK = """
window.requestAnimationFrame=cb=>setTimeout(()=>cb(performance.now()),100);
window.cancelAnimationFrame=id=>clearTimeout(id);
window.healthBars=[];
const p=WebGL2RenderingContext.prototype,o=p.bufferSubData;
p.bufferSubData=function(t,off,data,start=0,length){
 if(data&&data.buffer){
 const n=(length===undefined?data.length-start:length)*data.BYTES_PER_ELEMENT;
 if(n%64===0&&n>512&&n<300000){
 const a=new Float32Array(data.buffer,data.byteOffset+start*data.BYTES_PER_ELEMENT,n/4);
 let q=[];for(let i=0;i<a.length;i+=16)q.push(Array.from(a.slice(i,i+16)));
 if(q.some(v=>v[12]===3))window.healthBars=q.filter(v=>v[12]===1&&Math.abs(v[13]-devicePixelRatio)<.01&&Math.abs(v[3]-3*devicePixelRatio)<.01);
 }}return o.apply(this,arguments);};
"""

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
    for kind,at,hp in [('wolf',{'column':31,'row':24},300),('bear',{'column':34,'row':24},600),('boar',{'column':28,'row':24},60)]:
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
            await asyncio.sleep(1)
    app=web.Application()
    app.router.add_get('/ws',ws)
    app.router.add_get('/{path:.*}',file)
    runner=web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner,'127.0.0.1',8018).start()
    feeding=asyncio.create_task(feed())
    results=[]
    try:
        async with async_playwright() as p:
            browser=await p.chromium.launch(args=['--no-sandbox','--enable-unsafe-swiftshader'])
            for label,w,h,dpr in [('desktop',1280,800,1),('phone',390,844,2)]:
                for u in base['units']:
                    u['health']=100
                for animal in base['animals']:
                    animal.update(attack_seconds=0,heading=[1,0])
                page=await browser.new_page(viewport={'width':w,'height':h},device_scale_factor=dpr,has_touch=dpr==2)
                errors=[]
                page.on('pageerror',lambda e:errors.append(str(e)))
                await page.add_init_script(HOOK)
                await page.goto('http://127.0.0.1:8018/play',wait_until='networkidle')
                await page.locator('#loading').wait_for(state='detached',timeout=120000)
                samples=[]
                for fraction,name in [(1,'green'),(.5,'orange'),(.25,'red')]:
                    for u in base['units']:
                        u['health']=100*fraction
                    for animal in base['animals']:
                        animal['health']={'wolf':300,'bear':600,'boar':60}[animal['kind']]*fraction
                    await page.wait_for_function("""f=>window.healthBars.length===5 && window.healthBars.every(q=>Math.abs(q[2]-24*f*devicePixelRatio)<.01)""",arg=fraction,timeout=120000)
                    print(label,name,'geometry passed',flush=True)
                    bars=await page.evaluate('window.healthBars')
                    expected={1:(.27,.75,.30),.5:(.95,.55,.12),.25:(.88,.20,.15)}[fraction]
                    assert all(all(abs(q[8+i]-v)<.001 for i,v in enumerate(expected)) for q in bars)
                    await page.screenshot(path=str(out/f'{label}-{name}.png'))
                    samples.append(dict(fraction=fraction,bars=bars))
                assert not errors,errors
                results.append(dict(viewport=label,dpr=dpr,samples=samples,errors=errors))
                print(label,'health widths, colors and screenshots passed',flush=True)
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
