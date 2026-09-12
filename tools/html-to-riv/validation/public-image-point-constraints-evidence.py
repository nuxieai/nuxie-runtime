"""Recheck bounded public point image constraints without recompiling or rendering.

The default-alignment controls transfer visual inspection by exact PNG pairs only;
their different requests and computed alignment are not claimed equivalent.
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

    def visual_review(root):
        review = read(root / 'visual-r1/review-receipt.json')
        assert review['visualReviewCompleted']
        bindings(review['bindings'])
        bindings(review['inspectedSheets'])
        coverage = read(root / 'visual-r1/coverage.json')
        bindings(coverage['bindings'])
        bindings(coverage['artifacts'])
        representatives = {(r['name'], r['frame']) for r in coverage['representatives']}
        transfers = {(r['case'], r['frame']) for r in coverage['transfers']}
        assert not representatives.intersection(transfers)
        assert len(representatives | transfers) == review['counts']['frames']
        inspected = {(r['case'], r['frame']) for sheet in coverage['sheets'] for r in sheet['members']}
        assert representatives <= inspected
        assert {(r['path'], r['sha256']) for r in review['inspectedSheets']} == {
            (r['path'], r['sha256']) for r in coverage['sheets']}
        return review, representatives | transfers

    build = OUTPUT / 'public-image-point-constraints-build-r1'
    summary = read(build / 'summary.json')
    assert summary['status'] == 'public-build-and-transport-pass' and summary['sourceUnchanged']
    assert (summary['rustTests'], summary['nodeTests']) == (335, 56)
    bindings(read(build / 'frozen/source-bindings.json')['files'], live=True)
    bind(build / 'frozen/html-to-riv', summary['compilerSha256'])
    bind(build / 'frozen/compiler.wasm', summary['wasmSha256'])

    private = OUTPUT / 'image-point-constraints-candidate-r1'
    private_review, reviewed_frames = visual_review(private)
    assert private_review['counts']['frames'] == 336
    prior = read(private / 'combined-receipt.json')
    assert not prior['errors']
    prior_rows = {(r['name'], r['frame']): r for r in prior['rows']}
    assert set(prior_rows) == reviewed_frames
    transfers = []
    corpora = []
    pixel_failures = []
    unsupported_presence = []
    for name, parity, requests, files, frames, pixels in [
        ('public-image-point-constraints-r2', 'public-image-point-constraints-parity-r1', 49, 44, 352, 336),
        ('public-image-point-constraints-floor-r1', 'public-image-point-constraints-floor-parity-r1', 2, 2, 16, 16),
    ]:
        root = OUTPUT / name
        bindings(read(root / 'authoring-bindings.json'), live=True)
        emitted = read(root / 'compile-receipt.json')
        assert len(emitted) == requests and sum(r['compiled'] for r in emitted) == files
        assert all(r['matchesExpectation'] for r in emitted)
        indexed = {r['name']: r for r in emitted}
        for row in emitted:
            folder = root / row['name']
            bind(folder / 'request.json', row['requestSha256'])
            if row['compiled']:
                bind(folder / 'scene.riv', row['rivSha256'])
                bind(folder / 'scene.map.json', row['mapSha256'])
            else:
                assert not (folder / 'scene.riv').exists() and not (folder / 'scene.map.json').exists()
        native = read(root / 'combined-receipt.json')
        assert native['browser'] == '153.0.8010.12' and not native['errors']
        bindings(native['bindings'])
        bindings(native['sources'])
        bindings([native['compilerBuild']])
        assert len(native['rows']) == frames
        frame_keys = {(r['name'], r['frame']) for r in native['rows']}
        expected_keys = {(r['name'], frame) for r in emitted if r['compiled'] for frame in range(8)}
        assert len(frame_keys) == frames and frame_keys == expected_keys
        viewport_sequence = [(240, 240), (390, 320), (768, 560), (240, 240)]
        for row in native['rows']:
            assert (row['instance'], row['step']) == divmod(row['frame'], 4)
            assert (row['width'], row['height']) == viewport_sequence[row['step']]
        assert sum(not r['metricFailures'] for r in native['rows']) == frames
        assert sum(not r['pixelFailures'] for r in native['rows']) == pixels
        assert sum(all(v['passed'] for v in r['imagePresence'].values()) for r in native['rows']) == pixels
        for row in native['rows']:
            emitted_row = indexed[row['name']]
            for key in ['requestSha256', 'rivSha256', 'mapSha256']:
                assert row[key] == emitted_row[key]
            for suffix, key in [('.chrome.png', 'chromeSha256'), ('.native.png', 'nativeSha256')]:
                bind(row['prefix'] + suffix, row[key])
            bind(Path(row['probeDirectory']) / row['geometry'], row['geometrySha256'])
            bind(Path(row['probeDirectory']) / row['stream'], row['streamSha256'])
            evidence = row['evidence']
            bind(evidence['receipt'], evidence['receiptSha256'])
            bind(evidence['result'], evidence['resultSha256'])
            if row['pixelFailures']:
                pixel_failures.append({'case': row['name'], 'frame': row['frame'], 'failures': row['pixelFailures']})
            for presence in row['imagePresence'].values():
                if not presence['passed']:
                    assert presence['failure'] == 'unsupported-control'
                    assert presence['unsupported'] == ['No fully contained visible pixels for this content expectation']
                    assert presence['outerBox']['width'] == presence['outerBox']['height'] == 0
                    unsupported_presence.append({'case': row['name'], 'frame': row['frame'], 'presence': presence})
            if name == 'public-image-point-constraints-r2':
                default = row['name'].endswith('-default-alignment')
                donor_name = row['name'].removesuffix('-default-alignment')
                donor = prior_rows[(donor_name, row['frame'])]
                assert (donor_name, row['frame']) in reviewed_frames
                assert (row['width'], row['height'], row['instance'], row['step']) == (
                    donor['width'], donor['height'], donor['instance'], donor['step'])
                for key, suffix in [('chromeSha256', '.chrome.png'), ('nativeSha256', '.native.png')]:
                    assert row[key] == donor[key]
                    bind(donor['prefix'] + suffix, donor[key])
                if default:
                    assert row['requestSha256'] != donor['requestSha256']
                    assert row['browserMetrics']['images']['image']['alignSelf'] != donor['browserMetrics']['images']['image']['alignSelf']
                    assert not row['metricFailures'] and not row['pixelFailures']
                else:
                    assert measured(row) == measured(donor)
                    for filename, key in [('request.json', 'requestSha256'), ('scene.riv', 'rivSha256'), ('scene.map.json', 'mapSha256')]:
                        bind(private / donor_name / filename, emitted_row[key])
                transfers.append({'case': row['name'], 'frame': row['frame'], 'donorCase': donor_name,
                                  'donorFrame': donor['frame'], 'kind': 'exact-PNG-pair-only' if default else 'exact-file-and-measured-row',
                                  'chromeSha256': row['chromeSha256'], 'nativeSha256': row['nativeSha256'],
                                  'metadataTransferred': not default})
        if name.endswith('floor-r1'):
            floor_review, floor_coverage = visual_review(root)
            assert floor_review['counts']['frames'] == 16
            assert floor_coverage == {(r['name'], r['frame']) for r in native['rows']}
        transport = read(OUTPUT / parity / 'receipt.json')
        bindings(transport['bindings'])
        assert transport['cases'] == requests and transport['observations'] == transport['passed'] == 2 * requests
        corpora.append({'requests': requests, 'files': files, 'frames': frames, 'geometryPass': frames,
                        'pixelPass': pixels, 'presencePass': pixels, 'parityPass': 2 * requests})

    assert len(pixel_failures) == len(unsupported_presence) == 16
    assert {r['case'] for r in pixel_failures} == {
        'image-point-constraints-padded-border-max-width-column',
        'image-point-constraints-padded-content-max-width-column'}
    assert {r['case'] for r in unsupported_presence} == {
        'image-point-constraints-intrinsic-zero-max-row', 'image-point-constraints-intrinsic-zero-max-column'}
    assert sum(r['kind'] == 'exact-PNG-pair-only' for r in transfers) == 16
    transfer_path = OUTPUT / 'public-image-point-constraints-r2/visual-r1/visual-transfer-review.json'
    transfer_result = {
        'scope': 'Completed private visual review transfers to 336 exact public files/measurements and 16 exact PNG pairs. The latter have distinct source requests and alignSelf metadata; their own native gates are independently retained.',
        'visualReviewTransferred': True, 'freshSheetInspectionsClaimed': 0,
        'sourceReview': {'path': str(private / 'visual-r1/review-receipt.json'), 'sha256': bind(private / 'visual-r1/review-receipt.json')},
        'frames': 352, 'exactFileAndMeasuredRowTransfers': 336, 'exactPngPairOnlyTransfers': 16,
        'transfers': transfers}
    transfer_path.write_text(json.dumps(transfer_result, indent=2) + '\n')
    bind(transfer_path)

    old = read(OUTPUT / 'public-image-point-constraints-existing-r1/results/manifest.json')
    assert old['total'] == old['passed'] == 126
    bind(old['compiler'], summary['compilerSha256'])
    bind(old['priorManifest'], old['priorManifestSha256'])
    for row in old['results']:
        assert row['exact']
        bind(row['request'], row['requestSha256'])
        for filename, key, prior_key in [('scene.riv', 'rivSha256', 'priorRiv'), ('scene.map.json', 'mapSha256', 'priorMap')]:
            bind(Path(row['result']) / filename, row[key])
            bind(row[prior_key], row[key])
    history_root = OUTPUT / 'public-transport-malformed-point-constraints-regression-r1'
    history = read(history_root / 'receipt.json')
    assert history['total'] == history['passed'] == 794 and history['allExact']
    for row in history['results']:
        assert row['exact']
        reference = row['reference']
        for filename, digest in reference['hashes'].items():
            bind(Path(reference['reference']) / filename, digest)
        for filename, digest in row['actualHashes'].items():
            assert digest == reference['hashes'][filename]
            bind(history_root / 'results' / str(row['index']) / filename, digest)
    bind(Path(__file__).resolve())
    return {'scope': 'Bounded partial public point image min/max support; retained 16 pixel failures and 16 unsampleable zero-area presence controls.',
            'build': summary, 'corpora': corpora, 'historicalOutputsExact': 794, 'existingImageFilesExact': 126,
            'visualTransferProof': str(transfer_path), 'pixelFailures': pixel_failures,
            'unsupportedPresenceControls': unsupported_presence,
            'bindings': [{'path': path, 'sha256': digest} for path, digest in sorted(checked.items())]}


if __name__ == '__main__':
    result = verify()
    out = OUTPUT / 'public-image-point-constraints-checkpoint-r1'
    out.mkdir(exist_ok=True)
    (out / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'corpora': result['corpora'], 'artifactBindings': len(result['bindings'])}))
