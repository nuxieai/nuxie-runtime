"""Verify public responsive image constraints and exact private visual transfers.

No compilation or rendering occurs. Only the intended private recipes transfer;
unguarded control failures remain distinct evidence against omitting the guard.
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
        return {k: v for k, v in row.items() if k != 'evidence'}

    build = OUTPUT / 'public-image-responsive-constraints-build-r1'
    summary = read(build / 'summary.json')
    assert summary['status'] == 'public-build-and-transport-pass' and summary['sourceUnchanged']
    assert (summary['rustTests'], summary['nodeTests']) == (343, 56)
    source_bindings = read(build / 'frozen/source-bindings.json')['files']
    assert len(source_bindings) == 269
    bindings(source_bindings, live=True)
    bind(build / 'frozen/html-to-riv', summary['compilerSha256'])
    bind(build / 'frozen/compiler.wasm', summary['wasmSha256'])
    corpora, transfers, pixel_failures, negative_controls = [], [], [], []
    for public, private, parity, requests, pixels, reviewed_count in [
        ('public-image-responsive-constraints-r1', 'image-responsive-constraints-candidate-r1',
         'public-image-responsive-constraints-parity-r1', 19, 96, 160),
        ('public-image-responsive-constraints-parent-padding-r1', 'image-responsive-constraints-parent-padding-r1',
         'public-image-responsive-constraints-parent-padding-parity-r1', 12, 84, 96),
    ]:
        root, donor_root = OUTPUT / public, OUTPUT / private
        bindings(read(root / 'authoring-bindings.json'), live=True)
        # The public frozen inputs are the same full build, not a native-only candidate snapshot.
        assert read(root / 'frozen/source-bindings.json')['files'] == source_bindings
        bind(root / 'frozen/html-to-riv', summary['compilerSha256'])
        emitted = read(root / 'compile-receipt.json')
        assert len(emitted) == requests and len({r['name'] for r in emitted}) == requests
        assert sum(r['compiled'] for r in emitted) == 12 and all(r['matchesExpectation'] for r in emitted)
        indexed = {r['name']: r for r in emitted}
        for row in emitted:
            folder = root / row['name']
            bind(folder / 'request.json', row['requestSha256'])
            if row['compiled']:
                for filename, key in [('request.json', 'requestSha256'), ('scene.riv', 'rivSha256'), ('scene.map.json', 'mapSha256')]:
                    bind(folder / filename, row[key])
                    bind(donor_root / row['name'] / filename, row[key])
            else:
                assert not (folder / 'scene.riv').exists() and not (folder / 'scene.map.json').exists()

        review = read(donor_root / 'visual-r1/review-receipt.json')
        assert review['visualReviewCompleted'] and review['counts']['frames'] == reviewed_count
        bindings(review['bindings'])
        bindings(review['inspectedSheets'])
        coverage = read(donor_root / 'visual-r1/coverage.json')
        bindings(coverage['bindings'])
        bindings(coverage['artifacts'])
        representatives = {(r['name'], r['frame']) for r in coverage['representatives']}
        visual_transfers = {(r['case'], r['frame']) for r in coverage['transfers']}
        assert not representatives.intersection(visual_transfers)
        reviewed = representatives | visual_transfers
        assert len(reviewed) == reviewed_count
        inspected = {(r['case'], r['frame']) for sheet in coverage['sheets'] for r in sheet['members']}
        assert representatives <= inspected
        assert {(r['path'], r['sha256']) for r in review['inspectedSheets']} == {
            (r['path'], r['sha256']) for r in coverage['sheets']}

        prior = read(donor_root / 'combined-receipt.json')
        assert not prior['errors']
        prior_rows = {(r['name'], r['frame']): r for r in prior['rows']}
        assert len(prior_rows) == len(prior['rows']) == reviewed_count and set(prior_rows) == reviewed
        native = read(root / 'combined-receipt.json')
        assert native['browser'] == '153.0.8010.12' and not native['errors']
        bindings(native['bindings'])
        bindings(native['sources'])
        bindings([native['compilerBuild']])
        assert native['summary']['freshFrames'] == 0 and native['summary']['transferredFrames'] == 96
        frame_keys = {(r['name'], r['frame']) for r in native['rows']}
        expected_keys = {(r['name'], frame) for r in emitted if r['compiled'] for frame in range(8)}
        assert len(native['rows']) == len(frame_keys) == 96 and frame_keys == expected_keys
        assert sum(not r['metricFailures'] for r in native['rows']) == 96
        assert sum(not r['pixelFailures'] for r in native['rows']) == pixels
        assert all(v['passed'] for r in native['rows'] for v in r['imagePresence'].values())
        viewports = [(240, 240), (390, 320), (768, 560), (240, 240)]
        for row in native['rows']:
            key = row['name'], row['frame']
            assert key in reviewed and measured(row) == measured(prior_rows[key])
            assert (row['instance'], row['step']) == divmod(row['frame'], 4)
            assert (row['width'], row['height']) == viewports[row['step']]
            for hash_key in ['requestSha256', 'rivSha256', 'mapSha256']:
                assert row[hash_key] == indexed[row['name']][hash_key]
            for suffix, hash_key in [('.chrome.png', 'chromeSha256'), ('.native.png', 'nativeSha256')]:
                bind(row['prefix'] + suffix, row[hash_key])
            bind(Path(row['probeDirectory']) / row['geometry'], row['geometrySha256'])
            bind(Path(row['probeDirectory']) / row['stream'], row['streamSha256'])
            evidence = row['evidence']
            assert evidence['kind'] == 'exact-unchanged-transfer'
            bind(evidence['receipt'], evidence['receiptSha256'])
            bind(evidence['result'], evidence['resultSha256'])
            if row['pixelFailures']:
                assert row['pixelFailures'] == ['tail: local RGB error']
                pixel_failures.append({'case': row['name'], 'frame': row['frame'], 'failures': row['pixelFailures']})
            transfers.append({'publicRoot': str(root), 'case': row['name'], 'frame': row['frame'],
                              'kind': 'exact-request-file-map-and-measured-row',
                              'sourceReview': str(donor_root / 'visual-r1/review-receipt.json'),
                              'sourceReviewSha256': bind(donor_root / 'visual-r1/review-receipt.json'),
                              'chromeSha256': row['chromeSha256'], 'nativeSha256': row['nativeSha256']})
        if reviewed_count == 160:
            controls = [r for r in prior['rows'] if (r['name'], r['frame']) not in frame_keys]
            assert len(controls) == 64
            assert sum(not r['metricFailures'] for r in controls) == 28
            assert sum(not r['pixelFailures'] for r in controls) == 28
            negative_controls = [{'case': r['name'], 'frame': r['frame'], 'metricFailures': r['metricFailures'],
                                  'pixelFailures': r['pixelFailures']} for r in controls]
        transport = read(OUTPUT / parity / 'receipt.json')
        bindings(transport['bindings'])
        assert transport['cases'] == requests and transport['observations'] == transport['passed'] == 2 * requests
        corpora.append({'requests': requests, 'files': 12, 'frames': 96, 'geometryPass': 96,
                        'pixelPass': pixels, 'presencePass': 96, 'parityPass': 2 * requests})

    assert len(pixel_failures) == 12
    assert {r['case'] for r in pixel_failures} == {
        'image-responsive-constraints-parent-padding-width-max-height-point-column-guard',
        'image-responsive-constraints-parent-padding-width-max-height-percent-column-guard',
        'image-responsive-constraints-parent-padding-width-min-height-point-column-direct'}
    out = OUTPUT / 'public-image-responsive-constraints-checkpoint-r1'
    out.mkdir(exist_ok=True)
    transfer_path = out / 'visual-transfer-review.json'
    transfer_path.write_text(json.dumps({
        'scope': 'Only the 192 intended public frames transfer from completed private visual reviews; 64 unguarded control frames do not transfer to public admission.',
        'visualReviewTransferred': True, 'freshSheetInspectionsClaimed': 0,
        'exactPublicRequestFileMapAndMeasurementTransfers': 192, 'transfers': transfers}, indent=2) + '\n')
    bind(transfer_path)
    old = read(OUTPUT / 'public-image-responsive-constraints-existing-r1/results/manifest.json')
    assert old['total'] == old['passed'] == 172
    bind(old['compiler'], summary['compilerSha256'])
    bind(old['priorManifest'], old['priorManifestSha256'])
    for row in old['results']:
        assert row['exact']
        bind(row['request'], row['requestSha256'])
        for filename, key, prior_key in [('scene.riv', 'rivSha256', 'priorRiv'), ('scene.map.json', 'mapSha256', 'priorMap')]:
            bind(Path(row['result']) / filename, row[key])
            bind(row[prior_key], row[key])
    history_root = OUTPUT / 'public-transport-malformed-responsive-constraints-regression-r1'
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
    return {'scope': 'Partial public responsive image constraints; 12 following-tail pixel failures retained. Separate private unguarded controls retain 36 geometry/pixel failures.',
            'build': summary, 'corpora': corpora, 'historicalOutputsExact': 794, 'existingImageFilesExact': 172,
            'visualTransferProof': str(transfer_path), 'pixelFailures': pixel_failures,
            'privateUnguardedControls': negative_controls,
            'bindings': [{'path': path, 'sha256': digest} for path, digest in sorted(checked.items())]}


if __name__ == '__main__':
    result = verify()
    out = OUTPUT / 'public-image-responsive-constraints-checkpoint-r1'
    (out / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'corpora': result['corpora'], 'artifactBindings': len(result['bindings'])}))
