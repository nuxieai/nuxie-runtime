#!/usr/bin/env python3
"""Offline matched immutable-probe comparison; recomputes metrics, never times."""
from pathlib import Path
import hashlib,json,math,re,statistics
M=Path(__file__).resolve().parents[1];bindings={}
def bind(p,h=None):
 p=Path(p);a=hashlib.sha256(p.read_bytes()).hexdigest()
 if h is not None:assert a==h,p
 bindings[str(p.resolve())]=a;return a
def read(p):bind(p);return json.loads(Path(p).read_text())
def entry(v):bind(v['path'],v['sha256'])
def stats(v):
 v=sorted(v);return dict(samples=len(v),minNs=min(v),medianNs=statistics.median(v),p95Ns=v[max(0,math.ceil(len(v)*.95)-1)],maxNs=max(v))
def recipe_shape(v):
 if isinstance(v,dict):return {k:recipe_shape(x)for k,x in v.items()if k!='source'and x is not None}
 if isinstance(v,list):return list(map(recipe_shape,v))
 return v
receipts=[];resources=[];trial_count=measured_count=frame_count=measured_frame_count=0
for dirname,resource in [('wrapped-integral-lifecycle-release-r1','wrapped-integral-resource-r1'),('wrapped-original-lifecycle-release-r1','wrapped-original-resource-r2')]:
 r=read(M/'output'/dirname/'receipt.json');assert r['status']=='completed-native-lifecycle-measurement'and len(r['results'])==4
 for k in ['probe','toolchain','runner','cases','hardware']:entry(r[k])
 resource_root=M/'output'/resource;construction=read(resource_root/'construction-receipt.json');cases={c['name']:c for c in construction['cases']};entry(construction['constructor']);entry(construction['sourceBindings']);entry(construction['script'])
 if resource=='wrapped-original-resource-r2':
  assert construction['maxOwners'] is None and construction['largestAcceptedTested']==84
  assert construction['theoreticalBudgetOwners']==1562 and construction['theoreticalOneOverOwners']==1563
  for n in [1562,1563]:
   c=cases[f'owners-{n}'];assert c['expectedRejection']=='Derived: Arithmetic(Separation)' and c['noOutputArtifacts']
   entry(c['recipe'])
   for run in c['runs']:
    entry(run['log']);assert run['exitCode']==1 and 'Derived: Arithmetic(Separation)' in Path(run['log']['path']).read_text()
 for x in r['results']:
  assert x['exitCode']==0 and x['warmups']==2 and x['repeats']==9
  assert x['command'][0:3]==['/usr/bin/time','-l',r['probe']['path']] and x['command'][5:]==['2','9','320x200','160x320','96x240','320x200']
  entry(x['log']);entry(x['timings']);bind(x['scene'],x['sceneSha256']);assert Path(x['scene']).stat().st_size==x['fileBytes']
  case=cases[x['name']];entry(case['recipe'])
  for name,artifact in case['artifacts'].items():
   entry(artifact)
   for sub in ['first','repeat']:bind(resource_root/'cases'/x['name']/sub/name,artifact['sha256'])
  assert case['artifacts']['scene.riv']['sha256']==x['sceneSha256']
  proof=read(resource_root/'cases'/x['name']/'proof.json');costs=proof['recordCosts'];assert sum(costs)==case['expectedCosts']['total']==x['objects']+1
  if resource=='wrapped-original-resource-r2':
   n=case['owners'];assert costs==[7+6*n,58*n-33,0] and proof['originalPaintOwners']==proof['integralPaintOwners']==list(range(8,8+6*n,6)) and proof['paintBoxes']==[]
   assert (x['objects']+1,x['fileBytes'])=={1:(38,558),3:(166,2344),8:(486,7051),34:(2150,31615)}[n]
  trial_root=Path(x['timings']['path']).parent;bind(trial_root/'scene.riv',x['sceneSha256']);d=read(x['timings']['path']);assert len(d['trials'])==11
  measured=[]
  for i,t in enumerate(d['trials']):
   trial_count+=1;frame_count+=len(t['frames'])
   if i>=2:measured_count+=1;measured_frame_count+=len(t['frames'])
   assert t['trial']==i and t['warmup']==(i<2) and t['objects']==x['objects']and t['fileBytes']==x['fileBytes']and len(t['frames'])==8
   for j,f in enumerate(t['frames']):assert f['instance']==j//4 and f['step']==j%4 and [f['width'],f['height']]==[[320,200],[160,320],[96,240],[320,200]][j%4]
   if i>=2:measured.append(t)
  for k,s in x['stages'].items():assert stats([t[k]for t in measured])==s
  for k,s in x['frames'].items():assert stats([f[k]for t in measured for f in t['frames']])==s
  rss=re.search(r'^\s*(\d+)\s+maximum resident set size\s*$',Path(x['log']['path']).read_text(),re.M);assert rss and int(rss[1])==x['processPeakResidentBytes']
 receipts.append(r);resources.append(resource_root)
a,b=receipts
for k in ['probe','toolchain','runner','hardware']:assert a[k]['sha256']==b[k]['sha256']
tool=Path(a['toolchain']['path']).parent;tm=read(tool/'manifest.json');assert tm['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'and tm['profile']=='release'
for name,h in tm['files'].items():bind(tool/name,h)
for entry_ in read(tool/'linked-library-bindings.json'):entry(entry_)
bind(M.parents[1]/'Cargo.toml',tm['rootCargoTomlSha256']);bind(M.parents[1]/'Cargo.lock',tm['rootCargoLockSha256'])
comparisons=[]
for old,new in zip(a['results'],b['results']):
 assert old['name']==new['name'];name=old['name'];assert recipe_shape(read(resources[0]/'cases'/name/'recipe.json'))==recipe_shape(read(resources[1]/'cases'/name/'recipe.json'))
 os=old['stages']|old['frames'];ns=new['stages']|new['frames']
 comparisons.append(dict(name=name,integralRecords=old['objects']+1,originalRecords=new['objects']+1,integralBytes=old['fileBytes'],originalBytes=new['fileBytes'],integralPeakRssBytes=old['processPeakResidentBytes'],originalPeakRssBytes=new['processPeakResidentBytes'],stages={k:dict(integral=v,original=ns[k],medianSpeedRatio=v['medianNs']/ns[k]['medianNs'])for k,v in os.items()}))
assert (trial_count,measured_count,frame_count,measured_frame_count)==(88,72,704,576)
bind(__file__);bind(M/'validation/wrapped-original-lifecycle-review.md')
result=dict(status='verified-matched-original-lifecycle-comparison',publicCostQualification=False,scope='Same optimized immutable native probe, runner, hardware, numeric recipes and resize schedule; scene graphs differ intentionally. CPU recording only; RSS is process high-water, not retained memory.',cases=4,trials=trial_count,measuredTrials=measured_count,resizeDrawSamples=frame_count,measuredResizeDrawSamples=measured_frame_count,comparison=comparisons,bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())]);out=M/'output/wrapped-original-lifecycle-verification-r1.json';out.write_text(json.dumps(result,indent=2)+'\n');(M/'validation/wrapped-original-lifecycle-receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(status=result['status'],cases=4,bindings=len(bindings))))
