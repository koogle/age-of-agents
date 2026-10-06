"""Equal-size before/after evidence for the requested map line edit only."""
import sys
from review import BASE,ROOT,BG,Image,ImageDraw,stamp
batch=sys.argv[1] if len(sys.argv)>1 else '12c-openai-map-refinement'
before=BASE/'11d-map-shield-targeted-retries/command_explore/cutout.png'
after=BASE/batch/'command_explore/cutout.png'
out=Image.new('RGB',(1050,790),'#d6d8d7');d=ImageDraw.Draw(out)
d.text((15,10),'MAP FINE INK | FAL original before / disclosed OpenAI refinement after | UNAPPROVED',fill='#202020')
d.text((15,35),'Before / after at equal128px and24/32px. Original villager and approved UI ink references.',fill='#202020')
for j,p in enumerate(['assets/sprites/villager_master.jpg','assets/ui/icons/resource_wood.png','assets/ui/icons/resource_food.png','assets/ui/icons/tech_masonry.png']):stamp(out,ROOT/p,15+j*140,65,110)
for row,bg in enumerate(BG):
 y=205+row*143;d.rectangle((10,y,1035,y+140),fill=bg)
 color='#202020' if row<2 else '#ffffff'
 d.text((20,y+4),'Before128',fill=color);d.text((200,y+4),'After128',fill=color)
 stamp(out,before,20,y+12,128);stamp(out,after,200,y+12,128)
 for m,size in enumerate([24,32]):
  x=420+m*270;d.text((x,y+12),f'Before / after {size}px',fill=color)
  stamp(out,before,x,y+52,size);stamp(out,after,x+95,y+52,size)
out.save(BASE/'map-line-weight-comparison.jpg',quality=96)
