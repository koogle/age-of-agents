#!/usr/bin/env python3
"""Check the authored field-work cycle, cutouts and shared baseline."""
import json
from pathlib import Path

from PIL import Image

from check_sprite_resolution import audit

ROOT = Path(__file__).resolve().parents[1] / "assets/sprites"


def main():
    manifest = ROOT / "villager_field_preparation.json"
    assert audit(manifest, 512) == (12, 0)
    data = json.loads(manifest.read_text())
    atlas = Image.open(ROOT / "villager_field_preparation.png")
    assert atlas.format == "PNG" and atlas.mode == "RGBA"
    assert list(atlas.size) == data["size"]
    for who, animation in data["people"].items():
        poses = []
        for x, y, width, height in animation["front"]:
            frame = atlas.crop((x, y, x + width, y + height))
            alpha = frame.getchannel("A")
            assert all(alpha.getpixel(p) == 0 for p in ((0, 0), (511, 0), (0, 511), (511, 511)))
            box = alpha.point(lambda a: 255 if a > 128 else 0).getbbox()
            assert box is not None and 478 <= box[3] <= 481, (who, box)
            assert box[0] > 0 and box[1] > 0 and box[2] < 512
            poses.append(frame.tobytes())
        assert len(set(poses)) == 4, f"{who}: duplicate poses"
        for pair in range(2):
            source = Image.open(ROOT / "field_preparation_sources" / f"{who}_{pair}.png")
            assert source.width // 2 >= 512 and source.height >= 512
    print("PASS field preparation: 12 distinct transparent HD poses, aligned feet, original HD sources")


if __name__ == "__main__":
    main()
