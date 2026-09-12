"""Verify the private scalar/position checkpoint without compiling or rendering."""
import hashlib
import json
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1]
BUILD = MODULE / 'output/wrapped-scalar-position-build-r2'
IMAGES = MODULE / 'output/wrapped-scalar-position-existing-r1/manifest.json'
HISTORY = MODULE / 'output/public-transport-malformed-wrapped-scalar-position-regression-r1/receipt.json'
bindings = {}


def bind(path, expected=None):
    path = Path(path)
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if expected is not None:
        assert digest == expected, f'Changed artifact: {path}'
    bindings[str(path)] = digest
    return digest


def read(path):
    bind(path)
    return json.loads(Path(path).read_text())


summary = read(BUILD / 'summary.json')
assert summary['sourceUnchanged'] and summary['rustTests'] == 396 and summary['nodeTests'] == 56
for row in read(BUILD / 'frozen/source-bindings.json')['files']:
    bind(row['path'], row['sha256'])
    if 'snapshot' in row:
        bind(row['snapshot'], row['sha256'])
checks = read(BUILD / 'checks.json')
assert [row['name'] for row in checks] == [
    'full-rust', 'native-build', 'wasm-build', 'typescript', 'full-node', 'runtime-guard']
for row in checks:
    assert row['exitCode'] == 0
    bind(row['log'], row['logSha256'])
cli = summary['compilerSha256']
bind(BUILD / 'frozen/html-to-riv', cli)
bind(BUILD / 'frozen/compiler.wasm', summary['wasmSha256'])

images = read(IMAGES)
assert images['total'] == images['passed'] == len(images['results']) == 282
bind(images['compiler'], cli)
bind(images['priorManifest'], images['priorManifestSha256'])
bind(MODULE / 'validation/check-output-regression.py', images['scriptSha256'])
for row in images['results']:
    assert row['exitCode'] == 0 and row['exact']
    out = Path(row['result'])
    for prior_key, hash_key, filename in [('request', 'requestSha256', 'request.json'),
        ('priorRiv', 'rivSha256', 'scene.riv'), ('priorMap', 'mapSha256', 'scene.map.json')]:
        bind(row[prior_key], row[hash_key])
        bind(out / filename, row[hash_key])

history = read(HISTORY)
assert history['total'] == history['passed'] == len(history['results']) == 794
assert history['allExact']
for row in history['results']:
    assert row['exitCode'] == 0 and row['exact'] and row['termination'] is None
    bind(row['command'][0], cli)
    reference = row['reference']
    for filename, digest in reference['hashes'].items():
        bind(Path(reference['reference']) / filename, digest)
    bind(row['command'][1], reference['hashes']['request.json'])
    out = Path(row['command'][2]).parent
    for filename in ['scene.riv', 'scene.map.json']:
        digest = reference['hashes'][filename]
        assert row['actualHashes'][filename] == digest
        bind(out / filename, digest)
    for filename in ['stdout', 'stderr']:
        bind(out / f'{filename}.log', row[f'{filename}Sha256'])

for filename in ['wrapped-gap-normalizer-audit.md', 'wrapped-sizing-certificate-next.md', 'wrapped-composition-layout-isolation.md', 'wrapped-scalar-binding-review.md', 'wrapped-position-bound-review.md', 'wrapped-mask-coverage-audit.md', 'wrapped-scalar-scheduling-audit.md']:
    bind(MODULE / 'validation' / filename)
bind(__file__)
result = dict(scope='Private source-derived scalar/position candidate; no public wrapping or new pixel qualification',
    rustTests=396, nodeTests=56, imageOutputs=282, historicalOutputs=794,
    compilerSha256=cli, wasmSha256=summary['wasmSha256'],
    publicWrappingAdmitted=False, artifactBindings=len(bindings), bindings=bindings)
output = MODULE / 'output/wrapped-scalar-position-verification-r1.json'
output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: v for k, v in result.items() if k != 'bindings'}))
