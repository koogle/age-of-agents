#!/usr/bin/env python3
"""Check stationary, rising/fading NPC status labels in desktop/phone WebGL.
Uses loopback snapshots and a paused clock; no saves or production commands.
"""
import argparse
import asyncio
import hashlib
import json
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
P = argparse.ArgumentParser(description=__doc__)
P.add_argument('--output', type=Path, required=True)
args = P.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
clients = []
HOOK = """// Throttle initial software rendering before the paused clock takes over.
window.requestAnimationFrame=cb=>setTimeout(()=>cb(performance.now()),100);
window.cancelAnimationFrame=id=>clearTimeout(id);
window.statusQuads=[];
const p=WebGL2RenderingContext.prototype,o=p.bufferSubData;
p.bufferSubData=function(t,off,data,start=0,length){
 if(data&&data.buffer){
 const n=(length===undefined?data.length-start:length)*data.BYTES_PER_ELEMENT;
 if(n%64===0&&n>512&&n<300000){
 const a=new Float32Array(data.buffer,data.byteOffset+start*data.BYTES_PER_ELEMENT,n/4);
 let q=[];for(let i=0;i<a.length;i+=16)q.push(Array.from(a.slice(i,i+16)));
 if(q.some(v=>v[12]===3))window.statusQuads=q.filter(v=>v[12]===4&&Math.abs(v[8]-246/255)<0.001&&Math.abs(v[9]-240/255)<0.001);
 }}return o.apply(this,arguments);};"""

async def ws(req):
    sock = web.WebSocketResponse()
    await sock.prepare(req)
    clients.append(sock)
    async for _ in sock:
        pass
    return sock

async def file(req):
    path = req.match_info['path']
    candidate = (ROOT / ('web/index.html' if path == 'play' else path)).resolve()
    if not any(candidate.is_relative_to(ROOT / x) for x in ['web', 'assets']):
        raise web.HTTPNotFound()
    return web.FileResponse(candidate)

async def main():
    app = web.Application()
    app.router.add_get('/ws', ws)
    app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner, '127.0.0.1', 8017).start()
    results = []
    async with async_playwright() as p:
        browser = await p.chromium.launch(executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        for mode, w, h, dpr in [('desktop', 1280, 800, 1), ('phone', 390, 844, 2)]:
            state = json.loads(Path(__file__).with_name('presentation-fixture.json').read_text())
            unit = state['units'][0]
            state['simulation_speed'] = 1
            unit['action'] = {'type': 'idle'}
            page = await browser.new_page(viewport={'width': w, 'height': h}, device_scale_factor=dpr, is_mobile=mode == 'phone', has_touch=mode == 'phone')
            errors = []
            page.on('pageerror', lambda e: errors.append(str(e)))
            await page.add_init_script(HOOK)
            await page.goto('http://127.0.0.1:8017/play', wait_until='networkidle')
            async with asyncio.timeout(120):
                while not any(not s.closed for s in clients):
                    await asyncio.sleep(0.1)
            seq = 0
            async def send():
                nonlocal seq
                seq += 1
                state['tick'] = seq
                for sock in clients:
                    if not sock.closed:
                        await sock.send_json({'type': 'snapshot', 'sequence': seq, 'world': state})
            for _ in range(15):
                await send()
                await asyncio.sleep(0.1)
            await page.locator('#loading').wait_for(state='detached', timeout=120000)
            await page.clock.install()
            await page.clock.pause_at(await page.evaluate('Date.now() + 100'))
            unit['action'] = {'type': 'build', 'building_id': 'base-1'}
            await send()
            await asyncio.sleep(0.1)
            await page.clock.run_for(300)
            async def capture(name):
                quads = await page.evaluate('window.statusQuads')
                await page.screenshot(path=str(args.output / f'{mode}-{name}.png'))
                return {'name': name, 'quads': quads}
            start = await capture('start')
            assert start['quads'], (mode, 'missing status')
            unit['position']['x'] += 4
            unit['position']['y'] += 2
            await send()
            await asyncio.sleep(0.1)
            await page.clock.run_for(800)
            moved = await capture('moved')
            assert moved['quads'], (mode, 'missing fading status')
            a, b = start['quads'][0], moved['quads'][0]
            assert abs(a[0] - b[0]) < 0.2, (mode, a[0], b[0])
            assert b[1] < a[1], (mode, 'did not rise')
            assert b[11] < a[11], (mode, 'did not fade')
            await page.clock.run_for(600)
            expired = await capture('expired')
            assert not expired['quads'], (mode, 'did not expire')
            assert not errors, errors
            results.append({'mode': mode, 'viewport': [w, h], 'dpr': dpr, 'samples': [start, moved, expired], 'errors': errors})
            print(mode, 'stationary anchor, rise, fade, expiry pass', flush=True)
            await page.context.close()
        await browser.close()
    await runner.cleanup()
    (args.output / 'results.json').write_text(json.dumps({'wasm_sha256': hashlib.sha256((ROOT / 'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest(), 'results': results}, indent=2))

if __name__ == '__main__':
    asyncio.run(main())
