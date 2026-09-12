#!/usr/bin/env python3
"""Deterministic source-pixel fixtures for compiler-owned JPEG sampling tests."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent
ENCODER = Path('/opt/homebrew/bin/cjpeg')
sha = lambda value: hashlib.sha256(value).hexdigest()
files = []
for width, height in [(17, 23), (33, 9)]:
    # Odd dimensions cross MCU and component-block boundaries in both patterns.
    pixels = bytes(value for y in range(height) for x in range(width)
                   for value in ((x * 37 + y * 11) % 256,
                                 (x * 7 + y * 29) % 256,
                                 (x * 19 + y * 3) % 256))
    ppm = f'P6\n{width} {height}\n255\n'.encode() + pixels
    source = ROOT / f'{width}x{height}.ppm'
    source.write_bytes(ppm)
    for sampling, factors in [('422', '2x1,1x1,1x1'), ('420', '2x2,1x1,1x1')]:
        for progressive in [False, True]:
            name = f'{sampling}-{width}x{height}-{"progressive" if progressive else "baseline"}.jpg'
            command = [str(ENCODER), '-quality', '90', '-sample', factors,
                       '-restart', '1B']
            if progressive:
                command.append('-progressive')
            command.extend(['-outfile', str(ROOT / name), str(source)])
            result = subprocess.run(command, capture_output=True, timeout=10, check=True)
            assert not result.stdout and not result.stderr
            encoded = (ROOT / name).read_bytes()
            files.append({'name': name, 'source': source.name,
                          'width': width, 'height': height,
                          'sourceSha256': sha(ppm), 'sampling': factors,
                          'progressive': progressive, 'restartIntervalMcus': 1,
                          'command': command, 'bytes': len(encoded),
                          'sha256': sha(encoded)})
(ROOT / 'manifest.json').write_text(json.dumps({
    'scope': 'Authored pixels encoded independently for compiler admission tests; no native/Chrome qualification.',
    'generatorSha256': sha(Path(__file__).read_bytes()),
    'encoder': str(ENCODER), 'encoderSha256': sha(ENCODER.read_bytes()),
    'encoderVersion': subprocess.run([str(ENCODER), '-version'], capture_output=True,
                                     text=True, check=True).stderr.strip(),
    'files': files,
}, indent=2) + '\n')
