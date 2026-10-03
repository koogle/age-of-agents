#!/usr/bin/env python3
"""Repack recovered FAL sources and reviewed refinements into 512px gameplay cells.

Requires Pillow, NumPy and SciPy. No network or generation calls. Originals and
refinements are retained under assets/sprites/hd_sources with provenance.
"""
from pathlib import Path
import os
import shutil
import subprocess
import sys
import tempfile

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SPRITES = ROOT / "assets/sprites"
SOURCES = SPRITES / "hd_sources"
TOOLS = SPRITES / "tools"


def pack(script, target, **env):
    subprocess.run([sys.executable, str(TOOLS / script), str(SPRITES / target)],
                   cwd=SOURCES, env={**os.environ, "HD_SCALE": "2", **env}, check=True)


def main():
    for variant in ("", "woman", "elder"):
        pack("pack.py", "villager" + ("_" + variant if variant else ""),
             VARIANT=variant, TOOL_SCALE="height" if variant == "woman" else "")
    pack("res_pack.py", "resources")
    with tempfile.TemporaryDirectory(prefix="aoa-hd-units-") as directory:
        units = Path(directory)
        for source in (SOURCES / "units").glob("*_cut.png"):
            shutil.copyfile(source, units / source.name)
        for name in ("guard_action", "siege_cart_action"):
            shutil.copyfile(SOURCES / "refined" / (name + "_cut.png"),
                            units / (name + "_cut.png"))
        # The reviewed sheet has four front poses above four back poses. Split
        # only at the empty row gutter; preserve the full-resolution originals.
        motion = Image.open(SOURCES / "refined/siege_cart_motion_cut.png").convert("RGBA")
        alpha = np.asarray(motion)[..., 3]
        lo, hi = motion.height // 3, motion.height * 2 // 3
        split = lo + int(np.argmin((alpha[lo:hi] > 40).sum(axis=1)))
        for facing, bounds in (("front", (0, 0, motion.width, split)),
                               ("back", (0, split, motion.width, motion.height))):
            strip = motion.crop(bounds)
            strip.save(units / f"siege_cart_walk_{facing}_cut.png")
            # Idle reuses two reviewed poses of the same chassis and pennant.
            strip.crop((0, 0, strip.width // 2, strip.height)).save(
                units / f"siege_cart_idle_{facing}_cut.png")
        pack("unit_pack.py", "units", UNIT_SOURCE=str(units))


if __name__ == "__main__":
    main()
