#!/usr/bin/env python3
"""Verify retained public JPEG evidence; never compile, decode or render a scene.

The only subprocess is the read-only immutable-source guard. Historical evidence
is verified against its bound snapshots; the final public build is also checked
against live source. PNG equality below verifies review transfers, not new paint
qualification. No comparison gates are changed or silently recomputed.
"""
from collections import Counter
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys
from urllib.parse import urljoin
from PIL import Image

MODULE = Path(__file__).resolve().parents[1]
ROOT = MODULE.parents[1]
OUT = MODULE / 'output/public-jpeg-checkpoint-r1'
BASELINE = '6c7ac16617835b5f581784ff08a9e779bb52faf3'
BUILD = MODULE / 'output/public-jpeg-build-r2'
FROZEN = BUILD / 'frozen'
verified = {}
historical = []
request_serializations = {}


def digest(path):
    path = Path(path).resolve()
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    result = h.hexdigest()
    if str(path) in verified:
        assert verified[str(path)] == result, ('Changed during verification', path)
    verified[str(path)] = result
    return result


def check(path, expected):
    actual = digest(path)
    assert actual == expected, (str(path), expected, actual)


def bind(path):
    return dict(path=str(Path(path).resolve()), sha256=digest(path))


def read(path):
    digest(path)
    return json.loads(Path(path).read_text())


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def bindings(rows, current=False, overrides=None):
    for row in rows:
        original = Path(row['path'])
        snapshot = row.get('snapshot') or (overrides or {}).get(str(original))
        if not snapshot and str(original) == str(MODULE / 'validation/public-image-native.mjs') and row['sha256'] == '1b980bc6b4d9142c56ef9fea299bb84f0ba44217b994a6193e3390f8ceccd13c':
            snapshot = MODULE / 'output/public-image-layout-r1/recovered-reference-driver/public-image-native.mjs'
            recovery = read(MODULE / 'output/public-image-layout-r1/recovered-reference-driver/receipt.json')
            assert recovery['matchesCaptureBinding'] and recovery['sha256'] == row['sha256']
        if not snapshot and str(original) == str(MODULE / 'validation/public-image-visual.py') and row['sha256'] == 'f5567c190aa4c4b63fbdcd258b9c7fc3dc15f43d5980b987c745db61ac7608b1':
            snapshot = MODULE / 'output/public-image-layout-r2/visual/public-image-visual.py'
        check(snapshot or original, row['sha256'])
        if current:
            check(original, row['sha256'])
        elif snapshot and original.exists() and hashlib.sha256(original.read_bytes()).hexdigest() != row['sha256']:
            historical.append(dict(path=str(original), snapshot=str(snapshot), sha256=row['sha256']))


def compile_suite(label, total, admitted):
    directory = MODULE / 'output' / label
    build = read(directory / 'build-receipt.json')
    assert Path(build['build']) == BUILD
    check(BUILD / 'summary.json', build['summarySha256'])
    check(FROZEN / 'source-bindings.json', build['bindingsSha256'])
    check(FROZEN / 'html-to-riv', build['compilerSha256'])
    bindings(read(directory / 'authoring-bindings.json'), current=True)
    fixtures = read(directory / 'cases.json')
    rows = read(directory / 'compile-receipt.json')
    assert len(fixtures) == len(rows) == total
    by_name = {r['name']: r for r in rows}
    assert len(by_name) == total and sum(r['compiled'] for r in rows) == admitted
    for fixture in fixtures:
        row = by_name[fixture['name']]
        case = directory / row['name']
        assert read(case / 'compile-result.json') == row
        assert row['matchesExpectation'] and row['expected'] == fixture['expected']
        assert Path(row['command'][0]).resolve() == (FROZEN / 'html-to-riv').resolve()
        request = read(case / 'request.json')
        check(case / 'request.json', row['requestSha256'])
        assert {k: v for k, v in request.items() if k != 'assets'} == fixture['input']
        assert set(request['assets']) == set(fixture['assetFiles'])
        for key, asset in fixture['assetFiles'].items():
            assert request['assets'][key] == dict(kind='image', bytes=list((MODULE / asset).read_bytes()))
            digest(MODULE / asset)
        previous = Path(row['previous']['directory'])
        assert row['previous']['sameRequest'] and read(previous / 'request.json') == request
        if row['compiled']:
            assert row['status'] == 0 and row['diagnostics'] is None
            for filename, key in [('scene.riv', 'rivSha256'), ('scene.map.json', 'mapSha256')]:
                check(case / filename, row[key])
                check(previous / filename, row[key])
            assert row['previous']['sameRiv'] and row['previous']['sameMap']
        else:
            assert row['status'] != 0 and not (case / 'scene.riv').exists() and not (case / 'scene.map.json').exists()
            assert read(case / 'compile.log') == row['diagnostics']
            assert row['diagnostics'][0]['code'] == fixture['expected']['code']
            assert fixture['expected'].get('messageIncludes', '') in row['diagnostics'][0]['message']
    return directory, by_name


