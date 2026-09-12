"""Verify mixed image constraint evidence without compilation/rendering.

Successful private recipes transfer by exact requests, ordinary files, maps and
measured rows. Every failed pixel frame stays in the admitted corpus; no tolerance changes are inferred.
"""
import hashlib
import json
import re
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1]
OUTPUT = MODULE / 'output'


def verify():
    checked = {}

    def bind(path, expected=None):
        path = Path(path).resolve()
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if expected is not None:
            assert digest == expected, f'Changed artifact: {path}'
        checked[str(path)] = digest
        return digest

    def read(path):
        bind(path)
        return json.loads(Path(path).read_text())

    def bindings(rows, live=False):
        for row in rows:
            bind(row.get('snapshot', row['path']), row['sha256'])
            if live:
                bind(row['path'], row['sha256'])

    def measured(row):
        return {key: value for key, value in row.items() if key != 'evidence'}

    def visual(root, frames):
        review = read(root / 'visual-r1/review-receipt.json')
        assert review['visualReviewCompleted'] and review['counts']['frames'] == frames
        bindings(review['bindings'])
        bindings(review['inspectedSheets'])
        coverage = read(root / 'visual-r1/coverage.json')
        bindings(coverage['bindings'])
        bindings(coverage['artifacts'])
        representatives = {(r['name'], r['frame']) for r in coverage['representatives']}
        transfers = {(r['case'], r['frame']) for r in coverage['transfers']}
        assert not representatives.intersection(transfers)
        reviewed = representatives | transfers
        assert len(reviewed) == frames
        inspected = {(r['case'], r['frame']) for sheet in coverage['sheets'] for r in sheet['members']}
        assert representatives <= inspected
        assert {(r['path'], r['sha256']) for r in review['inspectedSheets']} == {
            (r['path'], r['sha256']) for r in coverage['sheets']}
        return reviewed

    build = OUTPUT / 'public-image-mixed-build-r1'
    summary = read(build / 'summary.json')
    assert summary['status'] == 'public-build-and-transport-pass' and summary['sourceUnchanged']
    assert (summary['rustTests'], summary['nodeTests']) == (359, 56)
    checks = read(build / 'checks.json')
    assert len(checks) == 6 and {c['name'] for c in checks} == {
        'full-rust', 'native-build', 'wasm-build', 'typescript', 'full-node', 'runtime-guard'}
    for check in checks:
        assert check['exitCode'] == 0
        bind(check['log'], check['logSha256'])
    check_by_name={c['name']:c for c in checks}
    rust_log=Path(check_by_name['full-rust']['log']).read_text()
    rust_results=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed',rust_log)
    assert rust_results and all(status=='ok' and failed=='0' for status,_,failed in rust_results)
    assert sum(int(passed) for _,passed,_ in rust_results)==359
    node_log=Path(check_by_name['full-node']['log']).read_text()
    assert 'ℹ tests 56' in node_log and 'ℹ pass 56' in node_log and 'ℹ fail 0' in node_log
    guard=json.loads(Path(check_by_name['runtime-guard']['log']).read_text())
    assert guard['status']=='pass' and guard['baseline']=='6c7ac16617835b5f581784ff08a9e779bb52faf3'
    assert guard['baselineTree']=='25ccbb131d88dbd0fdde8f4c660919143b252edf'
    assert guard['changesOutsideCompiler']=={'index':[],'worktree':[],'untracked':[]}
    source_bindings = read(build / 'frozen/source-bindings.json')['files']
    assert len(source_bindings) == 287
    bindings(source_bindings, live=True)
    bind(build / 'frozen/html-to-riv', summary['compilerSha256'])
    bind(build / 'frozen/compiler.wasm', summary['wasmSha256'])
    corpora, transfers, pixel_failures, prior_comparisons = [], [], [], []
    prior_index = {}
    for name in ['public-image-preferred-constraints-diagnostic-r1', 'public-image-preferred-constraints-prior-r1']:
        root = OUTPUT / name
        for row in read(root / 'compile-receipt.json'):
            prior_index[row['name']] = root, row

    for label, donor_name, requests, files, pixels, donor_frames in [
        ('primary', 'image-preferred-mixed-candidate-r1', 16, 16, 120, 128),
        ('combined', 'image-preferred-mixed-combined-r1', 8, 8, 54, 64),
        ('diagnostic', None, 6, 0, 0, 0),
        ('prior', None, 20, 2, 16, 0),
    ]:
        root = OUTPUT / f'public-image-mixed-{label}-r1'
        # Matrices added after the build have separate live/snapshot authoring bindings.
        bindings(read(root / 'authoring-bindings.json'), live=True)
        assert read(root / 'frozen/source-bindings.json')['files'] == source_bindings
        bind(root / 'frozen/html-to-riv', summary['compilerSha256'])
        emitted = read(root / 'compile-receipt.json')
        assert len(emitted) == len({r['name'] for r in emitted}) == requests
        assert sum(r['compiled'] for r in emitted) == files and all(r['matchesExpectation'] for r in emitted)
        indexed = {r['name']: r for r in emitted}
        cases=read(root/'cases.json')
        case_index={c['name']:c for c in cases}
        assert len(cases)==len(case_index)==requests and set(case_index)==set(indexed)
        for row in emitted:
            folder = root / row['name']
            bind(folder / 'request.json', row['requestSha256'])
            case=case_index[row['name']]
            assert case['expected']==row['expected']
            expected_request=dict(case['input'])
            expected_request['assets']={}
            for key,filename in case['assetFiles'].items():
                asset=MODULE/filename
                bind(asset)
                expected_request['assets'][key]={'kind':'image','bytes':list(asset.read_bytes())}
            assert read(folder/'request.json')==expected_request
            if row['compiled']:
                bind(folder / 'scene.riv', row['rivSha256'])
                bind(folder / 'scene.map.json', row['mapSha256'])
            else:
                assert not (folder / 'scene.riv').exists() and not (folder / 'scene.map.json').exists()
                assert row['diagnostics'] and all(d['code'] == 'unsupported-target-semantics' for d in row['diagnostics'])
            if label == 'prior':
                old_root, old = prior_index[row['name']]
                assert not old['compiled'] and old['requestSha256'] == row['requestSha256']
                bind(old_root / row['name'] / 'request.json', row['requestSha256'])
                assert row['compiled'] == (row['name'] in {'image-preferred-diagnostic-mixed-point-percent', 'image-responsive-constraints-diagnostic-preferred-axis-maximum'})
                prior_comparisons.append({'case': row['name'], 'priorRoot': str(old_root),
                    'requestSha256': row['requestSha256'], 'wasCompiled': False, 'compiled': row['compiled'],
                    'priorDiagnostics': old['diagnostics'], 'currentDiagnostics': row['diagnostics'],
                    'diagnosticsUnchanged': old['diagnostics'] == row['diagnostics']})
        transport = read(OUTPUT / f'public-image-mixed-{label}-parity-r1/receipt.json')
        bindings(transport['bindings'])
        assert transport['cases'] == requests and transport['observations'] == transport['passed'] == 2 * requests
        assert transport['sourceBindingsVerified']
        wasm_bindings=[b for b in transport['bindings'] if Path(b['path']).name=='compiler.wasm']
        assert len(wasm_bindings)==1 and wasm_bindings[0]['sha256']==summary['wasmSha256']
        assert len(transport['rows'])==2*requests
        assert {(r['name'],r['transport']) for r in transport['rows']}=={(name,kind) for name in indexed for kind in ['raw-abi','public-js']}
        for result in transport['rows']:
            reference=indexed[result['name']]
            assert result['matchesCli'] and result['ok']==reference['compiled']
            assert result['requestSha256']==reference['requestSha256']
            if result['ok']:
                assert result['rivSha256']==reference['rivSha256'] and result['mapSha256']==reference['mapSha256']
            else: assert result['diagnostics']==reference['diagnostics']
        if not files:
            corpora.append({'label': label, 'requests': requests, 'files': 0, 'frames': 0, 'parityPass': 2 * requests})
            continue
        native = read(root / 'combined-receipt.json')
        assert native['browser'] == '153.0.8010.12' and not native['errors']
        bindings(native['bindings'])
        native_tools = {Path(b['path']).name: b for b in native['bindings']}
        for tool, digest in [('baseline-probe', '7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a'),
                             ('renderer-replay', '276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f')]:
            assert native_tools[tool]['sha256'] == digest
            bind(native_tools[tool]['path'], digest)
        bindings(native['sources'])
        bindings([native['compilerBuild']])
        frames = files * 8
        frame_keys = {(r['name'], r['frame']) for r in native['rows']}
        expected_keys = {(r['name'], frame) for r in emitted if r['compiled'] for frame in range(8)}
        assert len(native['rows']) == len(frame_keys) == frames and frame_keys == expected_keys
        assert sum(not r['metricFailures'] for r in native['rows']) == frames
        assert sum(not r['pixelFailures'] for r in native['rows']) == pixels
        assert all(set(r['imagePresence']) == {'image'} for r in native['rows'])
        assert all(r['imagePresence']['image']['passed'] for r in native['rows'])
        if donor_name:
            donor_root = OUTPUT / donor_name
            reviewed = visual(donor_root, donor_frames)
            prior = read(donor_root / 'combined-receipt.json')
            assert not prior['errors']
            prior_rows = {(r['name'], r['frame']): r for r in prior['rows']}
            assert len(prior_rows) == len(prior['rows']) == donor_frames and set(prior_rows) == reviewed
            matches = read(root / 'candidate-match-receipt.json')
            assert len(matches) == files and {r['name'] for r in matches} == set(indexed)
            for match in matches:
                assert all(match['identities'].values()) and Path(match['donor']) == donor_root / match['name']
                row = indexed[match['name']]
                for filename, key in [('request.json', 'requestSha256'), ('scene.riv', 'rivSha256'), ('scene.map.json', 'mapSha256')]:
                    bind(donor_root / row['name'] / filename, row[key])
            assert native['summary']['freshFrames'] == 0 and native['summary']['transferredFrames'] == frames
        else:
            assert label == 'prior' and visual(root, 16) == frame_keys
            assert native['summary']['freshFrames'] == 16 and native['summary']['transferredFrames'] == 0
        viewports = [(240, 240), (390, 320), (768, 560), (240, 240)]
        for row in native['rows']:
            key = row['name'], row['frame']
            assert (row['instance'], row['step']) == divmod(row['frame'], 4)
            assert (row['width'], row['height']) == viewports[row['step']]
            for hash_key in ['requestSha256', 'rivSha256', 'mapSha256']:
                assert row[hash_key] == indexed[row['name']][hash_key]
            for suffix, hash_key in [('.chrome.png', 'chromeSha256'), ('.native.png', 'nativeSha256')]:
                bind(row['prefix'] + suffix, row[hash_key])
            bind(Path(row['probeDirectory']) / row['geometry'], row['geometrySha256'])
            bind(Path(row['probeDirectory']) / row['stream'], row['streamSha256'])
            evidence = row['evidence']
            bind(evidence['receipt'], evidence['receiptSha256'])
            receipt = read(evidence['receipt'])
            driver = [b for b in receipt['bindings'] if Path(b['path']).name == 'public-image-native.mjs']
            assert len(driver) == 1
            bindings(driver)
            driver_source = Path(driver[0].get('snapshot', driver[0]['path'])).read_text()
            assert "'--backend', 'rust-metal', '--mode', 'clockwise-atomic'" in driver_source
            native_log = Path(row['prefix'] + '.native.log')
            bind(native_log)
            assert f"backend=rust-metal frame={row['frame']} size={row['width']}x{row['height']}" in native_log.read_text()
            bind(evidence['result'], evidence['resultSha256'])
            if row['pixelFailures']:
                pixel_failures.append({'case': row['name'], 'frame': row['frame'], 'failures': row['pixelFailures']})
            if donor_name:
                assert key in reviewed and measured(row) == measured(prior_rows[key])
                assert evidence['kind'] == 'exact-unchanged-transfer'
                transfers.append({'publicRoot': str(root), 'case': row['name'], 'frame': row['frame'],
                    'kind': 'exact-request-file-map-and-measured-row',
                    'sourceReview': str(donor_root / 'visual-r1/review-receipt.json'),
                    'sourceReviewSha256': bind(donor_root / 'visual-r1/review-receipt.json'),
                    'chromeSha256': row['chromeSha256'], 'nativeSha256': row['nativeSha256']})
        corpora.append({'label': label, 'requests': requests, 'files': files, 'frames': frames,
                        'geometryPass': frames, 'pixelPass': pixels, 'presencePass': frames, 'parityPass': 2 * requests})

    expected_pixel_failures = [
        {
            "case": "image-preferred-mixed-width-preferred-percent-auto-point-max-row",
            "frame": 1,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-width-preferred-percent-auto-point-max-row",
            "frame": 5,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-width-preferred-percent-auto-percent-min-row",
            "frame": 1,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-width-preferred-percent-auto-percent-min-row",
            "frame": 5,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-width-point-min-higher-percent-max-column",
            "frame": 1,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-width-point-min-higher-percent-max-column",
            "frame": 2,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-width-point-min-higher-percent-max-column",
            "frame": 5,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-width-point-min-higher-percent-max-column",
            "frame": 6,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-width-point-min-percent-max-row",
            "frame": 1,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-width-point-min-percent-max-row",
            "frame": 5,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-height-point-min-percent-max-row",
            "frame": 1,
            "failures": [
                "mismatch ratio",
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-height-point-min-percent-max-row",
            "frame": 5,
            "failures": [
                "mismatch ratio",
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-height-point-min-percent-max-column",
            "frame": 1,
            "failures": [
                "mismatch ratio"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-height-point-min-percent-max-column",
            "frame": 5,
            "failures": [
                "mismatch ratio"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-width-percent-min-point-max-row",
            "frame": 1,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-width-percent-min-point-max-row",
            "frame": 5,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-height-percent-min-point-max-row",
            "frame": 2,
            "failures": [
                "tail: local RGB error"
            ]
        },
        {
            "case": "image-preferred-mixed-combined-height-percent-min-point-max-row",
            "frame": 6,
            "failures": [
                "tail: local RGB error"
            ]
        }
    ]
    assert pixel_failures == expected_pixel_failures
    assert len(pixel_failures) == 18 and len(transfers) == 192
    assert len(prior_comparisons) == 20 and sum(r['compiled'] for r in prior_comparisons) == 2
    changed_diagnostics = {r['case'] for r in prior_comparisons if not r['compiled'] and not r['diagnosticsUnchanged']}
    assert changed_diagnostics == {'image-preferred-diagnostic-automatic-minimum', 'image-preferred-diagnostic-both-axes', 'image-combined-constraints-diagnostic-preferred-axis', 'image-responsive-constraints-diagnostic-automatic-minimum', 'image-preferred-diagnostic-padding', 'image-preferred-diagnostic-mixed-percent-point', 'image-preferred-diagnostic-cross-stretch'}
    for row in prior_comparisons:
        if row['case'] in changed_diagnostics:
            assert len(row['priorDiagnostics']) == len(row['currentDiagnostics']) == 1
            assert row['priorDiagnostics'][0]['code'] == row['currentDiagnostics'][0]['code']
            assert row['priorDiagnostics'][0]['source'] == row['currentDiagnostics'][0]['source']
            assert row['currentDiagnostics'][0]['message'] == ('Preferred image bounds require one percentage preferred axis, point bounds, point minimum with percentage maximum, or preferred percentages with independent opposite bounds, zero image padding, and ordinary sizing without automatic cross stretch; reverse mixed bounds and preferred point bounds with opposite constraints need separate qualification')
    out = OUTPUT / 'public-image-mixed-checkpoint-r1'
    out.mkdir(exist_ok=True)
    transfer_path = out / 'visual-transfer-review.json'
    transfer_path.write_text(json.dumps({'scope': '192 public frames transfer from completed private reviews. Sixteen newly admitted prior-case frames have an independent direct review; all 18 pixel-failure frames remain included.',
        'visualReviewTransferred': True, 'freshPublicSheetInspectionsClaimedByTransfer': 0,
        'exactPublicRequestFileMapAndMeasurementTransfers': 192, 'transfers': transfers}, indent=2) + '\n')
    bind(transfer_path)
    old = read(OUTPUT / 'public-image-mixed-existing-r1/results/manifest.json')
    assert old['total'] == old['passed'] == 256
    bind(old['compiler'], summary['compilerSha256'])
    bind(old['priorManifest'], old['priorManifestSha256'])
    for row in old['results']:
        assert row['exact']
        bind(row['request'], row['requestSha256'])
        for filename, key, prior_key in [('scene.riv', 'rivSha256', 'priorRiv'), ('scene.map.json', 'mapSha256', 'priorMap')]:
            bind(Path(row['result']) / filename, row[key])
            bind(row[prior_key], row[key])
    history_root = OUTPUT / 'public-transport-malformed-mixed-regression-r1'
    history = read(history_root / 'receipt.json')
    assert history['total'] == history['passed'] == 794 and history['allExact']
    assert history['inputHashes']['html-to-riv'] == summary['compilerSha256']
    for row in history['results']:
        assert row['exact']
        reference = row['reference']
        for filename, digest in reference['hashes'].items():
            bind(Path(reference['reference']) / filename, digest)
        for filename, digest in row['actualHashes'].items():
            assert digest == reference['hashes'][filename]
            bind(history_root / 'results' / str(row['index']) / filename, digest)
    bind(Path(__file__).resolve())
    return {'scope': 'Partial public mixed image constraints; 18 pixel-failure frames preserved, including four mismatch-ratio failures.',
            'build': summary, 'corpora': corpora, 'historicalOutputsExact': 794, 'existingImageFilesExact': 256,
            'visualTransferProof': str(transfer_path), 'pixelFailures': pixel_failures,
            'priorDiagnosticComparisons': prior_comparisons,
            'bindings': [{'path': path, 'sha256': digest} for path, digest in sorted(checked.items())]}


if __name__ == '__main__':
    result = verify()
    (OUTPUT / 'public-image-mixed-checkpoint-r1/verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'corpora': result['corpora'], 'artifactBindings': len(result['bindings'])}))
