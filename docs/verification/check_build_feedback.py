#!/usr/bin/env python3
"""Loopback presentation checks for build details and transient NPC complaints."""
import argparse, asyncio, copy, hashlib, json
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--mode', choices=['both','desktop','phone'], default='both')
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
base = json.loads((ROOT / 'docs/verification/menu-icon-fixture.json').read_text())
base.update(animals=[], resources=[], buildings=[], ships=[], ship_connections=[], simulation_speed=1)
base['terrain'].update(cells='A' * 9600, heights='g' * 9600)
base['units'] = base['units'][:2]
for i, u in enumerate(base['units']):
    u.update(cell={'column':30+i*5,'row':21}, position={'x':30.0+i*5,'y':21.0},
             step=None, action={'type':'idle'}, health=100.0)
for stocks in [base['inventories'],base['stored_inventories']]:
    for stock in stocks:
        for k in stock: stock[k]=500.0
clients=[]
commands=[]
sequence=0
state=copy.deepcopy(base)

async def ws(req):
    sock=web.WebSocketResponse()
    await sock.prepare(req)
    clients.append(sock)
    async for msg in sock:
        commands.append(json.loads(msg.data))
    return sock

async def file(req):
    path=req.match_info['path']
    candidate=(ROOT / ('web/index.html' if path in ('','play') else path)).resolve()
    if not any(candidate.is_relative_to(ROOT / x) for x in ['assets','web']):
        raise web.HTTPNotFound()
    return web.FileResponse(candidate)

async def send():
    global sequence
    sequence+=1
    state['tick']=sequence
    for sock in clients:
        if not sock.closed:
            await sock.send_json({'type':'snapshot','sequence':sequence,'world':state})

HOOK="""window.requestAnimationFrame=cb=>setTimeout(()=>cb(performance.now()),500);window.cancelAnimationFrame=id=>clearTimeout(id);window.hudFrame=0;window.hudQuads=[];const p=WebGL2RenderingContext.prototype,o=p.bufferSubData;p.bufferSubData=function(t,off,data,start=0,length){if(data&&data.buffer){const n=(length===undefined?data.length-start:length)*data.BYTES_PER_ELEMENT;if(n%64===0&&n>512&&n<300000){const a=new Float32Array(data.buffer,data.byteOffset+start*data.BYTES_PER_ELEMENT,n/4);let q=[];for(let i=0;i<a.length;i+=16)q.push(Array.from(a.slice(i,i+16)));if(q.some(v=>v[12]===3)){window.hudQuads=q;window.hudFrame++;}}}return o.apply(this,arguments);};"""
COINS="""window.hudQuads.filter(q=>Math.abs(q[2]/devicePixelRatio-(innerWidth<600?44:52))<.1&&Math.abs(q[3]-q[2])<.1&&q[1]/devicePixelRatio>innerHeight/2).map(q=>q.slice(0,4).map(v=>v/devicePixelRatio)).sort((a,b)=>Math.abs(a[1]-b[1])>8?a[1]-b[1]:a[0]-b[0])"""
STATUS="""window.hudQuads.filter(q=>q[12]===4&&Math.abs(q[8]-246/255)<.001&&Math.abs(q[9]-240/255)<.001)"""

