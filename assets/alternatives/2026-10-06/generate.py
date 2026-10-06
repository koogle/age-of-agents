"""Explicit candidate batches; paid calls only for the batch named on the CLI."""
import json, hashlib, sys
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'assets/ui/tools'))
import falcall
falcall.LEDGER=str(OUT/'ledger.jsonl')
def generate(item,batch):
 dest=OUT/batch/item['name']; dest.mkdir(parents=True,exist_ok=True)
 if (dest/'original.png').exists() and (item['family']=='texture' or (dest/'cutout.png').exists()): return item['name']+' exists'
 refs=[dict(path=p,sha256=hashlib.sha256((ROOT/p).read_bytes()).hexdigest()) for p in item['references']]
 if (dest/'response.json').exists():
  # A resumed download must retain the prompt that actually produced the image.
  result=json.loads((dest/'response.json').read_text())
 else:
  templates=[item.get('era_prompt','era-prompt.txt'),item.get('cel_prompt','cel-prompt.txt')]
  prompt=item['prompt']+' '+' '.join((OUT/t).read_text().strip() for t in templates)
  record=dict(item,prompt=prompt,model='fal-ai/nano-banana/edit',reference_revision='1984e9b',references=refs,status='unapproved candidate; no runtime replacement',prompt_templates=[dict(path=t,sha256=hashlib.sha256((OUT/t).read_bytes()).hexdigest()) for t in templates])
  (dest/'provenance.json').write_text(json.dumps(record,indent=2)+'\n')
  args=dict(prompt=prompt,image_urls=[falcall.data_uri(ROOT/p['path']) for p in refs],num_images=1,output_format='png',aspect_ratio=item['aspect_ratio'])
  result=falcall.run(record['model'],args,.0398,f'alternative:{batch}:{item["name"]}')
  (dest/'response.json').write_text(json.dumps(result,indent=2)+'\n')
 falcall.download(result['images'][0]['url'],str(dest/'original.png'))
 if item['family']!='texture':
  cut=falcall.run('fal-ai/birefnet/v2',dict(image_url=falcall.data_uri(dest/'original.png'),model='General Use (Heavy)',operating_resolution='1024x1024',refine_foreground=True),.005,f'cutout:{batch}:{item["name"]}')
  (dest/'cutout-response.json').write_text(json.dumps(cut,indent=2)+'\n')
  falcall.download(cut['image']['url'],str(dest/'cutout.png'))
 return item['name']+' done'
if __name__=='__main__':
 batch=sys.argv[1]
 manifest=OUT/'batches.json'
 specs=json.loads(manifest.read_text())
 with ThreadPoolExecutor(max_workers=3) as pool:
  for result in pool.map(lambda x:generate(x,batch),specs[batch]): print(result,flush=True)
