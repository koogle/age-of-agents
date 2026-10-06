"""Offline current/rejected/calibration evidence for era and cel corrections."""
from review import *
V=ROOT/'assets/sprites/villager_master.jpg'

def sheet(filename,title,rows,preceding_label="REJECTED candidate",latest_label="NEW unapproved calibration",small_label="Small-scale old / reject / new"):
 out=Image.new('RGB',(1200,100+len(rows)*320),'#d6d8d7');d=ImageDraw.Draw(out)
 d.text((15,10),title,fill='#202020')
 for x,t in [(15,'RUNTIME (unchanged)'),(255,preceding_label),(495,latest_label),(735,'Original villager'),(925,small_label)]:d.text((x,45),t,fill='#202020')
 for n,(name,old,reject,new,kind,note) in enumerate(rows):
  y=90+n*320;d.text((15,y),name+' | '+note,fill='#8d2828')
  for x,p in [(15,old),(255,reject),(495,new)]:stamp(out,p,x,y+25,215)
  stamp(out,V,755,y+35,135)
  for k,bg in enumerate(['#f0e5ce','#173c34','#437995']):
   yy=y+30+k*85;d.rectangle((925,yy,1190,yy+78),fill=bg)
   for col,p in enumerate([old,reject,new]):
    for m,size in enumerate([24,32] if kind=='icon' else [64]):stamp(out,p,930+col*85,yy+3+m*36,size)
 out.save(BASE/filename,quality=94)

if __name__=='__main__':
 sheet('cel-correction-comparison.jpg','CEL CALIBRATION: broad expansion paused; source and small-scale comparisons',[
  ('Steel',source('resource_steel'),BASE/'02-resources/resource_steel/cutout.png',BASE/'10-cel-calibration/resource_steel/cutout.png','icon','simpler planes; edge shine/gradient still needs correction'),
  ('Cloth',source('resource_cloth'),BASE/'02-resources/resource_cloth/cutout.png',BASE/'10-cel-calibration/resource_cloth/cutout.png','icon','matte broad blue areas; no satin shine'),
  ('Wolf',BASE/'wolf-current-crop.png',BASE/'01-calibration/wolf_idle/cutout.png',BASE/'10-cel-calibration/wolf_idle/cutout.png','sprite','quiet fur masses; changed facing/proportions remain unapproved')])
 sheet('era-correction-comparison.jpg','ERA CALIBRATION: magnetic compass and modern-looking lace-up boot rejected',[
  ('Explore',source('command_explore'),BASE/'03-menu/command_explore/cutout.png',BASE/'09b-era-explore-calibration/command_explore/cutout.png','icon','runtime compass ERA REJECTED; new celestial star + map, no instrument'),
  ('Disembark',source('command_disembark'),BASE/'03-menu/command_disembark/cutout.png',BASE/'09-era-corrections/command_disembark/cutout.png','icon','modern candidate ERA REJECTED; new simple sandal on pier')])
