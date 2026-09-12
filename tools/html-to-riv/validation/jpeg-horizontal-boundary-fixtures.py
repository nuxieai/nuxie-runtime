"""Horizontal chroma controls distinguish 4:2:2 from vertical-only evidence.

Usage: SCRIPT FRESH_FIXTURE_DIRECTORY. Author source pixels without a browser.
One-row width 2..5 controls isolate horizontal reconstruction; a 4x17 pair
checks the same colors in a taller image. PNG preserves a same-size control.
"""
from pathlib import Path
from PIL import Image
import hashlib
import json
import subprocess
import sys

root = Path(sys.argv[1]).resolve()
root.mkdir(parents=True, exist_ok=False)
encoder = Path('/opt/homebrew/bin/cjpeg')
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
colors = [(236, 51, 62), (22, 173, 111), (111, 66, 193), (238, 115, 33)]
rows, cases = [], []
for width, height in [(2, 1), (3, 1), (4, 1), (5, 1), (4, 17)]:
    pixels = bytes(c for _ in range(height) for x in range(width)
                   for c in colors[4 * x // width])
    ppm = root / f'{width}x{height}.ppm'
    ppm.write_bytes(f'P6\n{width} {height}\n255\n'.encode() + pixels)
    variants = []
    if (width, height) == (4, 1):
        png = root / f'{width}x{height}.png'
        Image.frombytes('RGB', (width, height), pixels).save(png, compress_level=9)
        variants.append((png, None, 'png-control'))
    for sample, factors in [('422', '2x1,1x1,1x1'), ('420', '2x2,1x1,1x1')]:
        file = root / f'{width}x{height}-{sample}.jpg'
        command = [str(encoder), '-quality', '90', '-sample', factors,
                   '-baseline', '-outfile', str(file), str(ppm)]
        subprocess.run(command, capture_output=True, check=True)
        variants.append((file, command, sample))
    for file, command, variant in variants:
        name = f'{width}x{height}-{variant}'
        rows.append(dict(name=name, path=str(file), sha256=sha(file), width=width, height=height,
                         source=str(ppm), sourceSha256=sha(ppm), command=command))
        cases.append(dict(name=name, asset=file.name, request=dict(assetWidth=width, assetHeight=height,
            width=dict(unit='auto'), height=dict(unit='auto'), aspectRatio=False, fit=7,
            alignmentX=0., alignmentY=0., clip=True, nearest=False),
            css='#image{display:block;width:auto;height:auto;object-fit:fill;object-position:50% 50%;image-rendering:auto;background:#e9f0f6}'))
manifest = dict(scope='Horizontal-color and one-row JPEG chroma reconstruction controls',
    encoder=dict(path=str(encoder), sha256=sha(encoder),
                 version=subprocess.run([str(encoder), '-version'], capture_output=True, text=True).stderr.strip()),
    generator=dict(path=str(Path(__file__).resolve()), sha256=sha(Path(__file__))), files=rows)
(root / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
(root / 'private-cases.json').write_text(json.dumps(cases, indent=2) + '\n')
print(json.dumps(dict(cases=len(cases), root=str(root))))
