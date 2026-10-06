"""Validate retained candidate files, references and runtime isolation offline."""
import hashlib,json,subprocess
from pathlib import Path
from PIL import Image
BASE=Path(__file__).resolve().parent
ROOT=BASE.parents[2]
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
errors=[];outputs=[]
for p in sorted(BASE.glob('*/*/provenance.json')):
 rec=json.loads(p.read_text())
 for r in rec['references']:
  if digest(ROOT/r['path'])!=r['sha256']:errors.append('reference hash '+r['path'])
 for name in ['original.png','cutout.png']:
  f=p.parent/name
  if not f.exists():continue
  im=Image.open(f);im.verify();im=Image.open(f)
  entry={'path':str(f.relative_to(BASE)),'size':list(im.size),'mode':im.mode,'sha256':digest(f)}
  if im.format!='PNG':errors.append('not PNG '+str(f))
  if name=='cutout.png':
   if im.mode!='RGBA':errors.append('not RGBA '+str(f))
   else:
    entry['alpha_extrema']=im.getchannel('A').getextrema()
    entry['corner_alpha']=[im.getpixel(x)[3] for x in [(0,0),(im.width-1,0),(0,im.height-1),(im.width-1,im.height-1)]]
    if any(entry['corner_alpha']):errors.append('nontransparent corners '+str(f))
  outputs.append(entry)
ledger=[json.loads(x) for x in (BASE/'ledger.jsonl').read_text().splitlines()]
changed=subprocess.check_output(['git','diff','--name-only','1984e9b'],cwd=ROOT,text=True).splitlines()
runtime=[x for x in changed if x.startswith(('crates/','web/','src/','assets/sprites/','assets/ui/','assets/terrain/'))]
if runtime:errors.append('runtime changed '+repr(runtime))
batches=json.loads((BASE/'batches.json').read_text());covered={x['name'] for group in batches.values() for x in group}
scope=json.loads((BASE/'scope.json').read_text());names={Path(x).stem for x in scope['runtime_paths'] if x.startswith('assets/ui/')}
if names-covered:errors.append('missing UI subjects '+repr(names-covered))
report={'illustration_count':len(list(BASE.glob('*/*/original.png'))),'successful_requests':len(ledger),'estimated_usd':round(sum(x['est_usd'] for x in ledger),4),'excluded_failed_request':'01a1102b-18f2-7502-be2e-fdb5e85453cd: invalid aspect ratio; unknown billing','runtime_paths_changed':runtime,'errors':errors,'outputs':outputs}
(BASE/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
print({k:v for k,v in report.items() if k!='outputs'})
raise SystemExit(bool(errors))
