#!/usr/bin/env python3
"""Verify the private content-owner checkpoint without rerendering unchanged files."""
from pathlib import Path
from collections import Counter
import hashlib
import importlib.util
import json
import re
import subprocess
from PIL import Image

BASE = Path(__file__).resolve().parents[1]
OUT = BASE / 'output'
BINDINGS = {}


def sha(path):
    path = Path(path).resolve()
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    BINDINGS[str(path)] = digest
    return digest


def read(path):
    sha(path)
    return json.loads(Path(path).read_text())


def rgba(path):
    with Image.open(path) as image:
        return image.size, image.convert('RGBA').tobytes()


def equal_white_extension(left, right):
    with Image.open(left) as a, Image.open(right) as b:
        size = max(a.width, b.width), max(a.height, b.height)
        first = Image.new('RGBA', size, (255, 255, 255, 255))
        second = first.copy()
        first.paste(a.convert('RGBA'), (0, 0))
        second.paste(b.convert('RGBA'), (0, 0))
        return first.tobytes() == second.tobytes()


def verify_render(directory, compiler_hash):
    receipt = read(directory / 'receipt.json')
    assert receipt['browser'] == '153.0.8010.12'
    assert receipt['backend'] == 'rust-metal' and receipt['effectiveMode'] == 'RasterOrdering'
    assert receipt['toolHashes'] == dict(compiler=compiler_hash,
        probe='7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a',
        renderer='276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f')
    for name, path in receipt['tools'].items():
        assert sha(path) == receipt['toolHashes'][name]
    for binding in receipt['sourceBindings']:
        assert sha(binding['source']) == sha(directory / binding['snapshot']) == binding['sha256']
    rows = {}
    for row in receipt['rows']:
        key = row['name'], row['frame']
        assert key not in rows
        rows[key] = row
        case = directory / row['name']
        prefix = Path(row['prefix'])
        assert row['instance'] == row['frame'] // 4 and row['step'] == row['frame'] % 4
        assert [row['width'], row['height']] == [[240, 160], [390, 200], [768, 120], [240, 160]][row['step']]
        for file, field in [(case / 'request.json', 'requestSha256'), (case / 'scene.riv', 'rivSha256'),
                            (case / 'scene.map.json', 'sourceMapSha256'), (str(prefix) + '.chrome.png', 'chromeSha256'),
                            (str(prefix) + '.native.png', 'nativeSha256'),
                            (case / 'probe' / row['stream'], 'streamSha256')]:
            assert sha(file) == row[field]
        frames = read(case / 'probe/frames.json')['frames']
        geometry_file = case / 'probe' / frames[row['frame']]['geometry']
        assert sha(geometry_file) == row['geometrySha256']
        assert read(geometry_file) == row['geometry']
        mapping = read(case / 'scene.map.json')
        actual = {n['objectId']: n for n in row['geometry']}
        failures = []
        for node in mapping:
            value = actual[node['object_id']]
            assert value['worldMatrix'][:4] == [1, 0, 0, 1]
            measured = dict(x=value['worldMatrix'][4], y=value['worldMatrix'][5], width=value['width'], height=value['height'])
            for axis, number in measured.items():
                if abs(number - row['boxes'][node['id']][axis]) > .1:
                    failures.append(f"{node['id']}.{axis}: {number} vs {row['boxes'][node['id']][axis]}")
        assert len(failures) == len(row['geometryFailures'])
        image = rgba(str(prefix) + '.native.png')
        for clear in row['clearChecks']:
            assert sha(clear['path']) == clear['sha256'] and clear['samePixels']
            assert rgba(clear['path']) == image
        first = next(r for r in receipt['rows'] if r['name'] == row['name'] and (r['width'], r['height']) == (row['width'], row['height']))
        assert first['nativeSha256'] == row['nativeSha256'] and first['chromeSha256'] == row['chromeSha256']
    return rows


