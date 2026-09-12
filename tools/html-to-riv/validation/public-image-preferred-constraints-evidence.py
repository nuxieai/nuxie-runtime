"""Verify preferred-axis image constraint evidence without compilation/rendering.

Successful private recipes transfer by exact requests, ordinary files, maps and
measured rows. Failed wrappers are preserved separately, never admitted by count.
"""
import hashlib
import json
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

    build = OUTPUT / 'public-image-preferred-constraints-build-r1'
    summary = read(build / 'summary.json')
    assert summary['status'] == 'public-build-and-transport-pass' and summary['sourceUnchanged']
    assert (summary['rustTests'], summary['nodeTests']) == (354, 56)
    checks = read(build / 'checks.json')
    assert len(checks) == 6 and {c['name'] for c in checks} == {
        'full-rust', 'native-build', 'wasm-build', 'typescript', 'full-node', 'runtime-guard'}
    for check in checks:
        assert check['exitCode'] == 0
        bind(check['log'], check['logSha256'])
    source_bindings = read(build / 'frozen/source-bindings.json')['files']
    assert len(source_bindings) == 277
    bindings(source_bindings, live=True)
    bind(build / 'frozen/html-to-riv', summary['compilerSha256'])
    bind(build / 'frozen/compiler.wasm', summary['wasmSha256'])
    corpora, transfers, pixel_failures, wrapper_controls, prior_comparisons = [], [], [], [], []
    prior_index = {}
    for name in ['public-image-point-constraints-r2', 'public-image-combined-constraints-prior-r1',
                 'public-image-combined-constraints-r1']:
        root = OUTPUT / name
        for row in read(root / 'compile-receipt.json'):
            prior_index[row['name']] = root, row

    for label, donor_name, requests, files, pixels, donor_frames in [
        ('points', 'image-preferred-constraints-followup-r1', 8, 8, 50, 96),
        ('coefficients', 'image-preferred-constraints-coefficient-r1', 8, 8, 60, 64),
        ('boundary', 'image-preferred-constraints-boundary-r1', 16, 16, 104, 128),
        ('diagnostic', None, 6, 0, 0, 0),
        ('prior', None, 15, 1, 8, 0),
    ]:
        root = OUTPUT / f'public-image-preferred-constraints-{label}-r1'
        # Matrices added after the build have separate live/snapshot authoring bindings.
        bindings(read(root / 'authoring-bindings.json'), live=True)
        assert read(root / 'frozen/source-bindings.json')['files'] == source_bindings
        bind(root / 'frozen/html-to-riv', summary['compilerSha256'])
        emitted = read(root / 'compile-receipt.json')
        assert len(emitted) == len({r['name'] for r in emitted}) == requests
        assert sum(r['compiled'] for r in emitted) == files and all(r['matchesExpectation'] for r in emitted)
        indexed = {r['name']: r for r in emitted}
        for row in emitted:
            folder = root / row['name']
            bind(folder / 'request.json', row['requestSha256'])
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
                assert row['compiled'] == (row['name'] == 'image-point-constraints-reject-percentage-dimension')
                prior_comparisons.append({'case': row['name'], 'priorRoot': str(old_root),
                    'requestSha256': row['requestSha256'], 'wasCompiled': False, 'compiled': row['compiled'],
                    'priorDiagnostics': old['diagnostics'], 'currentDiagnostics': row['diagnostics'],
                    'diagnosticsUnchanged': old['diagnostics'] == row['diagnostics']})
        transport = read(OUTPUT / f'public-image-preferred-constraints-{label}-parity-r1/receipt.json')
        bindings(transport['bindings'])
        assert transport['cases'] == requests and transport['observations'] == transport['passed'] == 2 * requests
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
            assert label == 'prior' and visual(root, 8) == frame_keys
            assert native['summary']['freshFrames'] == 8 and native['summary']['transferredFrames'] == 0
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
                assert row['pixelFailures'] == ['tail: local RGB error']
                pixel_failures.append({'case': row['name'], 'frame': row['frame'], 'failures': row['pixelFailures']})
            if donor_name:
                assert key in reviewed and measured(row) == measured(prior_rows[key])
                assert evidence['kind'] == 'exact-unchanged-transfer'
                transfers.append({'publicRoot': str(root), 'case': row['name'], 'frame': row['frame'],
                    'kind': 'exact-request-file-map-and-measured-row',
                    'sourceReview': str(donor_root / 'visual-r1/review-receipt.json'),
                    'sourceReviewSha256': bind(donor_root / 'visual-r1/review-receipt.json'),
                    'chromeSha256': row['chromeSha256'], 'nativeSha256': row['nativeSha256']})
        if label == 'points':
            controls = [r for r in prior['rows'] if (r['name'], r['frame']) not in frame_keys]
            assert len(controls) == 32 and all('inner-ratio' in r['name'] for r in controls)
            assert sum(not r['metricFailures'] for r in controls) == 16
            assert sum(not r['pixelFailures'] for r in controls) == 14
            wrapper_controls = [{'case': r['name'], 'frame': r['frame'], 'metricFailures': r['metricFailures'],
                                 'pixelFailures': r['pixelFailures']} for r in controls]
        corpora.append({'label': label, 'requests': requests, 'files': files, 'frames': frames,
                        'geometryPass': frames, 'pixelPass': pixels, 'presencePass': frames, 'parityPass': 2 * requests})

    assert len(pixel_failures) == 42 and len(transfers) == 256
    assert len(prior_comparisons) == 15 and sum(r['compiled'] for r in prior_comparisons) == 1
    changed_diagnostics = {r['case'] for r in prior_comparisons if not r['compiled'] and not r['diagnosticsUnchanged']}
    assert changed_diagnostics == {'image-responsive-constraints-diagnostic-automatic-minimum',
        'image-responsive-constraints-diagnostic-preferred-axis-maximum', 'image-combined-constraints-diagnostic-preferred-axis'}
    for row in prior_comparisons:
        if row['case'] in changed_diagnostics:
            assert len(row['priorDiagnostics']) == len(row['currentDiagnostics']) == 1
            assert row['priorDiagnostics'][0]['code'] == row['currentDiagnostics'][0]['code']
            assert row['priorDiagnostics'][0]['source'] == row['currentDiagnostics'][0]['source']
            assert row['currentDiagnostics'][0]['message'] == ('Preferred image bounds require one percentage preferred axis, homogeneous point or percentage bounds on that axis only, zero image padding, and ordinary sizing without automatic cross stretch; mixed-unit or both-axis constraints need separate qualification')
    out = OUTPUT / 'public-image-preferred-constraints-checkpoint-r1'
    out.mkdir(exist_ok=True)
    transfer_path = out / 'visual-transfer-review.json'
    transfer_path.write_text(json.dumps({'scope': '256 exact intended public frames transfer from completed private reviews; 32 inner-wrapper control frames are excluded. Eight newly admitted prior-case frames have an independent direct review.',
        'visualReviewTransferred': True, 'freshPublicSheetInspectionsClaimedByTransfer': 0,
        'exactPublicRequestFileMapAndMeasurementTransfers': 256, 'transfers': transfers}, indent=2) + '\n')
    bind(transfer_path)
    old = read(OUTPUT / 'public-image-preferred-constraints-existing-r1/results/manifest.json')
    assert old['total'] == old['passed'] == 223
    bind(old['compiler'], summary['compilerSha256'])
    bind(old['priorManifest'], old['priorManifestSha256'])
    for row in old['results']:
        assert row['exact']
        bind(row['request'], row['requestSha256'])
        for filename, key, prior_key in [('scene.riv', 'rivSha256', 'priorRiv'), ('scene.map.json', 'mapSha256', 'priorMap')]:
            bind(Path(row['result']) / filename, row[key])
            bind(row[prior_key], row[key])
    history_root = OUTPUT / 'public-transport-malformed-preferred-constraints-regression-r1'
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
    return {'scope': 'Partial public preferred-axis image constraints; 42 following-tail pixel failures preserved. Private inner-wrapper controls remain separate.',
            'build': summary, 'corpora': corpora, 'historicalOutputsExact': 794, 'existingImageFilesExact': 223,
            'visualTransferProof': str(transfer_path), 'pixelFailures': pixel_failures,
            'privateWrapperControls': wrapper_controls, 'priorDiagnosticComparisons': prior_comparisons,
            'bindings': [{'path': path, 'sha256': digest} for path, digest in sorted(checked.items())]}


if __name__ == '__main__':
    result = verify()
    (OUTPUT / 'public-image-preferred-constraints-checkpoint-r1/verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'corpora': result['corpora'], 'artifactBindings': len(result['bindings'])}))
