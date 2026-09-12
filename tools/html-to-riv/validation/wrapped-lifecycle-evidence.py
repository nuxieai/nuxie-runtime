#!/usr/bin/env python3
"""Recheck frozen lifecycle timings and distributions; never reruns benchmarks."""
from pathlib import Path
import hashlib,json,math,statistics
M=Path(__file__).resolve().parents[1];bindings={}
def bind(p,h=None):
 p=Path(p);actual=hashlib.sha256(p.read_bytes()).hexdigest()
 if h is not None:assert h==actual,p
 bindings[str(p.resolve())]=actual;return actual
def read(p):bind(p);return json.loads(Path(p).read_text())
def stats(v):
 v=sorted(v);return dict(samples=len(v),minNs=min(v),medianNs=statistics.median(v),p95Ns=v[max(0,math.ceil(len(v)*.95)-1)],maxNs=max(v))
def entry(b):bind(b['path'],b['sha256'])
results=[]
for directory in ['wrapped-lifecycle-r1','wrapped-lifecycle-release-r1']:
 r=read(M/'output'/directory/'receipt.json');assert r['status']=='completed-native-lifecycle-measurement'
 for k in ['toolchain','probe','runner','cases','hardware']:entry(r[k])
 tool=Path(r['toolchain']['path']).parent;manifest=read(tool/'manifest.json')
 assert manifest['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
 for name,h in manifest['files'].items():bind(tool/name,h)
 bind(M.parents[1]/'Cargo.toml',manifest['rootCargoTomlSha256']);bind(M.parents[1]/'Cargo.lock',manifest['rootCargoLockSha256'])
 for b in read(tool/'linked-library-bindings.json'):entry(b)
 assert len(r['results'])==4
 for row in r['results']:
  assert row['exitCode']==0 and row['warmups']==2 and row['repeats']==9
  entry(row['log']);entry(row['timings']);bind(row['scene'],row['sceneSha256'])
  p=Path(row['timings']['path']).parent;bind(p/'scene.riv',row['sceneSha256'])
  d=read(row['timings']['path']);assert d['drawScope']=='CPU RecordingFactory command recording; no GPU render or presentation'
  assert len(d['trials'])==11
  measured=[]
  for i,t in enumerate(d['trials']):
   assert t['trial']==i and t['warmup']==(i<2) and t['objects']==row['objects'] and t['fileBytes']==row['fileBytes']
   assert t['recordedStreamBytes']>0 and len(t['frames'])==8
   for j,f in enumerate(t['frames']):
    assert f['instance']==j//4 and f['step']==j%4
    assert [f['width'],f['height']]==[[320,200],[160,320],[96,240],[320,200]][j%4]
    assert f['resizeUpdateNs']>=0 and f['drawRecordingNs']>=0
   if i>=2:measured.append(t)
  for k,v in row['stages'].items():assert stats([t[k]for t in measured])==v
  for k,v in row['frames'].items():assert stats([f[k]for t in measured for f in t['frames']])==v
  owners=int(row['name'].split('-')[1]);assert row['objects']+1==16*owners*owners+2376*owners-33
 results.append(r)
a,b=results
assert a['cases']['sha256']==b['cases']['sha256'] and a['runner']['sha256']==b['runner']['sha256'] and a['hardware']['sha256']==b['hardware']['sha256']
at=Path(a['toolchain']['path']).parent;bt=Path(b['toolchain']['path']).parent
assert bind(at/'wrapped-lifecycle-probe.rs')==bind(bt/'wrapped-lifecycle-probe.rs')==bind(M/'validation/wrapped-lifecycle-probe.rs')
release_command=read(bt/'build-command.json');assert '--release'in release_command['argv'];assert set(release_command['overrides'])=={'CARGO_TARGET_DIR','CARGO_INCREMENTAL'}
link=read(bt/'probe-command.json');assert '-O'in link and 'lto=fat'in link and 'codegen-units=1'in link
comparison=[]
for x,y in zip(a['results'],b['results']):
 assert x['name']==y['name'] and x['sceneSha256']==y['sceneSha256'] and x['fileBytes']==y['fileBytes'] and x['objects']==y['objects']
 comparison.append(dict(name=x['name'],fileBytes=x['fileBytes'],records=x['objects']+1,sceneSha256=x['sceneSha256'],devStages=x['stages']|x['frames'],releaseStages=y['stages']|y['frames'],devProcessPeakResidentBytes=x['processPeakResidentBytes'],releaseProcessPeakResidentBytes=y['processPeakResidentBytes']))
failed=M/'output/wrapped-lifecycle-toolchain-release-r1';f=read(failed/'failure-receipt.json')
assert f['status']=='standalone-probe-link-failed'
for name,h in f['files'].items():bind(failed/name,h)
bind(M/'output/wrapped-resource-r1/construction-receipt.json');bind(M/'validation/wrapped-lifecycle-review.md');bind(__file__)
out=M/'output/wrapped-lifecycle-verification-r1.json';result=dict(status='verified-native-lifecycle-measurements',publicQualification=False,scope='Matched development and optimized standard release runtime CPU timings; no GPU render, production latency guarantee or leak qualification.',cases=4,totalTrials=88,measuredTrials=72,resizeDrawSamples=704,comparison=comparison,bindings=[dict(path=p,sha256=h)for p,h in sorted(bindings.items())]);out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(status=result['status'],bindings=len(bindings),cases=4,totalTrials=88)))
