"""Show registered idle/windup/strike frames at the same scale as NPCs."""
from pathlib import Path
from PIL import Image, ImageDraw
ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
atlas = Image.open(ROOT/'assets/sprites/wildlife.png').convert('RGBA')
npc = Image.open(ROOT/'assets/sprites/villager_idle_hd.png').convert('RGBA')
guard = Image.open(ROOT/'assets/sprites/units.png').convert('RGBA')
board = Image.new('RGB', (1500, 850), '#e9e2cf')
draw = ImageDraw.Draw(board)
for row, name in enumerate(['WOLF', 'BEAR']):
    top = row*425
    draw.text((25, top+20), name+'  /  IDLE → WINDUP → STRIKE', fill='#493d30')
    for column, source_column in enumerate([0,2,3]):
        tile = atlas.crop((source_column*627,row*627,(source_column+1)*627,(row+1)*627))
        tile.thumbnail((365,365), Image.Resampling.LANCZOS)
        board.paste(tile,(column*385+15,top+45),tile)
    for i, sheet in enumerate([npc,guard]):
        tile = sheet.crop((0,0,512,512))
        # Same renderer world ratio; bear cells represent 1.3 vs wolf 0.95 units.
        size = round(365*(.78*512/352)/(1.3 if row else .95))
        tile.thumbnail((size,size),Image.Resampling.LANCZOS)
        bbox=tile.getchannel('A').getbbox();tile=tile.crop(bbox)
        board.paste(tile,(1190+i*140,top+390-tile.height),tile)
    draw.text((1190,top+400),'EXISTING NPCs',fill='#493d30')
board.save(OUT/'attack-comparison.png')
# A timing preview of the same cells: recovery, windup, strike.
frames=[]
for column in [0,2,3]:
    preview=Image.new('RGB',(700,350),'#e9e2cf')
    for row,world_size in [(0,.95),(1,1.3)]:
        tile=atlas.crop((column*627,row*627,(column+1)*627,(row+1)*627))
        side=round(world_size*240)
        tile=tile.resize((side,side),Image.Resampling.LANCZOS)
        preview.paste(tile,(175+row*350-side//2,310-round(590/627*side)),tile)
    frames.append(preview)
frames[0].save(OUT/'attack-cycle.gif',save_all=True,append_images=frames[1:],duration=[100,500,400],loop=0,disposal=2)
alpha_check=Image.new('RGB',(1080,380))
for bg_index,bg in enumerate(['#e9e2cf','#526849','#426679']):
    ImageDraw.Draw(alpha_check).rectangle((bg_index*360,0,(bg_index+1)*360,380),fill=bg)
    for row in range(2):
        for col in range(2):
            tile=atlas.crop(((col+2)*627,row*627,(col+3)*627,(row+1)*627))
            tile.thumbnail((170,170),Image.Resampling.LANCZOS)
            alpha_check.paste(tile,(bg_index*360+col*180+5,row*180+15),tile)
alpha_check.save(OUT/'attack-alpha.png')
