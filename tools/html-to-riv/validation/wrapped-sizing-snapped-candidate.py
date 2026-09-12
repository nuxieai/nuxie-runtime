"""Freeze actual private compiler modules and replace only sizing/paint graphs.

The raw three-slot recipes are retained historical compiler output. No browser
measurement is read by the generated adapter or native scene constructor.
"""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
M=Path(__file__).resolve().parents[1]
root=Path(sys.argv[1]).resolve();root.mkdir(parents=True,exist_ok=False)
prior=M/'output/wrapped-snapped-layout-r1'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def write(p,v):Path(p).write_text(json.dumps(v,indent=2)+'\n')
def bind(p):return {'path':str(Path(p).resolve()),'sha256':sha(p)}
bindings=[]
def freeze(source,target):
 source=Path(source);target=Path(target);target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target)
 b=bind(source);assert sha(target)==b['sha256'];b['snapshot']=str(target);bindings.append(b)
for p in sorted((M/'src').rglob('*')):
 if p.is_file():freeze(p,root/'inputs/src'/p.relative_to(M/'src'))
for n in ['Cargo.toml','Cargo.lock']:freeze(M/n,root/'inputs'/n)
for n in ['wrapping.rs','wrapping_paint.rs','wire.rs']:freeze(M/'src'/n,root/'sources'/n)
freeze(M/'validation/wrapped-sizing-snapped-harness.rs',root/'sources/harness.rs')
freeze(__file__,root/'candidate-builder.py')
for n in ['pairs.json','chrome-cases.json']:freeze(prior/n,root/n)
command=['cargo','build','--manifest-path',str(M/'Cargo.toml'),'--locked','--lib','--message-format=json']
r=subprocess.run(command,cwd=M,capture_output=True,text=True);(root/'cargo-build.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
artifacts=[json.loads(s) for s in r.stdout.splitlines() if s.startswith('{')]
libs={}
for a in artifacts:
 if a.get('reason')!='compiler-artifact':continue
 for f in a['filenames']:
  if Path(f).suffix in ['.rlib','.rmeta','.dylib','.so']:
   dest=root/'libraries'/Path(f).name;freeze(f,dest)
   if Path(f).suffix=='.rlib':libs[a['target']['name']]=dest
cmd=['rustc','--edition=2024',str(root/'sources/harness.rs'),'--extern','nuxie_schema='+str(libs['nuxie_schema']),'--extern','nuxie_html_to_riv='+str(libs['nuxie_html_to_riv']),'-L','dependency='+str(root/'libraries'),'-C','debuginfo=0','-o',str(root/'augment')]
r=subprocess.run(cmd,cwd=M,capture_output=True,text=True);(root/'harness-build.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
cases=json.loads((root/'pairs.json').read_text());assert len(cases)==48
for c in cases:
 old=prior/'render'/c['name'];dest=root/'raw'/c['name']
 req=json.loads((old/'request.json').read_text());assert req['html']==c['html'] and req['css']==c['css']
 for n in ['request.json','scene.raw.riv','scene.raw.map.json']:freeze(old/n,dest/n)
adapter='''#!/usr/bin/env python3
from pathlib import Path
import json,sys,subprocess,shutil
p=Path(__file__).resolve().parent
request,out=map(Path,sys.argv[1:]);x=json.loads(request.read_text())
c=next(c for c in json.loads((p/'pairs.json').read_text()) if c['html']==x['html'] and c['css']==x['css'])
raw=p/'raw'/c['name'];assert json.loads((raw/'request.json').read_text())==x
for n in ['scene.raw.riv','scene.raw.map.json']:shutil.copy2(raw/n,out.with_suffix('.raw.riv' if n.endswith('.riv') else '.raw.map.json'))
mapping=json.loads((raw/'scene.raw.map.json').read_text());ids={n['id']:n['object_id'] for n in mapping}
subprocess.run([str(p/'augment'),str(raw/'scene.raw.riv'),str(out),','.join(str(ids[n]) for n in ['p','sb','sa','sc','b','a','c','ob','oa','oc']),c['axis'],str(c['wrapValue']),str(c['level']),str(c['dval']),str(out.with_suffix('.sizing-trace.json')),str(out.with_suffix('.sized.riv'))],check=True)
out.with_suffix('.map.json').write_text(json.dumps([n for n in mapping if n['id'] not in ['sa','sb','sc']]))
'''
(root/'adapter.py').write_text(adapter);(root/'adapter.py').chmod(0o755)
assert all(sha(b['path'])==sha(b['snapshot'])==b['sha256'] for b in bindings)
write(root/'build-receipt.json',{'scope':'Private snapped sizing and existing snapped paint; retained source recipes, no public admission','epsilon':1/64,'epsilonIsAdmissionCertificate':False,'commands':[command,cmd],'bindings':bindings,'artifacts':[bind(root/n) for n in ['cargo-build.log','harness-build.log','augment','adapter.py']]})
print(json.dumps({'root':str(root),'cases':len(cases),'frozenBindings':len(bindings)}))
