#!/usr/bin/env python3
"""Sequential native lifecycle timing, explicitly excluding GPU execution.

usage: wrapped-lifecycle-run.py TOOLCHAIN CASES_JSON NEW_OUTPUT
CASES_JSON is [{name,scene,viewports:[[width,height],...]}]. Run only after the
coordinator declares the machine quiet; no benchmark runs during other captures.
"""
import hashlib,json,math,platform,re,statistics,subprocess,sys,time
from pathlib import Path
M=Path(__file__).resolve().parents[1];ROOT=M.parents[1]
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def write(p,d):p.write_text(json.dumps(d,indent=2)+'\n')
def command(args):return subprocess.check_output(args,text=True).strip()
def stats(values):
 values=sorted(values);return dict(samples=len(values),minNs=min(values),medianNs=statistics.median(values),p95Ns=values[max(0,math.ceil(len(values)*.95)-1)],maxNs=max(values))
def main():
 if len(sys.argv)!=4:raise SystemExit(__doc__)
 toolchain,cases_path,out=map(lambda s:Path(s).resolve(),sys.argv[1:]);assert not out.exists();out.mkdir(parents=True)
 manifest=json.loads((toolchain/'manifest.json').read_text());assert manifest['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
 for name,h in manifest['files'].items():assert sha(toolchain/name)==h,name
 assert sha(ROOT/'Cargo.toml')==manifest['rootCargoTomlSha256'] and sha(ROOT/'Cargo.lock')==manifest['rootCargoLockSha256']
 cases=json.loads(cases_path.read_text());assert len(cases)<=12 and len(set(c['name']for c in cases))==len(cases)
 hardware=dict(platform=platform.platform(),machine=platform.machine(),os=command(['sw_vers']),cpu=command(['sysctl','-n','machdep.cpu.brand_string']),memoryBytes=int(command(['sysctl','-n','hw.memsize'])),logicalCpus=int(command(['sysctl','-n','hw.ncpu'])))
 write(out/'hardware.json',hardware);(out/'cases.json').write_bytes(cases_path.read_bytes());(out/'runner.py').write_bytes(Path(__file__).read_bytes())
 results=[]
 for c in cases:
  assert re.fullmatch(r'[A-Za-z0-9_-]+',c['name']);scene=Path(c['scene']).resolve();digest=sha(scene);folder=out/c['name'];log=out/(c['name']+'.time.log')
  args=['/usr/bin/time','-l',str(toolchain/'wrapped-lifecycle-probe'),str(scene),str(folder),'2','9',*[f'{w}x{h}'for w,h in c['viewports']]]
  started=time.time_ns()
  with log.open('wb')as f:run=subprocess.run(args,stdout=f,stderr=f,cwd=ROOT)
  row=dict(name=c['name'],command=args,scene=str(scene),sceneSha256=digest,startUnixNs=started,endUnixNs=time.time_ns(),exitCode=run.returncode,log=dict(path=str(log),sha256=sha(log)))
  results.append(row);write(out/'partial.json',results)
  assert run.returncode==0,(c['name'],run.returncode)
  assert sha(scene)==sha(folder/'scene.riv')==digest
  d=json.loads((folder/'timings.json').read_text());trials=d['trials'];assert len(trials)==11 and sum(t['warmup']for t in trials)==2
  measured=[t for t in trials if not t['warmup']];assert len(set(t['objects']for t in trials))==1
  for t in trials:
   assert t['fileBytes']==scene.stat().st_size and len(t['frames'])==2*len(c['viewports'])
   for i,f in enumerate(t['frames']):assert f['instance']==i//len(c['viewports']) and f['step']==i%len(c['viewports']) and [f['width'],f['height']]==c['viewports'][f['step']]
  row.update(objects=trials[0]['objects'],fileBytes=trials[0]['fileBytes'],warmups=2,repeats=9,timings=dict(path=str(folder/'timings.json'),sha256=sha(folder/'timings.json')),stages={k:stats([t[k]for t in measured])for k in ['factorySetupNs','importNs','initialSettleNs','cloneNs','releaseNs']},frames={k:stats([f[k]for t in measured for f in t['frames']])for k in ['resizeUpdateNs','drawRecordingNs']})
  rss=re.search(r'^\s*(\d+)\s+maximum resident set size\s*$',log.read_text(),re.M);assert rss,'Darwin peak RSS missing';row['processPeakResidentBytes']=int(rss.group(1))
  write(out/'partial.json',results)
 receipt=dict(scope='Read-only ordinary immutable-runtime lifecycle CPU timings; draw is command recording, not GPU rendering. Two warmups then nine measured fresh-import trials per scene; peak RSS is whole-process high-water mark, not retained/leaked memory.',status='completed-native-lifecycle-measurement',toolchain=dict(path=str(toolchain/'manifest.json'),sha256=sha(toolchain/'manifest.json')),probe=dict(path=str(toolchain/'wrapped-lifecycle-probe'),sha256=sha(toolchain/'wrapped-lifecycle-probe')),runner=dict(path=str(out/'runner.py'),sha256=sha(out/'runner.py')),cases=dict(path=str(cases_path),sha256=sha(cases_path)),hardware=dict(path=str(out/'hardware.json'),sha256=sha(out/'hardware.json')),results=results)
 write(out/'receipt.json',receipt);print(json.dumps(dict(status=receipt['status'],cases=len(results),receipt=str(out/'receipt.json'))))
if __name__=='__main__':main()
