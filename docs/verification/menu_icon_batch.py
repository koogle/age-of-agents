#!/usr/bin/env python3
"""Persistent loopback-only browser worker for sequential menu icon verification.

A --queue directory receives one request.json at a time containing name/output.
Each result records the rebuilt bundle hash; a failure stops the job. No saved
world is read and wire commands are captured, never forwarded to production.
"""

import argparse, asyncio, copy, hashlib, json
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser()
parser.add_argument("--queue", type=Path, required=True)
args = parser.parse_args()
args.queue.mkdir(parents=True, exist_ok=True)
clients = []
commands = []
state = {}
sequence = 0
raw = json.loads((ROOT / "docs/verification/menu-icon-fixture.json").read_text())
PRODUCTS = {
    "resource_steel": ("smelter", "steel"),
    "resource_bricks": ("kiln", "bricks"),
    "resource_cloth": ("weaver", "cloth"),
    "resource_rations": ("kitchen", "rations"),
    "unit_guard": ("barracks", "guard"),
    "unit_archer": ("range", "archer"),
    "unit_healer": ("infirmary", "healer"),
    "unit_siege_cart": ("workshop", "siege_cart"),
    "transport_queue": ("dock", "transport_ship"),
}
HOOK = """window.hudQuads=[];const p=WebGL2RenderingContext.prototype,o=p.bufferSubData;p.bufferSubData=function(t,off,data,start=0,length){if(data&&data.buffer){const n=(length===undefined?data.length-start:length)*data.BYTES_PER_ELEMENT;if(n%64===0&&n>512&&n<300000){const a=new Float32Array(data.buffer,data.byteOffset+start*data.BYTES_PER_ELEMENT,n/4);let q=[];for(let i=0;i<a.length;i+=16)q.push(Array.from(a.slice(i,i+16)));if(q.some(v=>v[12]===3))window.hudQuads=q;}}return o.apply(this,arguments);};"""


def fixture(name, portrait=False):
    s = copy.deepcopy(raw)
    s.update(
        animals=[],
        resources=[],
        buildings=[],
        ships=[],
        ship_connections=[],
        simulation_speed=0,
    )
    s["terrain"].update(cells="A" * 9600, heights="g" * 9600)
    s["units"] = s["units"][:1]
    u = s["units"][0]
    u.update(
        cell={"column": 30, "row": 21},
        position={"x": 30.0, "y": 21.0},
        step=None,
        action={"type": "idle"},
        health=100.0,
    )
    for stocks in [s["inventories"], s["stored_inventories"]]:
        for stock in stocks:
            for k in stock:
                stock[k] = 500.0
    # Keep all current building categories visible; their costs/commands remain typed.
    s["available_buildings"] = [
        x["kind"] if isinstance(x, dict) else x for x in s["catalog"]["buildings"]
    ]
    if name in PRODUCTS and not portrait:
        kind, product = PRODUCTS[name]
        s["units"] = []
        b = copy.deepcopy(raw["buildings"][0])
        b.update(
            kind=kind,
            origin={"column": 28, "row": 19},
            columns=4,
            rows=4,
            produces=[product],
            researches=[],
            queue=[
                {
                    "id": 1,
                    "job": {
                        "type": "produce",
                        "product": product,
                        "elapsed_seconds": 0.0,
                    },
                }
            ],
            next_queue_id=2,
            job={"type": "produce", "product": product, "elapsed_seconds": 1.0},
        )
        s["buildings"] = [b]
        if kind == "dock":
            s["terrain"]["cells"] = "".join(
                "J" if y >= 23 else "I" for y in range(80) for x in range(120)
            )
    elif name.startswith("unit_"):
        u["kind"] = name[5:]
    elif name.startswith("command_") and name != "command_back":
        s["units"] = []
        s["terrain"]["cells"] = "J" * 9600
        ship = {
            "id": "menu-ship",
            "cell": {"column": 30, "row": 21},
            "step": None,
            "destination": None,
            "heading": [1, 0],
            "passengers": [copy.deepcopy(u)],
            "cargo": {k: 0.0 for k in s["inventories"][0]},
            "home_dock_id": None,
        }
        s["ships"] = [ship]
        if name == "command_sail":
            s["island_count"] = 2
            s["island_origins"].append({"column": 184, "row": 0})
            s["inventories"].append(copy.deepcopy(s["inventories"][0]))
            s["stored_inventories"].append(copy.deepcopy(s["stored_inventories"][0]))
        if name == "command_cargo":
            b = copy.deepcopy(raw["buildings"][0])
            b.update(
                kind="dock",
                origin={"column": 28, "row": 19},
                columns=4,
                rows=4,
                produces=["transport_ship"],
                researches=[],
            )
            s["buildings"] = [b]
            ship["cell"] = {"column": 30, "row": 23}
            s["terrain"]["cells"] = "".join(
                "J" if y >= 23 else "I" for y in range(80) for x in range(120)
            )
    elif name == "stop":
        u["action"] = {"type": "move", "to": {"column": 31, "row": 21}}
    return s


async def ws(req):
    sock = web.WebSocketResponse()
    await sock.prepare(req)
    clients.append(sock)
    async for msg in sock:
        commands.append(json.loads(msg.data))
    return sock


async def file(req):
    p = req.match_info["path"]
    p = "web/index.html" if p in ("", "play") else p
    c = (ROOT / p).resolve()
    if not any(c.is_relative_to(ROOT / x) for x in ["assets", "web"]):
        raise web.HTTPNotFound()
    response = web.FileResponse(c)
    response.headers["Cache-Control"] = "no-store"
    return response


