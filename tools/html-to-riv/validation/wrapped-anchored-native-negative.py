"""Run pre-repair emitter against exactly the same authored native recipes."""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
root=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
source=subprocess.run(['git','show','49169f2bfb:tools/html-to-riv/src/wrapping.rs'],check=True,capture_output=True).stdout
(out/'wrapping.rs').write_bytes(source)
for name in ['wire.rs','harness.rs']:shutil.copy2(root/'sources'/name,out/name)
libs=root/'libraries'
cmd=['rustc','--edition=2024',str(out/'harness.rs'),'--extern','nuxie_schema='+str(next(libs.glob('libnuxie_schema-*.rlib'))),'--extern','nuxie_html_to_riv='+str(next(libs.glob('libnuxie_html_to_riv*.rlib'))),'-L','dependency='+str(libs),'-C','debuginfo=0','-o',str(out/'candidate')]
r=subprocess.run(cmd,capture_output=True,text=True);(out/'build.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
receipt=json.loads((root/'receipt.json').read_text());observations=json.loads((root/'observations.json').read_text());rows=[]
for case in receipt['cases']:
 name=case['name'];dest=out/name;dest.mkdir();prior=root/'cases'/name
 r=subprocess.run([str(out/'candidate'),str(dest),*case['arguments']],capture_output=True,text=True);(dest/'candidate.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
 assert sha(dest/'base.riv')==sha(prior/'base.riv')
 r=subprocess.run([str(root/'node-probe'),str(dest/'scene.riv'),str(dest/'scene'),*[f'{w}x{h}'for w,h in case['sizes']]],capture_output=True,text=True);(dest/'probe.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
 assert sha(dest/'scene.riv')==sha(dest/'scene/scene.riv')
 ids=json.loads((dest/'trace.json').read_text())['visible'];cross=1 if case['arguments'][0]=='row' else 0
 frames=[]
 for reference in [r for r in observations if r['case']==name]:
  fi=reference['frame'];geometry=json.loads((dest/f'scene/frame-{fi}.geometry.json').read_text());A={o['objectId']:o for o in geometry};frames.append(geometry)
  errors=[]
  for item in reference['observations']:
   got=A[ids[item['item']]]['worldMatrix'][4+cross];expected=item['expected']
   if abs(got-expected)>.1:errors.append({'item':item['item'],'expected':expected,'actual':got,'delta':got-expected})
  rows.append({'case':name,'frame':fi,'errors':errors})
 assert frames[:4]==frames[4:];assert frames[0]==frames[3]
(out/'observations.json').write_text(json.dumps(rows,indent=2)+'\n')
artifacts=[{'path':str(p),'sha256':sha(p)}for p in sorted(out.rglob('*'))if p.is_file()]
result={'scope':'Negative control: pre-repair emitter on identical base records and independently expected geometry','sourceCommit':'49169f2bfb','sourceSha256':sha(out/'wrapping.rs'),'positiveReceipt':{'path':str(root/'receipt.json'),'sha256':sha(root/'receipt.json')},'positiveObservations':{'path':str(root/'observations.json'),'sha256':sha(root/'observations.json')},'probeSha256':sha(root/'node-probe'),'geometryTolerance':.1,'frames':len(rows),'failedFrames':sum(bool(r['errors'])for r in rows),'failedItems':sum(len(r['errors'])for r in rows),'originalCloneRepeatExact':True,'baseBytesExact':True,'command':cmd,'artifacts':artifacts}
(out/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k]for k in ['frames','failedFrames','failedItems']}))
