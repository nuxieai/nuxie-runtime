"""Revalidate P05 public opacity lifecycle references; optionally compare a completed gallery.

Usage: SCRIPT EVIDENCE_ROOT [COMPLETED_GALLERY]
Transfers require exact authored inputs, assets, RIV, requirements and full PNGs.
"""
import hashlib
import json
from pathlib import Path
import runpy
import sys


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def references(root):
    verify = runpy.run_path(str(Path(__file__).with_name('audit-opacity-diagnostic.py')))['audit']
    compiler = root / 'group-opacity-native-full-toolchain-r2/html-to-riv'
    corpora = [
        ('group-opacity-public-static-review', 'group-opacity-public-runtime-recording',
         'group-opacity-public-runtime-lifecycle', None, 30),
        ('group-opacity-boundary-public-review-r2', 'group-opacity-boundary-public-recording-r2',
         'group-opacity-boundary-public-lifecycle-r2', 'boundary', 28),
        ('group-opacity-overflow-composition-review', 'group-opacity-overflow-composition-recording',
         'group-opacity-overflow-composition-lifecycle', 'overflow-composition', 18),
        ('group-opacity-decoration-review', 'group-opacity-decoration-recording',
         'group-opacity-decoration-lifecycle', 'decoration', 12),
    ]
    refs, receipts = {}, []
    for review, recording, replay, oracle_name, count in corpora:
        oracle = (Path(__file__).resolve().parent.parent / 'tests/assets' /
                  f'group-opacity-{oracle_name}-oracle.json') if oracle_name else None
        result = verify(root / recording, root / replay, root / review, compiler, oracle)
        assert result['scenes'] == count and result['frames'] == count * 8
        for directory, filename in [(review, 'replay.json'), (replay, 'replay.json'),
                                     (recording, 'lifecycle.json')]:
            p = root / directory / filename
            receipts.append(dict(path=str(p), sha256=sha(p)))
        fixtures = {f['name']: f for f in read(root / recording / 'lifecycle.json')['cases']}
        for row in read(root / replay / 'replay.json')['cases']:
            if row['instance'] != 0 or row['frame'] > 2:
                continue
            name = row['name'].rsplit('-0-step', 1)[0]
            key = name, row['width']
            assert key not in refs
            refs[key] = dict(row=row, fixture=fixtures[name], recording=root / recording)
    assert len(refs) == 264
    return refs, receipts


def audit(root, gallery):
    refs, receipts = references(root)
    report = read(gallery / 'review.json')
    assert report['status'] == 'passed' and not report['errors']
    assert len(report['checks']) == report['expected']
    assert all(c['status'] == 'passed' and not c['errors'] and c['project'] == 'native' for c in report['checks'])
    rows, seen = [], set()
    assets_dir = Path(__file__).resolve().parent.parent / 'tests/assets'
    for case in report['cases']:
        key = case['name'], case['width']
        if key not in refs:
            continue
        assert key not in seen
        seen.add(key)
        ref = refs[key]
        f = ref['fixture']
        expected = dict(html=f['html'], css=f['css'], width=390, height=320)
        assets = {}
        if f['font']:
            assets['inter'] = dict(kind='font', family='Inter', weight=400,
                                   bytes=list((assets_dir / 'Inter-Regular.ttf').read_bytes()))
        if f['image']:
            assets['photo'] = dict(kind='image', bytes=list((assets_dir / 'quadrants.png').read_bytes()))
        if assets:
            expected['assets'] = assets
        prefix = case['artifactPrefix']
        assert read(Path(prefix + '.json')) == expected
        assert case['html'] == f['html'] and case['css'] == f['css']
        assert case['status'] == 'passed' and not case['errors'] and case['metrics']['backend'] == 'rust-metal'
        assert sha(Path(prefix + '.riv')) == sha(ref['recording'] / (key[0] + '.riv'))
        assert read(Path(prefix + '.requirements.json')) == f['runtimeRequirements']
        hashes = {kind + 'Sha256': sha(Path(prefix + '.' + kind + '.png')) for kind in ('browser', 'native')}
        rows.append(dict(name=key[0], width=key[1], **hashes,
                         reviewTransferred=all(ref['row'][k] == v for k, v in hashes.items())))
    assert seen == set(refs)
    return dict(reviewSha256=sha(gallery / 'review.json'), sourceReceipts=receipts,
                rows=rows, reviewedPairs=sum(r['reviewTransferred'] for r in rows),
                remaining=[r for r in rows if not r['reviewTransferred']])


if __name__ == '__main__':
    root = Path(sys.argv[1]).resolve()
    if len(sys.argv) == 2:
        refs, _ = references(root)
        print(json.dumps(dict(sourceViewsVerified=len(refs))))
    else:
        gallery = Path(sys.argv[2]).resolve()
        result = audit(root, gallery)
        output = gallery / 'opacity-lifecycle-comparison.json'
        if output.exists():
            assert read(output) == result
        else:
            output.write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(dict(reviewedPairs=result['reviewedPairs'], remaining=len(result['remaining']))))
