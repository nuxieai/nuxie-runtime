"""Compare experimental public compiler/probe geometry with the L10 Chromium oracle.
Run after building the publisher. Produces evidence even when coordinates fail.
"""
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[3]
compiler = Path(sys.argv[1]).resolve()
probe = Path(sys.argv[2]).resolve()
output = Path(sys.argv[3]).resolve()
output.mkdir(parents=True, exist_ok=False)
oracle_path = Path(sys.argv[4]) if len(sys.argv) > 4 else root / 'output/playwright/html-to-riv/content-auto-oracle/oracle.json'
oracle = json.loads(oracle_path.read_text())
font = list((root / 'tools/html-to-riv/tests/assets/Inter-Regular.ttf').read_bytes())
results = []
rejections = []
for case in oracle['cases']:
    prefix = output / case['name']
    request = {key: case[key] for key in ['html', 'css']}
    request.update(width=390, height=320)
    if case.get('font'):
        request['assets'] = {'inter': dict(kind='font', family='Inter', weight=400, bytes=font)}
    source = Path(str(prefix) + '.json')
    source.write_text(json.dumps(request))
    riv = Path(str(prefix) + '.riv')
    compilation = subprocess.run([str(compiler), str(source), str(riv)], capture_output=True, text=True)
    if compilation.returncode:
        rejections.append(dict(name=case['name'], request=str(source), exitCode=compilation.returncode, stdout=compilation.stdout, stderr=compilation.stderr))
        continue
    compiled_hash = hashlib.sha256(riv.read_bytes()).hexdigest()
    for viewport in case['viewports']:
        width = viewport['width']
        resized = str(prefix) + f'-{width}'
        subprocess.run([str(probe), str(riv), str(prefix) + '.map.json', str(width), '320', resized], check=True, capture_output=True, env={**os.environ, 'NUXIE_NATIVE_GLYPHS': '0'})
        actual = json.loads(Path(resized + '.bounds.json').read_text())
        assert set(actual) == set(viewport['boxes']), (case['name'], set(actual) ^ set(viewport['boxes']))
        assert all(not box.get('hidden', False) for box in actual.values())
        assert hashlib.sha256(riv.read_bytes()).hexdigest() == compiled_hash
        mismatches = []
        maximum = 0
        for identity, expected in viewport['boxes'].items():
            for axis in ['x', 'y', 'width', 'height']:
                observed = actual[identity][axis]
                assert isinstance(observed, (int, float)) and math.isfinite(observed), (case['name'], width, identity, axis, observed)
                assert math.isfinite(expected[axis]), (case['name'], 'invalid oracle', identity, axis)
                error = abs(observed - expected[axis])
                maximum = max(maximum, error)
                if error > .1:
                    mismatches.append(dict(id=identity, axis=axis, actual=actual[identity][axis], expected=expected[axis]))
        results.append(dict(name=case['name'], width=width, rivSha256=compiled_hash, maximumErrorPx=maximum, mismatches=mismatches))
receipt = dict(toolchain=[dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in [compiler, probe]], tolerancePx=.1, results=results, rejections=rejections)
(output / 'geometry.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(dict(viewports=len(results), failed=sum(bool(r['mismatches']) for r in results), rejected=len(rejections), maxErrorPx=max((r['maximumErrorPx'] for r in results), default=0))))
sys.exit(bool(rejections) or any(r['mismatches'] for r in results))
