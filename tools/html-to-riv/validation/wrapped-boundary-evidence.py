#!/usr/bin/env python3
"""Recheck actual wrapped boundary paint artifacts, retaining known failures without compiling or rendering."""
from pathlib import Path
import hashlib, json, math
M=Path(__file__).resolve().parents[1]
ROOT=M/'output/playwright/wrapped-boundary-r1'
C=M/'output/wrapped-boundary-constructor-r1'
F=M/'output/wrapped-rounded-constructor-r1'
P=M/'output/wrapped-rounded-product-build-r1'
bindings={}
def bind(path, expected=None):
 p=Path(path); h=hashlib.sha256(p.read_bytes()).hexdigest()
 if expected is not None: assert h==expected, str(p)
 bindings[str(p.resolve())]=h
 return h
def read(path):
 bind(path);return json.loads(Path(path).read_text())
summary=read(P/'summary.json')
assert summary['sourceUnchanged'] and summary['rustTests']==419 and summary['nodeTests']==56
identity=read(P/'prior-public-identity.json')
for row in identity['verifiedFiles']:bind(row['path'],row['sha256'])
for row in identity['binaries']:
 assert row['exactBytes'];bind(row['current'],row['sha256']);bind(row['prior'],row['sha256'])
checks=read(P/'checks.json')
assert [c['name']for c in checks]==['full-rust','native-build','wasm-build','typescript','full-node','runtime-guard']
for c in checks:assert c['exitCode']==0;bind(c['log'],c['logSha256'])
for b in read(F/'source-bindings.json'):bind(b['source'],b['sha256']);bind(b['snapshot'],b['sha256'])
cb=read(F/'receipt.json');assert cb['exitCode']==0
for key,name in [('candidateSha256','candidate'),('bridgeSha256','bridge.rs'),('patchesSha256','patches.json'),('sourceBindingsSha256','source-bindings.json'),('buildLogSha256','build.log')]:bind(F/name,cb[key])
for b in cb['effectiveCrateInputs']:bind(b['path'],b['sha256'])
ci=read(C/'constructor-identity.json');assert ci['sha256']==ci['copySha256']==cb['candidateSha256']
bind(ci['source'],ci['sha256']);bind(C/'candidate',ci['copySha256']);bind(ci['buildReceipt']['path'],ci['buildReceipt']['sha256'])
construction=read(C/'construction-receipt.json');assert len(construction['cases'])==32
for key,name in [('constructorSha256','candidate'),('recipesSha256','recipes.json'),('scriptSha256','construct.py')]:bind(C/name,construction[key])
constructed={c['name']:c for c in construction['cases']}
for c in construction['cases']:
 assert c['exitCode']==0 and c['deterministic']
 folder=C/'cases'/c['name'];bind(folder/'recipe.json',c['recipeSha256']);bind(folder/'construct.log',c['logSha256'])
 for name,h in c['artifacts'].items():
  for sub in ['', 'first', 'repeat']:bind(folder/sub/name,h)
  bind(ROOT/c['name']/'derived-construction'/name,h)
 proof=read(folder/'proof.json');assert len(proof['paintBoxes'])in [1,3]
 for b in proof['paintBoxes']:assert b['end']-b['start']==2310