async def main():
    app=web.Application()
    app.router.add_get('/ws',ws)
    app.router.add_get('/{path:.*}',file)
    runner=web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner,'127.0.0.1',8013).start()
    results=[]
    async with async_playwright() as p:
        browser=await p.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--enable-unsafe-swiftshader'])
        try:
            for mode,w,h,dpr in [('desktop',1280,800,1),('phone',390,844,2)]:
                if args.mode not in ('both', mode):
                    continue
                state.update(copy.deepcopy(base))
                commands.clear()
                page=await browser.new_page(viewport={'width':w,'height':h},device_scale_factor=dpr,is_mobile=mode=='phone',has_touch=mode=='phone')
                errors=[]
                page.on('pageerror',lambda e:errors.append(str(e)))
                await page.add_init_script(HOOK)
                connections = len(clients)
                await page.goto('http://127.0.0.1:8013/play',wait_until='networkidle')
                async with asyncio.timeout(120):
                    while len(clients) <= connections:
                        await asyncio.sleep(.1)
                print(mode, 'connected', flush=True)
                for _ in range(15):
                    await send()
                    await asyncio.sleep(.1)
                await page.locator('#loading').wait_for(state='detached',timeout=120000)
                print(mode, 'loaded', flush=True)
                # Control the game's presentation clock without asking Playwright to
                # replay every expensive software-rendered animation frame.
                await page.evaluate("""() => {
                    window.feedbackTime=performance.now();
                    performance.now=()=>window.feedbackTime;
                }""")
                async def advance(ms=200):
                    while ms > 0:
                        step=min(ms,200)
                        frame=await page.evaluate("step=>{window.feedbackTime+=step;return window.hudFrame}",step)
                        await page.wait_for_function("frame=>window.hudFrame>frame",arg=frame,timeout=30000)
                        ms-=step
                tap=page.touchscreen.tap if mode=='phone' else page.mouse.click
                async def project(column,row):
                    return await page.evaluate('''async ([c,r])=>{const m=await import('/web/pkg/aoa_client.js');const x=c*.5,z=r*.5;return Array.from(m.debug_screen_of(x,m.debug_height_at(x,z),z));}''',[column,row])
                async def coin(index):
                    q=await page.evaluate(COINS)
                    assert q, ('no coins',mode)
                    x,y,cw,ch=q[index]
                    await tap(x+cw/2,y+ch/2)
                    await advance()
                    print(mode, 'coin', index, flush=True)
                async def capture(name):
                    await page.screenshot(path=str(args.output / f'{mode}-{name}.png'))
                    assert not errors,errors
                    print(mode, 'captured', name, flush=True)
                point=await project(30,21)
                await tap(point[0],point[1]-15)
                await advance()
                await coin(0) # Build
                await coin(4) # Roads
                if mode=='desktop':
                    x,y,cw,ch=(await page.evaluate(COINS))[1]
                    await page.mouse.move(x+cw/2,y+ch/2)
                    await advance()
                    await capture('road-hover')
                await coin(1) # Stone road
                await page.mouse.move(w / 2,h / 3)
                await advance()
                await capture('road-placement')
                # No centered top banner. Only resource tabs and fixed controls remain above the world.
                top=await page.evaluate('''window.hudQuads.filter(q=>q[12]===1&&q[0]/devicePixelRatio>100&&q[2]/devicePixelRatio>200&&q[1]/devicePixelRatio<innerHeight/3)''')
                assert not top,(mode,'unexpected top banner',top)
                await page.keyboard.press('Escape')
                await advance()
                await coin(0)
                await coin(0) # Town
                await coin(1) # House
                # Footprint overlaps its builder, so local preview rejects it.
                point=await project(30,21)
                await tap(point[0],point[1])
                await advance(300)
                status=await page.evaluate(STATUS)
                assert status,(mode,'missing blocked-site feedback')
                await capture('blocked-building')
                await advance(1600)
                assert not await page.evaluate(STATUS),(mode,'feedback did not fade')
                await page.keyboard.press('Escape')
                await advance()
                # Delayed rejection belongs to the original unit after selecting the second.
                point=await project(28,25)
                await tap(point[0],point[1])
                await advance()
                async with asyncio.timeout(5):
                    while not commands: await asyncio.sleep(.05)
                command=commands[-1]
                assert command['command']['type'] in ('move','group_move'),command
                point2=await project(35,21)
                await tap(point2[0],point2[1]-15)
                await advance()
                for sock in clients:
                    if not sock.closed:
                        await sock.send_json({'type':'command_result','request_id':command['request_id'],'ok':False,'error':'destination cell is occupied'})
                await asyncio.sleep(.1)
                await advance(300)
                status=await page.evaluate(STATUS)
                assert status,(mode,'missing rejection')
                point1=await project(30,21)
                center=(min(q[0] for q in status)+max(q[0]+q[2] for q in status))/2/dpr
                assert abs(center-point1[0])<abs(center-point2[0]),(mode,center,point1,point2)
                await capture('server-rejection')
                await advance(1600)
                assert not await page.evaluate(STATUS)
                results.append({'mode':mode,'viewport':[w,h],'dpr':dpr,'errors':errors,'checks':['menu hint','placement detail','no top banner','blocked building overhead','fade expiry','delayed rejection original unit']})
                print(mode,'PASS',flush=True)
                await page.close()
        finally:
            await browser.close()
    await runner.cleanup()
    (args.output / f'results-{args.mode}.json').write_text(json.dumps({'wasm_sha256':hashlib.sha256((ROOT/'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest(),'results':results,'scope':'Controlled WebSocket presentation; not production or physical-device verification.'},indent=2)+'\n')

asyncio.run(main())
