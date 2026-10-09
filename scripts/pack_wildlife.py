#!/usr/bin/env python3
"""Register the retained NPC-style wildlife cells; never enlarge source pixels."""
from pathlib import Path
import json

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/sprites/wildlife_sources/npc-style-refined.png"
ATTACK_SOURCE = SOURCE.with_name("npc-attack-refined.png")
CELL = 627
BASELINE = 590


def main():
    source = Image.open(SOURCE).convert("RGBA")
    attacks = Image.open(ATTACK_SOURCE).convert("RGBA")
    assert source.size == (CELL * 2, CELL * 2)
    assert attacks.size == source.size
    atlas = Image.new("RGBA", (CELL * 4, CELL * 5))
    frames = {}
    # Planted rear-paw landmarks, measured in the original 627px cells.
    # A lifted/swiping front paw must not shift the whole animal's ground anchor.
    idle_rear = [(140, 580), (109, 494)]
    attack_rear = [[(136, 568), (112, 564)], [(114, 478), (89, 486)]]
    for row in range(2):
        idle_offset = None
        for col, pose in enumerate(["idle", "walk", "attack_windup", "attack_strike"]):
            sheet = source if col < 2 else attacks
            source_col = col % 2
            frame = sheet.crop((source_col * CELL, row * CELL, (source_col + 1) * CELL, (row + 1) * CELL))
            # Common scale across every pose; padding keeps silhouettes off cell edges.
            frame.thumbnail((589, 589), Image.Resampling.LANCZOS)
            bounds = frame.getchannel("A").point(lambda a: 255 if a > 128 else 0).getbbox()
            assert bounds is not None
            offset = ((CELL - frame.width) // 2, BASELINE - bounds[3])
            if col == 0:
                idle_offset = offset
            elif col >= 2:
                assert idle_offset is not None
                rear = attack_rear[row][col - 2]
                offset = tuple(round(idle_offset[i] + (idle_rear[row][i] - rear[i]) * 589 / CELL)
                               for i in range(2))
            assert 0 <= bounds[0] + offset[0] < bounds[2] + offset[0] < CELL
            assert 0 <= bounds[1] + offset[1] < bounds[3] + offset[1] < CELL
            atlas.alpha_composite(frame, (col * CELL + offset[0], row * CELL + offset[1]))
            species = "wolf" if row == 0 else "bear"
            frames[f"{species}_{pose}"] = [col * CELL, row * CELL, CELL, CELL]
    boars = Image.open(SOURCE.with_name("boar-approved-animation.png")).convert("RGBA")
    assert boars.size == (CELL * 2, CELL * 2)
    for col, pose in enumerate(["idle", "walk", "attack_windup", "attack_strike"]):
        x, y = col % 2 * CELL, col // 2 * CELL
        frame = boars.crop((x, y, x + CELL, y + CELL))
        bounds = frame.getchannel("A").point(lambda a: 255 if a > 128 else 0).getbbox()
        assert bounds is not None
        # Register planted rear hooves for attacks, retaining the forward head thrust.
        rear_y = [491, 491, 451, 455][col]
        dy = 491 - rear_y if col >= 2 else 0
        idle_bottom = 521
        offset = (-24, BASELINE - idle_bottom + dy)
        assert bounds[3] + offset[1] < CELL
        atlas.alpha_composite(frame, (col * CELL + offset[0], 2 * CELL + offset[1]))
        frames[f"boar_{pose}"] = [col * CELL, 2 * CELL, CELL, CELL]
    # Lions: 512px source cells placed unenlarged; the lowest (planted) paws of
    # every pose sit on the shared baseline so attacks stay grounded.
    for row, name in [(3, "lioness"), (4, "lion")]:
        sheet = Image.open(ROOT / f"assets/sprites/lion_sources/{name}_cutout_0.png").convert("RGBA")
        assert sheet.size == (1024, 1024)
        for col, pose in enumerate(["idle", "walk", "attack_windup", "attack_strike"]):
            x, y = col % 2 * 512, col // 2 * 512
            frame = sheet.crop((x, y, x + 512, y + 512))
            bounds = frame.getchannel("A").point(lambda a: 255 if a > 128 else 0).getbbox()
            assert bounds is not None
            offset = ((CELL - 512) // 2, BASELINE - bounds[3])
            assert 0 <= bounds[1] + offset[1] and bounds[2] + offset[0] < CELL
            atlas.alpha_composite(frame, (col * CELL + offset[0], row * CELL + offset[1]))
            frames[f"{name}_{pose}"] = [col * CELL, row * CELL, CELL, CELL]
    atlas.save(ROOT / "assets/sprites/wildlife.png")
    (ROOT / "assets/sprites/wildlife.json").write_text(
        json.dumps({"size": list(atlas.size), "frames": frames}, indent=2) + "\n")


if __name__ == "__main__":
    main()
