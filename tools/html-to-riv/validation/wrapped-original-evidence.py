"""Bind original-paint construction, fallback transfers and native observations."""
from pathlib import Path
import hashlib,json
M=Path(__file__).resolve().parents[1];bindings={}
def bind(p,h=None):
 p=Path(p).resolve();v=hashlib.sha256(p.read_bytes()).hexdigest()
 if h is not None:assert v==h,str(p)
 bindings[str(p)]=v;return v
def read(p):bind(p);return json.loads(Path(p).read_text())
C=M/'output/wrapped-original-constructor-r1';r=read(C/'construction-receipt.json');assert len(r['cases'])==80
bind(C/'candidate',r['constructorSha256'])
for b in read(C/'source-bindings.json'):bind(b['source'],b['sha256']);bind(b['snapshot'],b['sha256'])
for case in r['cases']:
 assert case['exitCode']==0 and case['deterministic']
 folder=C/'cases'/case['name']
 for name,h in case['artifacts'].items():
  for suffix in ['', 'first','repeat']:bind(folder/suffix/name,h)
 proof=read(folder/'proof.json');assert proof['originalPaintOwners'] is None
 for name in ['scene.riv','base.riv','scene.map.json','trace.json']:
  assert bind(folder/name)==bind(M/'output/wrapped-integral-constructor-r1/cases'/case['name']/name)
bind(M/'validation/wrapped-integral-receipt.json')
resource=read(M/'output/wrapped-original-resource-r2/construction-receipt.json');assert resource['maxOwners'] is None and resource['largestAcceptedTested']==84
for case in resource['cases']:
 bind(case['recipe']['path'],case['recipe']['sha256'])
 for run in case['runs']:bind(run['log']['path'],run['log']['sha256'])
 if case['owners']>=1562:
  assert case['expectedRejection']=='Derived: Arithmetic(Separation)' and case['noOutputArtifacts']
  assert all(run['exitCode']!=0 for run in case['runs'])
 else:
  assert case['deterministic'] and all(run['exitCode']==0 for run in case['runs'])
  for name,a in case['artifacts'].items():bind(a['path'],a['sha256'])
  proof=read(Path(case['artifacts']['proof.json']['path']));assert len(proof['originalPaintOwners'])==case['owners'] and proof['recordCosts'][2]==0
  assert sum(proof['recordCosts'])==64*case['owners']-26
root=M/'output/playwright/wrapped-original-resource-r1';capture=read(root/'receipt.json');assert len(capture['rows'])==72
for name,path in capture['tools'].items():bind(path,capture['toolHashes'][name])
bind(M/'validation/pixels.mjs',capture['pixelGateSha256'])
for row in capture['rows']:
 assert not row['geometryFailures'] and not row['pixelFailures']
 for kind in ['chrome','native']:bind(row['prefix']+'.'+kind+'.png',row[kind+'Sha256'])
 bind(Path(row['prefix']).parent/'scene.riv',row['rivSha256']);bind(Path(row['prefix']).parent/'probe'/row['stream'],row['streamSha256'])
initial=read(root/'original-observation/receipt.json');assert initial['counts']['streamFailures']==8
observation=read(root/'original-observation-r2/receipt.json');counts=observation['counts']
assert counts['frames']==72 and all(counts[k]==0 for k in ['streamFailures','scalarFailures','integralFailures','chromeClippedEdgeDifferences'])
for k in ['script','parser','fallbackObserver']:
 if k in observation:bind(observation[k]['path'],observation[k]['sha256'])
coverage=read(root/'visual/coverage-r2.json');assert coverage['counts']['representatives']==27 and coverage['counts']['transfers']==45
for sheet in coverage['sheets']:
 bind(sheet['path'],sheet['sha256'])
 for member in sheet['members']:
  for image in member['files']:bind(image['path'],image['sha256'])
bind(M/'validation/wrapped-original-resource-visual.md')
bind(M/'output/wrapped-original-fractional-r1/receipt.json');bind(M/'validation/wrapped-original-fractional-review.md')
bind(M/'output/wrapped-original-lifecycle-release-r1/receipt.json')
build=read(M/'output/wrapped-original-product-build-r1/summary.json');previous=read(M/'output/wrapped-integral-product-build-r1/summary.json');assert build['rustTests']==450 and build['nodeTests']==56
for name,key in [('html-to-riv','compilerSha256'),('compiler.wasm','wasmSha256')]:
 assert build[key]==previous[key]
 for d in ['wrapped-original-product-build-r1','wrapped-integral-product-build-r1']:bind(M/'output'/d/'frozen'/name,build[key])
bind(M/'validation/wrapped-normalized-public-regression-receipt.json')
bind(M/'validation/wrapped-original-transparent-review.md')
bind(M/'validation/wrapped-original-lifecycle-review.md')
bind(M/'validation/wrapped-original-lifecycle-receipt.json')
bind(M/'output/wrapped-original-lifecycle-verification-r1.json')
bind(__file__)
(M/'validation/wrapped-original-receipt.json').write_text(json.dumps(dict(scope='Private whole-scene nonoverlap original-paint optimization.80 existing fallback scenes retain exact RIV/map/trace bytes; their640previous native/Chrome frames transfer without rerendering.72new frames captured. Public wrapping remains unadmitted.',counts=counts,unchangedFallbackScenes=80,transferredFrames=640,productBuild=build,bindings=bindings),indent=2)+'\n')
print(json.dumps(dict(status='verified-original-paint-checkpoint',counts=counts,bindings=len(bindings))))
