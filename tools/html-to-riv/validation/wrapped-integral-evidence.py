"""Recheck private mixed integral-paint implementation and captured evidence."""
from pathlib import Path
import hashlib,json
M=Path(__file__).resolve().parents[1];bindings={}
def bind(p,h=None):
 p=Path(p).resolve();actual=hashlib.sha256(p.read_bytes()).hexdigest()
 if h is not None:assert actual==h,str(p)
 bindings[str(p)]=actual;return actual
def read(p):bind(p);return json.loads(Path(p).read_text())
C=M/'output/wrapped-integral-constructor-r1';construction=read(C/'construction-receipt.json')
bind(C/'candidate',construction['constructorSha256']);assert len(construction['cases'])==80
for entry in read(C/'source-bindings.json'):
 bind(entry['source'],entry['sha256']);bind(entry['snapshot'],entry['sha256'])
selections={}
for case in construction['cases']:
 assert case['exitCode']==0 and case['deterministic']
 for name,h in case['artifacts'].items():
  for suffix in ['', 'first','repeat']:bind(C/'cases'/case['name']/suffix/name,h)
 proof=read(C/'cases'/case['name']/'proof.json');n=len(proof['integralPaintOwners']);selections[n]=selections.get(n,0)+1
assert selections=={0:74,1:6}
counts={}
for campaign,frames in [('boundary',256),('regression',384),('resource',48)]:
 root=M/f'output/playwright/wrapped-integral-{campaign}-r1';r=read(root/'receipt.json');assert len(r['rows'])==frames
 for name,path in r['tools'].items():bind(path,r['toolHashes'][name])
 bind(M/'validation/pixels.mjs',r['pixelGateSha256'])
 prior=None
 if campaign!='resource':
  previous=read(M/f'output/playwright/wrapped-normalized-{campaign}-r1/receipt.json');prior={(x['name'],x['frame']):x for x in previous['rows']}
 same=0
 for row in r['rows']:
  assert not row['geometryFailures'] and not row['pixelFailures']
  for kind in ['chrome','native']:bind(row['prefix']+'.'+kind+'.png',row[kind+'Sha256'])
  bind(Path(row['prefix']).parent/'scene.riv',row['rivSha256']);bind(Path(row['prefix']).parent/'probe'/row['stream'],row['streamSha256'])
  if prior:
   old=prior[row['name'],row['frame']];assert row['chromeSha256']==old['chromeSha256'] and row['boxes']==old['boxes']
   for kind in ['chrome','native']:bind(old['prefix']+'.'+kind+'.png',old[kind+'Sha256'])
   same+=row['nativeSha256']==old['nativeSha256']
 observation=read(root/'integral-observation/receipt.json');o=observation['counts']
 assert o['frames']==frames and all(o[k]==0 for k in ['integralFailures','scalarFailures','streamFailures','chromeClippedEdgeDifferences','saturationIntersectionFailures'])
 coverage=read(root/'visual/coverage.json')
 for sheet in coverage['sheets']:
  bind(sheet['path'],sheet['sha256'])
  for member in sheet['members']:
   for image in member['files']:bind(image['path'],image['sha256'])
 counts[campaign]=dict(frames=frames,priorNativePNGMatches=same,observation=o)
assert counts['boundary']['priorNativePNGMatches']==248
assert counts['regression']['priorNativePNGMatches']==384
for file in ['wrapped-integral-boundary-visual.md','wrapped-integral-resource-visual.md','wrapped-normalized-review.md','wrapped-normalized-receipt.json','wrapped-rounded-review.md']:
 bind(M/'validation'/file)
resource=read(M/'output/wrapped-integral-resource-r1/construction-receipt.json');assert resource['maxOwners']==83 and resource['oneOverOwners']==84
for case in resource['cases']:
 bind(case['recipe']['path'],case['recipe']['sha256'])
 for run in case['runs']:bind(run['log']['path'],run['log']['sha256'])
 for image in case.get('artifacts',{}).values():bind(image['path'],image['sha256'])
 if case['owners']==84:assert case['noOutputArtifacts'] and all(r['exitCode']!=0 for r in case['runs'])
 else:assert case['deterministic'] and all(r['exitCode']==0 for r in case['runs'])
build=read(M/'output/wrapped-integral-product-build-r1/summary.json');prior=read(M/'output/wrapped-normalized-product-build-r1/summary.json')
assert build['rustTests']==442 and build['nodeTests']==56
for file,key in [('html-to-riv','compilerSha256'),('compiler.wasm','wasmSha256')]:
 assert build[key]==prior[key]
 for directory in ['wrapped-integral-product-build-r1','wrapped-normalized-product-build-r1']:bind(M/'output'/directory/'frozen'/file,build[key])
bind(M/'validation/wrapped-normalized-public-regression-receipt.json')
bind(M/'output/wrapped-integral-fractional-r1/receipt.json');bind(M/'validation/wrapped-integral-fractional-review.md')
bind(M/'output/wrapped-integral-lifecycle-release-r1/receipt.json')
bind(M/'validation/wrapped-integral-lifecycle-review.md')
bind(M/'validation/wrapped-integral-lifecycle-receipt.json')
bind(M/'output/wrapped-integral-lifecycle-verification-r1.json')
bind(__file__)
(M/'validation/wrapped-integral-receipt.json').write_text(json.dumps(dict(scope='Private source-certified integral paint with rounded fallback; no public wrapping admission. Regression direct-image review transfers through all384 exact Chrome/native pairs; geometry gates independently rerun.',selections=selections,counts=counts,productBuild=build,bindings=bindings),indent=2)+'\n')
print(json.dumps(dict(counts=counts,bindings=len(bindings))))
