#!/usr/bin/env python3
"""Exercise cargo paging against a presentation-only WebSocket fixture.

Run after scripts/build_web.sh with --output DIR, then again with --desktop.
Uses aiohttp, Playwright and /usr/bin/chromium; serves this checkout on loopback
8001. Mobile covers DPR-2 touch and portrait/landscape captures; desktop covers
DPR-1 mouse clicks/drags. Outgoing commands are inspected, not applied to a game.
The existing presentation fixture is adapted to the current inventory schema.
"""

import argparse, asyncio, json
from pathlib import Path
from aiohttp import web
from playwright.async_api import async_playwright

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--desktop", action="store_true")
args = parser.parse_args()
OUT = args.output.resolve()
OUT.mkdir(parents=True, exist_ok=True)
state = json.loads((ROOT / "docs/verification/presentation-fixture.json").read_text())
stock = state.pop("stockpile")
state.update(
    island_id=0,
    island_count=1,
    island_origins=[{"column": 0, "row": 0}],
    units=[],
    resources=[],
    buildings=[],
    inventories=[dict.fromkeys(stock, 100)],
    stored_inventories=[dict.fromkeys(stock, 100)],
    ship_connections=[{"ship_id": "ship", "island_id": 0, "docked": True}],
)
state["ships"] = [
    dict(
        id="ship",
        cell={"column": 30, "row": 21},
        step=None,
        destination=None,
        heading=[1, 0],
        passengers=[],
        cargo=dict.fromkeys(stock, 3),
        home_dock_id=None,
    )
]
state["terrain"]["cells"] = "".join(
    "A" if x < 27 else "I" if x == 27 else "J" for y in range(80) for x in range(120)
)
state["terrain"]["heights"] = "g" * 9600
state["terrain"]["rows"] = 80
state["simulation_speed"] = 0
commands = []


async def ws(req):
    sock = web.WebSocketResponse()
    await sock.prepare(req)
    print("CONNECTED", flush=True)
    for tick in range(1, 4):
        state["tick"] = tick
        await sock.send_json({"type": "snapshot", "sequence": tick, "world": state})
        await asyncio.sleep(0.1)
    async for msg in sock:
        if msg.type == web.WSMsgType.TEXT:
            commands.append(json.loads(msg.data))
            print("COMMAND", msg.data, flush=True)
    return sock


async def file(req):
    path = req.match_info["path"]
    path = "web/index.html" if path in ("", "play") else path
    candidate = (ROOT / path).resolve()
    if not any(candidate.is_relative_to(ROOT / f) for f in ["web", "assets"]):
        raise web.HTTPNotFound()
    return web.FileResponse(candidate)


