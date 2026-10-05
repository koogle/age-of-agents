"""Pack FAL renders/cutouts as matched 256px buttons; repair enclosed mask holes."""
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

HERE = Path(__file__).resolve().parent
images = []
for name in ('up', 'down'):
    original = np.array(Image.open(HERE / f'{name}-original.png').convert('RGBA'))
    cutout = np.array(Image.open(HERE / f'{name}-cutout.png').convert('RGBA'))
    # BiRefNet can mistake the pale inset for background. Only fill regions
    # enclosed by its opaque shield silhouette, using the original FAL pixels.
    mask = Image.fromarray(np.where(cutout[..., 3] > 32, 255, 0).astype('uint8')).copy()
    ImageDraw.floodfill(mask, (0, 0), 128, thresh=0)
    interior = np.array(mask) != 128
    cutout[interior, :3] = original[interior, :3]
    cutout[interior, 3] = 255
    image = Image.fromarray(cutout)
    # Remove the two-pixel white matte fringe before downsampling.
    image.putalpha(image.getchannel("A").filter(ImageFilter.MinFilter(5)))
    images.append(image)
boxes = [im.getchannel('A').point(lambda a: 255 if a > 32 else 0).getbbox() for im in images]
box = (min(b[0] for b in boxes), min(b[1] for b in boxes), max(b[2] for b in boxes), max(b[3] for b in boxes))
fit = 224 / max(box[2]-box[0], box[3]-box[1])
size = (round((box[2]-box[0])*fit), round((box[3]-box[1])*fit))
for name, image in zip(('up','down'),images):
    button = Image.new('RGBA',(256,256))
    button.alpha_composite(image.crop(box).resize(size,Image.Resampling.LANCZOS),((256-size[0])//2,(256-size[1])//2))
    button.save(HERE.parent.parent.parent / 'buttons' / f'shield_{name}.png')
