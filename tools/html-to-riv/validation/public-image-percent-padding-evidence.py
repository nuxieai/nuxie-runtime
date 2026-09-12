"""Recheck final source/file identities and transfer completed visual reviews."""
import hashlib
import json
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1]

def verify():
    checked = {}
    def bind(path, expected=None):
        path = Path(path).resolve()
        h = hashlib.sha256(path.read_bytes()).hexdigest()
        if expected is not None:
            assert h == expected, f'Changed artifact: {path}'
        checked[str(path)] = h
        return h
    def read(path):
        bind(path)
        return json.loads(Path(path).read_text())
    def bindings(rows):
        for b in rows:
            bind(b['path'], b['sha256'])
    build = MODULE / 'output/public-image-percent-padding-build-r2'
    summary = read(build / 'summary.json')
    assert summary['status'] == 'public-build-and-transport-pass'
    assert (summary['rustTests'], summary['nodeTests']) == (322, 56)
    for row in read(build / 'frozen/source-bindings.json')['files']:
        bind(row['path'], row['sha256'])
        bind(row.get('snapshot', row['path']), row['sha256'])
    bind(build / 'frozen/html-to-riv', summary['compilerSha256'])
    bind(build / 'frozen/compiler.wasm', summary['wasmSha256'])
    counts = []
    for final, reviewed, parity, frames, pixels in [
        ('public-image-percent-padding-r3', 'public-image-percent-padding-r2', 'public-image-percent-padding-parity-r2', 200, 140),
        ('public-image-percent-padding-row-floor-r2', 'public-image-percent-padding-row-floor-r1', 'public-image-percent-padding-floor-parity-r2', 8, 6),
    ]:
        final, reviewed = MODULE / 'output' / final, MODULE / 'output' / reviewed
        emitted = read(final / 'compile-receipt.json')
        assert all(r['matchesExpectation'] for r in emitted)
        for row in emitted:
            folder = final / row['name']
            bind(folder / 'request.json', row['requestSha256'])
            if row['compiled']:
                for name, key in [('scene.riv', 'rivSha256'), ('scene.map.json', 'mapSha256')]:
                    bind(folder / name, row[key])
                    assert bind(reviewed / row['name'] / name) == row[key]
            else:
                assert not (folder / 'scene.riv').exists() and not (folder / 'scene.map.json').exists()
        native = read(final / 'combined-receipt.json')
        prior = read(reviewed / 'combined-receipt.json')
        assert not native['errors'] and not prior['errors']
        measured = lambda rows: [{k: v for k, v in row.items() if k != 'evidence'} for row in rows]
        assert measured(native['rows']) == measured(prior['rows'])
        assert len(native['rows']) == frames
        assert sum(not r['metricFailures'] for r in native['rows']) == frames
        assert sum(not r['pixelFailures'] for r in native['rows']) == pixels
        assert all(v['passed'] for r in native['rows'] for v in r['imagePresence'].values())
        for row in native['rows']:
            for suffix, key in [('.chrome.png', 'chromeSha256'), ('.native.png', 'nativeSha256')]:
                bind(row['prefix'] + suffix, row[key])
        review = read(reviewed / 'visual-r1/review-receipt.json')
        assert review['visualReviewCompleted']
        assert review['counts']['frames'] == frames and review['counts']['pixelPass'] == pixels
        bindings(review['bindings'])
        for sheet in review['inspectedSheets']:
            bind(sheet['path'], sheet['sha256'])
        coverage = read(reviewed / 'visual-r1/coverage.json')
        bindings(coverage.get('bindings', []))
        transport = read(MODULE / 'output' / parity / 'receipt.json')
        bindings(transport['bindings'])
        assert transport['cases'] == len(emitted)
        assert transport['observations'] == transport['passed'] == 2 * len(emitted)
        counts.append({'requests':len(emitted), 'frames':frames, 'geometryPass':frames, 'pixelPass':pixels,
                       'reviewTransferredByExactFilesAndMeasurements':True})
    old = read(MODULE / 'output/public-image-percent-padding-existing-r2/manifest.json')
    assert old['total'] == old['passed'] == 100
    for row in old['results']:
        assert row['exact']
        for name, key in [('scene.riv','rivSha256'), ('scene.map.json','mapSha256')]:
            bind(Path(row['result']) / name, row[key])
    history = read(MODULE / 'output/public-transport-malformed-percent-padding-regression-r1/receipt.json')
    assert history['total'] == history['passed'] == 794 and history['allExact']
    bind(Path(__file__).resolve())
    return {'scope':'Bounded partial public percentage-padding image support; 62 pixel failures retained',
            'build':summary, 'corpora':counts, 'historicalOutputsExact':794, 'existingImageFilesExact':100,
            'bindings':[{'path':p,'sha256':h} for p,h in sorted(checked.items())]}

if __name__ == '__main__':
    result = verify()
    out = MODULE / 'output/public-image-percent-padding-checkpoint-r1'
    out.mkdir(exist_ok=True)
    (out / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'corpora':result['corpora'],'artifactBindings':len(result['bindings'])}))
