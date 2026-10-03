#!/usr/bin/env python3
"""Replay a presentation-only zigzag fixture through the checked-in WebGL client.

Requires Python aiohttp and playwright, and Chromium. Serves only this checkout's
web/assets on loopback :8001; never connects to production or reads a saved game.
The paused Playwright clock isolates animation timing from slow software GPUs.
WebGL upload observations supplement, but do not replace, screenshot inspection.
"""
import argparse
import asyncio
import hashlib
import json
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright
ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--chromium', default='/usr/bin/chromium')
args = parser.parse_args()
OUT = args.output.resolve()
OUT.mkdir(parents=True, exist_ok=True)
state = json.loads((Path(__file__).parent / 'presentation-fixture.json').read_text())
u = state['units'][0]
u['position'] = {'x': 30.0, 'y': 21.0}
u['cell'] = {'column': 30, 'row': 21}
clients = []

async def ws(request):
    sock = web.WebSocketResponse()
    await sock.prepare(request)
    clients.append(sock)
    async for msg in sock:
        pass
    return sock

async def file(request):
    path = request.match_info['path']
    path = 'web/index.html' if path == 'play' else path
    candidate = (ROOT / path).resolve()
    if not any((candidate.is_relative_to(ROOT / folder) for folder in ['web', 'assets'])):
        raise web.HTTPNotFound()
    return web.FileResponse(candidate)
# Sprite is 18 f32 values; one villager means one 72-byte instance upload.
# Read the rendered UV/anchor, without adding an application debug API.
HOOK = """
window.spriteSamples=[];
const proto=WebGL2RenderingContext.prototype, original=proto.bufferSubData;
proto.bufferSubData=function(target,offset,data,src=0,length){
 if(data && data.buffer){
 const n=length===undefined?data.length-src:length;
 const bytes=n*data.BYTES_PER_ELEMENT;
 if(bytes===72){
 const a=Array.from(new Float32Array(data.buffer,data.byteOffset+src*data.BYTES_PER_ELEMENT,18));
 if(a[3]>0.5 && a[3]<5 && a[12]===1 && a[16]===0 && a[17]===0)
 window.spriteSamples.push({t:performance.now(),a});
 }
 }
 return original.apply(this,arguments);
};
"""

async def main():
    app = web.Application()
    app.router.add_get('/ws', ws)
    app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner, '127.0.0.1', 8001).start()
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path=args.chromium, args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        page = await b.new_page(viewport={'width': 480, 'height': 360})
        errors = []
        page.on('pageerror', lambda e: errors.append(str(e)))
        await page.add_init_script(HOOK)
        await page.goto('http://127.0.0.1:8001/play', wait_until='networkidle')
        async with asyncio.timeout(60):
            while not clients:
                await asyncio.sleep(0.1)
        await page.clock.install()
        await page.clock.pause_at(await page.evaluate('Date.now() + 100'))
        tick = 0

        async def send(x, y, cargo=None):
            nonlocal tick
            tick += 1
            state['tick'] = tick
            u['position'] = {'x': x, 'y': y}
            u['cargo'] = cargo
            await clients[0].send_json({'type': 'snapshot', 'sequence': tick, 'world': state})
        for _ in range(30):
            await send(30.0, 21.0)
            await page.clock.run_for(100)
        await page.locator('#loading').wait_for(state='detached', timeout=60000)
        await page.mouse.move(240, 180)
        await page.mouse.wheel(0, -1200)
        await page.clock.run_for(500)
        phases = []

        async def phase(name, steps, dx=0, dy=0, cargo=None):
            nonlocal x, y
            start = await page.evaluate('performance.now()')
            for i in range(steps):
                x += dx if not isinstance(dx, list) else dx[i % len(dx)]
                y += dy if not isinstance(dy, list) else dy[i % len(dy)]
                await send(x, y, cargo)
                await asyncio.sleep(0.01)
                await page.clock.run_for(100)
                if i in [steps // 3, 2 * steps // 3, steps - 1]:
                    await page.screenshot(path=str(OUT / f'{name}-{i:03}.png'))
            phases.append({'name': name, 'start': start, 'end': await page.evaluate('performance.now()')})
        x, y = (30.0, 21.0)
        await phase('zigzag', 48, [0.1, 0], [0, 0.1])
        await phase('brief-stop', 1)
        await phase('resume', 20, [0.1, 0], [0, 0.1])
        await phase('idle', 20)
        await phase('reverse', 40, [-0.1, 0], [0, -0.1])
        await phase('carry-walk', 20, -0.1, 0, {'kind': 'wood', 'amount': 7})
        await phase('carry-stop', 20, 0, 0, {'kind': 'wood', 'amount': 7})
        samples = await page.evaluate('window.spriteSamples')
        (OUT / 'zigzag-samples.json').write_text(json.dumps({'wasm_sha256': hashlib.sha256((ROOT / 'web/pkg/aoa_client_bg.wasm').read_bytes()).hexdigest(), 'chromium': b.version, 'phases': phases, 'samples': samples, 'errors': errors}))
        print(json.dumps({'samples': len(samples), 'errors': errors}))
        assert samples and (not errors)
        await b.close()
    await runner.cleanup()
if __name__ == '__main__':
    asyncio.run(main())
