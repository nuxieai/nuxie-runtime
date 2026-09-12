"""Bind the finite candidate experiment, full-size visual review and reproduction."""
from pathlib import Path
from PIL import Image
import json,hashlib,subprocess,sys
base=Path(__file__).resolve().parent.parent
out=base/'output/solid-border-candidate-r1'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
load=lambda p:json.loads(Path(p).read_text())
runs={name:load(out/name/'receipt.json') for name in ['render','r2/render','r3/render']}
# These 48 pairs were inspected at native resolution in 16 unchanged-pixel sheets.
direct={}
for run,names in [('r2/render',None),('r3/render',None),('render',['nested-borders'])]:
 for row in runs[run]['rows']:
  if row['frame']>=3 or (names is not None and row['name']not in names):continue
  sheet=out/'visual'/((('initial-' if run=='render' else '')+row['name'])+'.png')
  image=Image.open(sheet).convert('RGBA');y=sum(v['height']+32 for v in runs[run]['rows']if v['name']==row['name']and v['frame']<row['frame'])+24
  for index,kind in enumerate(['chrome','native']):
   original=Image.open(row['prefix']+'.'+kind+'.png').convert('RGBA')
   assert image.crop((8+index*784,y,8+index*784+row['width'],y+row['height'])).tobytes()==original.tobytes()
  key=f"{run}/{row['name']}/{row['frame']}"
  direct[key]=dict(row=row,sheet=str(sheet.relative_to(base)),sheetSha256=sha(sheet))
coverage=[]
for run,receipt in runs.items():
 for row in receipt['rows']:
  for kind in ['chrome','native']:assert sha(row['prefix']+'.'+kind+'.png')==row[kind+'Sha256']
  key=f"{run}/{row['name']}/{row['frame']}";match=direct.get(key)
  if match is None:
   match=next(d for d in direct.values()if all(d['row'][k]==row[k]for k in ['name','width','height','requestSha256','chromeSha256','nativeSha256']))
   for kind in ['chrome','native']:assert Image.open(row['prefix']+'.'+kind+'.png').convert('RGBA').tobytes()==Image.open(match['row']['prefix']+'.'+kind+'.png').convert('RGBA').tobytes()
  coverage.append(dict(frame=key,method='direct full-size sheet'if key in direct else'exact full RGBA transfer',sheet=match['sheet'],sheetSha256=match['sheetSha256'],requestSha256=row['requestSha256'],rivSha256=row['rivSha256'],sourceMapSha256=row['sourceMapSha256'],geometrySha256=row['geometrySha256'],chromeSha256=row['chromeSha256'],nativeSha256=row['nativeSha256']))
(out/'visual-coverage.json').write_text(json.dumps(coverage,indent=2)+'\n')
reproduce=out/(sys.argv[1] if len(sys.argv)>1 else 'reproduction');reproduce.mkdir(exist_ok=False);reproductions=[]
for run in ['r2/render','r3/render']:
 for artifact in runs[run]['artifacts']:
  name=artifact['name'];prior=out/run/name;dest=reproduce/(name+'.riv')
  command=[str(out/'r3/candidate'),str(prior/'request.json'),str(dest)]
  p=subprocess.run(command,capture_output=True,text=True);(reproduce/(name+'.log')).write_text(p.stdout+p.stderr);p.check_returncode()
  assert sha(dest)==artifact['rivSha256'];assert sha(dest.with_suffix('.map.json'))==artifact['mapSha256']
  reproductions.append(dict(name=name,rivSha256=sha(dest),mapSha256=sha(dest.with_suffix('.map.json')),bytes=dest.stat().st_size))
summaries=[]
for run,r in runs.items():
 cases=[]
 for a in r['artifacts']:
  rows=[v for v in r['rows']if v['name']==a['name']]
  cases.append(dict(name=a['name'],frames=len(rows),geometryPass=sum(not v['geometryFailures']for v in rows),pixelPass=sum(not v['pixelFailures']for v in rows),clearPass=sum(c['samePixels']for v in rows for c in v['clearChecks'])))
 summaries.append(dict(path=str((out/run/'receipt.json').relative_to(base)),sha256=sha(out/run/'receipt.json'),cases=cases))
source_files=[base/'validation'/name for name in ['solid-border-candidate-emitter.rs','solid-border-candidate-build.py','solid-border-candidate-cases.json','solid-border-candidate-computed.mjs','solid-border-candidate-receipt.py','solid-border-candidate-review.md']]
receipt=dict(status='private candidate evidence; P01 public admission unresolved',target='6c7ac16617835b5f581784ff08a9e779bb52faf3',browser='153.0.8010.12',dpr=1,backend='rust-metal RasterOrdering',runs=summaries,sourceHashes={str(p.relative_to(base)):sha(p)for p in source_files},buildReceipts={str((out/run).relative_to(base)):sha(out/run)for run in ['build-receipt.json','r2/build-receipt.json','r3/build-receipt.json']},visualCoverage=dict(path=str((out/'visual-coverage.json').relative_to(base)),sha256=sha(out/'visual-coverage.json'),directPairs=len(direct),totalFrames=len(coverage),transferredFrames=sum(v['method']=='exact full RGBA transfer'for v in coverage)),reproductions=reproductions,chromeComputedWidths=dict(path=str((out/'chrome-computed-widths.json').relative_to(base)),sha256=sha(out/'chrome-computed-widths.json')),runtimeGuard=dict(path=str((out/'runtime-guard.json').relative_to(base)),sha256=sha(out/'runtime-guard.json')))
(base/'validation/solid-border-candidate-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(dict(visualFrames=len(coverage),directPairs=len(direct),transferredFrames=receipt['visualCoverage']['transferredFrames'],exactReproductions=len(reproductions))))
