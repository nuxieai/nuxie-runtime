"""Join changed public image renders with verified unchanged native/Chrome evidence."""
import hashlib
import json
from pathlib import Path
import sys

root, previous_path = map(lambda p: Path(p).resolve(), sys.argv[1:3])
fresh_path = root / 'native-receipt.json'
output = root / 'combined-receipt.json'
assert not output.exists()


def read(p):
    return json.loads(p.read_text())


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


fresh, previous = read(fresh_path), read(previous_path)
assert fresh['browser'] == previous['browser'] == '153.0.8010.12'
assert not fresh['errors'] and not previous['errors']
bindings = []
for name in ['baseline-probe', 'renderer-replay', 'reset.css']:
    a = [b for b in fresh['bindings'] if Path(b['path']).name == name]
    b = [b for b in previous['bindings'] if Path(b['path']).name == name]
    assert len(a) == len(b) == 1 and a[0]['sha256'] == b[0]['sha256'], name
    assert sha(Path(a[0]['path'])) == a[0]['sha256']
    assert sha(Path(b[0]['path'])) == b[0]['sha256']
    bindings.append(a[0])
reset = (root / 'frozen/inputs/src/reset.css').read_text()
compiled = read(root / 'compile-receipt.json')
rows, transfers = [], []
for case in (c for c in compiled if c['compiled']):
    name = case['name']
    request = read(root / name / 'request.json')
    assert sha(root / name / 'scene.riv') == case['rivSha256']
    assert sha(root / name / 'scene.map.json') == case['mapSha256']
    current = [r for r in fresh['rows'] if r['name'] == name]
    transferred = not current
    source_receipt = previous_path if transferred else fresh_path
    source = previous if transferred else fresh
    chosen = [r for r in source['rows'] if r['name'] == name]
    assert len(chosen) == 8, name
    assert [(r['instance'], r['step'], r['width'], r['height']) for r in chosen] == [
        (instance, step, w, h) for instance in [0, 1]
        for step, (w, h) in enumerate([(240, 240), (390, 320), (768, 560), (240, 240)])]
    for row in chosen:
        prefix = Path(row['prefix'])
        old_dir = prefix.parent.parent
        assert read(old_dir / 'request.json') == request, name
        assert row['rivSha256'] == case['rivSha256'] == sha(old_dir / 'scene.riv')
        assert row['mapSha256'] == case['mapSha256'] == sha(old_dir / 'scene.map.json')
        assert row['requestSha256'] == sha(old_dir / 'request.json')
        # This also rejects the original contaminated r1 references whose base
        # lacked the per-run/per-case namespace required by the corrected driver.
        base = f"http://html-to-riv.invalid/{source['outputLabel']}/{name}/"
        expected_html = f'<!doctype html><meta charset="utf-8"><base href="{base}"><style>{reset}\n{request["css"]}</style>{request["html"]}'
        html = prefix.parent / 'reference.html'
        assert html.read_text() == expected_html and sha(html) == row['htmlSha256']
        observed = Path(row['probeDirectory'])
        assert sha(observed / 'scene.riv') == case['rivSha256']
        for key, path in [('chromeSha256', Path(str(prefix) + '.chrome.png')),
                          ('nativeSha256', Path(str(prefix) + '.native.png')),
                          ('geometrySha256', observed / row['geometry']),
                          ('streamSha256', observed / row['stream'])]:
            assert sha(path) == row[key], (name, key)
        result_path = Path(str(prefix) + '.result.json')
        assert read(result_path) == row
        joined = dict(row, evidence=dict(kind='exact-unchanged-transfer' if transferred else 'fresh-render',
                     receipt=str(source_receipt), receiptSha256=sha(source_receipt),
                     result=str(result_path), resultSha256=sha(result_path)))
        rows.append(joined)
    if transferred:
        transfers.append(name)
counts = [dict(name=case['name'], frames=len(r := [r for r in rows if r['name'] == case['name']]),
               geometryPass=sum(not x['metricFailures'] for x in r), pixelPass=sum(not x['pixelFailures'] for x in r),
               presencePass=sum(all(v['passed'] for v in x['imagePresence'].values()) for x in r))
          for case in compiled if case['compiled']]
summary = dict(frames=len(rows), freshFrames=sum(r['evidence']['kind'] == 'fresh-render' for r in rows),
               transferredFrames=sum(r['evidence']['kind'] == 'exact-unchanged-transfer' for r in rows),
               geometryPass=sum(c['geometryPass'] for c in counts), pixelPass=sum(c['pixelPass'] for c in counts),
               presencePass=sum(c['presencePass'] for c in counts))
receipt = dict(scope='Public image candidate evidence join; full public integration and visual qualification separate',
               browser=fresh['browser'], bindings=bindings, compilerBuild=dict(path=str(root / 'build-receipt.json'),
               sha256=sha(root / 'build-receipt.json')), sources=[dict(path=str(p), sha256=sha(p)) for p in
               [fresh_path, previous_path, root / 'compile-receipt.json', Path(__file__).resolve()]],
               rows=rows, counts=counts, summary=summary, transferredCases=transfers, errors=[])
output.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(summary))
