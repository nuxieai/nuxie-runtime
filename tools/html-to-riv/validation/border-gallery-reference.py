"""Revalidate completed border lifecycle reviews for full-gallery comparison.

Usage: SCRIPT EVIDENCE_ROOT [COMPLETED_GALLERY]
Without a gallery, validate source evidence only. With one, require exact public
input, RIV/requirements and both full PNGs for each of the 192 border views.
"""
import hashlib
import json
from pathlib import Path
import runpy
import sys


def read(p):
    return json.loads(p.read_text())


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def references(root):
    here = Path(__file__).resolve().parent
    toolchain = root / 'border-sides-grouped-toolchain'
    initial = root / 'border-sides-grouped-initial-lifecycle'
    recording = root / 'border-sides-grouped-initial-recording'
    verify_initial = runpy.run_path(str(here / 'audit-border-side-public-review.py'))['audit']
    fresh = verify_initial(root / 'border-sides-miter-ray-static-review', recording, initial, toolchain)
    assert fresh == read(initial / 'public-visual-transfer.json')
    edge_review = root / 'border-sides-grouped-edge-static-review'
    edge_recording = root / 'border-sides-grouped-edge-recording'
    verify_edges = runpy.run_path(str(here / 'audit-border-edge-review-progress.py'))['audit']
    old_receipt = read(edge_review / 'prior-review-transfer.json')
    assert verify_edges(root / 'border-sides-v23-edge-static-review', edge_review,
                        root / 'border-sides-v23-edge-recording', edge_recording)['remaining'] == 0
    assert read(edge_review / 'prior-review-transfer.json') == old_receipt
    review = read(edge_review / 'visual-inspection.json')
    known = {(r['browserSha256'], r['nativeSha256']) for r in
             review['direct'] + old_receipt['exactInputArtifactImageTransfers']}
    edge = root / 'border-sides-grouped-edge-lifecycle'
    edge_replay = read(edge / 'replay.json')
    completion = read(root / 'border-sides-grouped-edge-completion.json')
    assert completion['status'] == 'complete'
    assert completion['sourceReplaySha256'] == sha(edge / 'replay.json')
    assert completion['recordingSha256'] == sha(edge_recording / 'lifecycle.json')
    assert completion['combinedReviewSha256'] == sha(edge_review / 'prior-review-transfer.json')
    assert completion['compilerSha256'] == sha(toolchain / 'html-to-riv')
    assert len(edge_replay['cases']) == 320
    for row in edge_replay['cases']:
        assert (row['browserSha256'], row['nativeSha256']) in known
        for kind in ('browser', 'native'):
            assert sha(Path(row['prefix'] + '.' + kind + '.png')) == row[kind + 'Sha256']
    refs = {}
    for replay_dir, record_dir in [(initial, recording), (edge, edge_recording)]:
        replay = read(replay_dir / 'replay.json')
        assert replay['browser'] == '153.0.8010.12'
        assert replay['lifecycleSha256'] == sha(record_dir / 'lifecycle.json')
        fixtures = {f['name']: f for f in read(record_dir / 'lifecycle.json')['cases']}
        for row in replay['cases']:
            assert not row['failures'] and not row['geometryFailures']
            assert row['qualification'] == 'public-compiler-lifecycle'
            if row['instance'] != 0 or row['frame'] > 2:
                continue
            name = row['name'].rsplit('-0-step', 1)[0]
            fixture = fixtures[name]
            assert not fixture['font'] and not fixture['image']
            for field in ('html', 'css', 'runtimeRequirements'):
                assert row[field] == fixture[field]
            key = name, row['width']
            assert key not in refs
            refs[key] = dict(row=row, recording=record_dir, fixture=fixture)
    assert len(refs) == 192
    return refs


def audit(root, gallery, refs=None):
    if refs is None:
        refs = references(root)
    report = read(gallery / 'review.json')
    assert report['status'] == 'passed' and not report['errors']
    assert len(report['checks']) == report['expected']
    assert all(c['status'] == 'passed' and not c['errors'] for c in report['checks'])
    seen, rows = set(), []
    for case in report['cases']:
        key = case['name'], case['width']
        if key not in refs:
            continue
        assert key not in seen
        assert case['status'] == 'passed' and not case['errors']
        assert case['metrics']['backend'] == 'rust-metal'
        seen.add(key)
        ref = refs[key]
        prefix = case['artifactPrefix']
        expected = dict(html=ref['fixture']['html'], css=ref['fixture']['css'], width=390, height=320)
        assert read(Path(prefix + '.json')) == expected
        assert case['html'] == expected['html'] and case['css'] == expected['css']
        assert sha(Path(prefix + '.riv')) == sha(ref['recording'] / (key[0] + '.riv'))
        assert read(Path(prefix + '.requirements.json')) == ref['fixture']['runtimeRequirements']
        hashes = {kind + 'Sha256': sha(Path(prefix + '.' + kind + '.png')) for kind in ('browser', 'native')}
        matches = all(value == ref['row'][field] for field, value in hashes.items())
        rows.append(dict(name=key[0], width=key[1], **hashes, reviewTransferred=matches))
    assert seen == set(refs)
    return dict(reviewSha256=sha(gallery / 'review.json'), rows=rows,
                reviewedPairs=sum(r['reviewTransferred'] for r in rows),
                remaining=[r for r in rows if not r['reviewTransferred']])


if __name__ == '__main__':
    root = Path(sys.argv[1]).resolve()
    if len(sys.argv) == 2:
        print(json.dumps(dict(sourceViewsVerified=len(references(root)))))
    else:
        gallery = Path(sys.argv[2]).resolve()
        result = audit(root, gallery)
        output = gallery / 'border-lifecycle-comparison.json'
        if output.exists():
            assert read(output) == result
        else:
            output.write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(dict(reviewedPairs=result['reviewedPairs'], remaining=len(result['remaining']))))
