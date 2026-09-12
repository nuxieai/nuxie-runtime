#!/usr/bin/env python3
"""Verify saved independent Chrome observations; never renders or admits CSS."""
import hashlib
import json
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1]
ROOT = MODULE / 'output/image-percent-padding-chrome-r1'
NATIVE = MODULE / 'output/image-percent-padding-candidate-r2/native-receipt.json'


def read(path):
    return json.loads(path.read_text())


def verify():
    checked = {}

    def bind(path, expected=None):
        path = Path(path)
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if expected is not None:
            assert digest == expected, f'Changed artifact: {path}'
        checked[str(path)] = digest
        return digest

    receipt = read(ROOT / 'receipt.json')
    sources = read(ROOT / 'source-receipt.json')
    native = read(NATIVE)
    for path in [Path(__file__).resolve(), ROOT / 'receipt.json', NATIVE]:
        bind(path)
    for b in receipt['bindings']:
        bind(b['path'], b['sha256'])
    for b in sources['bindings']:
        bind(b['snapshot'], b['sha256'])
        assert b['snapshotSha256'] == b['sha256']
    for b in sources['requests']:
        bind(b['path'], b['sha256'])
    cases = read(ROOT / 'inputs/validation/image-percent-padding-cases.json')
    assert receipt['browser'] == native['browser'] == '153.0.8010.12'
    assert receipt['errors'] == native['errors'] == []
    assert len(cases) == receipt['cases'] == 26
    rows = {(r['name'], r['frame']): r for r in receipt['rows']}
    assert len(rows) == len(receipt['rows']) == receipt['frames'] == 208
    sequence = [(240, 240), (390, 320), (768, 560), (240, 240)]
    for case in cases:
        for frame in range(8):
            r = rows[case['name'], frame]
            assert (r['width'], r['height']) == sequence[frame % 4]
            assert r['step'] == frame % 4
            assert r['mode'] == ('original' if frame < 4 else 'dom-clone')
            for key in ['request', 'reference', 'png']:
                bind(r[key]['path'], r[key]['sha256'])
            metric = ROOT / case['name'] / f'frame-{frame}.metrics.json'
            bind(metric)
            assert read(metric) == r
            for name in case['observeIds']:
                box = r['metrics']['rectangles'][name]
                observed = r['metrics']['resizeObserver'][name]['borderBox']
                assert all(box[k] == observed[k] for k in ['width', 'height'])
    expected_repeats = {(c['name'], a, b) for c in cases
                        for a, b in [(0, 3), (0, 4), (1, 5), (2, 6), (0, 7)]}
    repeats = receipt['exactRepeats']
    assert len(repeats) == 130
    assert {(r['name'], r['sourceFrame'], r['targetFrame']) for r in repeats} == expected_repeats
    for repeat in repeats:
        a = rows[repeat['name'], repeat['sourceFrame']]
        b = rows[repeat['name'], repeat['targetFrame']]
        assert a['metrics'] == b['metrics']
        assert a['png']['sha256'] == b['png']['sha256']
        assert repeat['metricsEqual'] and repeat['pngBytesEqual']
        assert repeat['sourcePng'] == a['png'] and repeat['targetPng'] == b['png']
    assert len(native['rows']) == 192
    assert len({(r['name'], r['frame']) for r in native['rows']}) == 192
    for r in native['rows']:
        independent = rows[r['name'], r['frame']]
        assert r['browserMetrics']['rectangles'] == independent['metrics']['rectangles']
        assert r['requestSha256'] == independent['request']['sha256']
        bind(r['prefix'] + '.chrome.png', r['chromeSha256'])
        assert r['chromeSha256'] == independent['png']['sha256']
        for name, image in r['browserMetrics']['images'].items():
            observed = independent['metrics']['resizeObserver'][name]['contentBox']
            assert all(image['contentBox'][k] == observed[k] for k in ['width', 'height'])
    return {'scope': 'Saved Chrome evidence verification only; no native qualification or public admission',
            'independentFrames': 208, 'exactRepeats': 130,
            'nativeCampaignChromeReferencesExactlyMatched': 192,
            'bindings': [{'path': p, 'sha256': h} for p, h in sorted(checked.items())]}


if __name__ == '__main__':
    result = verify()
    target = ROOT / 'verification.json'
    target.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: len(v) if k == 'bindings' else v for k, v in result.items()}))
