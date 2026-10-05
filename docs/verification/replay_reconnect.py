#!/usr/bin/env python3
"""Verify real WebSocket reconnect and suspended rendering against a local fixture.

Requires aiohttp, Playwright, Chromium and a rebuilt web/pkg. Uses loopback :8002
and never reads or modifies a saved/production game. Sprite uploads measure the
actual rendered position; the fixture is presentation-only, not domain gameplay.
"""
import argparse
import asyncio
import json
from pathlib import Path

from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
HOOK = """
window.spriteSamples = [];
// Hold rendering independently of WebSocket delivery, as in a background tab.
window.suspendFrames = false;
window.requestAnimationFrame = callback => {
  const pump = () => {
    if (window.suspendFrames) window.setTimeout(pump, 100);
    else callback(performance.now());
  };
  return window.setTimeout(pump, 100);
};
window.cancelAnimationFrame = id => window.clearTimeout(id);
const original = WebGL2RenderingContext.prototype.bufferSubData;
WebGL2RenderingContext.prototype.bufferSubData = function(target, offset, data, src=0, length) {
  if (data && data.buffer) {
    const n = length === undefined ? data.length - src : length;
    if (n * data.BYTES_PER_ELEMENT === 72) {
      const a = Array.from(new Float32Array(data.buffer, data.byteOffset + src * data.BYTES_PER_ELEMENT, 18));
      if (a[3] > 0.5 && a[3] < 5 && a[12] === 1 && a[16] === 0 && a[17] === 0)
        window.spriteSamples.push(a);
    }
  }
  return original.apply(this, arguments);
};
"""


async def verify(browser, output, phone):
    state = json.loads((Path(__file__).parent / 'presentation-fixture.json').read_text())
    sockets = []
    state['tick'] = 100
    state['simulation_speed'] = 1.0
    state['units'][0]['position'] = {'x': 30.0, 'y': 21.0}

    async def socket(request):
        ws = web.WebSocketResponse()
        await ws.prepare(request)
        sockets.append(ws)
        await ws.send_json({'type': 'snapshot', 'sequence': 100 if len(sockets) == 1 else 0, 'world': state})
        async for _ in ws:
            pass
        return ws

    async def file(request):
        path = request.match_info['path']
        candidate = (ROOT / ('web/index.html' if path == 'play' else path)).resolve()
        if not any(candidate.is_relative_to(ROOT / folder) for folder in ['web', 'assets']):
            raise web.HTTPNotFound()
        return web.FileResponse(candidate)

    app = web.Application()
    app.router.add_get('/ws', socket)
    app.router.add_get('/{path:.*}', file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner, '127.0.0.1', 8002).start()
    context = await browser.new_context(
        viewport={'width': 390 if phone else 960, 'height': 844 if phone else 640},
        device_scale_factor=2 if phone else 1, has_touch=phone, is_mobile=phone,
    )
    errors = []
    page = await context.new_page()
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.on('console', lambda msg: print(msg.text, flush=True) if 'bad server message' in msg.text else None)
    await page.add_init_script(HOOK)
    try:
        await page.goto('http://127.0.0.1:8002/play', wait_until='networkidle')
        await page.locator('#loading').wait_for(state='detached', timeout=120000)
        print('loaded', 'phone' if phone else 'desktop', flush=True)
        await page.wait_for_function('window.spriteSamples.length > 0')

        async def send(sequence, tick, x, speed=1.0):
            state['tick'] = tick
            state['simulation_speed'] = speed
            state['units'][0]['position']['x'] = x
            await sockets[-1].send_json({'type': 'snapshot', 'sequence': sequence, 'world': state})
            # WebSocket callbacks run even while requestAnimationFrame is held.
            await asyncio.sleep(0.03)

        for tick in range(101, 106):
            await send(tick, tick, 30.0 + (tick - 100) * 0.3)
            await asyncio.sleep(0.15)

        print('movement warmed', flush=True)
        # A short gap previously retained old history; pausing must not prevent
        # establishing the fresh baseline when the sequence restarts at zero.
        state['tick'] = 107
        state['simulation_speed'] = 0.0
        state['units'][0]['position']['x'] = 35.0
        await sockets[0].close()
        for _ in range(40):
            await asyncio.sleep(0.18)
            if len(sockets) == 2:
                break
        assert len(sockets) == 2, 'client did not reconnect'
        print('socket recovered', flush=True)
        await page.wait_for_function('window.spriteSamples.length > 0 && Math.abs(window.spriteSamples.at(-1)[0] - 17.5) < 0.001')
        samples = await page.evaluate('window.spriteSamples')
        assert samples and abs(samples[-1][0] - 17.5) < 0.001, 'reconnect replayed stale position'
        await page.evaluate('window.spriteSamples = []')
        await asyncio.sleep(0.4)
        samples = await page.evaluate('window.spriteSamples')
        assert samples and all(abs(s[0] - 17.5) < 0.001 for s in samples), 'paused reconnect drifted'

        # Deliver a long backlog without a render frame, then finish paused.
        await page.evaluate('window.suspendFrames = true')
        await asyncio.sleep(0.2)
        await page.evaluate('window.spriteSamples = []')
        for sequence in range(1, 41):
            await send(sequence, 107 + sequence, 35.0 - sequence * 0.1, 0.0 if sequence == 40 else 1.0)
        assert await page.evaluate('window.spriteSamples.length') == 0, 'render suspension failed'
        await page.evaluate('window.suspendFrames = false')
        await page.wait_for_function('window.spriteSamples.length > 0')
        samples = await page.evaluate('window.spriteSamples')
        assert samples and all(abs(s[0] - 15.5) < 0.001 for s in samples), 'backlog was animated'
        name = 'phone' if phone else 'desktop'
        await page.screenshot(path=str(output / f'{name}-reconnected.png'))
        assert not errors, errors
        print(f'{name}: reconnect to sequence zero, paused baseline and 40-snapshot backlog PASS', flush=True)
    finally:
        await context.close()
        await runner.cleanup()


async def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    async with async_playwright() as pw:
        browser = await pw.chromium.launch(
            executable_path='/usr/bin/chromium', args=['--no-sandbox', '--enable-unsafe-swiftshader'],
        )
        try:
            for phone in [False, True]:
                await verify(browser, args.output, phone)
        finally:
            await browser.close()


if __name__ == '__main__':
    asyncio.run(main())
