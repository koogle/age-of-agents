import sys, glob, os
from concurrent.futures import ThreadPoolExecutor
from falcall import run, download, data_uri
def go(f):
    k=os.path.basename(f)
    try:
        o=run("fal-ai/birefnet/v2",{"image_url":data_uri(f),"model":"General Use (Heavy)","operating_resolution":"1024x1024","output_format":"png","refine_foreground":True},0.005,"cut:"+k)
        download(o["image"]["url"],"cut/"+k); return k,"ok"
    except Exception as e: return k,str(e)[:300]
fs=sys.argv[1:] or sorted(glob.glob("icons/*.png"))
with ThreadPoolExecutor(6) as ex:
    for r in ex.map(go,fs): print(r)
