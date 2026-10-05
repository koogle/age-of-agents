#!/usr/bin/env python3
"""Manage and verify the Age of Agents Modal deployment."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP_NAME = "age-of-agents"
BASE_URL = "https://koogle-frick--age-of-agents-web.modal.run"


def modal(*arguments: str) -> None:
    subprocess.run([sys.executable, "-m", "modal", *arguments], cwd=ROOT, check=True)


def fetch(path: str, timeout: int = 120) -> bytes:
    request = urllib.request.Request(f"{BASE_URL}{path}", headers={"User-Agent": "age-of-agents-deploy-check"})
    with urllib.request.urlopen(request, timeout=timeout) as response:
        if response.status != 200:
            raise RuntimeError(f"GET {path} returned HTTP {response.status}")
        return response.read()



def unpack_terrain(encoded: str, limit: int) -> str:
    """Decode the snapshot's bounded ~count:character runs (or legacy text)."""
    if not encoded.isascii():
        raise RuntimeError("non-ASCII terrain")
    parts = []
    size = 0
    index = 0
    while index < len(encoded):
        count = 1
        if encoded[index] == "~":
            end = encoded.find(":", index + 1)
            digits = encoded[index + 1:end]
            if end < 0 or not digits.isdigit() or end + 1 >= len(encoded):
                raise RuntimeError("invalid terrain run")
            count = int(digits)
            index = end + 1
        if count <= 0 or count > limit - size:
            raise RuntimeError("terrain run exceeds map bounds")
        parts.append(encoded[index] * count)
        size += count
        index += 1
    if size != limit:
        raise RuntimeError("terrain length disagrees with map dimensions")
    return "".join(parts)


def verify_once() -> None:
    comparisons = {
        "/": ROOT / "web/index.html",
        "/play": ROOT / "web/index.html",
        "/assets/loading/favicon.png": ROOT / "assets/loading/favicon.png",
        "/assets/loading/favicon.ico": ROOT / "assets/loading/favicon.ico",
    }
    for remote_path, local_path in comparisons.items():
        remote = fetch(remote_path)
        local = local_path.read_bytes()
        if remote != local:
            raise RuntimeError(f"production {remote_path} does not match {local_path.relative_to(ROOT)}")

    wasm = fetch("/web/pkg/aoa_client_bg.wasm")
    if not wasm.startswith(b"\0asm"):
        raise RuntimeError("production does not serve the Rust web client")
    if b"All types" not in wasm:
        raise RuntimeError("production does not serve the grouped building menu")
    if b"Place field" not in wasm:
        raise RuntimeError("production does not serve replenishable fields")
    for sheet in ("buildings_economy", "buildings_crafts", "buildings_civic", "units",
                  "villager_field_preparation"):
        for extension in ("json", "png"):
            path = f"assets/sprites/{sheet}.{extension}"
            if fetch(f"/{path}") != (ROOT / path).read_bytes():
                raise RuntimeError(f"production {path} does not match the catalog")

    state = json.loads(fetch("/state"))
    # The continuous map grows; both terrain channels may contain encoded runs.
    encoded = state.get("terrain", {})
    columns, rows = state.get("columns"), state.get("rows")
    if (type(columns) is not int or type(rows) is not int
            or not 0 < columns <= 65535 or not 0 < rows <= 65535
            or encoded.get("columns") != columns or encoded.get("rows", rows) != rows):
        raise RuntimeError("unexpected world dimensions")
    size = columns * rows
    terrain = list(zip(unpack_terrain(encoded.get("cells", ""), size),
                       unpack_terrain(encoded.get("heights", ""), size)))
    units = state.get("units", [])
    if not units:
        raise RuntimeError("production state has no units")
    cells = [(unit["cell"]["column"], unit["cell"]["row"]) for unit in units]
    if len(set(cells)) != len(cells):
        raise RuntimeError("production units share a cell")
    unseen = [cell for cell in terrain if cell[0] == "."]
    if not unseen:
        raise RuntimeError("production state has no unseen terrain to verify")
    if any(height != "." for _, height in unseen):
        raise RuntimeError("production leaks elevation for unseen terrain")
    if state.get("simulation_speed") not in (0.0, 1.0, 2.0):
        raise RuntimeError(f"invalid production simulation speed: {state.get('simulation_speed')}")
    expected_resources = {
        "wood",
        "food",
        "stone",
        "gold",
        "iron",
        "coal",
        "clay",
        "fiber",
        "timber",
        "steel",
        "bricks",
        "cloth",
        "rations",
    }
    if set(state.get("stockpile", {})) != expected_resources:
        raise RuntimeError("production stockpile does not expose the explicit 13-resource catalog")

    print(
        "PASS production matches checkout; "
        f"terrain={len(terrain)}, units={len(units)}, unseen={len(unseen)}, "
        f"speed={state['simulation_speed']}"
    )


def verify(attempts: int = 24, delay_seconds: int = 5) -> None:
    last_error: Exception | None = None
    for attempt in range(1, attempts + 1):
        try:
            verify_once()
            return
        except (RuntimeError, urllib.error.URLError, TimeoutError) as error:
            last_error = error
            if attempt < attempts:
                print(f"production not ready ({attempt}/{attempts}): {error}", file=sys.stderr)
                time.sleep(delay_seconds)
    raise RuntimeError(f"production did not converge after {attempts} attempts") from last_error


def deploy(skip_verify: bool) -> None:
    modal("deploy", "modal_app.py")
    if not skip_verify:
        verify()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    deploy_parser = subparsers.add_parser("deploy", help="Deploy current checkout and verify production")
    deploy_parser.add_argument("--skip-verify", action="store_true")

    subparsers.add_parser("verify", help="Compare production with the current checkout")
    subparsers.add_parser("status", help="List Modal apps and containers")
    subparsers.add_parser("history", help="Show deployment history")

    logs_parser = subparsers.add_parser("logs", help="Show recent application logs")
    logs_parser.add_argument("--tail", type=int, default=100)
    logs_parser.add_argument("--since")
    logs_parser.add_argument("--follow", action="store_true")

    subparsers.add_parser("rollover", help="Restart production containers without rebuilding")

    stop_parser = subparsers.add_parser("stop", help="Permanently stop the deployed app")
    stop_parser.add_argument(
        "--confirm",
        metavar="APP_NAME",
        help=f"required safety confirmation; pass exactly {APP_NAME!r}",
    )
    return parser


def main() -> None:
    args = build_parser().parse_args()
    if args.command == "deploy":
        deploy(args.skip_verify)
    elif args.command == "verify":
        verify()
    elif args.command == "status":
        modal("app", "list")
        modal("container", "list")
    elif args.command == "history":
        modal("app", "history", APP_NAME)
    elif args.command == "logs":
        command = ["app", "logs", APP_NAME, "--tail", str(args.tail)]
        if args.since:
            command.extend(("--since", args.since))
        if args.follow:
            command.append("--follow")
        modal(*command)
    elif args.command == "rollover":
        modal("app", "rollover", APP_NAME)
    elif args.command == "stop":
        if args.confirm != APP_NAME:
            raise SystemExit(f"refusing to stop production; pass --confirm {APP_NAME}")
        modal("app", "stop", APP_NAME, "--yes")


if __name__ == "__main__":
    main()
