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



def thin_lines(image, shave=1, lighten=0.35):
    """Shave generated ink strokes to the family's fine line and lift them toward warm brown.

    Thin dark strokes vanish under a morphological opening while broad dark areas
    (tail tufts, ear backs, mouths) survive it and stay untouched. Shaved pixels
    take the colour of the nearest opaque non-line pixel.
    """
    px = np.asarray(image).astype(float)
    opaque = px[..., 3] > 128
    dark = opaque & (px[..., :3] @ [0.299, 0.587, 0.114] / 255 < 0.42)
    broad = ndimage.binary_opening(dark, iterations=5)
    line = dark & ~ndimage.binary_dilation(broad, iterations=2)
    core = ndimage.binary_erosion(line, iterations=shave)
    removed = line & ~core
    _, (iy, ix) = ndimage.distance_transform_edt(~(opaque & ~line), return_indices=True)
    out = px.copy()
    out[removed, :3] = px[iy[removed], ix[removed], :3]
    out[core, :3] = out[core, :3] * (1 - lighten) + np.array([96.0, 62.0, 40.0]) * lighten
    return Image.fromarray(out.clip(0, 255).astype(np.uint8), "RGBA")


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
    # Lions: 1024px source quadrants, downsampled by one common factor (never
    # enlarged). A pose is every opaque shape whose center lies in its quadrant,
    # so a tail crossing the quadrant edge stays with its body. The lowest
    # (planted) paws sit on the shared baseline.
    half, scale = 1024, 589 / 1024
    for row, name, source in [(3, "lioness", "huntress_natural"), (4, "lion", "nemean_natural")]:
        sheet = Image.open(ROOT / f"assets/sprites/lion_sources/{source}_cutout.png").convert("RGBA")
        assert sheet.size == (2 * half, 2 * half)
        sheet = thin_lines(sheet)
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
            atlas.alpha_composite(cell, (col * CELL, row * CELL))
            frames[f"{name}_{pose}"] = [col * CELL, row * CELL, CELL, CELL]
    atlas.save(ROOT / "assets/sprites/wildlife.png")
    (ROOT / "assets/sprites/wildlife.json").write_text(
        json.dumps({"size": list(atlas.size), "frames": frames}, indent=2) + "\n")


if __name__ == "__main__":
    main()
