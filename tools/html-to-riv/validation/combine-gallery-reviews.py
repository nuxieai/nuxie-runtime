"""Combine successful exact-image review transfers without dropping their proofs.

Usage: SCRIPT GALLERY COMPARISON_NAME [COMPARISON_NAME ...]
Every gallery pair must have a successful component row. Component records must
refer to the same unchanged completed gallery; disagreement is an error.
"""
import hashlib
import json
from pathlib import Path
import sys


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text())


def combine(gallery, names):
    report_path = gallery / 'review.json'
    report = read(report_path)
    assert report['status'] == 'passed' and not report['errors']
    assert len(report['checks']) == report['expected']
    assert all(c['status'] == 'passed' and not c['errors'] for c in report['checks'])
    cases = {(c['name'], c['width']): c for c in report['cases']}
    assert len(cases) == len(report['cases'])
    rows, proofs = {}, []
    for name in names:
        path = (gallery / name).resolve()
        data = read(path)
        assert data['reviewSha256'] == sha(report_path)
        seen = set()
        assert data['reviewedPairs'] == sum(r['reviewTransferred'] for r in data['rows'])
        for row in data['rows']:
            key = row['name'], row['width']
            assert key in cases and key not in seen
            seen.add(key)
            if not row['reviewTransferred']:
                continue
            if key in rows:
                for field in ('browserSha256', 'nativeSha256'):
                    assert rows[key][field] == row[field], 'Conflicting successful proofs'
                continue
            request = read(Path(cases[key]['artifactPrefix'] + '.json'))
            assert request['html'] == cases[key]['html'] and request['css'] == cases[key]['css']
            digest = hashlib.sha256(json.dumps(request, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
            assert row.get('compilerInputSha256', digest) == digest
            rows[key] = dict(row, compilerInputSha256=digest, sourceComparison=path.name)
        proofs.append(dict(path=str(path), sha256=sha(path)))
    assert set(rows) == set(cases), 'Missing visual coverage'
    result = dict(status='complete', scenePairs=len(cases), reviewedPairs=len(rows), remaining=[],
                  reviewSha256=sha(report_path), sourceComparisons=proofs,
                  rows=[rows[key] for key in cases])
    output = gallery / 'combined-visual-comparison.json'
    if output.exists():
        assert read(output) == result, 'Refusing to replace changed evidence'
    else:
        output.write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    gallery, *names = sys.argv[1:]
    assert names
    result = combine(Path(gallery), names)
    print(json.dumps(dict(status=result['status'], reviewedPairs=result['reviewedPairs'])))
