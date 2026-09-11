"""Audit axis discrimination and restored paint in direct renderer controls.

Usage: python3 audit-axis-renderer.py REPLAY_DIRECTORY
This verifies control strength, not visual review or compiler qualification.
"""
import hashlib
import itertools
import json
import sys
from pathlib import Path

from PIL import Image

folder = Path(sys.argv[1])
replay = folder / 'replay.json'
data = json.loads(replay.read_text())
assert data['profile'] == 'direct-axis-renderer-diagnostic'
cases = data['cases']
assert len(cases) == 144
groups = {}
restored = {}
for case in cases:
    assert not case['failures'], case['name']
    transform, axis, dimensions = case['name'].removeprefix('axis-renderer-').split('-')
    key = (transform, dimensions, case['width'])
    group = groups.setdefault(key, {})
    assert axis not in group
    group[axis] = {}
    for profile in ['browser', 'native']:
        path = Path(case['prefix'] + '.' + profile + '.png')
        assert hashlib.sha256(path.read_bytes()).hexdigest() == case[profile + 'Sha256']
        with Image.open(path) as image:
            image = image.convert('RGBA')
            assert image.size == (case['width'], 320)
            group[axis][profile] = hashlib.sha256(image.tobytes()).hexdigest()
            # Include white surround to detect a leaked transform or clip.
            crop = image.crop((5, 235, 95, 275)).tobytes()
            if profile in restored:
                assert crop == restored[profile], (case['name'], profile, 'restore')
            else:
                restored[profile] = crop

checks = 0
for key, group in groups.items():
    assert set(group) == {'x', 'y', 'both', 'none'}, key
    for left, right in itertools.combinations(group, 2):
        for profile in ['browser', 'native']:
            assert group[left][profile] != group[right][profile], (key, left, right, profile)
            checks += 1
assert len(groups) == 36
result = {
    'scope': 'Direct renderer control discrimination and restore; no compiler qualification',
    'replaySha256': hashlib.sha256(replay.read_bytes()).hexdigest(),
    'groups': len(groups),
    'distinctAxisPairChecks': checks,
    'restoredRegionChecks': len(cases) * 2,
}
target = folder / 'control-audit.json'
if target.exists():
    assert json.loads(target.read_text()) == result, 'Refusing to replace changed evidence'
else:
    target.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
