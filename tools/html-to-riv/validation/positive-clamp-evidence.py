#!/usr/bin/env python3
"""Recheck actual ordinary positive-predicate evidence; no new execution."""
from pathlib import Path
import hashlib,json,struct
M=Path(__file__).resolve().parents[1];R=M/'output/positive-clamp-r1';bindings={}
def bind(p,h=None):
 p=Path(p);v=hashlib.sha256(p.read_bytes()).hexdigest()
 if h is not None:assert v==h,p
 bindings[str(p.resolve())]=v
 return v
def read(p):bind(p);return json.loads(Path(p).read_text())
bits=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
for b in read(R/'source-bindings.json'):bind(b['source'],b['sha256']);bind(b['snapshot'],b['sha256'])
b=read(R/'receipt.json');bind(R/'candidate',b['candidateSha256'])
for key,file in [('patchesSha256','patches.json'),('bridgeSha256','bridge.rs'),('sourceBindingsSha256','source-bindings.json'),('buildLogSha256','build.log')]:bind(R/file,b[key])
for x in b['effectiveCrateInputs']:bind(x['path'],x['sha256'])
n=read(R/'native-receipt.json');assert n['frames']==352 and len(n['cases'])==22 and not n['failures']
bind(R/'candidate',n['constructorSha256']);bind(R/'node-probe',n['probeSha256']);bind(R/'runner.py',n['runnerSha256']);bind(R/'observations.json',n['observationsSha256'])
assert n['probeSha256']=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
for b in n['artifacts']:bind(b['path'],b['sha256'])
observations=read(R/'observations.json');assert len(observations)==352
for c in n['cases']:
 p=R/'cases'/c['name'];bind(p/'scene.riv',c['sceneSha256']);bind(p/'native/scene.riv',c['sceneSha256'])
 trace=read(p/'trace.json');frames=read(p/'native/frames.json')['frames'];assert len(frames)==16
 assert trace['gainBits']==bits(2**64) and len(trace['stages'])==4
 assert c['originalCloneExact'] and c['repeatExact']
 seen=[]
 for f in frames:
  row=next(x for x in observations if x['case']==c['name']and x['frame']==f['frame'])
  objects={x['objectId']:x for x in read(p/'native'/f['geometry'])};axis=4+(c['recipe']['axis']=='y')
  actual=[objects[i]['worldMatrix'][axis]for i in [trace['source']]+trace['stages']]
  for i in [trace['source']]+trace['stages']:assert objects[i]['worldMatrix'][:4]==[1,0,0,1]
  if c['recipe']['dynamic']:source=f['height'if axis==5 else'width']/2-50
  else:source=struct.unpack('<f',struct.pack('<I',c['recipe']['inputBits']))[0]
  expected=[source,max(0.,min(1.,source))]
  for _ in range(3):expected.append(max(0.,min(1.,f32(expected[-1]*2**64))))
  assert [bits(x)for x in actual]==[bits(x)for x in expected]==row['actualBits']==row['expectedBits']
  assert actual[-1]==(1. if source>0 else 0.) and not row['errors']
  seen.append(row['actualBits'])
 assert seen[:8]==seen[8:] and all(seen[i]==seen[0]for i in [3,6,8,11,14])
audit=read(R/'native-source-audit.json');assert audit['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3' and audit['baselineExact']
for k in ['source','snapshot']:bind(audit[k],audit['sha256'])
B=M/'output/immutable-baseline-toolchain-r2';manifest=read(B/'manifest.json')
for file,h in manifest['files'].items():bind(B/file,h)
for file in ['positive-clamp-bridge.rs','positive-clamp-build.py','positive-clamp-run.py','positive-clamp-review.md']:bind(M/'validation'/file)
bind(__file__)
result=dict(scope='Private positive scalar predicate; no rounding/paint/public qualification',cases=22,frames=352,observedScalars=1760,failedFrames=0,bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())])
(M/'output/positive-clamp-verification-r1.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='bindings'}|dict(artifactBindings=len(bindings))))
