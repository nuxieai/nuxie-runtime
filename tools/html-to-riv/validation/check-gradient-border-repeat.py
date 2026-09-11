"""Local CSS background-repeat border gate; aggregate tolerances cannot waive it.
Usage: SCRIPT REPLAY_DIRECTORY NEW_RECEIPT [--browser-self]
Uses pinned Chrome used border widths and reference geometry, not native bounds.
"""
import hashlib
import json
import math
from pathlib import Path
import sys
from PIL import Image

replay, output = (Path(arg).resolve() for arg in sys.argv[1:3])
self_control = sys.argv[3:] == ['--browser-self']
assert not sys.argv[3:] or self_control, 'Unknown argument'
assert not output.exists(), 'Refusing to overwrite evidence'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
report = json.loads((replay / 'replay.json').read_text())
oracle_path = Path(report['oracle'])
assert sha(oracle_path) == report['oracleSha256']
oracle = json.loads(oracle_path.read_text())
assert oracle['browser'] == '153.0.8010.12'
used_path = oracle_path.parent / 'fractional-border-used-values.json'
used = json.loads(used_path.read_text())
assert used['browser'] == oracle['browser']
name_prefix = 'linear-gradient-composition-'
rows = []
for kind in ['fractional-border', 'opaque-border']:
    fixture = next(c for c in oracle['cases'] if c['name'] == name_prefix + kind)
    if kind == 'fractional-border':
        assert (fixture['html'], fixture['css']) == (used['html'], used['css'])
    for width in [240, 390, 768]:
        matches = [c for c in report['cases'] if c['name'] == fixture['name'] and c['width'] == width]
        assert len(matches) == 1
        row = matches[0]
        assert row['html'] == fixture['html'] and row['css'] == fixture['css']
        box = next(v['boxes']['gradient'] for v in fixture['viewports'] if v['width'] == width)
        if kind == 'fractional-border':
            measured = next(v for v in used['rows'] if v['viewportWidth'] == width)
            assert measured['borderLeftWidth'] == measured['borderRightWidth'] == '1px'
            for axis in ['x', 'y', 'width', 'height']:
                assert measured[axis] == box[axis]
            # The single used border pixel is at the snapped outer edge, with
            # its center in the border strip. Samples avoid horizontal corners.
            xs = [math.floor(box['x'] + .5), math.floor(box['x'] + box['width'] + .5) - 1]
        else:
            assert 'border:12px solid black' in fixture['css']
            xs = [math.floor(box['x'] + 6), math.floor(box['x'] + box['width'] - 6)]
        images = []
        for label in ['browser', 'native']:
            p = Path(row['prefix'] + '.' + label + '.png')
            assert sha(p) == row[label + 'Sha256']
            images.append(Image.open(p).convert('RGBA'))
        browser, native = images
        assert browser.size == native.size == (width, 320)
        samples = []
        for side, x in zip(['left', 'right'], xs):
            for fraction in [.25, .5, .75]:
                y = math.floor(box['y'] + box['height'] * fraction)
                a = browser.getpixel((x, y))
                b = (browser if self_control else native).getpixel((x, y))
                error = max(abs(u - v) for u, v in zip(a, b))
                samples.append(dict(side=side, x=x, y=y, browser=a, actual=b,
                                    maxChannelError=error, passed=error <= 2))
        rows.append(dict(name=row['name'], width=width, samples=samples,
                         browserSha256=row['browserSha256'], nativeSha256=row['nativeSha256'],
                         passed=all(sample['passed'] for sample in samples)))
failed = sum(not row['passed'] for row in rows)
receipt = dict(status='failed' if failed else 'passed', mode='browser-self-control' if self_control else 'native-border-repeat',
               frames=len(rows), samples=sum(len(row['samples']) for row in rows), failed=failed,
               tolerance='Existing hard-stop local gate: max RGBA channel error <= 2; aggregate gates unchanged.',
               replaySha256=sha(replay / 'replay.json'), oracleSha256=sha(oracle_path),
               usedBorderEvidenceSha256=sha(used_path), usedBorderEvidence=str(used_path), rows=rows)
output.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({key:receipt[key] for key in ['status', 'mode', 'frames', 'samples', 'failed']}))
sys.exit(1 if failed else 0)
