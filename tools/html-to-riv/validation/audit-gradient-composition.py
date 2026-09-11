"""Rebuild static public gradient evidence and audit full visual-review coverage.
Usage: SCRIPT REPLAY_DIRECTORY
Does not qualify original/clone lifecycle or untested capabilities.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from PIL import Image, ImageDraw

read = lambda p: json.loads(p.read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
key = lambda row: (row['name'], row['width'])


def images(row):
    for kind in ['browser', 'native']:
        path = Path(row['prefix'] + '.' + kind + '.png')
        assert sha(path) == row[kind + 'Sha256'], path
        with Image.open(path) as im:
            assert im.size == (row['width'], 320)


def sheet(folder, name, rows):
    selected = sorted((r for r in rows if r['name'] == name), key=lambda r:r['width'])
    cw = max(r['width'] for r in selected) + 8
    expected = Image.new('RGB', (cw * 3, 350 * len(selected)), '#ddd')
    draw = ImageDraw.Draw(expected)
    for yindex, row in enumerate(selected):
        for xindex, kind in enumerate(['browser', 'native', 'diff']):
            x, y = xindex * cw, yindex * 350
            draw.text((x+4, y+4), f'{kind} / {row["width"]}', fill='black')
            with Image.open(row['prefix'] + '.' + kind + '.png') as im:
                expected.paste(im.convert('RGB'), (x+4, y+24))
    p = folder / 'review-sheets' / (name + '.png')
    with Image.open(p) as actual:
        assert actual.size == expected.size and actual.convert('RGB').tobytes() == expected.tobytes(), p
    return p


def audit(folder):
    report = read(folder/'replay.json')
    toolchain = Path(report['toolchain'])
    manifest = read(toolchain/'manifest.json')
    assert report['manifest'] == manifest
    for name in ['html-to-riv','html-to-riv.wasm','probe','renderer-replay']:
        assert sha(toolchain/name) == manifest['files'][name]['sha256'], name
    oracle_path = Path(report['oracle'])
    assert sha(oracle_path) == report['oracleSha256']
    oracle = read(oracle_path)
    assert oracle['browser'] == '153.0.8010.12'
    reference_receipt = read(oracle_path.parent/'receipt.json')
    assert reference_receipt['oracle']['sha256'] == sha(oracle_path)
    for name in ['source','reset','capture']:
        binding = reference_receipt[name]
        assert sha(Path(binding['path'])) == binding['sha256']
    source = read(Path(reference_receipt['source']['path']))
    authored = {c['name']:c for c in source if c['expectedCompile']['status']=='accepted'}
    current_source = Path(__file__).parent/'linear-gradient-composition-cases.json'
    assert read(current_source) == source
    assert len(authored) == len(oracle['cases']) == 18
    for fixture in oracle['cases']:
        assert {k:v for k,v in fixture.items() if k!='viewports'} == authored[fixture['name']]
        assert [v['width'] for v in fixture['viewports']] == [240,390,768]
    expected_keys = {(name,width) for name in authored for width in [240,390,768]}
    rows = {key(c):c for c in report['cases']}
    assert len(rows) == len(report['cases']) == 54 and rows.keys() == expected_keys
    for binding in reference_receipt['images']:
        assert sha(Path(binding['path'])) == binding['sha256']
    # Fresh frozen compilation, fresh checked host recording and real native rendering,
    # plus the unchanged standard geometry/pixel gates. This also rejects fabricated
    # passing metrics and streams unrelated to the public compiler artifacts.
    runner = Path(__file__).parent/'replay-oracle.mjs'
    with tempfile.TemporaryDirectory(prefix='gradient-composition-audit-') as temporary:
        fresh = Path(temporary)/'replay'
        run = subprocess.run(['node',str(runner),str(oracle_path),str(toolchain),str(fresh)],
            env={**os.environ,'NUXIE_NATIVE_GLYPHS':'0'},capture_output=True,text=True)
        assert run.returncode == 0, run.stdout + run.stderr
        rebuilt = read(fresh/'replay.json')
        rebuilt_rows = {key(c):c for c in rebuilt['cases']}
        assert rebuilt_rows.keys() == rows.keys()
        for name in authored:
            for suffix in ['.json','.riv','.map.json','.requirements.json']:
                assert (folder/(name+suffix)).read_bytes() == (fresh/(name+suffix)).read_bytes(), (name,suffix)
        for k,row in rows.items():
            actual = rebuilt_rows[k]
            assert {field:value for field,value in row.items() if field!='prefix'} == {
                field:value for field,value in actual.items() if field!='prefix'}, k
            images(row)
            for suffix in ['.stream','.bounds.json','.browser.png','.native.png','.diff.png']:
                assert Path(row['prefix']+suffix).read_bytes() == Path(actual['prefix']+suffix).read_bytes(), (k,suffix)
        edge = Path(temporary)/'border-gate.json'
        run = subprocess.run([sys.executable,str(Path(__file__).parent/'check-gradient-border-repeat.py'),str(folder),str(edge)],capture_output=True,text=True)
        assert run.returncode == 0, run.stdout + run.stderr
        edge_result = read(edge)
    review = read(folder/'composition-visual-inspection.json')
    assert review['replaySha256'] == sha(folder/'replay.json') and review['remaining'] == []
    seen = set()
    for entry in review['direct']:
        k = key(entry); assert k in rows and k not in seen; seen.add(k)
        row = rows[k]
        for field in ['browserSha256','nativeSha256']:assert entry[field] == row[field]
        assert entry['status'] != 'visual-failure'
        p = sheet(folder,entry['name'],report['cases'])
        assert Path(entry['sheet']) == p and entry['sheetSha256'] == sha(p)
    for entry in review['transferred']:
        k = key(entry); assert k in rows and k not in seen; seen.add(k)
        origin_path, inspection_path = Path(entry['sourceReplay']), Path(entry['sourceInspection'])
        assert sha(inspection_path) == entry['sourceInspectionSha256']
        inspection = read(inspection_path); assert inspection['replaySha256'] == sha(origin_path)
        origin = read(origin_path); old_rows = {key(c):c for c in origin['cases']}; old_row = old_rows[k]
        row = rows[k]
        for field in ['html','css','rivSha256','browserSha256','nativeSha256']:assert row[field] == old_row[field]
        images(old_row)
        for kind in ['browser','native']:
            assert Path(row['prefix']+'.'+kind+'.png').read_bytes() == Path(old_row['prefix']+'.'+kind+'.png').read_bytes()
        # r1 includes three within-run transfers to a directly inspected color-order alias.
        direct = next((d for d in inspection['direct'] if key(d)==k and d['status']!='visual-failure'),None)
        if direct is None:
            transfer = next(d for d in inspection['transferred'] if key(d)==k)
            source_key = (transfer['source'],entry['width'])
            direct = next(d for d in inspection['direct'] if key(d)==source_key and d['status']!='visual-failure')
            source_row = old_rows[source_key]; images(source_row)
            for kind in ['browser','native']:
                assert Path(source_row['prefix']+'.'+kind+'.png').read_bytes() == Path(old_row['prefix']+'.'+kind+'.png').read_bytes()
        reviewed_row = old_rows[key(direct)]
        for field in ['browserSha256','nativeSha256']:assert direct[field] == reviewed_row[field]
        sheet(origin_path.parent,direct['name'],origin['cases'])
    assert seen == expected_keys
    return dict(status='passed',scope='Static public compiler artifacts, checked probe, fresh native pixels and complete visual evidence. Original/clone lifecycle remains separate.',
                scenes=18,frames=54,directViews=len(review['direct']),transferredViews=len(review['transferred']),
                replaySha256=sha(folder/'replay.json'),visualInspectionSha256=sha(folder/'composition-visual-inspection.json'),
                toolchainManifestSha256=sha(toolchain/'manifest.json'),oracleSha256=sha(oracle_path),
                sourceSha256=sha(current_source),runnerSha256=sha(runner),auditSha256=sha(Path(__file__)),borderGate=edge_result)

if __name__=='__main__':
    folder = Path(sys.argv[1]).resolve()
    result = audit(folder)
    target = folder/'composition-artifact-audit.json'
    assert not target.exists(), 'Refusing to overwrite evidence'
    target.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({key:result[key] for key in ['status','scenes','frames','directViews','transferredViews']}))
