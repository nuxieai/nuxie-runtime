#!/usr/bin/env python3
"""Recheck actual wrapped rounded paint artifacts without compiling or rendering."""
from pathlib import Path
import hashlib, json, math
M=Path(__file__).resolve().parents[1]
ROOT=M/'output/playwright/wrapped-rounded-r1'
C=M/'output/wrapped-rounded-constructor-r1'
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
for b in read(C/'source-bindings.json'):bind(b['source'],b['sha256']);bind(b['snapshot'],b['sha256'])
cb=read(C/'receipt.json');assert cb['exitCode']==0
for key,name in [('candidateSha256','candidate'),('bridgeSha256','bridge.rs'),('patchesSha256','patches.json'),('sourceBindingsSha256','source-bindings.json'),('buildLogSha256','build.log')]:bind(C/name,cb[key])
for b in cb['effectiveCrateInputs']:bind(b['path'],b['sha256'])
construction=read(C/'construction-receipt.json');assert len(construction['cases'])==48
for key,name in [('constructorSha256','candidate'),('recipesSha256','recipes.json'),('scriptSha256','construct.py')]:bind(C/name,construction[key])
constructed={c['name']:c for c in construction['cases']}
for c in construction['cases']:
 assert c['exitCode']==0 and c['deterministic']
 folder=C/'cases'/c['name'];bind(folder/'recipe.json',c['recipeSha256']);bind(folder/'construct.log',c['logSha256'])
 for name,h in c['artifacts'].items():
  for sub in ['', 'first', 'repeat']:bind(folder/sub/name,h)
  bind(ROOT/c['name']/'derived-construction'/name,h)
 proof=read(folder/'proof.json');assert len(proof['paintBoxes'])==3
 for b in proof['paintBoxes']:assert b['end']-b['start']==2310
