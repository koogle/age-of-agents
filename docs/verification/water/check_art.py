#!/usr/bin/env python3
"""Presentation-only desktop/phone art review; no gameplay replay assertions.
Run cargo build -p aoa-game --example water_fixture --profile test and scripts/build_web.sh first.
No production connections or persisted game are used. Requires aiohttp/Playwright.
"""
import asyncio, json
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
            # Presentation fixture: expose the normally hidden zero-balance HUD pill.
            state['inventories'][0]['water'] = 20
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
                    window.requestAnimationFrame = cb => setTimeout(() => cb(performance.now()), 1000);
                    const Socket=window.WebSocket;
                    window.WebSocket=class extends Socket { constructor(...args) { super(...args);
                        this.addEventListener('message',event=>{const m=JSON.parse(event.data);
                            if(m.type==='snapshot') window.waterReceivedTick=m.world.tick;});
                    }};
                ''')
                await page.goto('http://127.0.0.1:8001/play',wait_until='networkidle')
                await page.locator('#loading').wait_for(state='detached',timeout=90000)
                await asyncio.sleep(2)
                if dpr == 2:
                    # Real touch pan: the riverbank starts left of a phone's view.
                    cdp = await page.context.new_cdp_session(page)
                    await cdp.send('Input.dispatchTouchEvent',dict(type='touchStart',touchPoints=[dict(x=100,y=450)]))
                    for x in [130,160,190,220,250]:
                        await cdp.send('Input.dispatchTouchEvent',dict(type='touchMove',touchPoints=[dict(x=x,y=450)]))
                    await cdp.send('Input.dispatchTouchEvent',dict(type='touchEnd',touchPoints=[]))
                    await asyncio.sleep(2)

                async def point(column,row,lift=0):
                    return await page.evaluate('''async ([c,r,l])=>{const m=await import('/web/pkg/aoa_client.js');let x=c*.5,z=r*.5;return Array.from(m.debug_screen_of(x,m.debug_height_at(x,z)+l,z));}''',[column,row,lift])

                await page.screenshot(path=str(OUT/f'{name}-art-riverbank.png'))
                xy=await point(25.5,22.5,.25)
                await page.mouse.move(*xy[:2])
                await page.mouse.wheel(0,-10000)
                await asyncio.sleep(2)
                await page.screenshot(path=str(OUT/f'{name}-art-maximum-zoom.png'))
                results.append(dict(platform=name, viewport=[w,h], dpr=dpr, scope="presentation only"))
            finally:
                if page is not None: await page.close()
                if process.returncode is None: process.terminate()
                await process.wait()
            print(name,'passed',flush=True)
        await browser.close()
    await runner.cleanup()
    assert not errors,errors
    (OUT/'art-results.json').write_text(json.dumps(dict(results=results,browser_errors=errors),indent=2)+'\n')

if __name__=='__main__': asyncio.run(main())
