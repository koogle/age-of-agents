#!/usr/bin/env python3
"""Register the retained NPC-style wildlife cells; never enlarge source pixels."""
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/sprites/wildlife_sources/npc-style-refined.png"
CELL = 627
BASELINE = 590


def main():
    source = Image.open(SOURCE).convert("RGBA")
    assert source.size == (CELL * 2, CELL * 2)
    atlas = Image.new("RGBA", source.size)
    for row in range(2):
        for col in range(2):
            frame = source.crop((col * CELL, row * CELL, (col + 1) * CELL, (row + 1) * CELL))
            # Common scale across every pose; padding keeps silhouettes off cell edges.
            frame.thumbnail((589, 589), Image.Resampling.LANCZOS)
            bounds = frame.getchannel("A").point(lambda a: 255 if a > 128 else 0).getbbox()
            assert bounds is not None
            atlas.alpha_composite(frame, (col * CELL + (CELL - frame.width) // 2,
                                          row * CELL + BASELINE - bounds[3]))
    atlas.save(ROOT / "assets/sprites/wildlife.png")


if __name__ == "__main__":
    main()
