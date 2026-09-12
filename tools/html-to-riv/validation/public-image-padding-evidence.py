#!/usr/bin/env python3
"""Verify the point-padding checkpoint without compiling or rendering again."""
from collections import Counter
from pathlib import Path
import hashlib
import json
import math
import re
import subprocess
import sys
from PIL import Image

MODULE = Path(__file__).resolve().parents[1]
BUILD = MODULE / 'output/public-image-padding-build-r2'
FROZEN = BUILD / 'frozen'
OUT = MODULE / 'output/public-image-padding-checkpoint-r2'
HISTORICAL = MODULE / 'output/public-image-padding-checkpoint-r1'
verified = {}


def sha(path):
    path = Path(path).resolve()
    value = hashlib.sha256(path.read_bytes()).hexdigest()
    assert str(path) not in verified or verified[str(path)] == value, path
    verified[str(path)] = value
    return value


def bind(path):
    return dict(path=str(Path(path).resolve()), sha256=sha(path))


def check(path, expected):
    assert sha(path) == expected, path


def read(path):
    sha(path)
    return json.loads(Path(path).read_text())


def bindings(items, current=False):
    for item in items:
        path = item.get('snapshot', item['path'])
        if (Path(path).name == 'public-image-native.mjs' and
                item['sha256'] == '1b980bc6b4d9142c56ef9fea299bb84f0ba44217b994a6193e3390f8ceccd13c'):
            path = MODULE / 'output/public-image-layout-r1/recovered-reference-driver/public-image-native.mjs'
        check(path, item['sha256'])
        if current:
            check(item['path'], item['sha256'])


def suite(label, total, admitted, parity_label, changed=False):
    root = MODULE / 'output' / label
    build = read(root / 'build-receipt.json')
    assert Path(build['build']) == BUILD
    check(BUILD / 'summary.json', build['summarySha256'])
    check(FROZEN / 'source-bindings.json', build['bindingsSha256'])
    assert build['compilerSha256'] == summary['compilerSha256']
    bindings(read(root / 'authoring-bindings.json'), current=True)
    cases = {c['name']: c for c in read(root / 'cases.json')}
    rows = read(root / 'compile-receipt.json')
    assert len(rows) == len(cases) == total and sum(r['compiled'] for r in rows) == admitted
    for row in rows:
        case = cases[row['name']]
        directory = root / row['name']
        assert row == read(directory / 'compile-result.json') and row['matchesExpectation']
        assert row['expected'] == case['expected']
        check(directory / 'request.json', row['requestSha256'])
        request = read(directory / 'request.json')
        assert {k: v for k, v in request.items() if k != 'assets'} == case['input']
        for key, asset in case['assetFiles'].items():
            assert request['assets'][key] == dict(kind='image', bytes=list((MODULE / asset).read_bytes()))
            sha(MODULE / asset)
        prior = row.get('previous')
        if prior:
            assert prior['sameRequest'] and read(Path(prior['directory']) / 'request.json') == request
        if row['compiled']:
            assert row['status'] == 0
            for name, key in [('scene.riv', 'rivSha256'), ('scene.map.json', 'mapSha256')]:
                check(directory / name, row[key])
                if prior and (Path(prior['directory']) / name).exists():
                    equal = sha(Path(prior['directory']) / name) == row[key]
                    assert prior['sameRiv' if name == 'scene.riv' else 'sameMap'] == equal
                    if not changed:
                        assert equal
            if prior and 'sameRiv' in prior:
                assert prior['sameRiv'] == (not changed)
                if not changed:
                    assert prior['sameMap']
            elif prior:
                assert row['name'] == 'image-padding' and case['priorExpected']['outcome'] == 'diagnostic'
        else:
            assert row['status'] != 0 and not (directory / 'scene.riv').exists() and not (directory / 'scene.map.json').exists()
            assert read(directory / 'compile.log') == row['diagnostics']
    parity = read(MODULE / 'output' / parity_label / 'receipt.json')
    bindings(parity['bindings'])
    assert parity['cases'] == total and parity['passed'] == parity['observations'] == len(parity['rows']) == total * 2
    by_name = {r['name']: r for r in rows}
    assert Counter(r['transport'] for r in parity['rows']) == {'raw-abi': total, 'public-js': total}
    for observation in parity['rows']:
        row = by_name[observation['name']]
        assert observation['matchesCli'] and observation['ok'] == row['compiled']
        for key in ('requestSha256', 'rivSha256', 'mapSha256', 'diagnostics'):
            if key in observation:
                assert observation[key] == row[key]
    return root, by_name


