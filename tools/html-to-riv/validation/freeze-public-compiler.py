#!/usr/bin/env python3
"""Freeze compiler artifacts and verify exact-byte reuse of public render receipts.
Usage: SCRIPT NATIVE_BINARY WASM_BINARY NEW_OUTPUT RUN_RECEIPT [RUN_RECEIPT ...]
No rendering or visual review is performed. Source snapshots are contemporaneous;
this script does not prove that a supplied binary was built from those sources.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

MODULE = Path(__file__).resolve().parents[1]

def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def read(p):
    return json.loads(Path(p).read_text())

def require(value, message):
    if not value:
        raise ValueError(message)

def source_files(module):
    paths = [p for name in ('src', 'js', 'tests') for p in (module/name).rglob('*') if p.is_file()]
    paths += [module/name for name in ('Cargo.toml', 'Cargo.lock', 'package.json', 'package-lock.json')]
    require(all(p.is_file() for p in paths), 'missing source manifest/lock')
    return {str(p.relative_to(module)): sha(p) for p in sorted(paths)}

def freeze(native, wasm, out, receipts, module=MODULE):
    native, wasm, out, module = [Path(p).resolve() for p in (native, wasm, out, module)]
    require(receipts, 'at least one render receipt is required')
    require(not out.exists(), 'fresh output directory required')
    receipts = [Path(p).resolve() for p in receipts]
    require(len(set(receipts)) == len(receipts), 'duplicate run receipt')
    snapshot = source_files(module)
    binaries = {'html-to-riv': (native, sha(native)), 'html-to-riv.wasm': (wasm, sha(wasm))}
    out.mkdir(parents=True)
    bindings = {}
    def bind(p, expected=None):
        p = Path(p).resolve(); digest = sha(p)
        require(expected is None or digest == expected, f'changed evidence: {p}')
        require(str(p) not in bindings or bindings[str(p)] == digest, f'evidence changed during run: {p}')
        bindings[str(p)] = digest
        return digest
    for relative, digest in snapshot.items():
        target = out/'source'/relative; target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(module/relative, target); require(sha(target) == digest, f'source changed while copying: {relative}')
    for name, (source, digest) in binaries.items():
        shutil.copy2(source, out/name);require(sha(out/name) == digest, 'binary changed while copying')
    # Historical byte snapshots may omit executable mode; adding owner execute
    # changes no binary bytes and never mutates the supplied snapshot.
    (out/'html-to-riv').chmod((out/'html-to-riv').stat().st_mode | 0o100)
    results = []
    invocations = []
    for run_index, receipt_path in enumerate(receipts):
        receipt_hash = bind(receipt_path); receipt = read(receipt_path); run = receipt_path.parent
        require(receipt['status'] == 'passed-public-baseline', 'receipt is not a terminal public pass')
        artifacts = receipt['artifacts']; rows = receipt['rows']
        require(artifacts and len(rows) == 8 * len(artifacts), 'incomplete original/clone receipt')
        names = [a['name'] for a in artifacts]
        require(len(set(names)) == len(names) and all(re.fullmatch('[a-z0-9-]+', n) for n in names), 'invalid artifact identities')
        require({r['name'] for r in rows} == set(names), 'receipt row/artifact mismatch')
        fixtures_path = run/'source-cases.json';bind(fixtures_path, receipt['fixturesSha256'])
        fixtures = {f['name']: f for f in read(fixtures_path)}
        require(len(fixtures) == len(read(fixtures_path)) and set(fixtures) == set(names), 'fixture identities mismatch')
        reset = run/'browser-reset.css';bind(reset)
        require(sha(reset) == snapshot['src/reset.css'], 'browser reset differs from frozen source')
        run_out = out/'runs'/str(run_index);run_out.mkdir(parents=True)
        for p in (receipt_path, fixtures_path, reset):shutil.copy2(p, run_out/p.name)
        drivers = []
        for filename, key in [('check-public-baseline.mjs', 'driverSha256'), ('pixels.mjs', 'pixelGateSha256')]:
            candidates = [run/filename, module/'validation'/filename]
            matching = next((p for p in candidates if p.is_file() and sha(p) == receipt[key]), None)
            if matching:
                bind(matching, receipt[key]);shutil.copy2(matching, run_out/filename)
            drivers.append(dict(name=filename,sha256=receipt[key],copied=matching is not None))
        for name in ('probe', 'renderer'):
            bind(receipt['tools'][name], receipt['toolHashes'][name])
        for artifact in artifacts:
            name = artifact['name']; folder = run/name; destination = run_out/name;destination.mkdir()
            request = folder/'request.json';riv = folder/'scene.riv';source_map = folder/'scene.map.json'
            for p, key in [(request,'requestSha256'),(riv,'rivSha256'),(source_map,'mapSha256'),(folder/'probe/frames.json','probeManifestSha256')]:bind(p,artifact[key])
            request_value=read(request)
            require(all(request_value[k] == fixtures[name][k] for k in ('html','css')), f'{name}: authored source mismatch')
            original_frames = read(folder/'probe/frames.json')['frames'];require(len(original_frames)==8, 'incomplete probe manifest')
            bind(folder/'probe/scene.riv', artifact['rivSha256'])
            case_rows = [r for r in rows if r['name']==name]
            require({r['frame'] for r in case_rows}==set(range(8)) and len(case_rows)==8,'duplicate/missing frame')
            for row in case_rows:
                frame = row['frame'];require(row['instance']==frame//4 and row['step']==frame%4, 'incorrect original/clone frame identity')
                require(not row['geometryFailures'] and not row['pixelFailures'], 'failed row in pass receipt')
                original=next(f for f in original_frames if f['frame']==frame)
                require(all(original[k]==row[k] for k in ('instance','step','width','height')), 'probe/render identity mismatch')
                require(row['requestSha256']==artifact['requestSha256'] and row['rivSha256']==artifact['rivSha256'] and row['sourceMapSha256']==artifact['mapSha256'],'row input hashes differ')
                require(Path(row['prefix']).resolve()==(folder/f'frame-{frame}').resolve(),'unexpected row prefix')
                for p,key in [(folder/'probe'/original['stream'],'streamSha256'),(folder/'probe'/original['geometry'],'geometrySha256'),(folder/f'frame-{frame}.chrome.png','chromeSha256'),(folder/f'frame-{frame}.native.png','nativeSha256')]:bind(p,row[key])
                for clear in row.get('clearChecks', []):
                    require(clear['samePixels'], 'failed canvas-clear independence check')
                    require(Path(clear['path']).resolve()==(folder/f'frame-{frame}.clear-{clear["name"]}.png').resolve(),'unexpected clear-check path')
                    bind(clear['path'],clear['sha256'])
            shutil.copy2(request,destination/'request.json')
            command=[str(out/'html-to-riv'),str(destination/'request.json'),str(destination/'scene.riv')]
            process=subprocess.run(command,capture_output=True,text=True)
            (destination/'compile.log').write_text(process.stdout+process.stderr)
            invocations.append(dict(argv=command,cwd=str(Path.cwd()),exitCode=process.returncode))
            require(process.returncode==0,f'{name}: frozen compiler rejected request')
            for name_out, original_path in [('scene.riv',riv),('scene.map.json',source_map)]:
                require((destination/name_out).read_bytes()==original_path.read_bytes(),f'{artifact["name"]}: complete {name_out} bytes differ')
            require(not (destination/'scene.requirements.json').exists(),'unexpected runtime policy sidecar')
            results.append(dict(run=run_index,name=name,requestSha256=artifact['requestSha256'],rivSha256=sha(destination/'scene.riv'),mapSha256=sha(destination/'scene.map.json'),frames=8))
        (run_out/'driver-bindings.json').write_text(json.dumps(drivers,indent=2)+'\n')
        require(bind(receipt_path)==receipt_hash,'run receipt changed')
    require(source_files(module)==snapshot,'source inventory/content changed during freeze')
    for name,(source,digest) in binaries.items():require(sha(source)==digest==sha(out/name),'binary changed during freeze')
    for p,digest in bindings.items():require(sha(p)==digest,f'evidence changed during freeze: {p}')
    manifest=dict(status='verified-byte-identical',scope='Exact request/Rive/source-map reproduction of terminal immutable render receipts; no rerender or new visual qualification',sourceBinding='Contemporaneous snapshot only; supplied binary build provenance must be bound separately',source=snapshot,files={name:dict(source=str(p),sha256=h) for name,(p,h) in binaries.items()},runs=[dict(path=str(p),sha256=bindings[str(p)]) for p in receipts],artifacts=results,invocations=invocations,evidence=[dict(path=p,sha256=h) for p,h in sorted(bindings.items())],scriptSha256=sha(__file__))
    manifest['frozenFiles']={str(p.relative_to(out)):sha(p) for p in sorted(out.rglob('*')) if p.is_file()}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    return manifest

if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('native',type=Path);parser.add_argument('wasm',type=Path);parser.add_argument('output',type=Path);parser.add_argument('receipts',type=Path,nargs='+');a=parser.parse_args()
    try:
        result=freeze(a.native,a.wasm,a.output,a.receipts)
    except (ValueError,KeyError,OSError) as e:
        parser.exit(1,f'Freeze rejected: {e}\n')
    print(json.dumps(dict(status=result['status'],artifacts=len(result['artifacts']),frames=sum(r['frames'] for r in result['artifacts']))))