async def main():
    app = web.Application()
    app.router.add_get("/ws", ws)
    app.router.add_get("/{path:.*}", file)
    runner = web.AppRunner(app)
    await runner.setup()
    await web.TCPSite(runner, "127.0.0.1", 8001).start()
    async with async_playwright() as p:
        b = await p.chromium.launch(
            executable_path="/usr/bin/chromium",
            args=["--no-sandbox", "--enable-unsafe-swiftshader"],
        )
        page = await b.new_page(
            viewport={
                "width": 1280 if args.desktop else 390,
                "height": 800 if args.desktop else 844,
            },
            device_scale_factor=1 if args.desktop else 2,
            is_mobile=not args.desktop,
            has_touch=not args.desktop,
        )
        page.on(
            "console",
            lambda m: (
                print("CONSOLE", m.text, flush=True)
                if m.type in ("warning", "error")
                else None
            ),
        )
        errors = []
        page.on("pageerror", lambda e: errors.append(str(e)))
        await page.goto("http://127.0.0.1:8001/play", wait_until="networkidle")
        print("LOADED PAGE", flush=True)
        await page.clock.install()
        await page.clock.pause_at(await page.evaluate("Date.now() + 100"))
        await page.clock.run_for(34)
        await page.locator("#loading").wait_for(state="detached", timeout=30000)
        print("READY", flush=True)
        await page.clock.run_for(34)
        if args.desktop:
            await page.mouse.click(640, 380)
        else:
            await page.touchscreen.tap(195, 400)
        await page.clock.run_for(34)
        print("SELECTED", flush=True)
        if args.desktop:
            await page.set_viewport_size({"width": 1280, "height": 800})
            await page.clock.run_for(34)
            await page.screenshot(path=str(OUT / "desktop.png"))
            commands.clear()
            await page.mouse.click(1024, 598)
            await page.clock.run_for(34)
            await page.mouse.click(240, 542)
            await page.clock.run_for(34)
            await asyncio.sleep(0.1)
            assert (
                len(commands) == 1 and commands[0]["command"]["kind"] == "timber"
            ), commands
            commands.clear()
            await page.mouse.move(240, 542)
            await page.mouse.down()
            await page.mouse.move(400, 542, steps=5)
            await page.mouse.up()
            await page.clock.run_for(34)
            assert not commands, commands
            await page.mouse.click(240, 542)
            await page.clock.run_for(34)
            await asyncio.sleep(0.1)
            assert (
                len(commands) == 1 and commands[0]["command"]["kind"] == "wood"
            ), commands
            print(
                "PASS: desktop paging and mouse drag select expected resources without accidental transfer",
                flush=True,
            )
        else:
            await page.screenshot(path=str(OUT / "phone.png"))
            await page.screenshot(
                path=str(OUT / "phone-cargo-row.png"),
                clip={"x": 35, "y": 585, "width": 320, "height": 95},
            )
            cdp = await page.context.new_cdp_session(page)

            async def swipe(start, end):
                await cdp.send(
                    "Input.dispatchTouchEvent",
                    {
                        "type": "touchStart",
                        "touchPoints": [{"x": start[0], "y": start[1]}],
                    },
                )
                for i in range(1, 6):
                    point = {
                        "x": start[0] + (end[0] - start[0]) * i / 5,
                        "y": start[1] + (end[1] - start[1]) * i / 5,
                    }
                    await cdp.send(
                        "Input.dispatchTouchEvent",
                        {"type": "touchMove", "touchPoints": [point]},
                    )
                await cdp.send(
                    "Input.dispatchTouchEvent", {"type": "touchEnd", "touchPoints": []}
                )
                await page.clock.run_for(34)

            async def check_load(kind):
                commands.clear()
                await page.touchscreen.tap(125, 634)
                await page.clock.run_for(34)
                await asyncio.sleep(0.1)
                assert len(commands) == 1, commands
                assert commands[0]["command"] == {
                    "type": "transfer_ship_cargo",
                    "ship_id": "ship",
                    "kind": kind,
                    "amount": 10.0,
                    "direction": "load",
                }, commands

            commands.clear()
            await swipe((250, 634), (125, 634))
            assert not commands, commands
            await check_load("stone")
            await page.touchscreen.tap(325, 633)
            await page.clock.run_for(34)
            await check_load("iron")
            await page.touchscreen.tap(65, 633)
            await page.clock.run_for(34)
            await check_load("stone")
            commands.clear()
            await swipe((125, 634), (250, 634))
            assert not commands, commands
            await check_load("wood")
            print(
                "PASS: touch swipes and chevrons select the expected resource pages; swipes emit no cargo command",
                flush=True,
            )
            await page.set_viewport_size({"width": 320, "height": 844})
            await page.clock.run_for(34)
            await page.screenshot(path=str(OUT / "phone320.png"))
            await page.set_viewport_size({"width": 844, "height": 390})
            await page.clock.run_for(34)
            await page.screenshot(path=str(OUT / "landscape.png"))
        assert not errors, errors
        await b.close()
    await runner.cleanup()


asyncio.run(main())
