#!/usr/bin/env python3
"""Build unchanged runtime/renderer through their own locked workspace.

Then link the read-only probe to the exact Cargo-produced runtime libraries.
The compiler's isolated workspace cannot affect this dependency resolution.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
MODULE = ROOT / 'tools/html-to-riv'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    if len(sys.argv) not in [2,3] or (len(sys.argv)==3 and sys.argv[2]!='--release'):
        raise SystemExit('usage: wrapped-lifecycle-build.py NEW_OUTPUT [--release]')
    release=len(sys.argv)==3
    profile='release' if release else 'debug'
    target=MODULE/('output/wrapped-lifecycle-release-build' if release else 'output/baseline-build')
    out = Path(sys.argv[1]).resolve()
    if out.exists():
        raise SystemExit('fresh output required')
    subprocess.run([sys.executable, str(MODULE/'validation/check-target-runtime.py')], cwd=ROOT, check=True)
    out.mkdir(parents=True)
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0')
    if not release:env['CARGO_PROFILE_DEV_DEBUG']='0'
    build = ['cargo', 'build', '--locked', '-p', 'rust-golden-runner', '-p', 'renderer-replay', '--features', 'renderer-replay/native-metal', '--message-format=json']
    if release:build.append('--release')
    (out/'build-command.json').write_text(json.dumps({'cwd':str(ROOT),'argv':build,'overrides':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_INCREMENTAL','CARGO_PROFILE_DEV_DEBUG'] if k in env},'rustflags':env.get('RUSTFLAGS'),'rustcWrapper':env.get('RUSTC_WRAPPER')},indent=2)+'\n')
    with (out/'build-messages.jsonl').open('w') as stdout, (out/'build.log').open('w') as stderr:
        subprocess.run(build,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,check=True)
    messages=[json.loads(line) for line in (out/'build-messages.jsonl').read_text().splitlines() if line.startswith('{')]
    command=['rustc','--edition=2024','-C','debuginfo=0',str(MODULE/'validation/wrapped-lifecycle-probe.rs'),'-o',str(out/'wrapped-lifecycle-probe'),'-L','dependency='+str(target/profile/'deps')]
    if release:command.extend(['-O','-C','lto=fat','-C','codegen-units=1'])
    for name in ['nuxie_runtime','nuxie_render_api']:
        matches=[m for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']==name]
        if len(matches)!=1: raise RuntimeError(f'ambiguous compiled artifact: {name}')
        libraries=[f for f in matches[0]['filenames'] if f.endswith('.rlib')]
        if len(libraries)!=1: raise RuntimeError(f'missing compiled rlib: {name}')
        command.extend(['--extern',name+'='+libraries[0]])
    for path in sorted({p for m in messages if m.get('reason')=='build-script-executed' for p in m['linked_paths']}):
        command.extend(['-L',path])
    (out/'probe-command.json').write_text(json.dumps(command,indent=2)+'\n')
    with (out/'probe-build.log').open('w') as log:
        subprocess.run(command,cwd=ROOT,stdout=log,stderr=log,check=True)
    shutil.copy2(target/profile/'renderer-replay',out/'renderer-replay')
    shutil.copy2(MODULE/'validation/wrapped-lifecycle-probe.rs',out/'wrapped-lifecycle-probe.rs')
    shutil.copy2(__file__,out/'wrapped-lifecycle-build.py')
    graph=['cargo','tree','--locked','-p','rust-golden-runner','-p','renderer-replay','--features','renderer-replay/native-metal','-e','normal,build,features']
    (out/'dependency-command.json').write_text(json.dumps(graph,indent=2)+'\n')
    (out/'dependency-features.txt').write_bytes(subprocess.check_output(graph,cwd=ROOT,env=env))
    (out/'rustc-version.txt').write_bytes(subprocess.check_output(['rustc','-vV'],cwd=ROOT))
    gate=subprocess.check_output([sys.executable,str(MODULE/'validation/check-target-runtime.py')],cwd=ROOT)
    (out/'source-identity.json').write_bytes(gate)
    for name in ['wrapped-lifecycle-probe','renderer-replay']:(out/name).chmod(0o555)
    linked=[]
    for p in sorted((target/profile/'deps').glob('*')):
        if p.is_file() and p.suffix in ['.rlib','.dylib','.a']:
            linked.append(dict(path=str(p),sha256=sha(p)))
    (out/'linked-library-bindings.json').write_text(json.dumps(linked,indent=2)+'\n')
    receipt={'profile':profile,'status':'built-immutable-baseline','scope':'Source identity and baseline-owned Cargo resolution; no feature qualification','baseline':'6c7ac16617835b5f581784ff08a9e779bb52faf3','rootCargoTomlSha256':sha(ROOT/'Cargo.toml'),'rootCargoLockSha256':sha(ROOT/'Cargo.lock'),'files':{p.name:sha(p) for p in sorted(out.iterdir()) if p.is_file()}}
    (out/'manifest.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({'status':receipt['status'],'manifest':str(out/'manifest.json')}))


if __name__=='__main__':main()
