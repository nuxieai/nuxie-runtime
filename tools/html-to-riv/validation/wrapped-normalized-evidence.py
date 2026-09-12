"""Bind normalized construction, unchanged references and native visual evidence."""
from pathlib import Path
import hashlib,json
M=Path(__file__).resolve().parents[1]
bindings={}
def bind(p,expected=None):
 p=Path(p).resolve();h=hashlib.sha256(p.read_bytes()).hexdigest()
 if expected is not None:assert h==expected,str(p)
 bindings[str(p)]=h;return h
def read(p):bind(p);return json.loads(Path(p).read_text())
roots=[M/f'output/wrapped-normalized-constructor-r{i}' for i in [1,2]]
construct=[]
for root in roots:
 receipt=read(root/'construction-receipt.json');bind(root/'candidate',receipt['constructorSha256']);bind(root/'recipes.json',receipt['recipesSha256'])
 assert len(receipt['cases'])==80
 for case in receipt['cases']:
  assert case['exitCode']==0 and case['deterministic']
  for name,h in case['artifacts'].items():
   for suffix in ['', 'first','repeat']:bind(root/'cases'/case['name']/suffix/name,h)
 construct.append({c['name']:c['artifacts']for c in receipt['cases']})
assert construct[0]==construct[1]
for entry in read(roots[1]/'source-bindings.json'):
 bind(entry['source'],entry['sha256']);bind(entry['snapshot'],entry['sha256'])
counts={}
for campaign,oldname,count in [('boundary','wrapped-boundary-r1',256),('regression','wrapped-rounded-r1',384)]:
 root=M/f'output/playwright/wrapped-normalized-{campaign}-r1';r=read(root/'receipt.json');old=read(M/'output/playwright'/oldname/'receipt.json')
 prior={(x['name'],x['frame']):x for x in old['rows']};assert len(r['rows'])==count
 for name,path in r['tools'].items():bind(path,r['toolHashes'][name])
 assert r['pixelGateSha256']==old['pixelGateSha256'];bind(M/'validation/pixels.mjs',r['pixelGateSha256'])
 failures_fixed=0;identical=0
 for row in r['rows']:
  assert not row['geometryFailures'] and not row['pixelFailures']
  prev=prior[row['name'],row['frame']]
  assert row['chromeSha256']==prev['chromeSha256'] and row['boxes']==prev['boxes']
  for k in ['chrome','native']:
   bind(row['prefix']+'.'+k+'.png',row[k+'Sha256']);bind(prev['prefix']+'.'+k+'.png',prev[k+'Sha256'])
  failures_fixed+=bool(prev['pixelFailures'])
  identical+=row['nativeSha256']==prev['nativeSha256'] and row['geometry']==prev['geometry'] and row['rivSha256']==prev['rivSha256']
 observation=read(root/('boundary-observation' if campaign=='boundary' else 'rounded-observation')/'receipt.json')['counts']
 assert observation['scalarFailures']==observation['streamFailures']==0
 coverage=read(root/'visual/coverage.json')
 for sheet in coverage['sheets']:
  bind(sheet['path'],sheet['sha256'])
  for member in sheet['members']:
   for f in member['files']:bind(f['path'],f['sha256'])
 if campaign=='boundary':
  assert failures_fixed==28 and observation['chromeClippedEdgeDifferences']==0 and observation['chromeEdgeDifferences']==64
  for who in ['root','agent']:bind(M/f'validation/wrapped-normalized-boundary-{who}-visual.md')
 else:
  assert identical==384
  for name in ['wrapped-rounded-review.md','wrapped-rounded-receipt.json']:bind(M/'validation'/name)
 counts[campaign]=dict(frames=count,priorPixelFailuresRepaired=failures_fixed,exactPriorNativeTransfers=identical,observation=observation)
build=read(M/'output/wrapped-normalized-product-build-r1/summary.json');assert build['rustTests']==431 and build['nodeTests']==56
bind(M/'validation/wrapped-normalized-source-review.md')
for path in ['output/wrapped-normalized-proof-verification-r1.json','output/wrapped-normalized-source-review-r1/receipt.json','output/wrapped-normalized-source-review-r2/receipt.json']:
 bind(M/path)
bind(M/'validation/wrapped-normalized-proof-check.py')
bind(M/'validation/wrapped-normalized-public-regression-receipt.json')
bind(M/'validation/wrapped-normalized-public-regression-review.md')
bind(__file__)
(M/'validation/wrapped-normalized-receipt.json').write_text(json.dumps(dict(scope='Private fixed-layout normalization; no public wrapping admission. Native frames captured using r1 constructor; r2 outputs proven identical for all80recipes.',counts=counts,constructorArtifactTransfers=400,productBuild=build,bindings=bindings),indent=2)+'\n')
print(json.dumps(dict(counts=counts,bindings=len(bindings))))
