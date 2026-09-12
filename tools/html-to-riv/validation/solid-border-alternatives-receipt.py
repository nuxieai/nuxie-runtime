"""Verify the retained alternative files and write their compact evidence receipt."""
from pathlib import Path
from PIL import Image
import json,hashlib,subprocess,sys
b=Path(__file__).resolve().parent.parent;o=b/'output/solid-border-alternatives-r1'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
load=lambda p:json.loads(Path(p).read_text())
runs={p:load(o/p/'receipt.json')for p in ['render','r2/render']}
fixtures=load(b/'validation/solid-border-alternatives-cases.json');prior_fixtures=load(b/'validation/solid-border-candidate-cases.json')
for c in fixtures:
 if c['mode']in ['layout-fill','shape-fill']:
  assert c['sourceCase']=='fixed-shape-primitive-control' and c['borders']==[] and c['html']=='<div id="p"></div>';continue
 prior=next(p for p in prior_fixtures if p['name']==c['sourceCase'])
 assert c['html']==prior['html'];assert c['css']==prior['css']+'/* '+c['mode']+' */';assert c['nativeCss']==prior['nativeCss'];assert c['borders']==prior['borders']
# Validate the current source-backed executable's exact bytes against both runs.
dest=o/(sys.argv[1]if len(sys.argv)>1 else'reproduction');dest.mkdir(exist_ok=False);reproductions=[]
for run,r in runs.items():
 assert all(x['nativeIdentical']and x['chromeIdentical']for x in r['repeated'])
 for a in r['artifacts']:
  request=o/run/a['name']/'request.json';path=dest/(a['name']+'.riv')
  command=[str(o/'r3/candidate'),str(request),str(path)]
  p=subprocess.run(command,capture_output=True,text=True);(dest/(a['name']+'.log')).write_text(p.stdout+p.stderr);p.check_returncode()
  assert sha(path)==a['rivSha256'];assert sha(path.with_suffix('.map.json'))==a['mapSha256']
  reproductions.append(dict(name=a['name'],path=str(path.relative_to(b)),bytes=path.stat().st_size,rivSha256=sha(path),mapSha256=sha(path.with_suffix('.map.json'))))
coverage=load(o/'visual-coverage.json');assert len(coverage['directPairs'])==32;assert len(coverage['frames'])==216
for frame in coverage['frames']:
 row=next(r for r in runs[frame['run']]['rows']if r['name']==frame['name']and r['frame']==frame['frame'])
 for field in ['requestSha256','rivSha256','sourceMapSha256','geometrySha256','streamSha256','chromeSha256','nativeSha256']:assert row[field]==frame[field]
 ref=frame['review'];assert sha(b/ref['sheet'])==ref['sheetSha256']
 if ref['review']=='prior verified review':assert sha(b/'validation/solid-border-candidate-receipt.json')==ref['reviewReceiptSha256']
 for kind in ['chrome','native']:
  path=Path(row['prefix']+'.'+kind+'.png');assert sha(path)==row[kind+'Sha256']
  assert Image.open(path).convert('RGBA').tobytes()==Image.open(ref['prefix']+'.'+kind+'.png').convert('RGBA').tobytes()
# Bind per-mode observations, including retained failures, without recategorizing.
summary=[]
for run,r in runs.items():
 per_case=[]
 for a in r['artifacts']:
  rows=[v for v in r['rows']if v['name']==a['name']]
  per_case.append(dict(name=a['name'],frames=len(rows),geometryPass=sum(not v['geometryFailures']for v in rows),pixelPass=sum(not v['pixelFailures']for v in rows),clearPass=sum(c['samePixels']for v in rows for c in v['clearChecks'])))
 summary.append(dict(path=str((o/run/'receipt.json').relative_to(b)),sha256=sha(o/run/'receipt.json'),cases=per_case))
prior_runs=[b/'output/solid-border-candidate-r1'/p/'receipt.json'for p in ['r2/render','r3/render']]
all_rows=[r for p in prior_runs for r in load(p)['rows']]+[r for receipt in runs.values()for r in receipt['rows']]
topology=[]
for name in ['fractional-origin-border-only','fractional-origin-alpha-border','fractional-edges','nested-borders']:
 for mode in ['','clipped-double-','inset-stroke-','sliced-ring-']:
  row=next(r for r in all_rows if r['name']==mode+name and r['frame']==0)
  pixels={kind:Image.open(row['prefix']+'.'+kind+'.png').convert('RGBA').getpixel((20,5))for kind in ['chrome','native']}
  topology.append(dict(name=name,mode=mode.rstrip('-')or'strips',width=row['width'],height=row['height'],metrics=row['metrics'],sampleCoordinate=[20,5],sampleRgba=pixels,chromeSha256=row['chromeSha256'],nativeSha256=row['nativeSha256']))
(o/'topology-comparison.json').write_text(json.dumps(topology,indent=2)+'\n')
files=[b/'validation'/('solid-border-alternatives-'+suffix)for suffix in ['emitter.rs','build.py','cases.json','visual.py','receipt.py','review.md','primitive.py']]
receipt=dict(status='private ordinary-file candidates; fractional-edge profile unresolved; no public admission',target='6c7ac16617835b5f581784ff08a9e779bb52faf3',browser='153.0.8010.12',dpr=1,backend='rust-metal RasterOrdering',sourceHashes={str(p.relative_to(b)):sha(p)for p in files},runs=summary,reproductions=reproductions,buildReceipts={str((o/p).relative_to(b)):sha(o/p)for p in ['build-receipt.json','r2/build-receipt.json','r3/build-receipt.json']},sourceAudit=dict(path=str((o/'source-audit.json').relative_to(b)),sha256=sha(o/'source-audit.json')),runtimeGuard=dict(path=str((o/'runtime-guard.json').relative_to(b)),sha256=sha(o/'runtime-guard.json')),priorReceipts={str(p.relative_to(b)):sha(p)for p in prior_runs},visualCoverage=dict(path=str((o/'visual-coverage.json').relative_to(b)),sha256=sha(o/'visual-coverage.json'),frames=216,distinctPairs=48,newDirectPairs=32,priorExactFrameTransfers=116,newExactFrameTransfers=68),topologyComparison=dict(path=str((o/'topology-comparison.json').relative_to(b)),sha256=sha(o/'topology-comparison.json')))
receipt['primitiveControl']=dict(path=str((o/'primitive-receipt.json').relative_to(b)),sha256=sha(o/'primitive-receipt.json'),cases=4,frames=32,geometryPass=32,pixelPass=16,clearPass=64,directPairs=2,exactReproductions=4)
(b/'validation/solid-border-alternatives-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(dict(exactReproductions=len(reproductions),geometryPass=216,pixelPass=156,clearPass=432,distinctPairs=48,newDirectPairs=32,visualFrameCoverage=216)))
