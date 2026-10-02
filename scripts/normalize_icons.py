#!/usr/bin/env python3
"""Center and size generated UI icons so they sit identically inside round coins.

Each icon is measured on its alpha mask:
- optical centre: halfway between the bounding-box centre and the alpha centroid;
- visual weight: square root of the covered area;
- reach: the farthest opaque pixel from the optical centre.

It is then rescaled so every icon has the same visual weight without any pixel
leaving the coin's safe circle, and recentred on the canvas.

    python3 scripts/normalize_icons.py --check          # report, exit 1 if off
    python3 scripts/normalize_icons.py --write          # normalize in place
    python3 scripts/normalize_icons.py --sheet out.png  # review sheet with guides
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ICONS = ROOT / "assets/ui/icons"
SIZE = 128
SAFE_RADIUS = 0.47  # of SIZE: keeps every pixel inside the coin face (hud.js draws the icon box at 1.5 r, so 0.47 reaches 0.7 r of a 0.8 r face)
WEIGHT = 0.52  # sqrt(covered area) as a fraction of SIZE
ALPHA = 40  # alpha counted as "ink"
CENTER_TOLERANCE = 1.5  # px
WEIGHT_TOLERANCE = 0.06  # relative


def measure(image: Image.Image) -> dict:
    alpha = np.asarray(image.getchannel("A"), dtype=np.float32)
    mask = alpha > ALPHA
    if not mask.any():
        raise ValueError("empty icon")
    ys, xs = np.nonzero(mask)
    box_center = np.array([(xs.min() + xs.max() + 1) / 2, (ys.min() + ys.max() + 1) / 2])
    weights = alpha[mask]
    centroid = np.array([(xs * weights).sum(), (ys * weights).sum()]) / weights.sum()
    center = (box_center + centroid) / 2
    reach = np.sqrt(((xs + 0.5 - center[0]) ** 2 + (ys + 0.5 - center[1]) ** 2).max())
    return {"center": center, "weight": np.sqrt(mask.sum()), "reach": reach}


def target_scale(m: dict, size: int) -> float:
    return min(WEIGHT * size / m["weight"], SAFE_RADIUS * size / m["reach"])


def normalize(image: Image.Image, size: int = SIZE) -> Image.Image:
    m = measure(image)
    scale = target_scale(m, size)
    # Supersample: work at 4x so the final downscale stays crisp.
    work = size * 4
    big = image.resize((round(image.width * scale * 4), round(image.height * scale * 4)), Image.LANCZOS)
    canvas = Image.new("RGBA", (work, work), (0, 0, 0, 0))
    offset = (round(work / 2 - m["center"][0] * scale * 4), round(work / 2 - m["center"][1] * scale * 4))
    canvas.alpha_composite(big, (max(offset[0], 0), max(offset[1], 0)),
                           (max(-offset[0], 0), max(-offset[1], 0)))
    return canvas.resize((size, size), Image.LANCZOS)


def problems(path: Path) -> list[str]:
    image = Image.open(path).convert("RGBA")
    issues = []
    if image.size != (SIZE, SIZE):
        issues.append(f"size {image.size} != {SIZE}")
    m = measure(image)
    dx, dy = m["center"] - np.array(image.size) / 2
    if max(abs(dx), abs(dy)) > CENTER_TOLERANCE:
        issues.append(f"off-centre by ({dx:+.1f}, {dy:+.1f}) px")
    scale = target_scale(m, image.width)
    if abs(scale - 1) > WEIGHT_TOLERANCE:
        issues.append(f"needs {scale:.2f}x to match the others")
    return issues


def sheet(paths: list[Path], out: Path) -> None:
    cell, cols = 160, 6
    rows = (len(paths) + cols - 1) // cols
    board = Image.new("RGBA", (cols * cell, rows * cell), (244, 238, 224, 255))
    draw = ImageDraw.Draw(board)
    for i, path in enumerate(paths):
        x, y = (i % cols) * cell, (i // cols) * cell
        cx, cy = x + cell // 2, y + cell // 2
        r = SIZE / 2
        draw.ellipse((cx - r, cy - r, cx + r, cy + r), fill=(250, 246, 236, 255), outline=(176, 128, 60, 255), width=3)
        safe = SAFE_RADIUS * SIZE
        draw.ellipse((cx - safe, cy - safe, cx + safe, cy + safe), outline=(200, 80, 60, 120))
        draw.line((cx - 6, cy, cx + 6, cy), fill=(200, 80, 60, 160))
        draw.line((cx, cy - 6, cx, cy + 6), fill=(200, 80, 60, 160))
        board.alpha_composite(Image.open(path).convert("RGBA"), (cx - SIZE // 2, cy - SIZE // 2))
        draw.text((x + 6, y + cell - 14), path.stem, fill=(60, 40, 30, 255))
    board.save(out)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("paths", nargs="*", type=Path, help=f"icons (default: {ICONS.relative_to(ROOT)}/*.png)")
    parser.add_argument("--check", action="store_true", help="report icons that are off; exit 1 if any")
    parser.add_argument("--write", action="store_true", help="normalize icons in place")
    parser.add_argument("--sheet", type=Path, help="write a review sheet with coin and safe-circle guides")
    args = parser.parse_args()
    # Portraits are full-bleed round coins of their own, not icons on a face.
    paths = args.paths or sorted(p for p in ICONS.glob("*.png") if not p.name.startswith("portrait_"))
    if args.write:
        for path in paths:
            if problems(path):
                normalize(Image.open(path).convert("RGBA")).save(path, optimize=True)
                print(f"normalized {path.name}")
    failed = False
    for path in paths:
        issues = problems(path)
        failed |= bool(issues)
        print(f"{'FAIL' if issues else 'PASS'} {path.name}" + (": " + "; ".join(issues) if issues else ""))
    if args.sheet:
        sheet(paths, args.sheet)
        print(f"sheet: {args.sheet}")
    return 1 if failed and (args.check or args.write) else 0


if __name__ == "__main__":
    sys.exit(main())