def native(root, compiled):
    document = read(root / 'combined-receipt.json')
    assert document['browser'] == '153.0.8010.12' and not document['errors']
    bindings(document['bindings'] + document['sources'] + [document['compilerBuild']])
    rows = document['rows']
    assert Counter(r['name'] for r in rows) == {n: 8 for n, c in compiled.items() if c['compiled']}
    for row in rows:
        prefix, probe = Path(row['prefix']), Path(row['probeDirectory'])
        source = prefix.parent.parent
        case = compiled[row['name']]
        evidence = row['evidence']
        check(evidence['receipt'], evidence['receiptSha256'])
        check(evidence['result'], evidence['resultSha256'])
        recorded = read(evidence['result'])
        assert recorded == {k: v for k, v in row.items() if k != 'evidence'}
        capture = read(evidence['receipt'])
        bindings(capture['bindings'])
        assert [r for r in capture['rows'] if (r['name'], r['frame']) == (row['name'], row['frame'])] == [recorded]
        assert read(source / 'request.json') == read(root / row['name'] / 'request.json')
        assert row['rivSha256'] == case['rivSha256'] and row['mapSha256'] == case['mapSha256']
        for key, path in [('requestSha256', source / 'request.json'), ('rivSha256', source / 'scene.riv'),
                          ('mapSha256', source / 'scene.map.json'), ('htmlSha256', prefix.parent / 'reference.html'),
                          ('chromeSha256', str(prefix) + '.chrome.png'), ('nativeSha256', str(prefix) + '.native.png'),
                          ('geometrySha256', probe / row['geometry']), ('streamSha256', probe / row['stream'])]:
            check(path, row[key])
        check(probe / 'scene.riv', row['rivSha256'])
        frame = read(probe / 'frames.json')['frames'][row['frame']]
        assert all(row[k] == v for k, v in frame.items())
        assert (row['instance'], row['step']) == (row['frame'] // 4, row['frame'] % 4)
        assert [row['width'], row['height']] == [[240, 240], [390, 320], [768, 560], [240, 240]][row['step']]
        if row.get('transferredNative'):
            old = row['transferredNative']
            check(old['result'], old['sha256'])
            previous = read(old['result'])
            for key in ('rivSha256', 'streamSha256', 'geometrySha256', 'nativeSha256', 'instance', 'step', 'width', 'height'):
                assert previous[key] == row[key]
        geometry = {g['objectId']: g for g in read(probe / row['geometry'])}
        mapping = {n['id']: n['object_id'] for n in read(source / 'scene.map.json')}
        for name, expected in row['browserMetrics']['rectangles'].items():
            measured = geometry[mapping[name]]
            assert measured == row['nativeBoxes'][name]
            actual = dict(x=measured['worldMatrix'][4], y=measured['worldMatrix'][5],
                          width=measured['width'], height=measured['height'])
            if not row['metricFailures']:
                assert all(abs(actual[k] - value) <= .1 for k, value in expected.items())
    counts = dict(frames=len(rows), geometryPass=sum(not r['metricFailures'] for r in rows),
                  pixelPass=sum(not r['pixelFailures'] for r in rows),
                  presencePass=sum(all(i['passed'] for i in r['imagePresence'].values()) for r in rows))
    assert all(document['summary'][k] == v for k, v in counts.items())
    return dict(**counts, receipt=bind(root / 'combined-receipt.json'), failures=[
        dict(name=r['name'], frame=r['frame'], pixelFailures=r['pixelFailures'], imagePresence=r['imagePresence'])
        for r in rows if r['metricFailures'] or r['pixelFailures'] or not all(i['passed'] for i in r['imagePresence'].values())])


def visual(root, label='visual-content-r2', target_root=None):
    directory = root / label
    receipt, coverage = read(directory / 'review-receipt.json'), read(directory / 'coverage.json')
    assert receipt['visualReviewCompleted'] and receipt['counts'] == coverage['counts']
    bindings(receipt['bindings'] + receipt['inspectedSheets'] + receipt.get('supplementaryInspection', []))
    bindings(coverage['bindings'] + coverage['artifacts'])
    assert {b['path'] for b in receipt['inspectedSheets']} == {b['path'] for b in coverage['sheets'] + coverage['beforeAfterSheets']}
    rows = {(r['name'], r['frame']): r for r in read(root / 'combined-receipt.json')['rows']}
    # Reuse a prior completed review only for the same public request, emitted
    # file/map and actual captured frame results. Compiler labels are not proof.
    if target_root is not None:
        target_rows = {(r['name'], r['frame']): r for r in read(target_root / 'combined-receipt.json')['rows']}
        for name in {r['name'] for r in rows.values()}:
            for file in ('request.json', 'scene.riv', 'scene.map.json'):
                check(target_root / name / file, sha(root / name / file))
        for key, row in rows.items():
            assert {k: v for k, v in row.items() if k != 'evidence'} == {
                k: v for k, v in target_rows[key].items() if k != 'evidence'}
    seen = set()
    for row in coverage['representatives']:
        key = (row['name'], row['frame'])
        assert key not in seen and row == rows[key]
        seen.add(key)
    representatives = set(seen)
    for transfer in coverage['transfers']:
        key = (transfer['case'], transfer['frame'])
        assert key not in seen and (transfer['case'], transfer['sourceFrame']) in representatives
        origin = rows[(transfer['case'], transfer['sourceFrame'])]
        target = rows[key]
        for field in ('rivSha256', 'mapSha256', 'requestSha256', 'htmlSha256', 'nativeBoxes'):
            assert origin[field] == target[field]
        assert origin['browserMetrics']['rectangles'] == target['browserMetrics']['rectangles']
        metadata = json.loads(json.dumps({k: origin[k] for k in ('browserMetrics', 'imagePresence')}))
        for difference in transfer.get('backgroundBoxMetadataDifferences', []):
            keys = difference['path']
            assert len(keys) == 6 and keys[-1] == 'borderBox' and isinstance(keys[-2], int)
            assert ((keys[:2] == ['browserMetrics', 'images'] and keys[3] == 'backgroundChain') or
                    (keys[0] == 'imagePresence' and keys[2:4] == ['background', 'chain']))
            left, right = metadata, target
            for key_part in keys[:-1]:
                left, right = left[key_part], right[key_part]
            assert left['borderBox'] == difference['sourceValue'] and right['borderBox'] == difference['targetValue']
            assert difference['transferred'] is False
            for box in (left['borderBox'], right['borderBox']):
                assert set(box) == {'x', 'y', 'width', 'height'} and all(math.isfinite(v) for v in box.values())
            left['borderBox'] = right['borderBox']
        assert metadata == {k: target[k] for k in ('browserMetrics', 'imagePresence')}
        assert transfer['backgroundBoxMetadataTransferred'] is False
        assert transfer['targetFrameEvidence'] == target['evidence']
        for key_part in ('metricFailures', 'pixelFailures', 'pixelMetrics', 'imagePresence'):
            assert transfer[key_part] == target[key_part]
        assert len(transfer['images']) == 2
        for side, images in zip(('chrome', 'native'), transfer['images']):
            assert images['source']['path'] == origin['prefix'] + '.' + side + '.png'
            assert images['target']['path'] == target['prefix'] + '.' + side + '.png'
            bindings([images['source'], images['target']])
            with Image.open(images['source']['path']) as a, Image.open(images['target']['path']) as b:
                size = (max(a.width, b.width), max(a.height, b.height))
                aa, bb = Image.new('RGBA', size, 'white'), Image.new('RGBA', size, 'white')
                aa.paste(a.convert('RGBA'), (0, 0)); bb.paste(b.convert('RGBA'), (0, 0))
                assert aa.tobytes() == bb.tobytes()
        seen.add(key)
    assert seen == set(rows)
    for comparison in coverage['beforeAfter']:
        current, previous = comparison['current'], comparison['previous']
        assert current == rows[(comparison['case'], comparison['frame'])]
        assert current['rivSha256'] != previous['rivSha256']
        for key in ('requestSha256', 'browserMetrics', 'width', 'height', 'instance', 'step'):
            assert current[key] == previous[key]
        assert comparison['currentGeometryPass'] == (not current['metricFailures'])
        assert comparison['previousGeometryPass'] == (not previous['metricFailures'])
        for side in ('chrome', 'native'):
            check(previous['prefix'] + '.' + side + '.png', previous[side + 'Sha256'])
        with Image.open(previous['prefix'] + '.chrome.png') as a, Image.open(current['prefix'] + '.chrome.png') as b:
            assert a.size == b.size and a.convert('RGBA').tobytes() == b.convert('RGBA').tobytes()
    result = dict(counts=receipt['counts'], receipt=bind(directory / 'review-receipt.json'))
    if target_root is not None:
        result['exactEvidenceTransferredTo'] = bind(target_root / 'combined-receipt.json')
    return result


summary = read(BUILD / 'summary.json')
assert summary['sourceUnchanged'] and (summary['rustTests'], summary['nodeTests']) == (311, 56)
source = read(FROZEN / 'source-bindings.json')['files']
bindings(source, current=True)
for check_result in read(BUILD / 'checks.json'):
    assert check_result['exitCode'] == 0
    check(check_result['log'], check_result['logSha256'])
assert sum(map(int, re.findall(r'test result: ok\. (\d+) passed;', (BUILD / 'full-rust.log').read_text()))) == 311
check(FROZEN / 'html-to-riv', summary['compilerSha256'])
check(FROZEN / 'compiler.wasm', summary['wasmSha256'])
tool_directory = MODULE / 'output/immutable-baseline-toolchain-r2'
toolchain = read(tool_directory / 'manifest.json')
assert toolchain['status'] == 'built-immutable-baseline' and toolchain['baseline'] == '6c7ac16617835b5f581784ff08a9e779bb52faf3'
for name, value in toolchain['files'].items():
    check(tool_directory / name, value)
check(MODULE.parents[1] / 'Cargo.toml', toolchain['rootCargoTomlSha256'])
check(MODULE.parents[1] / 'Cargo.lock', toolchain['rootCargoLockSha256'])
guard = json.loads(subprocess.check_output([sys.executable, str(MODULE / 'validation/check-target-runtime.py')]))
assert guard['status'] == 'pass' and guard['baseline'] == '6c7ac16617835b5f581784ff08a9e779bb52faf3'
suites = [suite(*args) for args in [
    ('public-image-padding-layout-r2', 24, 22, 'public-image-padding-parity-r2'),
    ('public-image-padding-regression-r2', 40, 28, 'public-image-padding-regression-parity-r2'),
    ('public-image-padding-jpeg-r2', 56, 34, 'public-image-padding-jpeg-parity-r2'),
    ('public-image-padding-aspect-r2', 12, 12, 'public-image-padding-aspect-parity-r2'),
    ('public-image-padding-alpha-r2', 2, 2, 'public-image-padding-alpha-parity-r2'),
    ('public-image-padding-percent-r2', 2, 2, 'public-image-padding-percent-parity-r2', True)]]
regression_path = MODULE / 'output/public-transport-malformed-padding-regression-r2/receipt.json'
regression = read(regression_path)
assert regression['allExact'] and regression['total'] == regression['passed'] == len(regression['results']) == 794
for row in regression['results']:
    assert row['exact'] and row['exitCode'] == 0 and row['termination'] is None
    for name, value in row['reference']['hashes'].items():
        check(Path(row['reference']['reference']) / name, value)
        if name != 'request.json':
            assert row['actualHashes'][name] == value
            check(Path(row['command'][-1]).parent / name, value)
native_results = [native(*suites[i]) for i in (0, 1, 4, 5)]
for result, expected in zip(native_results, [(176, 176, 152, 176), (224, 224, 206, 224), (16, 16, 16, 16), (16, 16, 16, 16)]):
    assert tuple(result[k] for k in ('frames', 'geometryPass', 'pixelPass', 'presencePass')) == expected
visual_results = [visual(MODULE / 'output/public-image-padding-layout-r1', target_root=suites[0][0]),
                  visual(MODULE / 'output/public-image-padding-alpha-r1', target_root=suites[4][0]),
                  visual(MODULE / 'output/public-image-padding-migration-r1', target_root=suites[1][0]),
                  visual(suites[5][0], label='visual-r2')]
assert visual_results[-1]['counts']['beforeAfterFrames'] == 16
before_root = MODULE / 'output/public-image-padding-percent-before-r1'
before_receipt = read(before_root / 'receipt.json')
bindings(before_receipt['bindings'])
before_compiled = {r['name']: r for r in read(before_root / 'compile-receipt.json')}
before_native = native(before_root, before_compiled)
assert tuple(before_native[k] for k in ('frames', 'geometryPass', 'pixelPass', 'presencePass')) == (16, 0, 0, 0)
before_visual = visual(before_root, label='visual-before')
before_rows = {(r['name'], r['frame']): r for r in read(before_root / 'native-receipt.json')['rows']}
after_coverage = read(suites[5][0] / 'visual-r2/coverage.json')
assert len(after_coverage['beforeAfter']) == len(before_rows) == 16
for comparison in after_coverage['beforeAfter']:
    assert comparison['previous'] == before_rows[(comparison['case'], comparison['frame'])]
older_path = MODULE / 'output/public-image-padding-regression-suite-r2/older73-receipt.json'
older = read(older_path)
assert older['allExact'] and older['total'] == older['passed'] == len(older['rows']) == 73
assert Counter(row['suite'] for row in older['rows']) == {'regression': 27, 'jpeg': 34, 'aspect': 12}
bindings([older['all98Reference'], older['driver']])
for row in older['rows']:
    bindings([v for v in row.values() if isinstance(v, dict) and 'path' in v and 'sha256' in v])
    r1 = next(r for r in read(row['r1Compile']['path']) if r['name'] == row['name'])
    r2 = next(r for r in read(row['r2Compile']['path']) if r['name'] == row['name'])
    assert r1['compiled'] and r2['compiled'] and r1['previous']['sameRequest']
    assert Path(r1['previous']['directory']) == Path(row['olderRiv']['path']).parent
    for suffix, filename in [('Request', 'request.json'), ('Riv', 'scene.riv'), ('Map', 'scene.map.json')]:
        left, right = row['older' + suffix], row['current' + suffix]
        assert left['sha256'] == right['sha256']
        assert Path(right['path']) == Path(row['r2Compile']['path']).parent / row['name'] / filename
preservation_path = MODULE / 'output/public-image-padding-candidate-r1/content-preservation-receipt.json'
preservation = read(preservation_path)
bindings(preservation['bindings'])
assert (preservation['frames'], preservation['originalPixelFailures'], preservation['originalPresenceFailures'], preservation['contentPresencePass']) == (176, 24, 28, 176)
for row in preservation['rows']:
    bindings([row['originalResult'], row['contentResult'], row['independentChrome']])
    assert all(row[k] for k in ('nativePixelsExact', 'chromePixelsExact', 'originalPixelGatesExact',
                               'originalBorderPresenceRetained', 'independentContentGeometryExact'))
helper_tests = read(HISTORICAL / 'image-content-tests.json')
assert helper_tests['exitCode'] == 0
bindings(helper_tests['sourceBindings'] + [helper_tests['log']])
log = Path(helper_tests['log']['path']).read_text()
assert re.search(r'(?:#|ℹ) pass 17\b', log) and re.search(r'(?:#|ℹ) fail 0\b', log)
transfer_tests = read(MODULE / 'output/public-image-transfer-check-r2/receipt.json')
assert transfer_tests['passed'] == transfer_tests['planned'] == 12 and transfer_tests['originalInputsUnchanged']
bindings([transfer_tests[k] for k in ('source', 'testedSnapshot', 'driver')] + transfer_tests['inputs'])
for row in transfer_tests['rows']:
    assert row['passed']
    bindings([v for v in row.values() if isinstance(v, dict) and 'path' in v and 'sha256' in v])
projection = read(MODULE / 'output/public-image-padding-layout-r1/visual-content-r2/projection-check-receipt.json')
bindings(projection['bindings'])
assert len(projection['metadataChecks']) == 18 and all(c['compatible'] == c['expectedCompatible'] for c in projection['metadataChecks'])
assert projection['allTransferMetadataAndTargetGatesVerified'] == 152 and projection['originalPixelFailureFrames'] == 24
package = read(MODULE / 'output/public-image-padding-layout-r1/visual-content-r2/package-receipt.json')
bindings(package['bindings'])
assert package['visualReviewCompleted'] and package['helperMatchesPreparationAndCompletion']
receipt = dict(scope='Bounded public point-padding checkpoint; recorded pixel failures and broader unfinished contexts remain',
               build=summary, sourceBindings=len(source), runtimeGuard=guard, regression=bind(regression_path),
               native=native_results, visual=visual_results, preservation=bind(preservation_path),
               percentageRepair=dict(before=before_native, beforeVisual=before_visual, after=native_results[-1]),
               api=dict(requests=136, abiAndJsComparisonsAgainstCli=272, allMatch=True),
               exactEarlierImageJpegRatioFiles=dict(count=73, receipt=bind(older_path)),
               validatorTests=dict(imageContent=17, transfer=12, visualProjection=18), verifier=bind(__file__), artifacts=[
                   dict(path=path, sha256=value) for path, value in sorted(verified.items())])
OUT.mkdir(exist_ok=True)
(OUT / 'verification.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(dict(sourceBindings=len(source), artifactBindings=len(verified), native=[
    {k: r[k] for k in ('frames', 'geometryPass', 'pixelPass', 'presencePass')} for r in native_results],
    exactRegressions=794, visualFrames=sum(r['counts']['frames'] for r in visual_results))))
