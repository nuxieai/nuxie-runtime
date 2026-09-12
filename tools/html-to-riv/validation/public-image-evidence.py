"""Bind final public image code, API results and measured ordinary-file evidence.

This verifies existing artifacts. It does not rerun compilers, browsers or renderers.
"""
from pathlib import Path
import hashlib
import json

module = Path(__file__).resolve().parents[1]


def read(path):
    return json.loads(Path(path).read_text())


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bind(path):
    path = Path(path).resolve()
    return dict(path=str(path), sha256=sha(path))


build = module / 'output/public-image-build-r2'
frozen = build / 'frozen'
summary = read(build / 'summary.json')
assert summary['sourceUnchanged'] and (summary['rustTests'], summary['nodeTests']) == (301, 56)
assert all(c['exitCode'] == 0 for c in read(build / 'checks.json'))
source_bindings = read(frozen / 'source-bindings.json')['files']
for b in source_bindings:
    assert sha(b['path']) == b['sha256']
    assert sha(b.get('snapshot', b['path'])) == b['sha256']

final_cases = module / 'output/public-image-layout-r3'
cases = read(final_cases / 'compile-receipt.json')
assert len(cases) == 40 and sum(c['compiled'] for c in cases) == 27
assert all(c['matchesExpectation'] and c['previous']['sameRequest'] for c in cases)
for case in cases:
    directory = final_cases / case['name']
    assert sha(directory / 'request.json') == case['requestSha256']
    if case['compiled']:
        assert case['previous']['sameRiv'] and case['previous']['sameMap']
        assert sha(directory / 'scene.riv') == case['rivSha256']
        assert sha(directory / 'scene.map.json') == case['mapSha256']
parity_path = module / 'output/public-image-parity-r1/receipt.json'
parity = read(parity_path)
assert (parity['cases'], parity['observations'], parity['passed']) == (40, 80, 80)
for b in parity['bindings']:
    assert sha(b['path']) == b['sha256']

native_path = module / 'output/public-image-layout-r2/combined-receipt.json'
native = read(native_path)
assert native['summary'] == dict(frames=216, freshFrames=32, transferredFrames=184,
                                  geometryPass=216, pixelPass=198, presencePass=216)
for b in native['bindings'] + native['sources']:
    assert sha(b['path']) == b['sha256']
for row in native['rows']:
    case = next(c for c in cases if c['name'] == row['name'])
    assert row['rivSha256'] == case['rivSha256'] and row['mapSha256'] == case['mapSha256']
    assert read(Path(row['prefix']).parent.parent / 'request.json') == read(final_cases / row['name'] / 'request.json')
    for key, path in [('chromeSha256', row['prefix'] + '.chrome.png'),
                      ('nativeSha256', row['prefix'] + '.native.png'),
                      ('geometrySha256', Path(row['probeDirectory']) / row['geometry']),
                      ('streamSha256', Path(row['probeDirectory']) / row['stream'])]:
        assert sha(path) == row[key]
    for key in ['receipt', 'result']:
        assert sha(row['evidence'][key]) == row['evidence'][key + 'Sha256']

visual = module / 'output/public-image-layout-r2/visual'
assert read(visual / 'review-receipt.json')['visualReviewCompleted']
for b in read(visual / 'artifact-bindings.json'):
    assert sha(b['path']) == b['sha256']
clear_path = module / 'output/public-image-clear-r1/receipt.json'
clear = read(clear_path)
assert clear['total'] == clear['passed'] == len(clear['rows']) == 32
for row in clear['rows']:
    assert row['samePixels'] and sha(row['output']) == row['outputSha256']
    assert sha(row['reference']) == row['referenceSha256']

assets_path = module / 'output/public-image-assets-r1/receipt.json'
assets = read(assets_path)
verification_path = assets_path.parent / 'verification.json'
verification = read(verification_path)
assert assets['passed'] and assets['completed'] == assets['planned'] == 86 and not assets['failures']
assert verification['passed'] and verification['receiptSha256'] == sha(assets_path)
assert verification['verifiedCaseArtifacts'] == 2373 and verification['frozenBindings'] == len(source_bindings)

regression_path = module / 'output/public-transport-malformed-image-regression-r1/receipt.json'
regression = read(regression_path)
assert regression['allExact'] and regression['total'] == regression['passed'] == 794
for row in regression['results']:
    assert row['exact'] and row['exitCode'] == 0
    directory = Path(row['command'][-1]).parent
    for name in ['scene.riv', 'scene.map.json']:
        assert sha(directory / name) == row['actualHashes'][name] == row['reference']['hashes'][name]
        assert sha(Path(row['reference']['reference']) / name) == row['reference']['hashes'][name]

files = [build / 'summary.json', build / 'checks.json', frozen / 'source-bindings.json',
         final_cases / 'build-receipt.json', final_cases / 'compile-receipt.json', parity_path,
         native_path, visual / 'review-receipt.json', visual / 'artifact-bindings.json', clear_path,
         assets_path, verification_path, regression_path,
         module / 'output/public-image-jpeg-review-r2/receipt.json',
         module / 'output/image-variable-recovery-r1/receipt.json',
         module / 'output/public-image-layout-r1/recovered-reference-driver/receipt.json',
         Path(__file__).resolve()]
files += [module / 'validation' / name for name in [
    'public-image-checkpoint-review.md', 'public-image-assets-review.md', 'public-image-jpeg-review.md',
    'public-image-visual-review.md', 'image-variable-recovery-review.md', 'public-image-compile.py',
    'public-image-native.mjs', 'public-image-transfer.py', 'public-image-parity.mjs', 'public-image-clear.mjs']]
receipt = dict(status='public-images-implemented-partial-visual-qualification',
    scope='Standalone compiler only; immutable baseline native consumer; all 99 backlog items remain in scope',
    baseline='6c7ac16617835b5f581784ff08a9e779bb52faf3', compiler=bind(frozen / 'html-to-riv'),
    wasm=bind(frozen / 'compiler.wasm'), sourceBindingsVerified=len(source_bindings),
    rustTests=301, nodeTests=56, priorExactOutputs=794, publicImageCases=40,
    rawAbiJsParityObservations=80, assetCases=86, assetInterfaceObservations=252,
    native=native['summary'], clearChecks=32, visualReviewCompleted=True,
    limitations='18 fractional pixel failures; broader codec/layout/asset representations and numeric domains remain open. No blanket browser fidelity or total memory/CPU guarantee.',
    artifacts=[bind(p) for p in files])
destination = module / 'validation/public-image-receipt.json'
destination.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(dict(receipt=str(destination), status=receipt['status'], sourceBindings=len(source_bindings),
                      native=receipt['native'], priorExactOutputs=794)))