def parity(label, compiled):
    p = MODULE / 'output' / label / 'receipt.json'
    data = read(p)
    bindings(data['bindings'])
    assert data['sourceBindingsVerified'] == len(source_bindings)
    assert data['cases'] == len(compiled)
    assert data['observations'] == data['passed'] == len(data['rows']) == 2 * len(compiled)
    assert Counter(r['transport'] for r in data['rows']) == {'raw-abi': len(compiled), 'public-js': len(compiled)}
    for row in data['rows']:
        case = compiled[row['name']]
        assert row['matchesCli'] and row['ok'] == case['compiled'] and row['requestSha256'] == case['requestSha256']
        if case['compiled']:
            assert (row['rivSha256'], row['mapSha256']) == (case['rivSha256'], case['mapSha256'])
        else:
            assert row['diagnostics'] == case['diagnostics']
    return bind(p)


def native(path, final_dir, compiled, expected):
    data = read(path)
    assert not data['errors'] and data['browser'] == '153.0.8010.12'
    bindings(data['bindings'])
    bindings(data.get('sources', []))
    if 'compilerBuild' in data:
        bindings([data['compilerBuild']])
    rows = data['rows']
    counts = dict(frames=len(rows), geometryPass=sum(not r['metricFailures'] for r in rows),
                  pixelPass=sum(not r['pixelFailures'] for r in rows),
                  presencePass=sum(all(i['passed'] for i in r['imagePresence'].values()) for r in rows))
    assert counts == expected, (path, counts, expected)
    assert Counter(r['name'] for r in rows) == {name: 8 for name, row in compiled.items() if row['compiled']}
    lookup = {}
    source_receipts = {}
    reset = (FROZEN / 'inputs/src/reset.css').read_text()
    for row in rows:
        key = (row['name'], row['frame'])
        assert key not in lookup
        lookup[key] = row
        case = compiled[row['name']]
        prefix, probe = Path(row['prefix']), Path(row['probeDirectory'])
        original_dir = prefix.parent.parent
        request = read(original_dir / 'request.json')
        assert request == read(final_dir / row['name'] / 'request.json')
        for field in ['rivSha256', 'mapSha256']:
            assert row[field] == case[field]
        if row['requestSha256'] != case['requestSha256']:
            request_serializations[(str(path), row['name'])] = dict(case=row['name'], prior=bind(original_dir / 'request.json'), current=bind(final_dir / row['name'] / 'request.json'), parsedRequestExactlyEqual=True)
        for field, p in [('requestSha256', original_dir / 'request.json'), ('rivSha256', original_dir / 'scene.riv'),
                         ('mapSha256', original_dir / 'scene.map.json'), ('htmlSha256', prefix.parent / 'reference.html'),
                         ('streamSha256', probe / row['stream']), ('geometrySha256', probe / row['geometry']),
                         ('chromeSha256', str(prefix) + '.chrome.png'), ('nativeSha256', str(prefix) + '.native.png')]:
            check(p, row[field])
        check(probe / 'scene.riv', row['rivSha256'])
        f = read(probe / 'frames.json')['frames'][row['frame']]
        for k in ['frame', 'instance', 'step', 'width', 'height', 'geometry', 'stream']:
            assert row[k] == f[k]
        assert (row['instance'], row['step']) == (row['frame'] // 4, row['frame'] % 4)
        assert [row['width'], row['height']] == [[240, 240], [390, 320], [768, 560], [240, 240]][row['step']]
        html = (prefix.parent / 'reference.html').read_text()
        base = re.search(r'<base href="([^"]+)">', html).group(1)
        assert html == f'<!doctype html><meta charset="utf-8"><base href="{base}"><style>{reset}\n{request["css"]}</style>{request["html"]}'
        asset_hashes = {hashlib.sha256(bytes(a['bytes'])).hexdigest() for a in request['assets'].values()}
        browser_assets = read(prefix.parent / 'browser-assets.json')
        assert {a['url']: a['sha256'] for a in browser_assets} == {
            urljoin(base, name): hashlib.sha256(bytes(a['bytes'])).hexdigest() for name, a in request['assets'].items()}
        requested = read(prefix.parent / 'browser-requests.json')
        assert requested and all(a['sha256'] in asset_hashes for a in requested)
        stream = (probe / row['stream']).read_text()
        decoded = re.findall(r'^decodeImage .* data=([0-9a-f]+)$', stream, re.M)
        assert decoded and {hashlib.sha256(bytes.fromhex(a)).hexdigest() for a in decoded} <= asset_hashes
        result = read(str(prefix) + '.result.json')
        # Combined receipts append provenance; the recorded measurement itself is identical.
        assert {k: v for k, v in row.items() if k != 'evidence'} == result
        if 'evidence' in row:
            e = row['evidence']
            for k in ['receipt', 'result']:
                check(e[k], e[k + 'Sha256'])
            if e['receipt'] not in source_receipts:
                src = read(e['receipt']); bindings(src['bindings'])
                source_receipts[e['receipt']] = {(r['name'], r['frame']): r for r in src['rows']}
            assert source_receipts[e['receipt']][key] == result
        map_by_id = {n['id']: n for n in read(original_dir / 'scene.map.json')}
        geometry = {n['objectId']: n for n in read(probe / row['geometry'])}
        measured = {}
        for id_, rect in row['browserMetrics']['rectangles'].items():
            owner = geometry[map_by_id[id_]['object_id']]
            assert row['nativeBoxes'][id_] == owner
            measured[id_] = dict(x=owner['worldMatrix'][4], y=owner['worldMatrix'][5], width=owner['width'], height=owner['height'])
        if not row['metricFailures']:
            assert all(abs(v - row['browserMetrics']['rectangles'][id_][k]) <= .1 for id_, rect in measured.items() for k, v in rect.items())
    failure_rows = [dict(name=r['name'], frame=r['frame'], pixelFailures=r['pixelFailures'], prefix=r['prefix']) for r in rows if r['pixelFailures']]
    return dict(**counts, scenes=len(compiled) - sum(not r['compiled'] for r in compiled.values()),
                failures=failure_rows, receipt=bind(path)), lookup


def visual(directory, native_rows, representatives, transfers, overrides=None):
    receipt = read(directory / 'review-receipt.json')
    assert receipt['visualReviewCompleted'] is True
    bindings(receipt['bindings'], overrides=overrides)
    bindings(receipt['inspectedSheets'])
    bindings(receipt.get('supplementary', []))
    bindings(receipt.get('supplementaryInspection', []))
    coverage = read(directory / 'coverage.json')
    bindings(coverage['bindings'], overrides=overrides)
    bindings(coverage['artifacts'], overrides=overrides)
    if (directory / 'artifact-bindings.json').exists():
        bindings(read(directory / 'artifact-bindings.json'), overrides=overrides)
    reps = coverage['representatives']; moved = coverage['transfers']
    assert len(reps) == representatives and len(moved) == transfers
    keys = {(r['name'], r['frame']) for r in reps}
    assert len(keys) == len(reps)
    for row in reps:
        assert row == native_rows[(row['name'], row['frame'])]
    for move in moved:
        key = (move['case'], move['frame'])
        assert key not in keys and (move['case'], move['sourceFrame']) in keys
        assert move.get('sameRequestSceneMapReferenceAndMeasurements') or move.get('fullRgbaEqual')
        target = native_rows[key]; source = native_rows[(move['case'], move['sourceFrame'])]
        for k in ['requestSha256', 'rivSha256', 'mapSha256', 'htmlSha256', 'browserMetrics', 'nativeBoxes']:
            assert source[k] == target[k]
        comparisons = move.get('images')
        if comparisons is None:
            comparisons = [dict(source=move['pairs'][side]['source'], target=move['pairs'][side]['destination']) for side in ['chrome', 'native']]
            assert source['width'] == target['width'] and source['height'] == target['height']
        assert len(comparisons) == 2
        for side, comparison in zip(['chrome', 'native'], comparisons):
            bindings([comparison['source'], comparison['target']])
            assert comparison['source']['path'] == source['prefix'] + '.' + side + '.png'
            assert comparison['target']['path'] == target['prefix'] + '.' + side + '.png'
            with Image.open(comparison['source']['path']) as a, Image.open(comparison['target']['path']) as b:
                size = (max(a.width, b.width), max(a.height, b.height))
                left = Image.new('RGBA', size, (255, 255, 255, 255)); left.paste(a.convert('RGBA'), (0, 0))
                right = Image.new('RGBA', size, (255, 255, 255, 255)); right.paste(b.convert('RGBA'), (0, 0))
                assert left.tobytes() == right.tobytes(), key
        keys.add(key)
    assert keys == set(native_rows)
    for sheet in coverage['sheets']:
        bindings([sheet])
        assert any(s['path'] == sheet['path'] and s['sha256'] == sheet['sha256'] for s in receipt['inspectedSheets'])
    return dict(representatives=representatives, transferredFrames=transfers, sheets=len(coverage['sheets']),
                visualReviewCompleted=True, receipt=bind(directory / 'review-receipt.json'))


OUT.mkdir(exist_ok=True)
summary = read(BUILD / 'summary.json')
assert summary['sourceUnchanged'] and (summary['rustTests'], summary['nodeTests']) == (309, 56)
checks = read(BUILD / 'checks.json')
assert len(checks) == 6 and all(c['exitCode'] == 0 for c in checks)
for c in checks:
    check(c['log'], c['logSha256'])
rust_counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;', (BUILD / 'full-rust.log').read_text())
assert sum(int(n) for n, _ in rust_counts) == 309 and all(int(n) == 0 for _, n in rust_counts)
assert re.search(r'pass 56\n', (BUILD / 'full-node.log').read_text()) and re.search(r'fail 0\n', (BUILD / 'full-node.log').read_text())
source_bindings = read(FROZEN / 'source-bindings.json')['files']
assert len(source_bindings) == 249
bindings(source_bindings, current=True)
check(FROZEN / 'html-to-riv', summary['compilerSha256'])
check(FROZEN / 'compiler.wasm', summary['wasmSha256'])
assert '[0x21 | 0x22, 0x11, 0x11] => width <= 4' in (FROZEN / 'inputs/src/assets.rs').read_text()
check(FROZEN / 'inputs/src/jpeg_validation.rs', 'a637966ee9fedd8e5689bae1d655248e78dfffe4e8605cc2cf2e82eda78ca8f9')

tool_dir = MODULE / 'output/immutable-baseline-toolchain-r2'
tools = read(tool_dir / 'manifest.json')
assert tools['baseline'] == BASELINE and tools['status'] == 'built-immutable-baseline'
for name, expected in tools['files'].items():
    check(tool_dir / name, expected)
check(ROOT / 'Cargo.toml', tools['rootCargoTomlSha256'])
check(ROOT / 'Cargo.lock', tools['rootCargoLockSha256'])
guard_path = MODULE / 'validation/check-target-runtime.py'
result = subprocess.run([sys.executable, str(guard_path)], cwd=ROOT, capture_output=True, check=True)
guard = json.loads(result.stdout)
assert guard['status'] == 'pass' and guard['baseline'] == BASELINE
write(OUT / 'runtime-guard.json', dict(**guard, command=[sys.executable, str(guard_path)], verifier=bind(guard_path)))

jpeg_dir, jpeg_cases = compile_suite('public-jpeg-layout-r2', 56, 34)
image_dir, image_cases = compile_suite('public-image-jpeg-regression-r2', 40, 27)
aspect_dir, aspect_cases = compile_suite('public-image-aspect-jpeg-r2', 12, 12)
parities = [parity(name, cases) for name, cases in [('public-jpeg-parity-r2', jpeg_cases),
    ('public-image-jpeg-parity-r2', image_cases), ('public-image-aspect-jpeg-parity-r2', aspect_cases)]]

jpeg_native, jpeg_rows = native(jpeg_dir / 'combined-receipt.json', jpeg_dir, jpeg_cases,
    dict(frames=272, geometryPass=272, pixelPass=272, presencePass=272))
assert read(jpeg_dir / 'combined-receipt.json')['summary']['transferredFrames'] == 272
image_native, image_rows = native(MODULE / 'output/public-image-layout-r2/combined-receipt.json', image_dir, image_cases,
    dict(frames=216, geometryPass=216, pixelPass=198, presencePass=216))
aspect_native, aspect_rows = native(MODULE / 'output/public-image-aspect-r1/native-receipt.json', aspect_dir, aspect_cases,
    dict(frames=96, geometryPass=96, pixelPass=80, presencePass=96))

asset_dir = MODULE / 'output/public-image-assets-jpeg-r2'
assets = read(asset_dir / 'receipt.json')
assert assets['passed'] and assets['planned'] == assets['selected'] == assets['completed'] == 86 and assets['failures'] == 0
assert assets['sourceBindingsUnchanged'] and assets['fixtureInputsUnchanged']
check(asset_dir / 'cases.json', assets['caseManifestSha256'])
for b in assets['inputs']:
    check(b['source'], b['sha256']); check(b['snapshot'], b['sha256'])
assert all(b['original'] and b['snapshot'] for b in assets['unchanged'])
manifest = read(asset_dir / 'cases.json')
for b in manifest['assets']:
    check(asset_dir / b['path'], b['sha256'])
for b in manifest['inputs']:
    check(b['source'], b['sha256'])
for b in manifest['expectationSources']:
    check(MODULE / b['path'], b['sha256'])
asset_observations = 0
asset_artifacts = 0
for case in assets['results']:
    assert case['passed'] and not case['issues']
    for observation in case['observations'].values():
        asset_observations += 1
        assert all(value is not False for value in observation['checks'].values())
    for b in case['artifacts']:
        check(asset_dir / b['path'], b['sha256']); asset_artifacts += 1
assert asset_observations == 252

reg_dir = MODULE / 'output/public-transport-malformed-jpeg-regression-r2'
regression = read(reg_dir / 'receipt.json')
assert regression['allExact'] and regression['total'] == regression['passed'] == len(regression['results']) == 794
assert regression['sourceBindingsVerified'] == len(source_bindings)
for name, value in regression['inputHashes'].items():
    check(reg_dir / 'frozen' / name, value)
check(reg_dir / 'references.json', regression['referencesSha256'])
for row in regression['results']:
    assert row['exact'] and row['exitCode'] == 0 and row['termination'] is None
    actual = Path(row['command'][-1]).parent
    reference = Path(row['reference']['reference'])
    assert Path(row['command'][1]) == reference / 'request.json'
    check(row['command'][0], summary['compilerSha256'])
    for name, expected in row['reference']['hashes'].items():
        check(reference / name, expected)
        if name != 'request.json':
            assert row['actualHashes'][name] == expected
            check(actual / name, expected)

# The tiny-image asset manifest names an earlier generator. Verify that exact
# recovered script rather than attributing the historical encoding to new code.
generation = MODULE / 'output/public-jpeg-cases-generation-r1'
tiny = read(MODULE / 'fixtures/images/jpeg-single-chroma-r1/manifest.json')
check(generation / 'recovered-reuse-generator.py', tiny['generator']['sha256'])
for row in tiny['files']:
    check(row['path'], row['sha256']); check(row['source'], row['sourceSha256'])
bindings([tiny['encoder']])
assert read(generation / 'reproduced-cases.json') == read(MODULE / 'validation/public-jpeg-cases.json') == read(jpeg_dir / 'cases.json')
read(generation / 'first-attempt-manifest.json')

# Prior failed evidence is retained, including the invalidated tiny exception.
boundary = MODULE / 'output/jpeg-decoder-boundary-r1'
bindings(read(boundary / 'review-bindings.json'))
revised = read(boundary / 'revised-gate-review.json')
bindings(revised['bindings'])
check(revised['source']['snapshot'], revised['source']['sha256'])
check(MODULE / 'src/assets.rs', revised['source']['sha256'])
assert len(revised['rows']) == 8
for row in revised['rows']:
    assert row['meanRgbError'] == 7.5 and row['pixelFailures']
    bindings([row[k] for k in ['request', 'scene', 'result', 'chrome', 'native']])
for name in ['validation/jpeg-subsampling-review.md', 'output/jpeg-subsampling-review-r1/receipt.json',
             'validation/jpeg-decoder-boundary-review.md']:
    digest(MODULE / name)

jpeg_visual = visual(jpeg_dir / 'visual', jpeg_rows, 34, 238)
image_visual = visual(MODULE / 'output/public-image-layout-r2/visual', image_rows, 45, 171)
aspect_visual = visual(MODULE / 'output/public-image-aspect-r1/visual', aspect_rows, 36, 60,
    overrides={str(MODULE / 'validation/public-image-visual.py'): str(MODULE / 'output/public-image-aspect-r1/source-snapshots/validation/public-image-visual.py')})

# Final JPEG visual completion package binds supplementary detail and gallery checks.
package = read(jpeg_dir / 'visual/package-receipt.json')
assert package['visualReviewCompleted'] and package['liveProductionMatchesFrozenBuild'] and package['helperMatchesCompletionSnapshot']
bindings(package['bindings'])
bindings(package['unchangedLiveProductionSources'], current=True)
details = read(jpeg_dir / 'visual/details-receipt.json')
bindings([details['driver'], details['coverage']])
bindings(details['sheets'])
retained = details['retainedFailure']
bindings([retained['sheet'], retained['oldReceipt'], retained['currentCompileReceipt'], *retained['fullImages'].values()])
assert retained['sameAuthoredRequest'] and retained['priorFrames'] == retained['priorPixelFailures'] == 8
assert retained['priorImageRegionMeanRgbError'] == 7.5 and retained['currentDiagnostic'] == jpeg_cases['single-chroma-2x1-420-baseline']['diagnostics']

digest(Path(__file__))
digest(MODULE / 'validation/public-jpeg-visual-review.md')
digest(OUT / 'runtime-guard.json')
write(OUT / 'artifact-bindings.json', [dict(path=p, sha256=s) for p, s in sorted(verified.items())])
receipt = dict(status='verified-bounded-public-jpeg-checkpoint', baseline=BASELINE,
    scope='Standalone compiler; original encoded assets; ordinary Rive files and immutable Mac native consumer; no new renders or compiles in this verifier.',
    compiler=bind(FROZEN / 'html-to-riv'), wasm=bind(FROZEN / 'compiler.wasm'),
    sourceBindingsVerified=len(source_bindings), rustTests=309, nodeTests=56,
    publicJpegCases=dict(total=56, admitted=34, diagnosed=22, parityObservations=112),
    native=jpeg_native, visual=jpeg_visual, visualReviewCompleted=True,
    priorPublicImage=dict(cases=40, admitted=27, parityObservations=80, native=image_native, visual=image_visual),
    additionalAspect=dict(cases=12, admitted=12, parityObservations=24, native=aspect_native, visual=aspect_visual),
    assets=dict(cases=86, interfaceObservations=asset_observations, artifactsVerified=asset_artifacts, receipt=bind(asset_dir / 'receipt.json')),
    priorExactOutputs=794, regression=bind(reg_dir / 'receipt.json'), parities=parities,
    gate='All 4:2:2 and 4:2:0 widths <=4 diagnosed unsupported-image; 4:4:4 unchanged; no tiny/content-specific exceptions qualified.',
    invalidatedTinyException=dict(failedFrames=8, meanImageRgbError=7.5, receipt=bind(boundary / 'revised-gate-review.json')),
    immutableTools=bind(tool_dir / 'manifest.json'), runtimeGuard=bind(OUT / 'runtime-guard.json'),
    historicalSourceSnapshots=historical, requestSerializationDifferences=list(request_serializations.values()),
    limitations=['Finite 34-scene JPEG qualification, not all sizes/colors/layouts/platforms or universal decoder equality.',
        'New JPEG evidence transfers all272 frames only after exact original request/RIV/map/reset/tool identity.',
        'Older image and aspect suites retain18 and16 pixel failures respectively; geometry and presence pass.',
        'Opaque-white RGBA review transfers certify captured pixels only; offscreen content is not inferred.',
        'Asset limits do not establish whole-process memory or CPU bounds; no runtime/dependency mutations.'],
    evidenceBindings=bind(OUT / 'artifact-bindings.json'))
write(MODULE / 'validation/public-jpeg-receipt.json', receipt)
write(OUT / 'verification.json', dict(status='pass', receipt=bind(MODULE / 'validation/public-jpeg-receipt.json'),
    verifiedArtifacts=len(verified), sourceBindings=len(source_bindings), nativeFrames=584, priorExactOutputs=794))
print(json.dumps(read(OUT / 'verification.json'), indent=2))
