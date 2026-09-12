"""Bind the partial content-box checkpoint; verify, never rerender or qualify.

Run after public-content-box-visual.py and the recorded manual inspection.
All failed native rows and superseded controls remain in the bound receipts.
"""
from pathlib import Path
from collections import Counter
import hashlib
import json
import re
import struct

module = Path(__file__).resolve().parent.parent
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
read = lambda p: json.loads(Path(p).read_text())
bind = lambda p: dict(path=str(Path(p)), sha256=sha(p))
r1 = module / 'output/public-content-box-r1'
r2 = module / 'output/public-content-box-r2'

freeze = read(r2 / 'source-bindings.json')
for item in freeze['files'] + [freeze['compiler'], freeze['wasm']]:
    assert sha(item['path']) == item['sha256'], item
assert len(freeze['files']) == 30
final_guard = read(r2 / 'final-runtime-guard.json')
assert final_guard['exitCode'] == 0 and final_guard['result']['status'] == 'pass'
assert final_guard['result']['baseline'] == '6c7ac16617835b5f581784ff08a9e779bb52faf3'
assert all(not changes for changes in final_guard['result']['changesOutsideCompiler'].values())

def counts(rows):
    return dict(frames=len(rows), geometryPass=sum(not r['geometryFailures'] for r in rows),
                pixelPass=sum(not r['pixelFailures'] for r in rows),
                clearPass=sum(c['samePixels'] for r in rows for c in r['clearChecks']))

def native(file):
    receipt = read(file)
    assert receipt['status'] == 'failed-public-baseline'
    assert (receipt['browser'], receipt['backend'], receipt['effectiveMode']) == (
        '153.0.8010.12', 'rust-metal', 'RasterOrdering')
    for key, value in receipt['tools'].items():
        assert sha(value) == receipt['toolHashes'][key]
    for item in receipt['sourceBindings']:
        assert sha(file.parent / item['snapshot']) == item['sha256']
    for artifact in receipt['artifacts']:
        scene = file.parent / artifact['name']
        for filename, key in [('request.json', 'requestSha256'), ('scene.riv', 'rivSha256'),
                              ('scene.map.json', 'mapSha256'), ('probe/frames.json', 'probeManifestSha256')]:
            assert sha(scene / filename) == artifact[key]
        assert (scene / 'scene.riv').read_bytes() == (scene / 'probe/scene.riv').read_bytes()
        assert not (scene / 'scene.requirements.json').exists()
    for row in receipt['rows']:
        prefix = Path(row['prefix'])
        for filename, key in [(str(prefix)+'.chrome.png', 'chromeSha256'),
                              (str(prefix)+'.native.png', 'nativeSha256'),
                              (prefix.parent/'probe'/f"frame-{row['frame']}.stream", 'streamSha256'),
                              (prefix.parent/'probe'/f"frame-{row['frame']}.geometry.json", 'geometrySha256')]:
            assert sha(filename) == row[key]
        for clear in row['clearChecks']:
            assert sha(clear['path']) == clear['sha256'] and clear['samePixels']
    assert all(r['nativeIdentical'] and r['chromeIdentical'] for r in receipt['repeated'])
    return receipt

original_file = r1 / 'render/receipt.json'
boundary_file = r2 / 'boundary-render/receipt.json'
original, boundary = native(original_file), native(boundary_file)
for key in ['probe', 'renderer']:
    assert original['toolHashes'][key] == boundary['toolHashes'][key]
for key in ['driverSha256', 'resetSha256', 'pixelGateSha256']:
    assert original[key] == boundary[key]
assert boundary['toolHashes']['compiler'] == freeze['compiler']['sha256']
cases = read(module / 'validation/public-content-box-cases.json')
boundaries = read(module / 'validation/public-content-box-boundary-cases.json')
assert len(cases) == 34 and len(boundaries) == 5
assert boundaries == [c for c in cases if c['classification'] == 'numeric-boundary']
assert len(read(module / 'validation/public-content-box-rejections.json')) == 14
aggregate_file = r2 / 'native-aggregate.json'
aggregate = read(aggregate_file)
assert aggregate['cases'] == bind(module / 'validation/public-content-box-cases.json')
assert aggregate['sourceReceipts'] == [bind(original_file), bind(boundary_file)]
for case in cases:
    matches = [r for r in aggregate['rows'] if r['name'] == case['name']]
    assert len(matches) == 8
    for row in matches:
        assert read(Path(row['prefix']).parent/'request.json') == dict(
            html=case['html'], css=case['css'], width=390, height=160)
assert counts(aggregate['rows']) == dict(frames=272, geometryPass=264, pixelPass=266, clearPass=544)
assert counts(original['rows']+boundary['rows']) == dict(frames=312, geometryPass=304, pixelPass=306, clearPass=624)

