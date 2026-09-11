"""Revalidate partial public border reviews across a runtime correction.

Usage: SCRIPT SOURCE_REVIEW TARGET_REVIEW SOURCE_RECORDING TARGET_RECORDING
Only individually reviewed source rows with identical authored input, manifest,
Rive bytes and both full PNGs can transfer. Changed rows remain unreviewed.
"""
from pathlib import Path
import hashlib
import json
import runpy
import sys


def read(p):
    return json.loads(p.read_text())


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def audit(source, target, source_recording, target_recording):
    record = runpy.run_path(str(Path(__file__).with_name('record-visual-review.py')))['record']
    indexes = []
    for folder, recording in [(source, source_recording), (target, target_recording)]:
        record(folder, audit=True)
        projection = read(folder / 'replay.json')
        full_path = Path(projection['sourceReplay'])
        assert sha(full_path) == projection['sourceReplaySha256']
        full = read(full_path)
        assert full['lifecycleSha256'] == sha(recording / 'lifecycle.json')
        assert read(recording / 'lifecycle.json')['kind'] == 'border-public'
        by_name = {r['name']: r for r in full['cases']}
        assert len(by_name) == len(full['cases'])
        rows = {}
        for row in projection['cases']:
            original = dict(row)
            original['name'] = original.pop('originalName')
            assert original == by_name[original['name']]
            assert row['qualification'] == 'public-compiler-lifecycle'
            key = row['name'], row['width']
            assert key not in rows
            rows[key] = row
        indexes.append(rows)
    old, new = indexes
    reviewed_source = {(r['name'], r['width']) for r in read(source / 'visual-inspection.json')['direct']}
    transfers = []
    for key, row in new.items():
        if key not in reviewed_source:
            continue
        prior = old[key]
        if any(prior[k] != row[k] for k in ['html', 'css', 'runtimeRequirements', 'browserSha256', 'nativeSha256']):
            continue
        for suffix in ['.riv', '.requirements.json']:
            assert sha(source_recording / (key[0] + suffix)) == sha(target_recording / (key[0] + suffix))
        for kind in ['browser', 'native']:
            for image in [prior, row]:
                assert sha(Path(image['prefix'] + '.' + kind + '.png')) == image[kind + 'Sha256']
        transfers.append(dict(name=key[0], width=key[1], browserSha256=row['browserSha256'], nativeSha256=row['nativeSha256']))
    review = read(target / 'visual-inspection.json')
    covered = {(r['name'], r['width']) for r in review['direct'] + review['exactImageTransfers'] + transfers}
    remaining = [dict(name=k[0], width=k[1]) for k in new if k not in covered]
    result = dict(status='partial' if remaining else 'complete', sourceReviewSha256=sha(source / 'visual-inspection.json'),
                  sourceReplaySha256=sha(source / 'replay.json'), targetReplaySha256=sha(target / 'replay.json'),
                  targetDirectReviewSha256=sha(target / 'visual-inspection.json'),
                  exactInputArtifactImageTransfers=transfers, reviewed=len(covered), remaining=remaining)
    (target / 'prior-review-transfer.json').write_text(json.dumps(result, indent=2) + '\n')
    return dict(reviewed=len(covered), remaining=len(remaining), transfers=len(transfers))


if __name__ == '__main__':
    args = [Path(p).resolve() for p in sys.argv[1:]]
    assert len(args) == 4
    print(json.dumps(audit(*args)))
