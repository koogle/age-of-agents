"""Usage: python3 capture_sequence.py OUT_DIR FIRST.json [NEXT.json...]

Serve a snapshot sequence: the first until the page loads, then each later one, then capture."""
import asyncio, json, sys
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright
ROOT = Path(__file__).resolve().parents[3]
OUT = Path(sys.argv[1]); OUT.mkdir(parents=True, exist_ok=True)
STATES = [json.loads(Path(s).read_text()) for s in sys.argv[2:]]
clients = []
async def ws(request):
    sock = web.WebSocketResponse(); await sock.prepare(request); clients.append(sock)
    async for _ in sock: pass
    return sock
async def file(request):
    path = request.match_info['path']
    path = 'web/index.html' if path in ('play', '') else path
    c = (ROOT / path).resolve()
    if not any(c.is_relative_to(ROOT / f) for f in ['web', 'assets']): raise web.HTTPNotFound()
    return web.FileResponse(c)
async def main():
    app = web.Application(); app.router.add_get('/ws', ws); app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app); await runner.setup(); await web.TCPSite(runner, '127.0.0.1', 8018).start()
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path='/opt/pw-browsers/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        for name, vp, dpr, mobile in [('desktop', {'width': 1280, 'height': 800}, 1, False), ('phone', {'width': 390, 'height': 844}, 2, True)]:
            clients.clear(); tick = 0
            ctx = await b.new_context(viewport=vp, device_scale_factor=dpr, is_mobile=mobile, has_touch=mobile)
            page = await ctx.new_page(); errors = []
            page.on('pageerror', lambda e: errors.append(str(e)))
            await page.goto('http://127.0.0.1:8018/play', wait_until='networkidle')
            async with asyncio.timeout(120):
                while not clients: await asyncio.sleep(0.1)
            async def send(state, n):
                nonlocal tick
                for _ in range(n):
                    tick += 1; state['tick'] = tick
                    await clients[0].send_json({'type': 'snapshot', 'sequence': tick, 'world': state})
                    await asyncio.sleep(0.1)
            await send(STATES[0], 30)
            await page.locator('#loading').wait_for(state='detached', timeout=240000)
            # Software rendering is slow: freeze page time and step it, as
            # docs/verification/replay_status.py does, so playback never resyncs.
            await page.clock.install()
            await page.clock.pause_at(await page.evaluate('Date.now() + 100'))
            for state in STATES[1:]:
                await send(state, 1)
                await page.clock.run_for(400)
            await asyncio.sleep(0.5)
            await page.screenshot(path=str(OUT / f'sequence-{name}.png'))
            print(name, 'errors:', errors)
            await ctx.close()
        await b.close()
    await runner.cleanup()
asyncio.run(main())