reproductions = []
for folder, source_file, expected in [('reproduce-r1', original_file, 34),
                                      ('reproduce-boundaries', boundary_file, 5)]:
    file = r2 / folder / 'receipt.json'
    result = read(file)
    assert result['compilerSha256'] == freeze['compiler']['sha256']
    assert result['sourceReceiptSha256'] == sha(source_file)
    assert result['passed'] == result['total'] == expected
    for row in result['results']:
        assert sha(row['command'][1]) == row['requestSha256']
        assert sha(row['command'][2]) == row['rivSha256']
        assert sha(Path(row['command'][2]).with_suffix('.map.json')) == row['mapSha256']
    reproductions.append(dict(receipt=bind(file), exactFilesAndMaps=expected))

prior_file = module / 'output/content-box-prior-regression-r2/manifest.json'
prior = read(prior_file)
assert prior['compilerSha256'] == freeze['compiler']['sha256']
assert prior['passed'] == prior['total'] == 629
assert all(r['exact'] and r['exitCode'] == 0 for r in prior['results'])

visual_file = r2 / 'visual-evidence.json'
visual = read(visual_file)
assert visual['nativeAggregate'] == bind(aggregate_file)
assert len(visual['directPairs']) == 60 and len(visual['placements']) == 120
assert len(visual['sheets']) == 10 and len(visual['transfers']) == 212
assert Counter(t['sourceKind'] for t in visual['transfers']) == {'historical': 72, 'current-direct': 140}
assert len(visual['historicalParentOnlyTransfers']) == 40
for entry in visual['sheets'] + [p['source'] for p in visual['placements']]:
    assert sha(entry['path']) == entry['sha256']
for key in ['historicalReview', 'historicalReceipt', 'historicalFrames']:
    assert sha(visual[key]['path']) == visual[key]['sha256']

depth_file = r1 / 'depth-comparison/receipt.json'
depth = read(depth_file)
assert next(b for b in depth['bindings'] if b['label'] == 'repaired-content-box')['sha256'] == freeze['wasm']['sha256']
assert len(depth['rows']) == 18
for row in depth['rows']:
    assert sha(row['requestFile']) == row['requestSha256']

rust_log = r1 / 'tests-after-stack-refactor.log'
rust_count = sum(map(int, re.findall(r'test result: ok\. (\d+) passed', rust_log.read_text())))
assert rust_count == 233
assert 'ℹ pass 39' in (r1 / 'transport-final.log').read_text()
assert '3 passed; 0 failed' in (r2 / 'strengthened-tests-r2.log').read_text()
assert 'ℹ pass 1' in (r2 / 'strengthened-transport-r3.log').read_text()
assert 'ℹ fail 1' in (r2 / 'strengthened-transport-r2.log').read_text()
tests = [dict(kind='full Rust before strengthened fixture/depth129 edits', passed=233, log=bind(rust_log)),
         dict(kind='full Node before strengthened fixture/depth129 edits', passed=39, log=bind(r1/'transport-final.log')),
         dict(kind='current-fixture focused Rust', passed=3, log=bind(r2/'strengthened-tests-r2.log')),
         dict(kind='current-fixture focused Node on frozen r2 CLI/WASM', passed=1,
              sceneViewportPairs=102, diagnosticPairs=14, successfulDepths=[64,128], diagnosticDepths=[129,130],
              log=bind(r2/'strengthened-transport-r3.log')),
         dict(kind='strict TypeScript', exitCode=0, log=bind(r1/'types.log'))]

numeric = []
as_f32 = lambda number: struct.unpack('f', struct.pack('f', number))[0]
for case in boundaries:
    row = next(r for r in boundary['rows'] if r['name']==case['name'] and r['frame']==0)
    source_map = read(Path(row['prefix']).parent/'scene.map.json')
    observed = {}
    for item in source_map:
        g = next(g for g in row['geometry'] if g['objectId']==item['object_id'])
        observed[item['id']] = dict(rawProbeWidth=g['width'], binary32Width=as_f32(g['width']),
                                    chromeRect=row['boxes'][item['id']], computedStyle=row['computedStyles'][item['id']])
    numeric.append(dict(name=case['name'], html=case['html'], css=case['css'],
                        frame=0, observed=observed, geometryFailures=row['geometryFailures'], pixelFailures=row['pixelFailures']))
failing = next(n for n in numeric if n['name']=='content-large-rounded-outer')['observed']
assert failing['p']['binary32Width'] == failing['p']['chromeRect']['width'] == 3000000
assert failing['c']['binary32Width'] == 1000000 and failing['c']['chromeRect']['width'] == 999999.75
assert failing['c']['computedStyle']['width'] == '1e+06px'

