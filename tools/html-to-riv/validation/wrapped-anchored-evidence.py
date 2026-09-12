"""Verify the private anchored positioning checkpoint without compiling or rendering."""
import hashlib
import json
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1]
BUILD = MODULE / 'output/wrapped-anchored-build-r1'
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
assert summary['sourceUnchanged'] and summary['rustTests'] == 397 and summary['nodeTests'] == 56
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

# The public CLI is byte-identical to the prior checkpoint. Bind its
# actual artifacts and receipts; do not recompile unchanged public output.
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

for filename in ['wrapped-gap-normalizer-audit.md', 'wrapped-sizing-certificate-next.md', 'wrapped-composition-layout-isolation.md', 'wrapped-scalar-binding-review.md', 'wrapped-position-bound-review.md', 'wrapped-mask-coverage-audit.md', 'wrapped-scalar-scheduling-audit.md', 'wrapped-anchored-scalar-review.md', 'wrapped-anchored-position-review.md', 'wrapped-visible-measurement-options.md']:
    bind(MODULE / 'validation' / filename)
# Recheck native geometry from actual ordinary imports, not receipt booleans.
NATIVE = MODULE / 'output/wrapped-anchored-native-r2'
NEGATIVE = MODULE / 'output/wrapped-anchored-native-negative-r2'
native = read(NATIVE / 'receipt.json')
negative = read(NEGATIVE / 'receipt.json')
assert native['epsilon'] == 1/64 and not native['epsilonIsAdmissionCertificate']
assert native['frames'] == 384 and len(native['cases']) == 48 and native['failures'] == []
for row in native['bindings']:
    bind(row['path'], row['sha256']); bind(row['snapshot'], row['sha256'])
for receipt in [native, negative]:
    for row in receipt['artifacts']: bind(row['path'], row['sha256'])
assert negative['sourceCommit'] == '49169f2bfb'
bind(NEGATIVE / 'wrapping.rs', '18460f64c664b7c73a86fef81cd30e84deb76da916e9ef2c3674128f1e899d70')
for key in ['positiveReceipt', 'positiveObservations']:
    bind(negative[key]['path'], negative[key]['sha256'])
bind(NATIVE / 'node-probe', '2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53')
positive_positions = failed_frames = failed_positions = 0
for case in native['cases']:
    dest = NATIVE / 'cases' / case['name']
    old = NEGATIVE / case['name']
    assert bind(dest / 'base.riv') == bind(old / 'base.riv')
    trace = read(dest / 'trace.json')
    prior_trace = read(old / 'trace.json')
    assert trace['visible'] == prior_trace['visible'] and trace['slots'] == prior_trace['slots']
    frames = read(dest / 'scene/frames.json')['frames']
    assert len(frames) == 8
    main = 0 if case['arguments'][0] == 'row' else 1
    cross = 1-main
    ext, mainext = ['width','height'][cross], ['width','height'][main]
    reverse_cross = case['arguments'][2] == '2'
    f = int(case['arguments'][3])/2
    if reverse_cross: f = 1-f
    actual_frames = []; old_frames = []
    for frame in frames:
        actual = read(dest / 'scene' / frame['geometry'])
        baseline = read(dest / 'base' / frame['geometry'])
        old_geometry = read(old / 'scene' / frame['geometry'])
        A = {o['objectId']:o for o in actual}; B = {o['objectId']:o for o in baseline}
        O = {o['objectId']:o for o in old_geometry}
        groups = [[]]; used = 0
        for i, slot in enumerate(trace['slots']):
            size = B[slot][mainext]
            if groups[-1] and used+size > case['sizes'][frame['step']][main]:
                groups.append([]); used=0
            groups[-1].append(i); used+=size
        assert len(groups) == [1,2,3,1][frame['step']]
        errors = 0
        for group in groups:
            maximum = max(B[trace['slots'][i]][ext] for i in group)
            for i in group:
                slot,visible = trace['slots'][i], trace['visible'][i]
                a = [0,.5,1][i]
                if reverse_cross: a=1-a
                line_origin = B[slot]['worldMatrix'][4+cross] - (maximum-B[slot][ext])*f
                expected = line_origin + (maximum-B[visible][ext])*a
                assert A[visible]['worldMatrix'][4+cross] == expected
                assert A[slot] == B[slot]
                assert all(A[visible][key] == B[visible][key] for key in ['width','height'])
                assert A[visible]['worldMatrix'][4+main] == B[visible]['worldMatrix'][4+main]
                positive_positions += 1
                errors += abs(O[visible]['worldMatrix'][4+cross]-expected) > .1
        failed_frames += bool(errors); failed_positions += errors
        actual_frames.append(actual); old_frames.append(old_geometry)
    for sequence in [actual_frames,old_frames]:
        assert sequence[:4] == sequence[4:] and sequence[0] == sequence[3]
assert (positive_positions,failed_frames,failed_positions) == (1152,384,768)
assert (negative['failedFrames'],negative['failedItems']) == (failed_frames,failed_positions)
reproduced = read(MODULE / 'output/wrapped-anchored-native-reproduce-r1/receipt.json')
assert reproduced['cases'] == 48 and reproduced['exactArtifacts'] == len(reproduced['results']) == 144
for key in ['candidate','sourceReceipt','driver']:
    bind(reproduced[key]['path'], reproduced[key]['sha256'])
for row in reproduced['results']:
    bind(row['path'], row['sha256']); bind(row['original'], row['sha256'])
for filename in ['wrapped-anchored-native-negative.py','wrapped-anchored-native-review.md']:
    bind(MODULE / 'validation' / filename)
bind(__file__)
result = dict(scope='Private source-derived anchored positioning candidate; no public wrapping or new pixel qualification',
    rustTests=397, nodeTests=56, imageOutputs=282, historicalOutputs=794,
    compilerSha256=cli, wasmSha256=summary['wasmSha256'],
    publicWrappingAdmitted=False, nativeFrames=384, nativePositionsExact=1152,
    negativeFramesFailed=384, negativePositionsFailed=768,
    artifactBindings=len(bindings), bindings=bindings)
output = MODULE / 'output/wrapped-anchored-verification-r1.json'
output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: v for k, v in result.items() if k != 'bindings'}))
