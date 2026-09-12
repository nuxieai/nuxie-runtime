#!/usr/bin/env python3
"""Recheck standalone paint evidence without recompiling or recapturing."""
from pathlib import Path
import hashlib,json
from PIL import Image
M=Path(__file__).resolve().parents[1];ROOT=M/'output/playwright/fractional-paint-r1';C=M/'output/fractional-paint-constructor-r1';bindings={}
def bind(p,h=None):
 p=Path(p);actual=hashlib.sha256(p.read_bytes()).hexdigest()
 if h is not None:assert actual==h,p
 bindings[str(p.resolve())]=actual
 return actual
def read(p):bind(p);return json.loads(Path(p).read_text())
for b in read(C/'source-bindings.json'):
 bind(b['source'],b['sha256']);bind(b['snapshot'],b['sha256'])
build=read(C/'receipt.json');bind(C/'candidate',build['candidateSha256'])
for key,name in [('bridgeSha256','bridge.rs'),('patchesSha256','patches.json'),('sourceBindingsSha256','source-bindings.json'),('buildLogSha256','build.log')]:bind(C/name,build[key])
for b in build['effectiveCrateInputs']:bind(b['path'],b['sha256'])
construction=read(C/'construction-receipt.json');assert len(construction['cases'])==20
for key,name in [('constructorSha256','candidate'),('recipesSha256','recipes.json'),('casesSha256','cases.json')]:bind(C/name,construction[key])
for c in construction['cases']:
 assert c['exitCode']==0
 for name,h in c['artifacts'].items():
  bind(C/'cases'/c['name']/name,h);bind(ROOT/c['name']/'fractional-construction'/name,h)
# Effective immutable baseline tool build identity is distinct from source guard.
B=M/'output/immutable-baseline-toolchain-r2';bm=read(B/'manifest.json')
assert bm['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
for name,h in bm['files'].items():bind(B/name,h)
bind(M.parents[1]/'Cargo.toml',bm['rootCargoTomlSha256']);bind(M.parents[1]/'Cargo.lock',bm['rootCargoLockSha256'])
r=read(ROOT/'receipt.json');assert len(r['rows'])==160 and len(r['artifacts'])==20
assert r['status']=='failed-private-fractional-paint-baseline' and r['browser']=='153.0.8010.12'
assert r['effectiveMode']=='RasterOrdering'
for k,h in r['toolHashes'].items():bind(r['tools'][k],h)
assert r['toolHashes']['probe']=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
assert r['toolHashes']['renderer']==bm['files']['renderer-replay']
for b in r['sourceBindings']:bind(b['source'],b['sha256']);bind(ROOT/b['snapshot'],b['sha256'])
for a in r['artifacts']:
 for k,name in [('requestSha256','request.json'),('rivSha256','scene.riv'),('mapSha256','scene.map.json'),('probeManifestSha256','probe/frames.json')]:bind(ROOT/a['name']/name,a[k])
 c=next(c for c in construction['cases']if c['name']==a['name']);assert a['rivSha256']==c['artifacts']['scene.riv'] and a['mapSha256']==c['artifacts']['scene.map.json']
for x in r['rows']:
 p=Path(x['prefix']);f=read(p.parent/'probe/frames.json')['frames'][x['frame']]
 for k in ['geometry','stream']:bind(p.parent/'probe'/f[k],x[k+'Sha256'])
 for k in ['chrome','native']:bind(str(p)+'.'+k+'.png',x[k+'Sha256'])
 bind(str(p)+'.diff.png');assert not x['geometryFailures']
 for c in x['clearChecks']:bind(c['path'],c['sha256']);assert c['samePixels']
 if x['pixelFailures']:assert x['pixelFailures']==['mismatch ratio'] and '-64d5-px' in x['name'] and x['step']!=1
assert sum(not x['pixelFailures']for x in r['rows'])==136
assert sum(len(x['clearChecks'])for x in r['rows'])==320
assert len(r['repeated'])==100 and all(x['nativeIdentical']and x['chromeIdentical']for x in r['repeated'])
paths=read(ROOT/'path-receipt.json');assert len(paths['frames'])==160 and len(paths['routePairs'])==80
for key in ['observer','pathParser']:bind(paths[key]['path'],paths[key]['sha256'])
for x in paths['frames']:bind(x['stream']['path'],x['stream']['sha256']);assert len(x['draws'])==2
for p in paths['routePairs']:
 a=next(x for x in r['rows']if x['name']==p['layout']and x['frame']==p['frame']);b=next(x for x in r['rows']if x['name']==p['shape']and x['frame']==p['frame'])
 assert p['nativeEqual'] and p['chromeEqual'] and a['boxes']==b['boxes']
 assert a['nativeSha256']==b['nativeSha256'] and a['chromeSha256']==b['chromeSha256']
for c in read(ROOT/'wrapping-local-reproduction.json')['comparisons']:
 for k in ['prior','standalone']:bind(c[k]['path'],c[k]['sha256'])
 a=Image.open(c['prior']['path']).convert('RGBA').crop(c['band']);b=Image.open(c['standalone']['path']).convert('RGBA').crop(c['band']);assert a.tobytes()==b.tobytes() and c['rgbaEqual']
coverage=read(ROOT/'visual/coverage.json');expected={s['path']:s for s in coverage['sheets']};notes={}
for file in ['inspection-agent.json','inspection-root.json']:
 for s in read(ROOT/'visual'/file)['inspectedSheets']:
  observation=s.get('observation',s.get('observations'));assert observation
  s=dict(s,observation=observation)
  assert s['path']not in notes;bind(s['path'],s['sha256']);assert s['sha256']==expected[s['path']]['sha256'];notes[s['path']]=s
assert set(notes)==set(expected) and len(notes)==18
for s in expected.values():
 for m in s['members']:
  for b in m['files']:bind(b['path'],b['sha256'])
for t in coverage['transfers']:
 a=next(x for x in r['rows']if x['name']==t['name']and x['frame']==t['frame']);b=next(x for x in r['rows']if x['name']==t['name']and x['frame']==t['donorFrame'])
 for k in ['chrome','native']:bind(t[k]['path'],t[k]['sha256']);assert t[k]['sha256']==a[k+'Sha256']==b[k+'Sha256']
 assert a['geometry']==b['geometry']and a['boxes']==b['boxes']
assert coverage['counts']==dict(frames=160,representatives=60,transfers=100,sheets=18,geometryPass=160,pixelPass=136)
review=dict(scope='60 distinct full pairs directly inspected;100 exact same-case repeat/clone transfers. All24pixel failures preserved; no public paint qualification.',visualReviewCompleted=True,counts=coverage['counts'],coverageSha256=bind(ROOT/'visual/coverage.json'),inspectedSheets=list(notes.values()))
(ROOT/'visual/review-receipt.json').write_text(json.dumps(review,indent=2)+'\n');bind(ROOT/'visual/review-receipt.json')
S=M/'output/fractional-paint-chrome-source-r1';source=read(S/'manifest.json');assert source['tag']=='153.0.8010.12'
bind(S/'tag-commit.json',source['commit_response_sha256'])
for f in source['files']:bind(S/f['path'],f['sha256'])
for name in ['fractional-paint-cases.py','fractional-paint-constructor-review.md','fractional-paint-chrome-rule.md']:bind(M/'validation'/name)
bind(__file__)
result=dict(status='verified-standalone-paint-with-retained-failures',publicAdmission=False,counts=coverage['counts'],routePairs=80,identicalNativeRoutes=80,bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())])
out=M/'output/fractional-paint-verification-r1.json';out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(status=result['status'],bindings=len(bindings),**coverage['counts'])))
