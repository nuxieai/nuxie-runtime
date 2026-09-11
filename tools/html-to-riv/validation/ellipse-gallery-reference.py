"""Revalidate P04 lifecycle references; optionally compare a completed gallery.

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
    verify = runpy.run_path(str(Path(__file__).with_name('audit-border-composition-review.py')))['audit']
    corpora = [
        ('elliptical-radii-public-static-review', 'elliptical-radii-public-recording-r3',
         'elliptical-radii-public-lifecycle', 'elliptical-radii-public-toolchain', 16),
        ('elliptical-radii-edge-dual-static-review', 'elliptical-radii-edge-dual-recording',
         'elliptical-radii-edge-dual-lifecycle', 'elliptical-radii-dual-clip-toolchain', 16),
        ('elliptical-radii-composition-native-static-review', 'elliptical-radii-composition-native-recording',
         'elliptical-radii-composition-native-lifecycle', 'elliptical-radii-dual-clip-toolchain', 52),
    ]
    refs, receipts = {}, []
    for review, recording, replay, toolchain, count in corpora:
        verify(root / review, root / recording, root / replay, root / toolchain, expected_scenes=count)
        for filename in ('visual-inspection.json', 'replay.json'):
            p = root / review / filename
            receipts.append(dict(path=str(p), sha256=sha(p)))
        fixtures = {f['name']: f for f in read(root / recording / 'lifecycle.json')['cases']}
        for row in read(root / replay / 'replay.json')['cases']:
            if row['instance'] != 0 or row['frame'] > 2:
                continue
            name = row['name'].rsplit('-0-step', 1)[0]
            key = name, row['width']
            assert key not in refs
            refs[key] = dict(row=row, fixture=fixtures[name], recording=root / recording)
    assert len(refs) == 252
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
        output = gallery / 'ellipse-lifecycle-comparison.json'
        if output.exists():
            assert read(output) == result
        else:
            output.write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(dict(reviewedPairs=result['reviewedPairs'], remaining=len(result['remaining']))))
