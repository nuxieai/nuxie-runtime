"""Audit coverage combining direct/within-run and cross-run visual reviews.

Both underlying receipts are revalidated against current inputs and images.
This records coverage only; it does not inspect images or waive failures.
"""
import argparse
import hashlib
import json
from pathlib import Path
import runpy


def audit(folder):
    folder = Path(folder).resolve()
    scripts = Path(__file__).parent
    direct_path = folder / 'visual-inspection.json'
    transfer_path = folder / 'visual-transfer.json'
    direct = json.loads(direct_path.read_text())
    transferred = json.loads(transfer_path.read_text())
    runpy.run_path(str(scripts / 'record-visual-review.py'))['record'](folder, audit=True)
    runpy.run_path(str(scripts / 'transfer-visual-review.py'))['transfer'](
        transferred['sourceReplay'], folder, audit=True,
        combined_source=transferred.get('sourceReceiptKind') == 'combined')
    cases = json.loads((folder / 'replay.json').read_text())['cases']
    keys = lambda rows: {(r['name'], r['width']) for r in rows}
    expected = keys(cases)
    inspected = keys(direct['direct'])
    within = keys(direct['exactImageTransfers'])
    cross = keys(transferred['exactSourceAndImageTransfers'])
    covered = inspected | within | cross
    if covered - expected:
        raise ValueError('Review includes unknown cases')
    result = dict(
        status='complete' if covered == expected else 'partial',
        total=len(expected), reviewed=len(covered),
        direct=len(inspected), withinRunTransfers=len(within),
        crossRunTransfers=len(cross),
        overlappingCoverage=len(inspected) + len(within) + len(cross) - len(covered),
        remaining=[dict(name=n, width=w) for n, w in sorted(expected - covered)],
        evidenceSha256={p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in [folder / 'replay.json', direct_path, transfer_path]})
    (folder / 'visual-coverage.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('folder', type=Path)
    args = parser.parse_args()
    print(json.dumps(audit(args.folder)))
