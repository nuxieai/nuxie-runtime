"""Author a finite codec-pixel corpus; never render HTML into source assets.

FRESH_FIXTURE_DIRECTORY; preserves encoder identity, original pixels and bytes.
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
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
colors = [(236,51,62),(31,121,220),(22,173,111),(245,187,37),
          (111,66,193),(23,167,180),(238,115,33),(35,42,59)]
rows, cases = [], []
for width, height in [(96,64),(97,65),(31,47),(1,17)]:
    pixels = bytes(c for y in range(height) for x in range(width)
                   for c in colors[(4*x//width + 2*(4*y//height)) % 8])
    ppm = root / f'{width}x{height}.ppm'
    ppm.write_bytes(f'P6\n{width} {height}\n255\n'.encode() + pixels)
    png = root / f'{width}x{height}.png'
    Image.frombytes('RGB',(width,height),pixels).save(png,compress_level=9)
    variants = [(png,None,'png-control')]
    for sample, factors in [('444','1x1,1x1,1x1'),('422','2x1,1x1,1x1'),('420','2x2,1x1,1x1')]:
        for mode in (['baseline'] if sample == '444' else ['baseline','progressive']):
            file = root / f'{width}x{height}-{sample}-{mode}.jpg'
            command = [str(encoder),'-quality','90','-sample',factors,'-baseline']
            if mode == 'progressive': command += ['-progressive']
            command += ['-outfile',str(file),str(ppm)]
            run = subprocess.run(command,capture_output=True,text=True,check=True)
            variants.append((file,command,f'{sample}-{mode}'))
    for file, command, variant in variants:
        name = f'{width}x{height}-{variant}'
        rows.append(dict(name=name,path=str(file),sha256=sha(file),width=width,height=height,
                         source=str(ppm),sourceSha256=sha(ppm),command=command))
        cases.append(dict(name=name,asset=file.name,request=dict(assetWidth=width,assetHeight=height,
            width=dict(unit='auto'),height=dict(unit='auto'),aspectRatio=False,fit=7,
            alignmentX=0.,alignmentY=0.,clip=True,nearest=False),
            css='#image{display:block;width:auto;height:auto;object-fit:fill;object-position:50% 50%;image-rendering:auto;background:#e9f0f6}'))
manifest = dict(scope='Authored codec-pixel controls: PNG and JPEG4:4:4 controls plus4:2:2/4:2:0 baseline/progressive, even/odd/portrait/skinny sizes',
    encoder=dict(path=str(encoder),sha256=sha(encoder),version=subprocess.run([str(encoder),'-version'],capture_output=True,text=True).stderr.strip()),
    generator=dict(path=str(Path(__file__).resolve()),sha256=sha(Path(__file__))),files=rows)
(root/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
(root/'private-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
print(json.dumps(dict(cases=len(cases),assets=len(rows),root=str(root))))
