"""Pack retained refinement sources using the standard icon normalizer."""
import argparse
import importlib.util
from pathlib import Path
from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
spec = importlib.util.spec_from_file_location("normalize", ROOT / "scripts/normalize_icons.py")
norm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(norm)
NAMES = ["command_cargo", "category_gathering", "category_town", "command_disembark"]


def pack(name):
    if name == "command_cargo":
        source = Image.open(HERE / "cargo-draft.png").convert("RGBA")
    else:
        sheet = Image.open(HERE / "simplified-three-icon-draft.png").convert("RGBA")
        cell = sheet.width // 3
        index = NAMES.index(name) - 1
        source = sheet.crop((index * cell, 0, (index + 1) * cell, sheet.height))
    return norm.normalize(source)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("names", nargs="*", choices=NAMES)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    for name in args.names or NAMES:
        path = args.output / (name + ".png")
        pack(name).save(path, optimize=True)
        assert not norm.problems(path), norm.problems(path)
        print(path)
