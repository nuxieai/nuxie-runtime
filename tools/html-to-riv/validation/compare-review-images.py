"""Compare a finished visual review with explicit prior reviewed runs.

Usage: python3 compare-review-images.py CURRENT BASELINE [BASELINE ...]
Later baselines take precedence. Source HTML/CSS must match before image identity
can transfer earlier review evidence; missing/changed cases require inspection.
"""
import hashlib
import json
import sys
from pathlib import Path

current = Path(sys.argv[1])
baselines = {}
for directory in sys.argv[2:]:
    for case in json.loads((Path(directory) / 'review.json').read_text())['cases']:
        baselines[(case['name'], case['width'], case['height'])] = (directory, case)

def digest(case, kind):
    image = Path(case['artifactPrefix'] + '.' + kind + '.png')
    return hashlib.sha256(image.read_bytes()).hexdigest() if image.exists() else None

rows = []
for case in json.loads((current / 'review.json').read_text())['cases']:
    row = {'name': case['name'], 'width': case['width'], 'status': case['status']}
    baseline = baselines.get((case['name'], case['width'], case['height']))
    if baseline:
        directory, previous = baseline
        row['baseline'] = directory
        row['sourceIdentical'] = all(case.get(k) == previous.get(k) for k in ['html', 'css'])
        for kind in ['browser', 'native']:
            before, after = digest(previous, kind), digest(case, kind)
            row[kind] = {'before': before, 'after': after, 'identical': before is not None and before == after}
    else:
        row['baseline'] = None
    row['reviewTransferred'] = bool(baseline and row['sourceIdentical'] and all(row[k]['identical'] for k in ['browser', 'native']))
    rows.append(row)
report = {'cases': len(rows), 'identical': sum(r['reviewTransferred'] for r in rows), 'needsInspection': [r for r in rows if not r['reviewTransferred']], 'rows': rows}
(current / 'baseline-comparison.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k: v for k, v in report.items() if k != 'rows'}, indent=2))
