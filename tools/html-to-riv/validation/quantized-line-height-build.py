"""Freeze/build a private source-driven candidate, then emit/observe ordinary files.
Usage: python3 validation/quantized-line-height-build.py NEW_OUTPUT
"""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
module=Path(__file__).resolve().parents[1]
repo=module.parents[1]
root=Path(sys.argv[1]).resolve()
assert not root.exists(), root
root.mkdir(parents=True)
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
source_paths=[module/'Cargo.toml',module/'Cargo.lock',module/'examples/quantized-line-height.rs',Path(__file__).resolve(),module/'validation/quantized-line-height-cases.json',module/'validation/quantized-line-height-native.mjs',module/'validation/pixels.mjs',module/'fixtures/fonts/roboto-regular.ttf',repo/'Cargo.lock',repo/'Cargo.toml']
source_paths+=sorted((module/'src').rglob('*.rs'))
source_paths+=sorted((repo/'crates/nuxie-schema').rglob('*.rs'))+[repo/'crates/nuxie-schema/Cargo.toml']
bindings=[]
for i,p in enumerate(source_paths):
 dest=root/'inputs'/str(i)/p.name;dest.parent.mkdir(parents=True);shutil.copyfile(p,dest)
 bindings.append(dict(path=str(p),snapshot=str(dest),sha256=sha(p)))
command=['cargo','build','--locked','--manifest-path',str(module/'Cargo.toml'),'--example','quantized-line-height']
r=subprocess.run(command,cwd=repo,capture_output=True,text=True)
(root/'build.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
for b in bindings:assert sha(Path(b['path']))==b['sha256']
shutil.copy2(module/'target/debug/examples/quantized-line-height',root/'generator')
probe=module/'output/reduced-line-height-r1/text-probe'
shutil.copy2(probe,root/'text-probe')
for p in [root/'generator',root/'text-probe']:
 bindings.append(dict(path=str(p),sha256=sha(p)))
write(root/'build-receipt.json',dict(command=command,bindings=bindings,scope='Private example build; no public API admission or runtime change.'))
cases=json.loads((module/'validation/quantized-line-height-cases.json').read_text())
write(root/'cases.json',cases)
commands=[]
for c in cases:
 d=root/c['name'];d.mkdir();write(d/'request.json',c['request'])
 cmd=[str(root/'generator'),str(d/'request.json'),str(module/'fixtures/fonts/roboto-regular.ttf'),str(d/'scene.riv')]
 r=subprocess.run(cmd,capture_output=True,text=True);(d/'generation.log').write_text(r.stdout+r.stderr)
 assert r.returncode==0,(c['name'],r.stderr)
 write(d/'plan.json',json.loads(r.stdout))
 cmd2=[str(root/'text-probe'),str(d/'scene.riv'),str(d/'text-observation'),'240x160','390x200','768x120','240x160']
 r2=subprocess.run(cmd2,capture_output=True,text=True);(d/'text-probe.log').write_text(r2.stdout+r2.stderr)
 assert r2.returncode==0,(c['name'],r2.stderr)
 commands.append(dict(name=c['name'],generation=cmd,observer=cmd2,rivSha256=sha(d/'scene.riv')))
write(root/'generation-receipt.json',commands)
print(json.dumps(dict(cases=len(cases),buildBindings=len(bindings),output=str(root))))