source_files = ['validation/public-content-box-review.md', 'validation/public-content-box-receipt.py',
                'validation/public-content-box-visual.py', 'validation/public-content-box-fixtures.py',
                'validation/public-content-box-cases.json', 'validation/public-content-box-boundary-cases.json',
                'validation/public-content-box-rejections.json', 'validation/public-content-box-depth.mjs',
                'validation/content-box-review.md', 'validation/content-box-numeric-audit.md',
                'tests/content-box.rs', 'tests/padding.rs', 'tests/transport-parity.mjs',
                'validation/check-public-baseline.mjs', 'validation/pixels.mjs', 'js/index.mjs']
failures = [dict(name=r['name'], frame=r['frame'], instance=r['instance'], step=r['step'],
                 width=r['width'], height=r['height'], geometry=r['geometryFailures'],
                 pixels=r['pixelFailures'], metrics=r['metrics'])
            for r in aggregate['rows'] if r['geometryFailures'] or r['pixelFailures']]
assert len(failures) == 14
assert Counter(f['name'] for f in failures) == {'content-large-rounded-outer': 8, 'content-point-fractional-paint-control': 6}
receipt = dict(
    status='partial-public-implementation-with-known-qualification-failures', backlog='L12',
    baseline='6c7ac16617835b5f581784ff08a9e779bb52faf3',
    scope='Bounded compiler lowering only. Current acceptance does not establish full qualification; large-coordinate child geometry and fractional paint failures remain. No runtime impossibility claim.',
    sourceFreeze=bind(r2/'source-bindings.json'), verifiedCurrentInputs=30,
    compiler=freeze['compiler'], wasm=freeze['wasm'], sourceFiles=[bind(module/p) for p in source_files],
    historicalCompilerFreeze=bind(r1/'source-bindings.json'),
    originalNative=dict(receipt=bind(original_file), fixtureSnapshot=bind(r1/'render/source-cases.json'),
                        cases=34, **counts(original['rows'])),
    strengthenedNative=dict(receipt=bind(boundary_file), fixtureSnapshot=bind(r2/'boundary-render/source-cases.json'),
                            cases=5, **counts(boundary['rows'])),
    currentCorpus=dict(cases=34, reusedExactSourceOriginalCases=29, strengthenedChildCases=5,
                       receipt=bind(aggregate_file), **counts(aggregate['rows'])),
    supersededParentOnly=aggregate['historicalParentOnly'], allRetainedEvidence=aggregate['allRetainedEvidence'],
    tools={k:dict(path=boundary['tools'][k],sha256=boundary['toolHashes'][k]) for k in ['probe','renderer']},
    browser=boundary['browser'], backend=boundary['backend'], effectiveMode=boundary['effectiveMode'],
    geometryToleranceCssPx=.1, pixelGateSha256=boundary['pixelGateSha256'], resetSha256=boundary['resetSha256'],
    originalAndCloneResizeSequence=[[240,160],[390,200],[768,120],[240,160]], compileViewport=[390,160],
    exactContentBoxReproductions=reproductions,
    priorOutputRegression=dict(manifest=bind(prior_file), exactFilesAndMaps=629, scope='Exact output regression; no new render qualification'),
    visual=dict(evidence=bind(visual_file), review=bind(module/'validation/public-content-box-review.md'),
                directPairsActuallyViewed=60, historicalCurrentTransfers=72, currentDirectTransfers=140,
                currentPairsCovered=272, supersededParentOnlyTransfers=40, retainedPairsCovered=312,
                scope='Complete pair RGBA identity transfers inspection only; offscreen geometry remains independently gated'),
    tests=tests, historicalDepthComparison=bind(depth_file),
    depthScope='Historical comparison64/128/130, old and initial traps retained. Final focused tests separately cover first-invalid129. Default stack and source limit unchanged.',
    numericBoundaryFrameZero=numeric, failures=failures,
    historicalRuntimeGuard=dict(receipt=bind(r1/'runtime-guard.json'), scope='Earlier source immutability receipt'),
    finalRuntimeGuard=dict(receipt=bind(r2/'final-runtime-guard.json'), scope='Final pre-commit source/index/nonignored additions check; effective native build identity is separate'),
    logs=[bind(p) for p in sorted(r1.glob('*.log'))]+[bind(p) for p in sorted(r2.glob('*.log'))],
    nextRequiredWork=['Isolate large-coordinate native/Chrome rectangle discrepancy and add sound admission guard or ordinary-file composition.',
                      'Keep fractional paint control failing until an independently verified ordinary solution is available.',
                      'Investigate responsive mixed point/percentage compositions without viewport baking.'],
)
target = module/'validation/public-content-box-receipt.json'
target.write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(dict(status=receipt['status'], current=counts(aggregate['rows']),
                     retained=counts(original['rows']+boundary['rows']), sourceInputs=30, priorExact=629,
                     originalExact=34, strengthenedExact=5, directVisualPairs=60, currentVisualPairs=272,
                     retainedVisualPairs=312, rustFull=233, nodeFull=39, rustFocused=3, nodeFocused=1)))
