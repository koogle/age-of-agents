#!/usr/bin/env python3
"""Verify menu presentation and browser wire dispatch using a controlled snapshot.

Run from any directory with --output PATH. Requires aiohttp, Playwright and
/usr/bin/chromium. Uses only loopback :8011; never reads or mutates saved games.
The fixture deliberately isolates HUD rendering; it is not a gameplay proof.
"""

import asyncio, copy, json, argparse, hashlib
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
P = argparse.ArgumentParser()
P.add_argument("--name", default="stop")
P.add_argument("--output", type=Path, required=True)
args = P.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
base = json.loads(Path(__file__).with_name("menu-icon-fixture.json").read_text())
# Controlled presentation fixture; command capture checks real browser dispatch,
# not authoritative validation. All terrain is flat visible meadow.
base["terrain"]["cells"] = "A" * (base["columns"] * base["rows"])
base["terrain"]["heights"] = "g" * (base["columns"] * base["rows"])
base["resources"] = []
base["buildings"] = []
base["ships"] = []
base["units"] = base["units"][:1]
u = base["units"][0]
u.update(
    cell={"column": 30, "row": 21},
    position={"x": 30.0, "y": 21.0},
    step=None,
    action={"type": "move", "to": {"column": 31, "row": 21}},
)
base["simulation_speed"] = 0
state = copy.deepcopy(base)
clients = []
commands = []
seq = 0


async def ws(req):
    global seq
    sock = web.WebSocketResponse()
    await sock.prepare(req)
    clients.append(sock)
    async for msg in sock:
        data = json.loads(msg.data)
        commands.append(data)
    return sock


async def file(req):
    path = req.match_info["path"]
    path = "web/index.html" if path in ("", "play") else path
    candidate = (ROOT / path).resolve()
    if not any(candidate.is_relative_to(ROOT / x) for x in ["assets", "web"]):
        raise web.HTTPNotFound()
    return web.FileResponse(candidate)


async def send():
    global seq
    seq += 1
    state["tick"] = seq
    for sock in clients:
        if not sock.closed:
            await sock.send_json({"type": "snapshot", "sequence": seq, "world": state})


HOOK = """window.hudQuads=[];const p=WebGL2RenderingContext.prototype,o=p.bufferSubData;p.bufferSubData=function(t,off,data,start=0,length){if(data&&data.buffer){const n=(length===undefined?data.length-start:length)*data.BYTES_PER_ELEMENT;if(n%64===0&&n>512&&n<300000){const a=new Float32Array(data.buffer,data.byteOffset+start*data.BYTES_PER_ELEMENT,n/4);let q=[];for(let i=0;i<a.length;i+=16)q.push(Array.from(a.slice(i,i+16)));if(q.some(v=>v[12]===3))window.hudQuads=q;}}return o.apply(this,arguments);};"""


async def main():
    app = web.Application()
    app.router.add_get("/ws", ws)
    app.router.add_get("/{path:.*}", file)
    r = web.AppRunner(app)
    await r.setup()
    await web.TCPSite(r, "127.0.0.1", 8011).start()
    async with async_playwright() as p:
        browser = await p.chromium.launch(
            executable_path="/usr/bin/chromium",
            args=["--no-sandbox", "--enable-unsafe-swiftshader"],
        )
        for mode, w, h, dpr in [("desktop", 1280, 800, 1), ("phone", 390, 844, 2)]:
            page = await browser.new_page(
                viewport={"width": w, "height": h},
                device_scale_factor=dpr,
                is_mobile=mode == "phone",
                has_touch=mode == "phone",
            )
            errors = []
            page.on("pageerror", lambda e: errors.append(str(e)))
            await page.add_init_script(HOOK)
            await page.goto("http://127.0.0.1:8011/play", wait_until="networkidle")
            async with asyncio.timeout(120):
                while not any(not s.closed for s in clients):
                    await asyncio.sleep(0.1)
            print(mode, "connected", flush=True)
            for _ in range(15):
                await send()
                await asyncio.sleep(0.1)
            print(mode, "snapshots sent", flush=True)
            await page.locator("#loading").wait_for(state="detached", timeout=120000)
            tap = page.touchscreen.tap if mode == "phone" else page.mouse.click
            await tap(w / 2, h / 2 - 20)
            await asyncio.sleep(0.5)
            await page.screenshot(path=str(args.output / f"{args.name}-{mode}.png"))
            quads = await page.evaluate("window.hudQuads")
            (args.output / f"{args.name}-{mode}-quads.json").write_text(
                json.dumps(quads)
            )
            # The selected villager has Build and Stop. The bar is centered on desktop,
            # bottom left on phone, using the production shared layout.
            if args.name == "stop":
                await tap(
                    w / 2 + 31 if mode == "desktop" else 94,
                    h - 50 if mode == "desktop" else h - 40,
                )
                await asyncio.sleep(0.5)
                await asyncio.sleep(0.2)
                assert any(
                    x.get("command", {}).get("type") == "stop" for x in commands
                ), (mode, commands)
                commands.clear()
            assert not errors, errors
            print(
                mode,
                "browser loaded, screenshot captured, dispatch checked",
                flush=True,
            )
            await page.close()
        await browser.close()
    await r.cleanup()
    (args.output / "result.json").write_text(
        json.dumps(
            {
                "scene": args.name,
                "wasm_sha256": hashlib.sha256(
                    (ROOT / "web/pkg/aoa_client_bg.wasm").read_bytes()
                ).hexdigest(),
                "viewports": ["1280x800 DPR1 mouse", "390x844 DPR2 touch"],
                "errors": [],
                "limitation": "Controlled snapshot presentation and wire dispatch; not an authoritative gameplay fixture.",
            },
            indent=2,
        )
        + "\n"
    )


asyncio.run(main())
