"""Freeze a public image candidate and compile its finite admission corpus.

ROOT must be fresh. --build reuses a completed public-value-build.py snapshot;
otherwise this performs a source-bound native candidate build only.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('root', type=Path)
parser.add_argument('--build', type=Path)
parser.add_argument('--previous', type=Path)
args = parser.parse_args()
module = Path(__file__).resolve().parents[1]
root = args.root.resolve()
root.mkdir(parents=True, exist_ok=False)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, data):
    path.write_text(json.dumps(data, indent=2) + '\n')


if args.build:
    build = args.build.resolve()
    summary = json.loads((build / 'summary.json').read_text())
    assert summary['status'] == 'public-build-and-transport-pass'
    frozen = build / 'frozen'
    for row in json.loads((frozen / 'source-bindings.json').read_text())['files']:
        assert sha(Path(row.get('snapshot', row['path']))) == row['sha256']
    (root / 'frozen').symlink_to(frozen, target_is_directory=True)
    write(root / 'build-receipt.json', dict(scope='Reuses completed public compiler build; image native qualification is separate',
          build=str(build), summarySha256=sha(build / 'summary.json'), compilerSha256=sha(frozen / 'html-to-riv'),
          bindingsSha256=sha(frozen / 'source-bindings.json')))
else:
    paths = sorted({*(p for directory in ['src', 'js', 'fixtures/images'] for p in (module / directory).rglob('*') if p.is_file()),
                    *(module / name for name in ['Cargo.toml', 'Cargo.lock', 'validation/public-image-cases.json']),
                    Path(__file__).resolve()})
    before = [(p, sha(p)) for p in paths]
    command = ['cargo', 'build', '--locked', '--manifest-path', str(module / 'Cargo.toml'), '--bin', 'html-to-riv']
    with (root / 'build.log').open('wb') as log:
        subprocess.run(command, cwd=module, stdout=log, stderr=subprocess.STDOUT, check=True)
    assert all(sha(p) == digest for p, digest in before), 'Source changed during candidate build'
    frozen = root / 'frozen'
    bindings = []
    for source, digest in before:
        snapshot = frozen / 'inputs' / source.relative_to(module)
        snapshot.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, snapshot)
        assert sha(snapshot) == digest
        bindings.append(dict(path=str(source), snapshot=str(snapshot), sha256=digest))
    shutil.copy2(module / 'target/debug/html-to-riv', frozen / 'html-to-riv')
    write(root / 'build-receipt.json', dict(scope='Public native image candidate only; full module/WASM validation separate',
          command=command, bindings=bindings, compilerSha256=sha(frozen / 'html-to-riv'), logSha256=sha(root / 'build.log')))

cases_source = frozen / 'inputs/validation/public-image-cases.json'
shutil.copy2(cases_source, root / 'cases.json')
cases = json.loads(cases_source.read_text())
results = []
for case in cases:
    directory = root / case['name']
    directory.mkdir()
    request = dict(case['input'])
    request['assets'] = {key: dict(kind='image', bytes=list((frozen / 'inputs' / file).read_bytes()))
                         for key, file in case['assetFiles'].items()}
    write(directory / 'request.json', request)
    command = [str(frozen / 'html-to-riv'), str(directory / 'request.json'), str(directory / 'scene.riv')]
    result = subprocess.run(command, capture_output=True, timeout=30)
    (directory / 'compile.log').write_bytes(result.stdout + result.stderr)
    compiled = result.returncode == 0
    expected = case['expected']
    diagnostics = None if compiled else json.loads(result.stderr)
    matches = (compiled and expected['outcome'] == 'compile') or (
        not compiled and expected['outcome'] == 'diagnostic' and diagnostics[0]['code'] == expected['code']
        and expected.get('messageIncludes', '') in diagnostics[0]['message'])
    row = dict(name=case['name'], command=command, status=result.returncode, compiled=compiled,
               expected=expected, diagnostics=diagnostics, requestSha256=sha(directory / 'request.json'), matchesExpectation=matches)
    if compiled:
        row.update(rivSha256=sha(directory / 'scene.riv'), mapSha256=sha(directory / 'scene.map.json'))
    else:
        assert not (directory / 'scene.riv').exists() and not (directory / 'scene.map.json').exists()
    if args.previous:
        prior = args.previous.resolve() / case['name']
        row['previous'] = dict(directory=str(prior), sameRequest=json.loads((prior / 'request.json').read_text()) == request)
        if compiled and (prior / 'scene.riv').exists():
            row['previous'].update(sameRiv=sha(prior / 'scene.riv') == row['rivSha256'], sameMap=sha(prior / 'scene.map.json') == row['mapSha256'])
    results.append(row)
    write(directory / 'compile-result.json', row)
    write(root / 'compile-receipt.json', results)
print(json.dumps(dict(cases=len(results), compiled=sum(r['compiled'] for r in results),
                     matches=sum(r['matchesExpectation'] for r in results),
                     changed=[r['name'] for r in results if r.get('previous', {}).get('sameRiv') is False])))
assert all(r['matchesExpectation'] for r in results)
