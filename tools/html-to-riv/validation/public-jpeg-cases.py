"""Build the finite public JPEG corpus and authored single-chroma controls.

Run once from the module. Existing encoded fixtures are reused byte-for-byte.
Diagnostic cases are named explicitly from the independent pixel experiments.
"""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess

module = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--reuse-tiny', action='store_true', help='Verify and reuse existing authored tiny assets without rewriting them')
parser.add_argument('--output', type=Path, default=module / 'validation/public-jpeg-cases.json')
args = parser.parse_args()
target = args.output.resolve()
assert not target.exists(), 'Choose a fresh case-corpus output'
tiny = module / 'fixtures/images/jpeg-single-chroma-r1'
reuse = args.reuse_tiny
tiny.mkdir(exist_ok=reuse)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
encoder = Path('/opt/homebrew/bin/cjpeg')
colors = [(236, 51, 62), (22, 173, 111), (111, 66, 193), (238, 115, 33)]
files = []
for width, height in [(1, 1), (1, 2), (2, 1), (2, 2)]:
    ppm = tiny / f'{width}x{height}.ppm'
    pixels = bytes(c for y in range(height) for x in range(width) for c in colors[y * 2 + x])
    source = f'P6\n{width} {height}\n255\n'.encode() + pixels
    if reuse:
        assert ppm.read_bytes() == source
    else:
        ppm.write_bytes(source)
    for mode in (['baseline', 'progressive'] if width == height == 2 else ['baseline']):
        file = tiny / f'{width}x{height}-420-{mode}.jpg'
        command = [str(encoder), '-quality', '90', '-sample', '2x2,1x1,1x1', '-baseline']
        if mode == 'progressive': command.append('-progressive')
        command += ['-outfile', str(file), str(ppm)]
        if not reuse:
            subprocess.run(command, capture_output=True, check=True)
        else:
            prior = json.loads((tiny / 'manifest.json').read_text())
            old = next(row for row in prior['files'] if row['name'] == file.stem)
            assert sha(file) == old['sha256'] and sha(ppm) == old['sourceSha256']
        files.append(dict(name=file.stem, path=str(file), sha256=sha(file), width=width, height=height,
                          source=str(ppm), sourceSha256=sha(ppm), command=command))
if not reuse:
    (tiny / 'manifest.json').write_text(json.dumps(dict(
        scope='Single chroma sample controls, original authored encoded bytes', files=files,
        encoder=dict(path=str(encoder), sha256=sha(encoder),
                     version=subprocess.run([str(encoder), '-version'], capture_output=True, text=True).stderr.strip()),
        generator=dict(path=str(Path(__file__).resolve()), sha256=sha(Path(__file__)))), indent=2) + '\n')

groups = [
    ('jpeg-pixels-r1', 'codec', {'1x17-420-baseline', '1x17-420-progressive', '1x17-422-baseline', '1x17-422-progressive'}),
    ('jpeg-sampling-r1', 'scans', set()),
    ('jpeg-chroma-boundary-r1', 'vertical', {'1x17-420', '2x17-420', '3x17-420', '4x17-420', '4x17-422'}),
    ('jpeg-horizontal-boundary-r1', 'horizontal', {'2x1-422', '2x1-420', '3x1-422', '3x1-420', '4x1-422', '4x1-420', '4x17-422', '4x17-420'}),
    ('jpeg-single-chroma-r1', 'single-chroma', {'1x1-420-baseline', '1x2-420-baseline', '2x1-420-baseline', '2x2-420-baseline', '2x2-420-progressive'}),
]
cases = []
for directory, group, diagnostics in groups:
    asset_root = module / 'fixtures/images' / directory
    manifest = json.loads((asset_root / 'manifest.json').read_text())
    for row in manifest['files']:
        file = Path(row.get('path', row['name']))
        if not file.is_absolute(): file = asset_root / file
        assert sha(file) == row['sha256']
        name = row.get('name', file.stem).removesuffix('.jpg')
        expected = (dict(outcome='diagnostic', code='unsupported-image', messageIncludes='narrow subsampled JPEG')
                    if name in diagnostics else dict(outcome='compile'))
        cases.append(dict(name=f'{group}-{name}', group=f'jpeg-{group}',
            assetFiles=dict(picture=str(file.relative_to(module))),
            input=dict(html='<img id="image" src="picture"><div id="tail"></div>',
                       css='#image{align-self:flex-start;background:#e9f0f6}#tail{width:8px;height:8px;background:#17212b}',
                       width=390, height=320), expected=expected, observeIds=['image', 'tail'],
            notes=['Independent source assets; intrinsic image dimensions and following sibling; original/clone resize without recompilation.']))
target.write_text(json.dumps(cases, indent=2) + '\n')
print(json.dumps(dict(cases=len(cases), compiled=sum(c['expected']['outcome']=='compile' for c in cases))))
