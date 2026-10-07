"""Serve exported snapshots to the checked-in WebGL client and capture desktop/phone views."""
import asyncio, json, sys
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright
ROOT = Path('/home/user/age-of-agents')
# Usage: python3 capture.py OUT_DIR SNAPSHOT.json... (snapshots from the archipelago_preview example)
OUT = Path(sys.argv[1]); OUT.mkdir(parents=True, exist_ok=True)
SNAPS = sys.argv[2:]
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
    runner = web.AppRunner(app); await runner.setup(); await web.TCPSite(runner, '127.0.0.1', 8017).start()
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path='/opt/pw-browsers/chromium' if Path('/opt/pw-browsers/chromium').exists() else None, args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        for snap in SNAPS:
            state = json.loads(Path(snap).read_text())
            for name, vp, dpr, mobile in [('desktop', {'width': 1280, 'height': 800}, 1, False), ('phone', {'width': 390, 'height': 844}, 2, True)]:
                clients.clear()
                ctx = await b.new_context(viewport=vp, device_scale_factor=dpr, is_mobile=mobile, has_touch=mobile)
                page = await ctx.new_page(); errors = []
                page.on('pageerror', lambda e: errors.append(str(e)))
                await page.goto('http://127.0.0.1:8017/play', wait_until='networkidle')
                async with asyncio.timeout(120):
                    while not clients: await asyncio.sleep(0.1)
                for t in range(40):
                    state['tick'] = t + 1
                    await clients[0].send_json({'type': 'snapshot', 'sequence': t + 1, 'world': state})
                    await asyncio.sleep(0.05)
                await page.locator('#loading').wait_for(state='detached', timeout=180000)
                await asyncio.sleep(3)
                tag = Path(snap).stem
                await page.screenshot(path=str(OUT / f'{tag}-{name}.png'))
                print(tag, name, 'errors:', errors)
                await ctx.close()
        await b.close()
    await runner.cleanup()
asyncio.run(main())
