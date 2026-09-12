"""Verify and bind the complete primitive-value recovery checkpoint."""
import hashlib
import json
from pathlib import Path
import re
from PIL import Image

module = Path(__file__).resolve().parents[1]
repo = module.parents[1]
build = module / 'output/public-value-build-r1'
native = module / 'output/public-value-native-r1'
validation = module / 'validation'


def read(path):
    return json.loads(Path(path).read_text())


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def resolve(path):
    path = Path(path)
    if path.is_absolute():
        return path
    matches = [p for p in [repo / path, module / path] if p.is_file()]
    assert matches, path
    return matches[0]


bindings = {}


def bind(path, digest=None):
    path = resolve(path)
    actual = sha(path)
    assert digest is None or actual == digest, f'Changed evidence: {path}'
    assert str(path) not in bindings or bindings[str(path)] == actual
    bindings[str(path)] = actual
    return path


freeze_path = build / 'frozen/source-bindings.json'
freeze = read(bind(freeze_path))
for item in freeze['files']:
    bind(item['path'], item['sha256'])
    if 'snapshot' in item:
        bind(item['snapshot'], item['sha256'])
summary = read(bind(build / 'summary.json'))
assert summary['sourceUnchanged'] and summary['rustTests'] == 262 and summary['nodeTests'] == 42
checks = read(bind(build / 'checks.json'))
assert len(checks) == 6 and all(row['exitCode'] == 0 for row in checks)
for row in checks:
    bind(row['log'], row['logSha256'])
assert sum(map(int, re.findall(r'test result: ok\. (\d+) passed;', (build / 'full-rust.log').read_text()))) == 262
assert read(build / 'runtime-guard.log')['status'] == 'pass'
cli_hash = summary['compilerSha256']

components = {}
for name in ['token', 'native', 'regression']:
    path = validation / f'public-value-{name}-receipt.json'
    receipt = read(bind(path))
    for row in receipt.get('bindings', []):
        if 'path' in row and 'sha256' in row:
            bind(row['path'], row['sha256'])
    components[name] = dict(path=str(path), sha256=sha(path), status=receipt['status'])

regression = read(bind(module / 'output/public-value-regression-r1/manifest.json'))
assert regression['compilerSha256'] == cli_hash
assert regression['total'] == regression['passed'] == 731
assert all(row['exitCode'] == 0 and row['exact'] for row in regression['results'])
render_path = bind(native / 'render/receipt.json')
render = read(render_path)
assert render['status'] == 'passed-public-baseline'
assert render['browser'] == '153.0.8010.12'
assert render['backend'] == 'rust-metal' and render['effectiveMode'] == 'RasterOrdering'
assert render['toolHashes']['compiler'] == cli_hash
for name, path in render['tools'].items():
    bind(path, render['toolHashes'][name])
rows = {(row['name'], row['frame']): row for row in render['rows']}
assert len(rows) == 280 and len(render['artifacts']) == 35
assert len(render['repeated']) == 175 and all(r['nativeIdentical'] and r['chromeIdentical'] for r in render['repeated'])
for row in rows.values():
    assert not row['geometryFailures'] and not row['pixelFailures']
    assert (row['instance'], row['step']) == (row['frame'] // 4, row['frame'] % 4)
    for role in ['native', 'chrome']:
        bind(row['prefix'] + f'.{role}.png', row[f'{role}Sha256'])
    assert len(row['clearChecks']) == 2
    for check in row['clearChecks']:
        assert check['samePixels']
        bind(check['path'], check['sha256'])

coverage = read(bind(native / 'visual-coverage.json'))
assert coverage['renderReceiptSha256'] == sha(render_path)
assert coverage['directPairs'] == 37 and coverage['transferredPairs'] == 243
assert len(coverage['coverage']) == len(rows)
direct = {(r['name'], r['frame']) for r in coverage['coverage'] if r['method'] == 'direct'}
assert len(direct) == 37
seen = set()
for item in coverage['coverage']:
    key = (item['name'], item['frame'])
    assert key not in seen and key in rows
    seen.add(key)
    if item['method'] == 'direct':
        continue
    assert item['method'] == 'complete-rgba-white-canvas'
    reference_key = (item['name'], item['referenceFrame'])
    assert reference_key in direct
    row, reference = rows[key], rows[reference_key]
    assert all(row[k] == reference[k] for k in ['requestSha256', 'rivSha256', 'sourceMapSha256'])
    for role in ['chrome', 'native']:
        images = [Image.open(r['prefix'] + f'.{role}.png').convert('RGBA') for r in [row, reference]]
        size = tuple(max(a, b) for a, b in zip(images[0].size, images[1].size))
        canvases = []
        for image in images:
            canvas = Image.new('RGBA', size, 'white')
            canvas.paste(image, (0, 0))
            canvases.append(canvas.tobytes())
        assert canvases[0] == canvases[1], (key, reference_key, role)
assert seen == set(rows)
sheets = read(bind(native / 'review-sheets.json'))
assert sheets['coverageSha256'] == sha(native / 'visual-coverage.json')
assert len(sheets['sheets']) == 10
assert {(p['name'], p['frame']) for sheet in sheets['sheets'] for p in sheet['pairs']} == direct
for sheet in sheets['sheets']:
    bind(sheet['path'], sheet['sha256'])
native_evidence = read(validation / 'public-value-native-receipt.json')
assert native_evidence['visualReviewCompleted'] is True

bridge = module / 'output/public-value-bridge-build-r1/build-receipt.json'
bind(bridge)
for name in ['SUPPORT.md', 'VALIDATION.md', 'BACKLOG.md', 'validation/progress-state.json',
             'validation/public-value-checkpoint-review.md', 'validation/public-value-build.py']:
    bind(module / name)
bind(Path(__file__).resolve())
statuses = [line.split('|')[3].strip().split(' (')[0] for line in (module / 'BACKLOG.md').read_text().splitlines()
            if re.match(r'\| [ASLPITRQ]\d{2}[a-z]? \|', line)]
assert len(statuses) == 99
assert [statuses.count(k) for k in ['qualified', 'partial', 'investigating', 'pending']] == [13, 14, 3, 69]
result = dict(status='qualified-primitive-recovery-with-explicit-unresolved-grammar',
              scope='Bounded public primitive receiving grammar and token boundaries; no general CSS, text, asset, layout or renderer qualification.',
              baseline='6c7ac16617835b5f581784ff08a9e779bb52faf3',
              rustTests=262, nodeTests=42, publicCases=610, cliWasmRequestControlPairs=1876,
              exactPriorOutputs=731, nativeScenes=35, nativeFrames=280, geometryPass=280, pixelPass=280,
              clearPass=560, directlyReviewedPairs=37, exactRgbaTransfers=243, reviewedSheets=10,
              compilerSha256=cli_hash, wasmSha256=summary['wasmSha256'], components=components,
              remaining='Opaque functions and unresolved receiving grammar; existing fractional-paint and large-padding failures; full99-item backlog remains active.',
              bindings=[dict(path=p, sha256=h) for p, h in sorted(bindings.items())])
(validation / 'public-value-checkpoint-receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: result[k] for k in ['status', 'rustTests', 'nodeTests', 'exactPriorOutputs', 'nativeFrames', 'directlyReviewedPairs', 'exactRgbaTransfers']}))
