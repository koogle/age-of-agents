"""Rebuild contact_sheet.png from manifest.json (run from assets/ui)."""
from PIL import Image, ImageDraw
import json
m = json.load(open("manifest.json"))
items = [(g, n, p) for g, d in m["icons"].items() for n, p in d.items()]
cols, cw, ch = 6, 236, 150
rows = (len(items)+cols-1)//cols
sheet = Image.new("RGB", (cols*cw+20, rows*ch+380+20), (250,246,238))
def checker(w,h):
    c=Image.new("RGB",(w,h),(214,214,214)); d=ImageDraw.Draw(c)
    for y in range(0,h,8):
        for x in range(0,w,8):
            if (x//8+y//8)%2: d.rectangle((x,y,x+7,y+7),fill=(255,255,255))
    return c
def paper(w,h):
    t=Image.open("paper_tile.png").convert("RGBA"); c=Image.new("RGBA",(w,h))
    for y in range(0,h,512):
        for x in range(0,w,512): c.paste(t,(x,y))
    return c
for i,(g,n,p) in enumerate(items):
    im = Image.open(p); ox, oy = 10+(i%cols)*cw, 10+(i//cols)*ch
    sheet.paste(checker(128,128),(ox,oy)); sheet.paste(im,(ox,oy),im)
    for j,bg in enumerate([(240,229,206),(58,46,36)]):
        t = Image.new("RGB",(40,128),bg)
        for sz,yy in ((32,14),(24,62),(20,98)):
            s=im.resize((sz,sz),Image.LANCZOS); t.paste(s,((40-sz)//2,yy),s)
        sheet.paste(t,(ox+134+j*44,oy))
y0 = 10+rows*ch+10
sheet.paste(paper(360,360).convert("RGB"),(10,y0))
fr = Image.open("border_frame.png"); bg = checker(310,310); bg.paste(fr,(0,0),fr); sheet.paste(bg,(390,y0+25))
W,H=620,310; panel = paper(W,H); B=m["textures"]["border_frame"]["nine_slice"]["inset"]; S=fr.size[0]; L=S-2*B
pc = lambda x0,y0,x1,y1: fr.crop((x0,y0,x1,y1))
for x in range(B,W-B,L):
    w=min(L,W-B-x); panel.alpha_composite(pc(B,0,B+w,B),(x,0)); panel.alpha_composite(pc(B,S-B,B+w,S),(x,H-B))
for y in range(B,H-B,L):
    h=min(L,H-B-y); panel.alpha_composite(pc(0,B,B,B+h),(0,y)); panel.alpha_composite(pc(S-B,B,S,B+h),(W-B,y))
for sx,sy,dx,dy in [(0,0,0,0),(S-B,0,W-B,0),(0,S-B,0,H-B),(S-B,S-B,W-B,H-B)]: panel.alpha_composite(pc(sx,sy,sx+B,sy+B),(dx,dy))
for i,p in enumerate([p for g,n,p in items if g=="resource"]):
    panel.alpha_composite(Image.open(p).resize((40,40),Image.LANCZOS),(60+i*72,64))
for i,p in enumerate([p for g,n,p in items if g in ("tech","command")]):
    panel.alpha_composite(Image.open(p).resize((32,32),Image.LANCZOS),(60+i*62,146))
for i,p in enumerate([p for g,n,p in items if g=="portrait"]):
    panel.alpha_composite(Image.open(p).resize((56,56),Image.LANCZOS),(60+i*80,206))
for i,p in enumerate([p for g,n,p in items if g=="resource"]):
    panel.alpha_composite(Image.open(p).resize((24,24),Image.LANCZOS),(330+i*34,222))
sheet.paste(panel.convert("RGB"),(720,y0+25))
sheet.quantize(256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save("contact_sheet.png", optimize=True)
