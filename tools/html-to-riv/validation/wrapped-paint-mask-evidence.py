"""Verify the private paint/mask checkpoint without compiling or rendering."""
import hashlib
import json
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1]
BUILD = MODULE / 'output/wrapped-paint-mask-build-r1'
IMAGES = MODULE / 'output/wrapped-paint-mask-existing-r1/manifest.json'
HISTORY = MODULE / 'output/public-transport-malformed-wrapped-paint-mask-regression-r1/receipt.json'
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
assert summary['sourceUnchanged'] and summary['rustTests'] == 403 and summary['nodeTests'] == 56
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

for filename in ['wrapped-gap-normalizer-audit.md', 'wrapped-sizing-certificate-next.md', 'wrapped-composition-layout-isolation.md', 'wrapped-paint-binding-review.md', 'wrapped-mask-domain-review.md', 'wrapped-paint-arithmetic-scheduling-audit.md', 'wrapped-clip-stream-review.md', 'wrapped-clip-observation-plan.md']:
    bind(MODULE / 'validation' / filename)
# Historical command observations are bound back to their original frozen
# checkpoint, not relabelled as captures of this source-derived candidate.
import importlib.util
clip_script = MODULE / 'validation/wrapped-clip-stream-check.py'
clip = read(MODULE / 'output/wrapped-clip-stream-r1/receipt.json')
bind(clip_script, clip['scriptSha256'])
historical_path = MODULE / 'output/wrapped-sizing-snapped-checkpoint-r1/verification.json'
bind(historical_path, '6d0e82d25b2e3f3b83922671f24df7390c06a9384612d904fdb993c6008fd36c')
historical = {r['path']:r['sha256'] for r in read(historical_path)['bindings']}
spec = importlib.util.spec_from_file_location('clip_observation',clip_script)
observer = importlib.util.module_from_spec(spec); spec.loader.exec_module(observer)
assert clip['frameCount'] == len(clip['frames']) == 384
assert clip['clipCount'] == 4992 and clip['emptyDrawCount'] == 2304
assert len(clip['negativeControls']) == 4
control = None
for row in clip['frames']:
    stream = MODULE / row['stream']; manifest = stream.parent / 'frames.json'
    assert historical[str(stream)] == row['streamSha256']
    assert historical[str(manifest)] == row['framesSha256']
    bind(stream,row['streamSha256']); bind(manifest,row['framesSha256'])
    frame = json.loads(manifest.read_text())['frames'][row['frame']]
    assert frame['stream'] == stream.name and frame['instance'] == row['instance'] and frame['step'] == row['step']
    lines = observer.extract(stream,row['frame'])
    actual = observer.observe(lines,frame['width'],frame['height'])
    assert actual['clips'] == row['clips'] and actual['draws'] == row['draws']
    if control is None: control = observer.controls(lines,frame['width'],frame['height'])
assert control == clip['negativeControls']
# This is the existing frozen immutable observer; no compiler-linked renderer.
probe = MODULE / 'output/wrapped-snapped-gate-r1/node-probe'
bind(probe,'2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53')
bind(__file__)
result = dict(scope='Private source-derived paint/mask candidate; no public wrapping or new pixel qualification',
    rustTests=403, nodeTests=56, imageOutputs=282, historicalOutputs=794,
    compilerSha256=cli, wasmSha256=summary['wasmSha256'],
    publicWrappingAdmitted=False, historicalClipFrames=384, historicalClipCommands=4992,
    historicalEmptyClipDraws=2304, artifactBindings=len(bindings), bindings=bindings)
output = MODULE / 'output/wrapped-paint-mask-verification-r1.json'
output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: v for k, v in result.items() if k != 'bindings'}))
