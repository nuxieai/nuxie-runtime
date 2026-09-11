"""Focused regression for fractional rounded content clips, beyond area metrics.
Usage: python3 clip-edge-samples.py EDGE_LIFECYCLE_REPLAY_DIRECTORY
The fixture has a constant left content clip edge through all viewport sizes.
"""
import json
import sys
from pathlib import Path
from PIL import Image

folder = Path(sys.argv[1])
replay = json.loads((folder / 'replay.json').read_text())
name = 'elliptical-edge-ellipse-content-margin-border-box-'
rows = [row for row in replay['cases'] if row['name'].startswith(name)]
assert len(rows) == 8, 'Expected original/clone four-frame lifecycle'
failures = []
for row in rows:
    browser = Image.open(row['prefix'] + '.browser.png').convert('RGB')
    native = Image.open(row['prefix'] + '.native.png').convert('RGB')
    # This is partial coverage between orange (234,164,61) and teal (48,141,165).
    # Guard the reference so moving/removing the clip cannot make this vacuous.
    expected = browser.getpixel((29, 80))
    assert 60 < expected[0] < 220, 'Reference no longer exercises fractional coverage'
    for point in [(28,80), (29,80), (30,80)]:
        expected = browser.getpixel(point)
        actual = native.getpixel(point)
        if max(abs(a-b) for a,b in zip(expected,actual)) > 2:
            failures.append(dict(name=row['name'],point=point,browser=expected,native=actual))
print(json.dumps(dict(frames=len(rows),failures=failures)))
sys.exit(bool(failures))
