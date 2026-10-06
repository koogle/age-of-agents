"""Normalize the retained Disembark platform refinement without repainting."""
from pathlib import Path
import importlib.util
from PIL import Image
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
spec=importlib.util.spec_from_file_location('normalize',ROOT/'scripts/normalize_icons.py')
norm=importlib.util.module_from_spec(spec);spec.loader.exec_module(norm)
if __name__=='__main__':
 output=HERE/'command_disembark.png'
 norm.normalize(Image.open(HERE/'refined.png').convert('RGBA')).save(output,optimize=True)
 assert not norm.problems(output),norm.problems(output)
 print(output)