def main():
    spec = importlib.util.spec_from_file_location('candidate', BASE / 'validation/content-owner-candidate.py')
    candidate = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(candidate)
    builds = {}
    for revision, packing, bounds in [('r1', 'column', False), ('r2', 'parent-axis', False), ('r3', 'parent-axis', True)]:
        directory = OUT / f'content-owner-candidate-build-{revision}'
        build = read(directory / 'build-receipt.json')
        for name, expected in build['originalSourceHashes'].items():
            assert sha(BASE / 'src' / name) == expected
            actual = directory / 'source' / name
            assert sha(actual) == build['experimentalSourceHashes'][name]
            if name == 'compiler.rs':
                assert actual.read_text() == candidate.patch((BASE / 'src/compiler.rs').read_text(), packing, bounds)
            else:
                assert sha(actual) == expected
        assert sha(directory / 'compiler') == build['compilerSha256']
        assert sha(directory / 'compiler.patch') == build['patchSha256']
        assert sha(directory / 'harness.rs') == build['harnessSha256']
        assert sha(directory / 'invoked-driver.py') == build['scriptSha256']
        assert sha(BASE / 'Cargo.toml') == build['cargoManifestSha256']
        assert sha(BASE / 'Cargo.lock') == build['cargoLockSha256']
        for path, expected in build['directDependencyHashes'].items():
            assert sha(path) == expected
        builds[revision] = build['compilerSha256']
    public_hash = '034084cd868e9b33ba849bc328ea4c540d63115192eba7918e7d7ec00c3c2b2d'
    public = verify_render(OUT / 'content-owner-public-r1/render', public_hash)
    public.update(verify_render(OUT / 'content-owner-public-r1/stretch/render', public_hash))
    first = verify_render(OUT / 'content-owner-candidate-native-r1', builds['r1'])
    first.update(verify_render(OUT / 'content-owner-stretch-native-r1', builds['r1']))
    revised = verify_render(OUT / 'content-owner-candidate-native-r2', builds['r2'])
    assert public.keys() == first.keys() == revised.keys() and len(revised) == 272
    for key, row in revised.items():
        before = public[key]
        assert before['boxes'] == row['boxes'] == first[key]['boxes']
        assert before['requestSha256'] == row['requestSha256'] == first[key]['requestSha256']
        assert before['chromeSha256'] == row['chromeSha256'] == first[key]['chromeSha256']
        assert before['pixelFailures'] == row['pixelFailures']
        assert rgba(before['prefix'] + '.native.png') == rgba(row['prefix'] + '.native.png')
    counts = {name: dict(frames=len(rows), geometry=sum(not r['geometryFailures'] for r in rows.values()),
                         pixels=sum(not r['pixelFailures'] for r in rows.values()))
              for name, rows in [('public', public), ('r1', first), ('r2', revised)]}
    assert counts == {'public': {'frames': 272, 'geometry': 264, 'pixels': 266},
                      'r1': {'frames': 272, 'geometry': 256, 'pixels': 250},
                      'r2': {'frames': 272, 'geometry': 272, 'pixels': 266}}
    preflights = {revision: read(OUT / f'content-owner-candidate-preflight-{revision}/receipt.json') for revision in ['r2', 'r3']}
    assert preflights['r2']['cases'] == preflights['r3']['cases'] == 39
    for a, b in zip(preflights['r2']['rows'], preflights['r3']['rows'], strict=True):
        assert a['name'] == b['name'] and a['requestSha256'] == b['requestSha256']
        for field in ['exitCode', 'rivSha256', 'mapSha256', 'diagnostic', 'owners']:
            assert a['outputs'][0].get(field) == b['outputs'][0].get(field)
        for revision, row in [('r2', a), ('r3', b)]:
            directory = OUT / f'content-owner-candidate-preflight-{revision}' / row['name']
            assert sha(directory / 'request.json') == row['requestSha256']
            for attempt, sample in zip(['scene', 'repeat'], row['outputs'], strict=True):
                assert sha(directory / f'{attempt}.log') == sample['logSha256']
                if sample['exitCode'] == 0:
                    assert sha(directory / f'{attempt}.riv') == sample['rivSha256']
                    assert sha(directory / f'{attempt}.map.json') == sample['mapSha256']
                    rendered = revised[(row['name'], 0)]
                    assert sample['rivSha256'] == rendered['rivSha256'] and sample['mapSha256'] == rendered['sourceMapSha256']
                    assert read(directory / 'request.json') == read(Path(rendered['prefix']).parent / 'request.json')
    bounds = read(OUT / 'content-owner-bounds-r1/receipt.json')
    assert bounds['summary'] == {'public': {'accepted': 18, 'rejected': 2}, 'r2': {'accepted': 18, 'rejected': 2}, 'r3': {'accepted': 12, 'rejected': 8}}
    assert sha(OUT / 'content-owner-bounds-r1/cases.json') == bounds['casesSha256']
    assert bounds['compilerHashes'] == dict(public=public_hash, r2=builds['r2'], r3=builds['r3'])
    for row in bounds['rows']:
        directory = OUT / 'content-owner-bounds-r1' / row['name']
        assert sha(directory / 'request.json') == row['requestSha256']
        for label, sample in row['outputs'].items():
            assert sha(directory / f'{label}.log') == sample['logSha256']
            if sample['exitCode'] == 0:
                assert sha(directory / f'{label}.riv') == sample['rivSha256']
                assert sha(directory / f'{label}.map.json') == sample['mapSha256']
            else:
                assert not (directory / f'{label}.riv').exists()
                assert not (directory / f'{label}.map.json').exists()
    visual = read(OUT / 'content-owner-visual-review-r1/receipt.json')
    assert visual['visualReviewCompleted']
    for binding in visual['bindings'] + visual['sourceReceipts'] + visual['inspectedSheets']:
        assert sha(binding['path']) == binding['sha256']
    coverage = {(item['name'], item['frame']): item for item in visual['frameMapping']}
    direct = {key for key, item in coverage.items() if item['method'] == 'direct'}
    assert len(direct) == visual['representativeCount'] == 50
    assert len(coverage) - len(direct) == visual['transferredFrameCount'] == 222
    assert coverage.keys() == revised.keys()
    reviewed = {(item['name'], item['frame']) for sheet in visual['inspectedSheets'] for item in sheet['triples']}
    assert reviewed == direct and len(visual['inspectedSheets']) == 15
    for key, item in coverage.items():
        if key in direct:
            continue
        assert item['method'] == 'complete-rgba-white-canvas-all-three-roles'
        reference = item['name'], item['referenceFrame']
        assert reference in direct
        for records, suffix in [(public, '.chrome.png'), (public, '.native.png'), (revised, '.native.png')]:
            assert equal_white_extension(records[key]['prefix'] + suffix, records[reference]['prefix'] + suffix)
    guard = subprocess.run(['python3', str(BASE / 'validation/check-target-runtime.py')], capture_output=True, text=True, check=True)
    target = json.loads(guard.stdout)
    assert target['status'] == 'pass'
    backlog = [line.split('|') for line in (BASE / 'BACKLOG.md').read_text().splitlines() if re.match(r'^\| [A-Z]\d{2}[a-z]? \|', line)]
    statuses = dict(Counter(row[3].strip().split(' (')[0] for row in backlog))
    assert len(backlog) == 99 and statuses == dict(qualified=13, partial=14, investigating=3, pending=69)
    for pattern in ['content-owner-*.py', 'content-owner-*.md', 'content-owner-*.json']:
        for file in (BASE / 'validation').glob(pattern):
            if file.name != 'content-owner-checkpoint-receipt.json':
                sha(file)
    for name in ['BACKLOG.md', 'SUPPORT.md', 'VALIDATION.md', 'validation/progress-state.json']:
        sha(BASE / name)
    receipt = dict(scope='Private ordinary-file content-owner experiment; no public compiler admission change.',
                   counts=counts, r3ExactRenderedScenes=34, preservedDiagnostics=5,
                   bounds=bounds['summary'], visualReviewCompleted=True, backlog=statuses,
                   runtimeGuard=target, bindings=BINDINGS)
    (BASE / 'validation/content-owner-checkpoint-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({k: v for k, v in receipt.items() if k != 'bindings'}, indent=2))


if __name__ == '__main__':
    main()
