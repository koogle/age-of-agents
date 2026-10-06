from PIL import Image,ImageDraw
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=Path(__file__).resolve().parent
npc=Image.open(root/'assets/sprites/villager_idle_hd.png').convert('RGBA')
guard=Image.open(root/'assets/sprites/units.png').convert('RGBA')
old=Image.open(root/'assets/sprites/wildlife_sources/refined.png').convert('RGBA')
new=Image.open(root/'assets/sprites/wildlife.png').convert('RGBA')
items=[('Villager',npc.crop((0,0,512,512)),.78/352),('Woman',npc.crop((0,512,512,1024)),.78/352),('Elder',npc.crop((0,1024,512,1536)),.78/352),('Guard',guard.crop((0,0,512,512)),.78/352),('Old wolf',old.crop((0,0,627,627)),.95/627),('New wolf',new.crop((0,0,627,627)),.95/627),('Old bear',old.crop((0,627,627,1254)),1.3/627),('New bear',new.crop((0,627,627,1254)),1.3/627)]
canvas=Image.new('RGB',(1600,760),'#ece6d4');d=ImageDraw.Draw(canvas)
d.text((20,16),'EXISTING NPCs / SHIPPED ANIMALS / NPC-MATCHED REFINEMENT',fill='#443b2d')
for row,(bg,scale,title) in enumerate([('#ece6d4',150,'Close comparison (common world scale)'),('#56674b',65,'Gameplay-scale comparison'),('#466572',65,'Blue alpha check')]):
 top=[50,340,535][row];height=[275,170,170][row];d.rectangle((0,top,1600,top+height),fill=bg);d.text((20,top+8),title,fill='#443b2d' if row==0 else '#fff4df')
 for i,(name,im,worldpx) in enumerate(items):
  bounds=im.getchannel('A').point(lambda a:255 if a>128 else 0).getbbox();cut=im.crop(bounds);cut=cut.resize((max(1,round(cut.width*worldpx*scale)),max(1,round(cut.height*worldpx*scale))),Image.Resampling.LANCZOS)
  x=100+i*200;baseline=top+height-35;canvas.paste(cut,(x-cut.width//2,baseline-cut.height),cut);d.text((x-35,baseline+12),name,fill='#443b2d' if row==0 else '#fff4df')
canvas.save(out/'npc-comparison.png')
