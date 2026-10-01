from PIL import Image, ImageDraw
import numpy as np, glob, os
def norm(f, size=128, pad=0.06):
    im = Image.open(f).convert("RGBA"); a = np.array(im)
    al = a[...,3].astype(np.float32)
    al[al < 12] = 0                                   # kill faint matte noise
    a[...,3] = al.astype(np.uint8)
    im = Image.fromarray(a)
    x0,y0,x1,y1 = im.getchannel("A").point(lambda v: 255 if v>40 else 0).getbbox()
    side = max(x1-x0, y1-y0); side = int(side*(1+2*pad))
    cx, cy = (x0+x1)//2, (y0+y1)//2
    canvas = Image.new("RGBA", (side, side), (0,0,0,0))
    canvas.paste(im.crop((cx-side//2, cy-side//2, cx-side//2+side, cy-side//2+side)), (0,0))
    return canvas.resize((size,size), Image.LANCZOS)
if __name__ == "__main__":
    for f in sorted(glob.glob("cut/*.png")):
        norm(f).save("out/"+os.path.basename(f), optimize=True)