B=M/'output/immutable-baseline-toolchain-r2';bm=read(B/'manifest.json')
assert bm['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
for name,h in bm['files'].items():bind(B/name,h)
bind(M.parents[1]/'Cargo.toml',bm['rootCargoTomlSha256']);bind(M.parents[1]/'Cargo.lock',bm['rootCargoLockSha256'])
r=read(ROOT/'receipt.json')
assert r['status']=='passed-private-derived-baseline' and r['browser']=='153.0.8010.12' and r['effectiveMode']=='RasterOrdering'
assert len(r['rows'])==384 and len(r['artifacts'])==48
for k,h in r['toolHashes'].items():bind(r['tools'][k],h)
assert r['toolHashes']['probe']=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
assert r['toolHashes']['renderer']==bm['files']['renderer-replay']
for b in r['sourceBindings']:bind(b['source'],b['sha256']);bind(ROOT/b['snapshot'],b['sha256'])
# The gate module is byte-identical to the historical campaign; do not widen it.
old=read(M/'output/playwright/wrapped-derived-r2/receipt.json')
assert r['pixelGateSha256']==old['pixelGateSha256'];bind(M/'validation/pixels.mjs',r['pixelGateSha256'])
oldrows={(x['name'],x['frame']):x for x in old['rows']}
rows={(x['name'],x['frame']):x for x in r['rows']}
assert len(rows)==len(oldrows)==384
manifests={};maps={}
for a in r['artifacts']:
 folder=ROOT/a['name']
 for k,n in [('requestSha256','request.json'),('rivSha256','scene.riv'),('mapSha256','scene.map.json'),('probeManifestSha256','probe/frames.json')]:bind(folder/n,a[k])
 bind(folder/'probe/scene.riv',a['rivSha256'])
 assert a['rivSha256']==constructed[a['name']]['artifacts']['scene.riv']
 assert a['mapSha256']==constructed[a['name']]['artifacts']['scene.map.json']
 manifests[a['name']]=read(folder/'probe/frames.json')['frames'];maps[a['name']]=read(folder/'scene.map.json')
 assert len(manifests[a['name']])==8 and not (folder/'scene.requirements.json').exists()
comparisons=[]
for x in r['rows']:
 p=Path(x['prefix']);f=manifests[x['name']][x['frame']]
 assert f['instance']==x['instance']==x['frame']//4 and f['step']==x['step']==x['frame']%4
 assert [f['width'],f['height']]==[x['width'],x['height']]
 for k in ['geometry','stream']:bind(p.parent/'probe'/f[k],x[k+'Sha256'])
 assert json.loads((p.parent/'probe'/f['geometry']).read_text())==x['geometry']
 for k in ['chrome','native']:bind(str(p)+'.'+k+'.png',x[k+'Sha256'])
 bind(str(p)+'.diff.png');assert not x['geometryFailures'] and not x['pixelFailures']
 actual={a['objectId']:a for a in x['geometry']}
 for owner in maps[x['name']]:
  g=actual[owner['object_id']];assert g['worldMatrix'][:4]==[1,0,0,1]
  measured=dict(x=g['worldMatrix'][4],y=g['worldMatrix'][5],width=g['width'],height=g['height'])
  for k,v in measured.items():assert isinstance(v,(int,float)) and math.isfinite(v) and abs(v-x['boxes'][owner['id']][k])<=.1
 for c in x['clearChecks']:bind(c['path'],c['sha256']);assert c['samePixels']
 prior=oldrows[(x['name'],x['frame'])]
 assert prior['boxes']==x['boxes'] and prior['chromeSha256']==x['chromeSha256'] and prior['requestSha256']==x['requestSha256']
 for k in ['chrome','native']:bind(str(prior['prefix'])+'.'+k+'.png',prior[k+'Sha256'])
 assert not prior['geometryFailures']
 comparisons.append(dict(name=x['name'],frame=x['frame'],requestSha256=x['requestSha256'],chromeSha256=x['chromeSha256'],priorPixelFailures=prior['pixelFailures'],currentPixelFailures=x['pixelFailures'],priorNativeSha256=prior['nativeSha256'],currentNativeSha256=x['nativeSha256']))
assert sum(bool(x['pixelFailures'])for x in old['rows'])==48
assert sum(len(x['clearChecks'])for x in r['rows'])==768
assert len(r['repeated'])==240
for repeat in r['repeated']:
 assert repeat['nativeIdentical'] and repeat['chromeIdentical']
 a=rows[(repeat['name'],repeat['frame'])];b=rows[(repeat['name'],repeat['firstFrame'])]
 assert all(a[k+'Sha256']==b[k+'Sha256']for k in ['chrome','native'])
ba=dict(scope='Exact same authored requests, Chrome pixels and DOM boxes; native records/paint change. Historical failures retained.',priorReceipt=dict(path=str(M/'output/playwright/wrapped-derived-r2/receipt.json'),sha256=bind(M/'output/playwright/wrapped-derived-r2/receipt.json')),currentReceipt=dict(path=str(ROOT/'receipt.json'),sha256=bind(ROOT/'receipt.json')),comparisons=comparisons)
(ROOT/'before-after-receipt.json').write_text(json.dumps(ba,indent=2)+'\n');bind(ROOT/'before-after-receipt.json')
# Observer and direct visual review coverage are mandatory, never inferred from pixels.
paths=read(ROOT/'rounded-observation/receipt.json')
coverage=read(ROOT/'visual/coverage.json')

import importlib.util
spec=importlib.util.spec_from_file_location('wrapped_rounded_observer',M/'validation/wrapped-rounded-observe.py');observer=importlib.util.module_from_spec(spec);spec.loader.exec_module(observer)
for key in ['source','script','parser']:bind(paths[key]['path'],paths[key]['sha256'])
assert paths['counts']==dict(frames=384,scalarChecks=52992,scalarFailures=0,zeroSignDifferences=0,streamFailures=0,chromeEdgeComparisons=2304,chromeEdgeDifferences=0)
assert len(paths['frames'])==384 and len(paths['negativeControls'])==5
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
assert coverage['counts']==dict(frames=384,representatives=144,transfers=240,sheets=36,geometryPass=384,pixelPass=384)
for key in ['receipt','script','observation']:bind(coverage[key]['path'],coverage[key]['sha256'])
expected={s['path']:s for s in coverage['sheets']};notes={}
for file in ['inspection-root.json','inspection-agent.json']:
 inspection=read(ROOT/'visual'/file)
 for s in inspection.get('inspectedSheets',inspection.get('sheets')):
  assert s.get('observation',s.get('observations'));assert s['path']not in notes
  bind(s['path'],s['sha256']);assert s['sha256']==expected[s['path']]['sha256'];notes[s['path']]=s
assert set(notes)==set(expected) and len(notes)==36
for s in expected.values():
 for m in s['members']:
  for b in m['files']:bind(b['path'],b['sha256'])
for t in coverage['transfers']:
 a=rows[(t['name'],t['frame'])];b=rows[(t['name'],t['donorFrame'])]
 for k in ['chrome','native']:bind(t[k]['path'],t[k]['sha256']);assert t[k]['sha256']==a[k+'Sha256']==b[k+'Sha256']
 assert a['geometry']==b['geometry'] and a['boxes']==b['boxes']
review=dict(scope='144 distinct full-size pairs directly inspected in saved root/agent notes;240 exact same-case transfers. Private Derived campaign only.',visualReviewCompleted=True,counts=coverage['counts'],coverageSha256=bind(ROOT/'visual/coverage.json'),inspectedSheets=list(notes.values()))
(ROOT/'visual/review-receipt.json').write_text(json.dumps(review,indent=2)+'\n');bind(ROOT/'visual/review-receipt.json')
for name in ['wrapped-rounded-domain-review.md','wrapped-rounded-review.md']:bind(M/'validation'/name)
bind(__file__)
result=dict(status='verified-private-wrapped-rounded-paint',publicAdmission=False,counts=coverage['counts'],observationCounts=paths['counts'],clips=clip_count,draws=draw_count,clearChecks=768,preservedPriorPixelFailures=48,rustTests=419,nodeTests=56,maxMismatchedPixels=max(x['metrics']['mismatchedPixels']for x in r['rows']),maxMeanChannelError=max(x['metrics']['meanChannelError']for x in r['rows']),bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())])
out=M/'output/wrapped-rounded-verification-r1.json';out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='bindings'}|dict(bindings=len(bindings))))
