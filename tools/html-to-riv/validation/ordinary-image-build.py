"""Freeze compiler-owned image experiment and authored assets; emit/observe files.
Usage: SCRIPT FRESH_OUTPUT_DIRECTORY
"""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
module=Path(__file__).resolve().parents[1];repo=module.parents[1];root=Path(sys.argv[1]).resolve();root.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
sources=[Path(__file__).resolve(),module/'examples/ordinary-image.rs',module/'Cargo.toml',module/'Cargo.lock',module/'package.json',module/'package-lock.json',module/'validation/ordinary-image-cases.json',module/'validation/ordinary-image-fixtures.py',module/'validation/ordinary-image-native.mjs',module/'validation/pixels.mjs']
assert all(p.is_file() for p in sources), 'Missing required source before freeze'
sources+=[p for p in sorted((module/'src').rglob('*')) if p.is_file()]+sorted((module/'fixtures/images/ordinary-r1').glob('*'))
bindings=[]
for i,p in enumerate(sources):
 target=root/'inputs'/str(i)/p.name;target.parent.mkdir(parents=True);shutil.copy2(p,target);bindings.append(dict(path=str(p),snapshot=str(target),sha256=sha(p)))
cmd=['cargo','build','--locked','--manifest-path',str(module/'Cargo.toml'),'--example','ordinary-image']
r=subprocess.run(cmd,cwd=repo,capture_output=True,text=True);(root/'build.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
for b in bindings:assert sha(Path(b['path']))==b['sha256']
for old,name in [(module/'target/debug/examples/ordinary-image','generator'),(module/'output/ordinary-image-source-r1/probe-build/ordinary-image-probe','image-probe')]:
 target=root/name;shutil.copy2(old,target);bindings.append(dict(path=str(target),sha256=sha(target)))
write(root/'build-receipt.json',dict(command=cmd,bindings=bindings,scope='Private authored-image experiment; immutable runtime tools; no public asset admission.'))
cases=json.loads((module/'validation/ordinary-image-cases.json').read_text());write(root/'cases.json',cases)
commands=[]
for c in cases:
 d=root/c['name'];d.mkdir();write(d/'request.json',c['request']);asset=module/'fixtures/images/ordinary-r1'/c['asset'];shutil.copy2(asset,d/c['asset'])
 cmd=[str(root/'generator'),str(d/'request.json'),str(d/c['asset']),str(d/'scene.riv')]
 r=subprocess.run(cmd,capture_output=True,text=True);(d/'generation.log').write_text(r.stdout+r.stderr);assert r.returncode==0,(c['name'],r.stderr)
 write(d/'plan.json',json.loads(r.stdout))
 observe=[str(root/'image-probe'),str(d/'scene.riv'),str(d/'image-observation'),'240x240','390x320','768x560','240x240']
 r=subprocess.run(observe,capture_output=True,text=True);(d/'image-probe.log').write_text(r.stdout+r.stderr);assert r.returncode==0,(c['name'],r.stderr)
 commands.append(dict(name=c['name'],generation=cmd,observer=observe,assetSha256=sha(asset),rivSha256=sha(d/'scene.riv')))
write(root/'generation-receipt.json',commands);print(json.dumps(dict(cases=len(cases),bindings=len(bindings))))
