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
receipts=[];resources=[]
for dirname,resource in [('wrapped-lifecycle-release-r1','wrapped-resource-r1'),('wrapped-integral-lifecycle-release-r1','wrapped-integral-resource-r1')]:
 r=read(M/'output'/dirname/'receipt.json');assert r['status']=='completed-native-lifecycle-measurement'and len(r['results'])==4
 for k in ['probe','toolchain','runner','cases','hardware']:entry(r[k])
 resource_root=M/'output'/resource;construction=read(resource_root/'construction-receipt.json');cases={c['name']:c for c in construction['cases']};entry(construction['constructor']);entry(construction['sourceBindings']);entry(construction['script'])
 for x in r['results']:
  assert x['exitCode']==0 and x['warmups']==2 and x['repeats']==9
  assert x['command'][0:3]==['/usr/bin/time','-l',r['probe']['path']] and x['command'][5:]==['2','9','320x200','160x320','96x240','320x200']
  entry(x['log']);entry(x['timings']);bind(x['scene'],x['sceneSha256']);assert Path(x['scene']).stat().st_size==x['fileBytes']
  case=cases[x['name']];entry(case['recipe'])
  for name,artifact in case['artifacts'].items():
   entry(artifact)
   for sub in ['first','repeat']:bind(resource_root/'cases'/x['name']/sub/name,artifact['sha256'])
  assert case['artifacts']['scene.riv']['sha256']==x['sceneSha256']
  costs=read(resource_root/'cases'/x['name']/'proof.json')['recordCosts'];assert sum(costs)==case['expectedCosts']['total']==x['objects']+1
  trial_root=Path(x['timings']['path']).parent;bind(trial_root/'scene.riv',x['sceneSha256']);d=read(x['timings']['path']);assert len(d['trials'])==11
  measured=[]
  for i,t in enumerate(d['trials']):
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
 comparisons.append(dict(name=name,roundedRecords=old['objects']+1,integralRecords=new['objects']+1,roundedBytes=old['fileBytes'],integralBytes=new['fileBytes'],roundedPeakRssBytes=old['processPeakResidentBytes'],integralPeakRssBytes=new['processPeakResidentBytes'],stages={k:dict(rounded=v,integral=ns[k],medianSpeedRatio=v['medianNs']/ns[k]['medianNs'])for k,v in os.items()}))
bind(__file__);bind(M/'validation/wrapped-integral-lifecycle-review.md')
result=dict(status='verified-matched-integral-lifecycle-comparison',publicCostQualification=False,scope='Same optimized immutable native probe, runner, hardware, numeric recipes and resize schedule; scene graphs differ intentionally. CPU recording only; RSS is process high-water, not retained memory.',cases=4,trials=88,measuredTrials=72,resizeDrawSamples=704,comparison=comparisons,bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())]);out=M/'output/wrapped-integral-lifecycle-verification-r1.json';out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(status=result['status'],cases=4,bindings=len(bindings))))