B=M/'output/immutable-baseline-toolchain-r2';bm=read(B/'manifest.json')
assert bm['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
for name,h in bm['files'].items():bind(B/name,h)
bind(M.parents[1]/'Cargo.toml',bm['rootCargoTomlSha256']);bind(M.parents[1]/'Cargo.lock',bm['rootCargoLockSha256'])
r=read(ROOT/'receipt.json')
assert r['status']=='failed-private-derived-baseline' and r['browser']=='153.0.8010.12' and r['effectiveMode']=='RasterOrdering'
assert len(r['rows'])==256 and len(r['artifacts'])==32
for k,h in r['toolHashes'].items():bind(r['tools'][k],h)
assert r['toolHashes']['probe']=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
assert r['toolHashes']['renderer']==bm['files']['renderer-replay']
for b in r['sourceBindings']:bind(b['source'],b['sha256']);bind(ROOT/b['snapshot'],b['sha256'])
# The gate module is byte-identical to the historical campaign; do not widen it.
old=read(M/'validation/wrapped-rounded-receipt.json')
# The historical evidence binds the exact gate and driver source, not merely metrics.
prior=read(M/'output/wrapped-rounded-verification-r1.json')
prior_bindings={x['path']:x['sha256']for x in prior['bindings']}
bind(M/'validation/pixels.mjs',r['pixelGateSha256'])
assert prior_bindings[str(M/'validation/pixels.mjs')]==r['pixelGateSha256']
assert prior_bindings[str(M/'validation/check-wrapped-rounded-baseline.mjs')]==bind(M/'validation/check-wrapped-rounded-baseline.mjs')
rows={(x['name'],x['frame']):x for x in r['rows']};assert len(rows)==256
manifests={};maps={}
for a in r['artifacts']:
 folder=ROOT/a['name']
 for k,n in [('requestSha256','request.json'),('rivSha256','scene.riv'),('mapSha256','scene.map.json'),('probeManifestSha256','probe/frames.json')]:bind(folder/n,a[k])
 bind(folder/'probe/scene.riv',a['rivSha256'])
 assert a['rivSha256']==constructed[a['name']]['artifacts']['scene.riv']
 assert a['mapSha256']==constructed[a['name']]['artifacts']['scene.map.json']
 manifests[a['name']]=read(folder/'probe/frames.json')['frames'];maps[a['name']]=read(folder/'scene.map.json')
 assert len(manifests[a['name']])==8 and not (folder/'scene.requirements.json').exists()

for x in r['rows']:
 p=Path(x['prefix']);f=manifests[x['name']][x['frame']]
 assert f['instance']==x['instance']==x['frame']//4 and f['step']==x['step']==x['frame']%4
 assert [f['width'],f['height']]==[x['width'],x['height']]
 for k in ['geometry','stream']:bind(p.parent/'probe'/f[k],x[k+'Sha256'])
 assert json.loads((p.parent/'probe'/f['geometry']).read_text())==x['geometry']
 for k in ['chrome','native']:bind(str(p)+'.'+k+'.png',x[k+'Sha256'])
 bind(str(p)+'.diff.png');assert not x['geometryFailures']
 actual={a['objectId']:a for a in x['geometry']}
 for owner in maps[x['name']]:
  g=actual[owner['object_id']];assert g['worldMatrix'][:4]==[1,0,0,1]
  measured=dict(x=g['worldMatrix'][4],y=g['worldMatrix'][5],width=g['width'],height=g['height'])
  for k,v in measured.items():assert isinstance(v,(int,float)) and math.isfinite(v) and abs(v-x['boxes'][owner['id']][k])<=.1
 for c in x['clearChecks']:bind(c['path'],c['sha256']);assert c['samePixels']
assert sum(bool(x['pixelFailures'])for x in r['rows'])==28
assert sum(len(x['clearChecks'])for x in r['rows'])==512
assert len(r['repeated'])==160
for repeat in r['repeated']:
 assert repeat['nativeIdentical'] and repeat['chromeIdentical']
 a=rows[(repeat['name'],repeat['frame'])];b=rows[(repeat['name'],repeat['firstFrame'])]
 assert all(a[k+'Sha256']==b[k+'Sha256']for k in ['chrome','native'])
# Observer and direct visual review coverage are mandatory, never inferred from pixels.
paths=read(ROOT/'boundary-observation/receipt.json')
coverage=read(ROOT/'visual/coverage.json')

import importlib.util
spec=importlib.util.spec_from_file_location('wrapped_rounded_observer',M/'validation/wrapped-boundary-observe.py');observer=importlib.util.module_from_spec(spec);spec.loader.exec_module(observer)
for key in ['source','script','parser']:bind(paths[key]['path'],paths[key]['sha256'])
assert paths['counts']==dict(frames=256,scalarChecks=20608,scalarFailures=0,zeroSignDifferences=0,streamFailures=0,chromeEdgeComparisons=896,chromeEdgeDifferences=92,chromeClippedEdgeDifferences=28,saturationIntersectionFailures=0)
assert len(paths['frames'])==256 and len(paths['negativeControls'])==5
clip_count=draw_count=0
for o in paths['frames']:
 row=rows[(o['name'],o['frame'])];case=Path(row['prefix']).parent/'derived-construction'
 calculated=observer.observe(row,case)
 assert calculated==o['scalar'] and not calculated['failures']
 for key in ['proof','recipe','trace']:bind(calculated[key]['path'],calculated[key]['sha256'])
 bind(o['stream']['path'],o['stream']['sha256']);assert o['stream']['sha256']==row['streamSha256']
 lines=observer.clips.extract(Path(o['stream']['path']),o['frame'])
 observed=observer.stream_observe(lines,row['width'],row['height'],calculated['replicas'])
 assert observed==o['streamObservation'] and o['streamError'] is None
 clip_count+=len(observed['clips']);draw_count+=len(observed['draws'])
 if o is paths['frames'][0]:assert list(observer.controls(lines,row['width'],row['height'],calculated['replicas']))==paths['negativeControls']
assert coverage['counts']==dict(frames=256,representatives=96,transfers=160,sheets=26,geometryPass=256,pixelPass=228)
for key in ['receipt','script','observation']:bind(coverage[key]['path'],coverage[key]['sha256'])
expected={s['path']:s for s in coverage['sheets']};notes={}
for file in ['inspection-root.json','inspection-agent.json']:
 inspection=read(ROOT/'visual'/file)
 for s in inspection.get('inspectedSheets',inspection.get('sheets')):
  assert s.get('observation',s.get('observations'));assert s['path']not in notes
  bind(s['path'],s['sha256']);assert s['sha256']==expected[s['path']]['sha256'];assert s['members']==expected[s['path']]['members'];notes[s['path']]=s
assert set(notes)==set(expected) and len(notes)==26
for s in expected.values():
 for m in s['members']:
  for b in m['files']:bind(b['path'],b['sha256'])
for t in coverage['transfers']:
 a=rows[(t['name'],t['frame'])];b=rows[(t['name'],t['donorFrame'])]
 for k in ['chrome','native']:bind(t[k]['path'],t[k]['sha256']);assert t[k]['sha256']==a[k+'Sha256']==b[k+'Sha256']
 assert a['geometry']==b['geometry'] and a['boxes']==b['boxes']
review=dict(scope='96 distinct full-size pairs directly inspected in saved root/agent notes;160 exact same-case transfers. Known pixel failures preserved.',visualReviewCompleted=True,counts=coverage['counts'],coverageSha256=bind(ROOT/'visual/coverage.json'),inspectedSheets=list(notes.values()))
(ROOT/'visual/review-receipt.json').write_text(json.dumps(review,indent=2)+'\n');bind(ROOT/'visual/review-receipt.json')
for name in ['wrapped-rounded-domain-review.md','wrapped-boundary-review.md','wrapped-boundary-quantization-review.md']:bind(M/'validation'/name)
case_summary=read(ROOT/'boundary-observation/case-summary.json')
for b in case_summary['source']:bind(b['path'],b['sha256'])
failure_counts={name:sum(bool(x['pixelFailures'])for x in r['rows']if x['name']==name)for name in constructed}
assert {k:v for k,v in failure_counts.items()if v}=={'boundary-row-thin-next-above':8,'boundary-column-thin-next-above':8,'boundary-row-accumulated-decimal':6,'boundary-column-accumulated-decimal':6}
assert sum(not e['same']and e['clippedSame']for o in paths['frames']for e in o['scalar']['edges'])==64
assert sum(len(o['scalar']['checks'])for o in paths['frames'])==20608
assert sum(len(o['scalar']['edges'])for o in paths['frames'])==896
assert sum(not e['same']for o in paths['frames']for e in o['scalar']['edges'])==92
assert sum(not e['clippedSame']for o in paths['frames']for e in o['scalar']['edges'])==28
assert all(e['saturationPreservesNativeIntersection']for o in paths['frames']for e in o['scalar']['edges'])
for name,c in case_summary['cases'].items():
 actual=[x for x in r['rows']if x['name']==name];assert c['frames']==len(actual)==8
 assert c['pixelFailureFrames']==[x['frame']for x in actual if x['pixelFailures']]
 for field,predicate in [('rawEdgeDifferenceFrames',lambda e:not e['same']),('clippedEdgeDifferenceFrames',lambda e:not e['clippedSame']),('saturationFailureFrames',lambda e:not e['saturationPreservesNativeIntersection'])]:
  expected=[]
  for o in paths['frames']:
   if o['name']!=name:continue
   edges=[e for e in o['scalar']['edges']if predicate(e)]
   if edges:expected.append(dict(frame=o['frame'],edges=edges))
  assert c[field]==expected
 assert {x['frame']for x in c['clippedEdgeDifferenceFrames']}==set(c['pixelFailureFrames'])

bind(__file__)
result=dict(status='verified-private-boundary-evidence-with-retained-failures',publicAdmission=False,counts=coverage['counts'],observationCounts=paths['counts'],clips=clip_count,draws=draw_count,clearChecks=512,preservedPixelFailures=28,saturationOnlyRawDifferences=64,pixelFailureCounts=failure_counts,rustTests=419,nodeTests=56,maxMismatchedPixels=max(x['metrics']['mismatchedPixels']for x in r['rows']),maxMeanChannelError=max(x['metrics']['meanChannelError']for x in r['rows']),bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())])
out=M/'output/wrapped-boundary-verification-r1.json';out.write_text(json.dumps(result,indent=2)+'\n')
compact={k:v for k,v in result.items()if k!='bindings'}
compact.update(artifactBindings=len(bindings),baseline='6c7ac16617835b5f581784ff08a9e779bb52faf3',scope='Private immutable Derived boundary evidence; known Chrome quantization failures remain unqualified.',evidence=[dict(path=str(p),sha256=bind(p))for p in [out,ROOT/'receipt.json',ROOT/'boundary-observation/receipt.json',ROOT/'visual/review-receipt.json',M/'validation/wrapped-boundary-review.md',M/'validation/wrapped-boundary-quantization-review.md']])
(M/'validation/wrapped-boundary-receipt.json').write_text(json.dumps(compact,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items()if k!='bindings'}|dict(bindings=len(bindings))))
