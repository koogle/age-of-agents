#!/usr/bin/env python3
"""Real WebGL mouse/touch input backed by aoa-game, with accelerated fixed ticks.
Run cargo build -p aoa-game --example water_fixture --profile test and scripts/build_web.sh first.
No production connections or persisted game are used. Requires aiohttp/Playwright.
"""
import asyncio, json, math
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent

async def main():
    process = None
    sockets, commands, errors = [], [], []
    lock = asyncio.Lock()
    state, sequence = None, 0

    async def advance(value, publish=True):
        nonlocal state, sequence
        async with lock:
            process.stdin.write((json.dumps(value)+'\n').encode())
            await process.stdin.drain()
            line = await process.stdout.readline()
            assert line, 'domain fixture stopped'
            result = json.loads(line)
            state = result['world']
            sequence += 1
            if publish:
                for sock in sockets:
                    await sock.send_json(dict(type='snapshot', sequence=sequence, world=state))
            return result

    async def ws(req):
        sock = web.WebSocketResponse()
        await sock.prepare(req)
        sockets.append(sock)
        await advance({'ticks': 1})
        async for msg in sock:
            if msg.type == web.WSMsgType.TEXT:
                value = json.loads(msg.data)
                if value['type'] == 'command':
                    result = await advance({'command': value['command']})
                    commands.append(dict(command=value['command'], error=result['error']))
                    await sock.send_json(dict(type='command_result',request_id=value['request_id'],ok=result['error'] is None,error=result['error'],applied_sequence=sequence))
        sockets.remove(sock)
        return sock

    async def file(req):
        path = req.match_info['path']
        candidate = (ROOT / ('web/index.html' if path in ('','play') else path)).resolve()
        if not any(candidate.is_relative_to(ROOT / folder) for folder in ['web','assets']):
            raise web.HTTPNotFound()
        return web.FileResponse(candidate)

    app = web.Application()
    app.router.add_get('/ws', ws)
    app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner,'127.0.0.1',8001).start()
    results=[]
    async with async_playwright() as pw:
        browser = await pw.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--enable-unsafe-swiftshader'])
        for name,w,h,dpr in [('desktop',1100,750,1),('phone',430,932,2)]:
            process = await asyncio.create_subprocess_exec(str(ROOT/'target/debug/examples/water_fixture'),'--serve',stdin=asyncio.subprocess.PIPE,stdout=asyncio.subprocess.PIPE)
            page = None
            try:
                commands.clear()
                page = await browser.new_page(viewport={'width':w,'height':h},device_scale_factor=dpr,has_touch=dpr==2)
                page.on('pageerror',lambda e: errors.append(str(e)))
                await page.add_init_script('''
                    window.requestAnimationFrame = cb => setTimeout(() => cb(performance.now()), 100);
                    const Socket=window.WebSocket;
                    window.WebSocket=class extends Socket { constructor(...args) { super(...args);
                        this.addEventListener('message',event=>{const m=JSON.parse(event.data);
                            if(m.type==='snapshot') window.waterReceivedTick=m.world.tick;});
                    }};
                ''')
                await page.goto('http://127.0.0.1:8001/play',wait_until='networkidle')
                await page.locator('#loading').wait_for(state='detached',timeout=90000)
                await asyncio.sleep(.5)
                if dpr == 2:
                    # Real touch pan: the riverbank starts left of a phone's view.
                    cdp = await page.context.new_cdp_session(page)
                    await cdp.send('Input.dispatchTouchEvent',dict(type='touchStart',touchPoints=[dict(x=100,y=450)]))
                    for x in [130,160,190,220,250]:
                        await cdp.send('Input.dispatchTouchEvent',dict(type='touchMove',touchPoints=[dict(x=x,y=450)]))
                    await cdp.send('Input.dispatchTouchEvent',dict(type='touchEnd',touchPoints=[]))
                    await asyncio.sleep(.5)

                async def tap(x,y):
                    if dpr==2: await page.touchscreen.tap(x,y)
                    else:
                        await page.mouse.click(x,y)
                        await page.mouse.move(w/2,h/2)
                    await asyncio.sleep(.35)

                async def point(column,row,lift=0):
                    return await page.evaluate('''async ([c,r,l])=>{const m=await import('/web/pkg/aoa_client.js');let x=c*.5,z=r*.5;return Array.from(m.debug_screen_of(x,m.debug_height_at(x,z)+l,z));}''',[column,row,lift])

                async def world_tap(column,row,lift=0):
                    xy=await point(column,row,lift)
                    assert 0<xy[0]<w and 90<xy[1]<h-120, (name,'target offscreen',xy)
                    await tap(*xy[:2])

                async def button(index,count,labels=False):
                    narrow=w<600 or h<500
                    gap=8 if narrow else 10; pad=8 if narrow else 14; edge=12 if narrow else 18
                    gx=w-edge-(80 if narrow else 136);gy=h-edge-(80 if narrow else 136)
                    if narrow:
                        size=44;room=(gx-28-24)-2*pad+gap;per=max(1,math.floor(room/(size+gap)))
                    else: size=max(40,min(52,(w-52+gap)/count-gap));per=count
                    step=size+gap+(26 if labels else 0);rows=math.ceil(count/per)
                    bw=min(count,per)*(size+gap)-gap+2*pad;bh=rows*step-gap+12
                    left=12 if narrow else (w-bw)/2
                    top=h-edge-bh if narrow or left+bw<=gx-8 else gy-48-bh
                    await tap(left+pad+(index%per)*(size+gap)+size/2,top+6+(index//per)*step+size/2)

                async def until(predicate,limit=100):
                    for _ in range(limit):
                        if predicate():
                            await advance({'ticks':0})
                            await page.wait_for_function('(tick)=>window.waterReceivedTick>=tick',arg=state['tick'])
                            await page.evaluate('async()=>{for(let i=0;i<3;i++) await new Promise(requestAnimationFrame)}')
                            return
                        await advance({'ticks':20},publish=False)
                        await asyncio.sleep(.02)
                    raise AssertionError((name,'condition timeout',state['units'],state['inventories']))

                await page.screenshot(path=str(OUT/f'{name}-riverbank.png'))
                await world_tap(26.5,22.5,.45)
                selected=await point(26.5,22.5)
                assert selected[2]==1,(name,'unit selection',selected)
                await world_tap(25.5,22.5,.25)
                assert commands[-1]['command']['type']=='gather',commands
                await advance({'ticks':20})
                await asyncio.sleep(.5)
                await page.screenshot(path=str(OUT/f'{name}-collecting.png'))
                await until(lambda:state['inventories'][0]['water']>=20)
                # Stop through the existing medallion; allow a reserved step to finish.
                await asyncio.sleep(.4)
                await button(1,2)
                assert commands[-1]['command']['type']=='stop',commands
                await advance({'ticks':20})
                await asyncio.sleep(.4)
                def owned_water():
                    return state['inventories'][0]['water'] + sum(u.get('cargo',{}).get('amount',0) for u in state['units'] if (u.get('cargo') or {}).get('kind')=='water')
                water=owned_water()
                assert water>=20
                await page.screenshot(path=str(OUT/f'{name}-water-stored.png'))
                await button(0,1) # Build
                await button(1,5,True) # Gathering
                await button(2,4,True) # Field
                await page.screenshot(path=str(OUT/f'{name}-field-cost.png'))
                await world_tap(30.5,25.5)
                assert commands[-1]['command']['type']=='plant_field',commands
                assert commands[-1]['error'] is None,commands
                assert abs(owned_water()-(water-10))<1e-6
                await until(lambda:any(r.get('field') is not None and r['amount']>0 for r in state['resources']))
                await asyncio.sleep(.5)
                await page.screenshot(path=str(OUT/f'{name}-field.png'))
                await until(lambda:state['inventories'][0]['food']>=120,200)
                field=next(r for r in state['resources'] if r.get('field') is not None)
                assert field['amount']==0 and field['field']['work'] is None
                # The selected harvester stays selected after finishing its explicit order.
                await asyncio.sleep(.5)
                await world_tap(field['cell']['column']+1.5,field['cell']['row']+1.5,.05)
                assert commands[-1]['command']['type']=='cultivate',commands
                assert commands[-1]['error'] is None,commands
                assert abs(owned_water()-(water-20))<1e-6
                await until(lambda:state['inventories'][0]['food']>=240,200)
                assert next(r for r in state['resources'] if r['kind']=='water')['amount']==120
                assert all(c['error'] is None for c in commands),commands
                results.append(dict(platform=name,viewport=[w,h],dpr=dpr,commands=list(commands),water_collected=water,water_after_two_fields=state['inventories'][0]['water'],food=state['inventories'][0]['food']))
                xy=await point(25.5,22.5,.25)
                await page.mouse.move(*xy[:2])
                await page.mouse.wheel(0,-10000)
                await asyncio.sleep(.5)
                await page.screenshot(path=str(OUT/f'{name}-maximum-zoom.png'))
            finally:
                if page is not None: await page.close()
                if process.returncode is None: process.terminate()
                await process.wait()
            print(name,'passed',flush=True)
        await browser.close()
    await runner.cleanup()
    assert not errors,errors
    (OUT/'results.json').write_text(json.dumps(dict(results=results,browser_errors=errors),indent=2)+'\n')

if __name__=='__main__': asyncio.run(main())
