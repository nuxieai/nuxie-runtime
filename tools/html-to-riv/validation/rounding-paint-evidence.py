#!/usr/bin/env python3
"""Recheck standalone paint evidence without recompiling or recapturing."""
from pathlib import Path
import hashlib,json
M=Path(__file__).resolve().parents[1];ROOT=M/'output/playwright/rounding-paint-r1';C=M/'output/rounding-paint-constructor-r1';bindings={}
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
construction=read(C/'construction-receipt.json');assert len(construction['cases'])==28
for key,name in [('constructorSha256','candidate'),('recipesSha256','recipes.json'),('casesSha256','cases.json')]:bind(C/name,construction[key])
for c in construction['cases']:
 assert c['exitCode']==0 and c['deterministic']
 bind(C/'cases'/c['name']/'recipe.json',c['recipeSha256'])
 assert read(C/'cases'/c['name']/'construction.json')['records']==2321
 for name,h in c['artifacts'].items():
  bind(C/'cases'/c['name']/name,h);bind(ROOT/c['name']/'rounding-construction'/name,h)
  for run in ['first','repeat']:bind(C/'cases'/c['name']/run/name,h)
# Effective immutable baseline tool build identity is distinct from source guard.
B=M/'output/immutable-baseline-toolchain-r2';bm=read(B/'manifest.json')
assert bm['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
for name,h in bm['files'].items():bind(B/name,h)
bind(M.parents[1]/'Cargo.toml',bm['rootCargoTomlSha256']);bind(M.parents[1]/'Cargo.lock',bm['rootCargoLockSha256'])
r=read(ROOT/'receipt.json');assert len(r['rows'])==224 and len(r['artifacts'])==28
assert r['status']=='passed-private-rounded-paint-baseline' and r['browser']=='153.0.8010.12'
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
 assert not x['pixelFailures']
assert sum(len(x['clearChecks'])for x in r['rows'])==448
assert len(r['repeated'])==140 and all(x['nativeIdentical']and x['chromeIdentical']for x in r['repeated'])
paths=read(ROOT/'path-receipt.json');assert len(paths['frames'])==224
for key in ['observer','pathParser']:bind(paths[key]['path'],paths[key]['sha256'])
for x in paths['frames']:
 bind(x['stream']['path'],x['stream']['sha256']);assert len(x['draws'])==2 and len(x['clips'])==5
 a=next(a for a in r['rows']if a['name']==x['name']and a['frame']==x['frame'])
 import math
 box=a['boxes']['v']; edges=[]
 for start,size in [(box['x'],box['width']),(box['y'],box['height'])]:
  lo=math.floor(start+.5);hi=math.floor(start+size+.5)
  if hi==lo and size>1/16:hi+=1
  edges.append([lo,hi])
 assert x['chromeSnappedAxes']==edges
 (l,rr),(t,b)=edges
 assert x['clips'][1:]==[[l,0,l+32768,32768],[rr-32768,0,rr,32768],[0,t,32768,t+32768],[0,b-32768,32768,b]]
coverage=read(ROOT/'visual/coverage.json');expected={s['path']:s for s in coverage['sheets']};notes={}
for file in ['inspection-agent.json','inspection-root.json']:
 inspection=read(ROOT/'visual'/file)
 for s in inspection.get('inspectedSheets',inspection.get('sheets')):
  assert s.get('observation',s.get('observations'));assert s['path']not in notes
  bind(s['path'],s['sha256']);assert s['sha256']==expected[s['path']]['sha256'];notes[s['path']]=s
assert set(notes)==set(expected) and len(notes)==24
for s in expected.values():
 for m in s['members']:
  for b in m['files']:bind(b['path'],b['sha256'])
for t in coverage['transfers']:
 a=next(x for x in r['rows']if x['name']==t['name']and x['frame']==t['frame']);b=next(x for x in r['rows']if x['name']==t['name']and x['frame']==t['donorFrame'])
 for k in ['chrome','native']:bind(t[k]['path'],t[k]['sha256']);assert t[k]['sha256']==a[k+'Sha256']==b[k+'Sha256']
 assert a['geometry']==b['geometry']and a['boxes']==b['boxes']
assert coverage['counts']==dict(frames=224,representatives=84,transfers=140,sheets=24,geometryPass=224,pixelPass=224)
review=dict(scope='84 distinct full pairs directly inspected in saved root/agent notes; 140 exact same-case transfers. Private prototype only.',visualReviewCompleted=True,counts=coverage['counts'],coverageSha256=bind(ROOT/'visual/coverage.json'),inspectedSheets=list(notes.values()))
(ROOT/'visual/review-receipt.json').write_text(json.dumps(review,indent=2)+'\n');bind(ROOT/'visual/review-receipt.json')
ba=read(ROOT/'before-after-receipt.json');old=read(ba['priorReceipt']['path']);bind(ba['priorReceipt']['path'],ba['priorReceipt']['sha256']);bind(ba['currentReceipt']['path'],ba['currentReceipt']['sha256'])
assert len(ba['comparisons'])==160
oldfail=0
for c in ba['comparisons']:
 a=next(x for x in old['rows']if x['name']==c['prior']and x['frame']==c['frame']);b=next(x for x in r['rows']if x['name']==c['current']and x['frame']==c['frame'])
 assert a['chromeSha256']==b['chromeSha256']==c['chromeSha256'] and a['boxes']==b['boxes']
 bind(str(a['prefix'])+'.chrome.png',c['chromeSha256'])
 assert c['priorPixelFailures']==a['pixelFailures'] and not c['currentPixelFailures'] and not b['pixelFailures']
 oldfail+=bool(a['pixelFailures'])
 for key in ['priorNative','currentNative']:bind(c[key]['path'],c[key]['sha256'])
assert oldfail==24
for sheet in ba['beforeAfterSheets']:
 bind(sheet['path'],sheet['sha256'])
 for a in sheet['files']:bind(a['path'],a['sha256'])
bind(ROOT/'visual/before-after-inspection.json')
# Scalar captures are checked numerically as IEEE f32; signed zeros may normalize.
import struct
f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
bits=lambda x:struct.pack('<f',x)
S=M/'output/rounding-scalar-r1';S2=M/'output/rounding-scalar-r2'
sr=read(S/'native-receipt.json');assert sr['frames']==1696 and not sr['failures'] and len(sr['cases'])==106
for a in sr['artifacts']:bind(a['path'],a['sha256'])
bind(S/'candidate',sr['constructorSha256']);bind(S/'node-probe',sr['probeSha256']);bind(S/'runner.py',sr['runnerSha256']);bind(S/'observations.json',sr['observationsSha256'])
obs=read(S/'observations.json');count=zero_sign=0
for o in obs:
 geometry={a['objectId']:a for a in read(S/'cases'/o['case']/'native'/f"frame-{o['frame']}.geometry.json")}
 axis=4 if o['case'].startswith('x-')else 5
 for c in o['checks']:
  actual=f32(geometry[c['id']]['worldMatrix'][axis]);assert actual==c['actual']==c['expected'] and c['pass_'];count+=1
  if bits(actual)!=bits(c['expected']):assert actual==0 and c['expected']==0;zero_sign+=1
assert count==410432
rep=read(S/'reproduction.json');assert len(rep['pairs'])==3922
bind(S/'native-receipt.json',rep['firstReceiptSha256']);bind(S2/'native-receipt.json',rep['secondReceiptSha256'])
for a in rep['pairs']:
 assert a['exact'] and a['firstSha256']==a['secondSha256'];bind(S/a['path'],a['firstSha256']);bind(S2/a['path'],a['secondSha256'])
correction=sr['observerCorrection'];initial=S/'initial-json-decimal-observer'
for key,name in [('initialReceiptSha256','native-receipt.json'),('initialObservationsSha256','observations.json'),('initialRunnerSha256','runner.py')]:bind(initial/name,correction[key])
assert len(read(initial/'native-receipt.json')['failures'])==1088
for directory in [S]:
 build=read(directory/'receipt.json')
 for b in read(directory/'source-bindings.json'):bind(b['source'],b['sha256']);bind(b['snapshot'],b['sha256'])
 for b in build['effectiveCrateInputs']:bind(b['path'],b['sha256'])
 for key,name in [('bridgeSha256','bridge.rs'),('patchesSha256','patches.json'),('buildLogSha256','build.log')]:bind(directory/name,build[key])
P=M/'output/rounding-product-build-r1';summary=read(P/'summary.json');assert summary['rustTests']==406 and summary['nodeTests']==56 and summary['sourceUnchanged']
identity=read(P/'prior-public-identity.json')
for a in identity['verifiedFiles']:bind(a['path'],a['sha256'])
for a in identity['binaries']:bind(a['current'],a['sha256']);bind(a['prior'],a['sha256']);assert a['exactBytes']
for name in ['rounding-scalar-review.md','rounding-paint-composition-review.md','rounding-paint-review.md']:bind(M/'validation'/name)
bind(__file__)
result=dict(status='verified-private-signed-rounding-and-paint',publicAdmission=False,counts=coverage['counts'],scalarFrames=1696,scalarNumericalChecks=count,scalarZeroSignDifferences=zero_sign,reproductionArtifacts=3922,clips=1120,clearChecks=448,preservedPriorPixelFailures=24,rustTests=406,nodeTests=56,bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())])
out=M/'output/rounding-paint-verification-r1.json';out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='bindings'}|dict(bindings=len(bindings))))
