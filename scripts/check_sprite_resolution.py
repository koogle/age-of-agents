#!/usr/bin/env python3
"""Audit world-sprite frame resolution, including every action, not just idle.

Use --root with an extracted sprite branch to check its assets before integration.
The check measures authored frame pixels, not DPI metadata or atlas dimensions.
Passing this check still requires source review: enlarging a low-resolution frame
does not recover detail. --report-only audits existing low-resolution assets
without treating the known migration work as a build failure.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

MANIFESTS = (
    "villager.json", "villager_woman.json", "villager_elder.json",
    "villager_idle_hd.json", "resources.json", "towncenter.json",
    "buildings_hd.json", "buildings_economy.json", "buildings_crafts.json",
    "buildings_civic.json", "units.json", "transport.json",
)


def frame_rects(value: object):
    """Walk nested action/facing/stage maps, yielding authored cell rectangles."""
    if isinstance(value, dict):
        for key, child in value.items():
            for path, rect in frame_rects(child):
                yield f"{key}/{path}".rstrip("/"), rect
    elif isinstance(value, list):
        if len(value) == 4 and all(isinstance(n, (int, float)) for n in value):
            yield "", value
        else:
            for index, child in enumerate(value):
                for path, rect in frame_rects(child):
                    yield f"{index}/{path}".rstrip("/"), rect


def audit(path: Path, minimum: int) -> tuple[int, int]:
    data = json.loads(path.read_text())
    frames = data.get("animations", data.get("frames", data.get("people", data.get("nodes"))))
    if "units" in data:
        frames = {kind: entry["animations"] for kind, entry in data["units"].items()}
    found = list(frame_rects(frames))
    if not found:
        print(f"FAIL {path.name}: no frame rectangles")
        return 0, 1
    with Image.open(path.with_suffix(".png")) as image:
        if image.format != "PNG" or image.mode != "RGBA":
            raise ValueError(f"{path.name}: expected lossless RGBA PNG")
        if "size" in data and list(image.size) != data["size"]:
            raise ValueError(f"{path.name}: manifest size does not match actual pixels")
        for name, (x, y, w, h) in found:
            if min(x, y) < 0 or min(w, h) <= 0 or x + w > image.width or y + h > image.height:
                raise ValueError(f"{path.name}/{name}: frame outside actual image")
            alpha = image.crop((x, y, x + w, y + h)).getchannel("A")
            if alpha.getbbox() is None:
                raise ValueError(f"{path.name}/{name}: empty frame")
    low = [(name, rect) for name, rect in found if min(rect[2:]) < minimum]
    dimensions = sorted({(rect[2], rect[3]) for _, rect in found})
    status = "NEEDS HD REPACK" if low else "PASS"
    print(f"{status} {path.name}: {len(found)} frames, cells {dimensions}")
    # Names make it clear that walking, carrying and work/action strips were audited.
    if low:
        depth = 2 if "units" in data else 1
        actions = sorted({"/".join(name.split("/")[:depth]) for name, _ in low})
        print(f"  below {minimum}px: {', '.join(actions)}")
    return len(found), len(low)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1] / "assets/sprites")
    parser.add_argument("--min-frame-px", type=int, default=512)
    parser.add_argument("--report-only", action="store_true")
    args = parser.parse_args()
    if args.min_frame_px <= 0:
        parser.error("--min-frame-px must be positive")
    checked = failed = 0
    for name in MANIFESTS:
        path = args.root / name
        if path.exists():
            try:
                count, low = audit(path, args.min_frame_px)
            except ValueError as error:
                print(f"FAIL {error}")
                return 1
            checked += count
            failed += low
    if not checked:
        parser.error("no sprite manifests found")
    print(f"Checked {checked} frames; {failed} below the HD frame minimum.")
    return int(bool(failed) and not args.report_only)


if __name__ == "__main__":
    raise SystemExit(main())
