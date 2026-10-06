"""Offline candidate comparison sheets; does not touch runtime images."""
import json, math, html
from pathlib import Path
from PIL import Image, ImageDraw, ImageOps
BASE=Path(__file__).resolve().parent
ROOT=BASE.parents[2]
BG=['#d6d8d7','#f0e5ce','#173c34','#437995']

def fit(path,size):
 im=Image.open(path).convert('RGBA')
 if im.getextrema()[3][0]<255:
  box=im.getchannel('A').point(lambda x:255 if x>40 else 0).getbbox()
  if box: im=im.crop(box)
 im.thumbnail((size,size),Image.Resampling.LANCZOS)
 out=Image.new('RGBA',(size,size));out.alpha_composite(im,((size-im.width)//2,(size-im.height)//2));return out

def stamp(canvas,path,x,y,size):
 canvas.paste(fit(path,size),(x,y),fit(path,size))

def source(name):
 family='buttons' if name.startswith(('shield_','coin_')) else 'icons'
 return ROOT/f'assets/ui/{family}/{name}.png'

if __name__=='__main__':
 batches=json.loads((BASE/'batches.json').read_text()); gallery=[]
 for batch,items in batches.items():
  icons=[i for i in items if i['family']=='icon' and (BASE/batch/i['name']/'cutout.png').exists()]
  if not icons:continue
  sheet=Image.new('RGB',(1100,150+len(icons)*190),'#d6d8d7');d=ImageDraw.Draw(sheet)
  d.text((15,10),batch+' | UNAPPROVED alternatives | original villager + UI refs',fill='#202020')
  for n,p in enumerate(['assets/sprites/villager_master.jpg','assets/ui/icons/resource_wood.png','assets/ui/icons/resource_food.png','assets/ui/icons/tech_masonry.png']):stamp(sheet,ROOT/p,20+n*130,35,95)
  for row,i in enumerate(icons):
   y=150+190*row; name=i['name'];cand=BASE/batch/name/'cutout.png'; old=source(name)
   d.text((320,y),i.get('review_status','UNAPPROVED'),fill='#9b2525');d.text((10,y),name,fill='#202020');d.text((15,y+20),'current',fill='#202020');d.text((170,y+20),'candidate',fill='#202020')
   stamp(sheet,old,10,y+43,128);stamp(sheet,cand,165,y+43,128)
   for k,bg in enumerate(BG):
    x=320+190*k;d.rectangle((x,y+20,x+180,y+177),fill=bg)
    d.text((x+6,y+24),'old / new  24/32px',fill='white' if k>1 else '#303030')
    for m,size in enumerate([24,32]):
     stamp(sheet,old,x+15,y+53+m*56,size);stamp(sheet,cand,x+85,y+53+m*56,size)
   gallery.append((batch,name))
  sheet.save(BASE/(batch+'-comparison.jpg'),quality=93)
 links=['<!doctype html><meta charset="utf-8"><title>Unapproved art alternatives</title><style>body{font:16px system-ui;background:#d6d8d7;margin:30px}img{max-width:100%}section{margin-bottom:32px}a{color:#124565}</style><h1>Unapproved alternatives — 6 October 2026</h1><p>Original villagers and runtime assets unchanged. Latest cel and era corrections appear first; red captions mark rejected historical/style attempts retained below. No gameplay approval.</p>']
 for p in sorted(BASE.glob('*comparison.jpg'),key=lambda p:(-1 if p.name.startswith('painted-') else 0 if p.name.startswith(('cel-','era-')) else 1,p.name)):links.append(f'<h2>{html.escape(p.stem)}</h2><img src="{p.name}">')
 for batch,items in batches.items():
  for i in items:
   folder=BASE/batch/i['name']
   if not (folder/'original.png').exists():continue
   rel=f'{batch}/{i["name"]}'
   links.append(f'<section><h2>{html.escape(rel)}</h2><p><strong>{html.escape(i["review_status"])}</strong></p><a href="{rel}/original.png">Full original</a> · <a href="{rel}/provenance.json">Prompt/references</a><br><img width="420" src="{rel}/original.png"></section>')
 (BASE/'index.html').write_text('\n'.join(links))
 print('Review sheets written for',len(gallery),'icon attempts')
