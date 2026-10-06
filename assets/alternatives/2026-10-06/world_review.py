"""World candidate comparisons and mirrored texture repeats, for review only."""
from review import *
B=BASE
rows=[('Wolf',ROOT/'assets/sprites/wildlife.png',(0,0,627,627),B/'01-calibration/wolf_idle/cutout.png'),('Bear',ROOT/'assets/sprites/wildlife.png',(0,627,627,1254),B/'05-world/bear_idle/cutout.png'),('Dock front',ROOT/'assets/sprites/building_sources/dock_complete.png',None,B/'05-world/dock_front/cutout.png'),('Dock rear',ROOT/'assets/sprites/building_sources/directions/dock_north_complete.png',None,B/'07-targeted-repairs/dock_rear_single_sail/cutout.png')]
sheet=Image.new('RGB',(1100,1300),'#d6d8d7');d=ImageDraw.Draw(sheet)
d.text((20,10),'UNAPPROVED WORLD ALTERNATIVES | current left, candidate middle; small-scale previews right',fill='#202020')
for j,(name,old,box,new) in enumerate(rows):
 y=40+j*310;d.text((20,y),name,fill='#202020')
 if box:
  dst=B/(name.lower()+'-current-crop.png');Image.open(old).crop(box).save(dst);old=dst
 stamp(sheet,old,15,y+25,270);stamp(sheet,new,330,y+25,270)
 for k,bg in enumerate(['#829765','#f0e5ce']):
  x=650+k*220;d.rectangle((x,y+30,x+210,y+270),fill=bg)
  stamp(sheet,old,x+5,y+80,95);stamp(sheet,new,x+110,y+80,95)
  stamp(sheet,ROOT/'assets/sprites/villager_master.jpg',x+75,y+180,60)
sheet.save(B/'world-comparison.jpg',quality=94)
sheet=Image.new('RGB',(1100,1150),'#d6d8d7');d=ImageDraw.Draw(sheet)
for j,(name,cand) in enumerate([('road_stone',B/'01-calibration/road_stone/original.png'),('road_dirt',B/'05-world/road_dirt/original.png')]):
 y=20+j*570;d.text((20,y),name+' | current / candidate / mirrored 2x2 candidate',fill='#202020')
 for k,p in enumerate([ROOT/f'assets/terrain/{name}.png',cand]):
  im=Image.open(p).convert('RGB').resize((320,320),Image.Resampling.LANCZOS);sheet.paste(im,(20+k*340,y+30))
 im=Image.open(cand).convert('RGB').resize((180,180),Image.Resampling.LANCZOS)
 for dy in range(2):
  for dx in range(2):
   a=ImageOps.mirror(im) if dx else im
   a=ImageOps.flip(a) if dy else a
   sheet.paste(a,(710+dx*180,y+30+dy*180))
 d.text((20,y+420),'Source swatches only; runtime mirrored/clamped sampling not changed. No gameplay acceptance claimed.',fill='#202020')
sheet.save(B/'terrain-comparison.jpg',quality=94)
