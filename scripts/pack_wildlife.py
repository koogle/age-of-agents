#!/usr/bin/env python3
"""Register the retained NPC-style wildlife cells; never enlarge source pixels."""
from pathlib import Path
import json

import numpy as np
from PIL import Image
from scipy import ndimage

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets/sprites/wildlife_sources/npc-style-refined.png"
ATTACK_SOURCE = SOURCE.with_name("npc-attack-refined.png")
CELL = 627
BASELINE = 590
# Two species per atlas row (four poses each) keep the atlas square and within
# common 8192px WebGL texture limits.
SPECIES = 16
POSES = ["idle", "walk", "attack_windup", "attack_strike"]


def origin(species, col):
    return ((species % 2) * 4 + col) * CELL, (species // 2) * CELL




def main():
    source = Image.open(SOURCE).convert("RGBA")
    attacks = Image.open(ATTACK_SOURCE).convert("RGBA")
    assert source.size == (CELL * 2, CELL * 2)
    assert attacks.size == source.size
    atlas = Image.new("RGBA", (CELL * 8, CELL * SPECIES // 2))
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
            x, y = origin(row, col)
            atlas.alpha_composite(frame, (x + offset[0], y + offset[1]))
            species = "wolf" if row == 0 else "bear"
            frames[f"{species}_{pose}"] = [x, y, CELL, CELL]
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
        ax, ay = origin(2, col)
        atlas.alpha_composite(frame, (ax + offset[0], ay + offset[1]))
        frames[f"boar_{pose}"] = [ax, ay, CELL, CELL]
    # Lions, snakes, barbarians and monsters: 1024px source quadrants, downsampled by one common factor
    # (never enlarged) and otherwise untouched: line work and shading are the
    # image model's job, per Jakob (2026-10-10), not pixel processing here. A pose is every opaque shape whose center lies in its quadrant,
    # so a tail crossing the quadrant edge stays with its body. The lowest
    # (planted) paws sit on the shared baseline.
    half = 1024
    sheets = [(3, "lioness", "lion_sources/huntress_ink"), (4, "lion", "lion_sources/nemean_ink"),
              (5, "python", "snake_sources/rock_python_ink"), (6, "sea_serpent", "snake_sources/sea_serpent_ink"),
              (7, "viper", "snake_sources/marble_viper_ink"),
              (8, "barbarian", "barbarian_sources/raider"), (9, "chieftain", "barbarian_sources/chieftain"),
              (10, "torchbearer", "barbarian_sources/torchbearer"),
              (11, "skeleton", "monster_sources/skeleton"), (12, "centaur", "monster_sources/centaur")]
    for row, name, source in sheets:
        scale = 589 / 1024
        sheet = Image.open(ROOT / f"assets/sprites/{source}_cutout.png").convert("RGBA")
        assert sheet.size == (2 * half, 2 * half)
        alpha = np.asarray(sheet.getchannel("A")) > 128
        labels, count = ndimage.label(alpha)
        centers = ndimage.center_of_mass(alpha, labels, range(1, count + 1))
        pixels = np.asarray(sheet)
        for col, pose in enumerate(["idle", "walk", "attack_windup", "attack_strike"]):
            x, y = col % 2 * half, col // 2 * half
            shapes = [i + 1 for i, (cy, cx) in enumerate(centers)
                      if x <= cx < x + half and y <= cy < y + half]
            # Keep soft edge pixels of the pose's own shapes only.
            mask = ndimage.binary_dilation(np.isin(labels, shapes), iterations=4)
            frame = Image.fromarray(np.where(mask[..., None], pixels, 0).astype(np.uint8), "RGBA")
            frame = frame.crop((x - half // 4, y, x + half + half // 4, y + half))
            frame = frame.resize((round(frame.width * scale), round(frame.height * scale)),
                                 Image.Resampling.LANCZOS)
            bounds = frame.getchannel("A").point(lambda a: 255 if a > 128 else 0).getbbox()
            assert bounds is not None
            offset = ((CELL - round(half * scale)) // 2 - round(half // 4 * scale), BASELINE - bounds[3])
            assert 0 <= bounds[0] + offset[0] and bounds[2] + offset[0] < CELL
            assert 0 <= bounds[1] + offset[1]
            cell = Image.new("RGBA", (CELL, CELL))
            cell.alpha_composite(frame.crop(bounds), (bounds[0] + offset[0], bounds[1] + offset[1]))
            ax, ay = origin(row, col)
            atlas.alpha_composite(cell, (ax, ay))
            frames[f"{name}_{pose}"] = [ax, ay, CELL, CELL]
    # The giants: one cutout per pose, drawn large so their pen lines stay as fine
    # as the family's at display size. One factor per species fits its widest and
    # tallest pose in the cell; each pose's mass centre sits on the cell centre and
    # its lowest opaque row on the baseline.
    for row, name in [(13, "cyclops"), (14, "cyclops_general"), (15, "minotaur")]:
        poses = [Image.open(ROOT / f"assets/sprites/monster_sources/{name}_{pose}_cutout.png").convert("RGBA")
                 for pose in POSES]
        boxes = [p.getchannel("A").point(lambda a: 255 if a > 128 else 0).getbbox() for p in poses]
        scale = min(min((CELL - 24) / (b[2] - b[0]), (BASELINE - 12) / (b[3] - b[1])) for b in boxes)
        for col, (pose, frame, box) in enumerate(zip(POSES, poses, boxes)):
            frame = frame.crop(box).convert("RGBa")
            frame = frame.resize((round(frame.width * scale), round(frame.height * scale)),
                                 Image.Resampling.LANCZOS).convert("RGBA")
            alpha = np.asarray(frame.getchannel("A")) > 128
            cx = ndimage.center_of_mass(alpha)[1]
            x = min(max(round(CELL / 2 - cx), 4), CELL - frame.width - 4)
            cell = Image.new("RGBA", (CELL, CELL))
            cell.alpha_composite(frame, (x, BASELINE - frame.height))
            ax, ay = origin(row, col)
            atlas.alpha_composite(cell, (ax, ay))
            frames[f"{name}_{pose}"] = [ax, ay, CELL, CELL]
    atlas.save(ROOT / "assets/sprites/wildlife.png")
    (ROOT / "assets/sprites/wildlife.json").write_text(
        json.dumps({"size": list(atlas.size), "frames": frames}, indent=2) + "\n")


if __name__ == "__main__":
    main()
