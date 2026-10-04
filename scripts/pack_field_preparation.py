#!/usr/bin/env python3
"""Pack original generated pose pairs without enlarging source pixels."""
import json
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[1] / "assets/sprites"
SOURCES = ROOT / "field_preparation_sources"
PEOPLE = ("villager", "villager_woman", "villager_elder")


def main():
    atlas = Image.new("RGBA", (2048, 1536))
    people = {}
    for row, who in enumerate(PEOPLE):
        frames = []
        for pair in range(2):
            source = Image.open(SOURCES / f"{who}_{pair}.png").convert("RGBA")
            assert source.width // 2 >= 512 and source.height >= 512
            for side in range(2):
                frame = source.crop((side * source.width // 2, 0,
                                     (side + 1) * source.width // 2, source.height))
                # Ignore imperceptible alpha when measuring, but preserve the
                # original antialiased cutout inside the measured rectangle.
                box = frame.getchannel("A").point(lambda a: 255 if a > 128 else 0).getbbox()
                assert box is not None
                frames.append(frame.crop(box))
        # One scale for the whole cycle; waist bends must not resize the body.
        scale = min(352 / max(frame.height for frame in frames),
                    440 / max(frame.width for frame in frames))
        assert scale <= 1, "HD frames must come from original detail"
        for column, frame in enumerate(frames):
            frame = frame.resize((round(frame.width * scale), round(frame.height * scale)),
                                 Image.Resampling.LANCZOS)
            # Keep the connected figure/tool cutout and its antialiased edge;
            # discard isolated generation specks before anchoring the feet.
            alpha = frame.getchannel("A")
            mask = alpha.point(lambda a: 255 if a > 128 else 0)
            seed = (round(frame.width * 0.65), frame.height // 2)
            assert mask.getpixel(seed) == 255
            ImageDraw.floodfill(mask, seed, 128)
            mask = mask.point(lambda a: 255 if a == 128 else 0).filter(ImageFilter.MaxFilter(5))
            frame.putalpha(ImageChops.multiply(alpha, mask))
            # Sandals are the lowest opaque pixels; centre their span rather
            # than the tool, so the villager's ground anchor stays fixed.
            feet = frame.getchannel("A").crop((0, frame.height - 12, frame.width, frame.height))
            box = feet.point(lambda a: 255 if a > 128 else 0).getbbox()
            assert box is not None
            anchor_x = (box[0] + box[2]) // 2
            x, y = 256 - anchor_x, 480 - frame.height
            assert x >= 0 and x + frame.width <= 512 and y >= 0
            atlas.alpha_composite(frame, (column * 512 + x, row * 512 + y))
        people[who] = {"front": [[x * 512, row * 512, 512, 512] for x in range(4)]}
    atlas.save(ROOT / "villager_field_preparation.png")
    (ROOT / "villager_field_preparation.json").write_text(json.dumps({
        "size": [2048, 1536], "cell": [512, 512], "anchor": [256, 480],
        "figureHeight": 352, "fps": 4, "people": people,
    }, indent=2) + "\n")
    preview = []
    for column in range(4):
        frame = Image.new("RGBA", (384, 128), (231, 223, 193, 255))
        for row in range(3):
            pose = atlas.crop((column * 512, row * 512,
                               (column + 1) * 512, (row + 1) * 512))
            frame.alpha_composite(pose.resize((128, 128), Image.Resampling.LANCZOS),
                                  (row * 128, 0))
        preview.append(frame.convert("RGB"))
    preview[0].save(ROOT / "villager_field_preparation_preview.gif", save_all=True,
                    append_images=preview[1:], duration=250, loop=0, disposal=2)


if __name__ == "__main__":
    main()