async def send():
    global sequence
    sequence += 1
    state["tick"] = sequence
    for sock in clients:
        if not sock.closed:
            await sock.send_json(
                {"type": "snapshot", "sequence": sequence, "world": state}
            )


async def coin(page, mode, index):
    q = await page.evaluate(
        """window.hudQuads.filter(q=>Math.abs(q[2]/devicePixelRatio-(innerWidth<600?44:52))<.1 && Math.abs(q[3]-q[2])<.1 && q[1]/devicePixelRatio>innerHeight/2).map(q=>q.slice(0,4).map(v=>v/devicePixelRatio)).sort((a,b)=>a[1]-b[1]||a[0]-b[0])"""
    )
    assert q, ("no command coins", mode)
    rect = q[index]
    tap = page.touchscreen.tap if mode == "phone" else page.mouse.click
    await tap(rect[0] + rect[2] / 2, rect[1] + rect[3] / 2)
    await asyncio.sleep(0.6)


async def run(browser, request):
    global state
    name = request["name"]
    out = Path(request["output"])
    out.mkdir(parents=True, exist_ok=True)
    errors = []
    scenes = [False, True] if name.startswith("unit_") else [False]
    context = await browser.new_context(
        viewport={"width": 1280, "height": 800}, has_touch=True
    )
    page = await context.new_page()
    page.on("pageerror", lambda e: errors.append(str(e)))
    await page.add_init_script(HOOK)
    cdp = await context.new_cdp_session(page)
    loaded = False
    for mode, w, h, dpr in [("desktop", 1280, 800, 1), ("phone", 390, 844, 2)]:
        await cdp.send(
            "Emulation.setDeviceMetricsOverride",
            {
                "width": w,
                "height": h,
                "deviceScaleFactor": dpr,
                "mobile": mode == "phone",
                "screenWidth": w,
                "screenHeight": h,
            },
        )
        for portrait in scenes:
            state = fixture(name, portrait)
            commands.clear()
            if not loaded:
                await page.goto("http://127.0.0.1:8012/play", wait_until="networkidle")
                async with asyncio.timeout(120):
                    while not any(not sock.closed for sock in clients):
                        await asyncio.sleep(0.1)
                loaded = True
            for _ in range(15):
                await send()
                await asyncio.sleep(0.1)
            await page.locator("#loading").wait_for(state="detached", timeout=120000)
            metrics = await page.evaluate("[innerWidth,innerHeight,devicePixelRatio]")
            assert metrics == [w, h, dpr], metrics
            tap = page.touchscreen.tap if mode == "phone" else page.mouse.click
            await tap(w / 2, h / 2 - 20)
            await asyncio.sleep(0.8)
            if name.startswith("category_") or name == "command_back":
                await coin(page, mode, 0)
                if name == "command_back":
                    await coin(page, mode, 0)
            label = name + ("-portrait" if portrait else "")
            await page.screenshot(
                path=str(out / f"{label}-{mode}.jpg"), type="jpeg", quality=88
            )
            (out / f"{label}-{mode}-quads.json").write_text(
                json.dumps(await page.evaluate("window.hudQuads"))
            )
            if name in PRODUCTS and not portrait:
                await coin(page, mode, 0)
                assert any(
                    x.get("command", {}).get("type") == "produce" for x in commands
                ), (name, mode, commands)
            elif name in ["command_disembark", "command_sail", "command_explore"]:
                await coin(page, mode, 0 if name == "command_disembark" else 1)
                expected = "disembark" if name == "command_disembark" else "voyage"
                assert any(
                    x.get("command", {}).get("type") == expected for x in commands
                ), (name, mode, commands)
            elif name == "command_back":
                await coin(page, mode, -1)
            assert not errors, errors
            print(name, mode, "portrait" if portrait else "menu", "PASS", flush=True)
    await context.close()
    return {
        "name": name,
        "wasm_sha256": hashlib.sha256(
            (ROOT / "web/pkg/aoa_client_bg.wasm").read_bytes()
        ).hexdigest(),
        "errors": errors,
        "viewports": ["1280x800 DPR1 mouse", "390x844 DPR2 touch"],
        "limitation": "Controlled snapshot rendering and wire dispatch, not domain acceptance or physical-phone evidence.",
    }


async def main():
    app = web.Application()
    app.router.add_get("/ws", ws)
    app.router.add_get("/{path:.*}", file)
    r = web.AppRunner(app)
    await r.setup()
    await web.TCPSite(r, "127.0.0.1", 8012).start()
    async with async_playwright() as p:
        browser = await p.chromium.launch(
            executable_path="/usr/bin/chromium",
            args=["--no-sandbox", "--enable-unsafe-swiftshader"],
        )
        print("Browser worker ready", flush=True)
        while True:
            request = args.queue / "request.json"
            if not request.exists():
                await asyncio.sleep(0.5)
                continue
            spec = json.loads(request.read_text())
            request.unlink()
            try:
                result = await run(browser, spec)
            except Exception as e:
                result = {"name": spec["name"], "error": repr(e)}
                print(result, flush=True)
            out = Path(spec["output"])
            out.mkdir(parents=True, exist_ok=True)
            (out / "result.json").write_text(json.dumps(result, indent=2) + "\n")
            (args.queue / "done.json").write_text(json.dumps(result) + "\n")


asyncio.run(main())
