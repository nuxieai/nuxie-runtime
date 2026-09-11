"""Record human image inspection, or audit an existing replay review receipt.

This does not inspect images. Invoke only after viewing each named review sheet.
Exact-image transfers apply within this replay; cross-run transfers additionally
need source identity and are intentionally a separate operation.
"""
from pathlib import Path
import argparse
import hashlib
import json


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def record(out, names=(), notes=None, audit=False):
    out = Path(out)
    receipt_path = out / 'visual-inspection.json'
    receipt = json.loads(receipt_path.read_text()) if receipt_path.exists() else {'direct': []}
    cases = json.loads((out / 'replay.json').read_text())['cases']
    by_key = {(c['name'], c['width']): c for c in cases}
    if len(by_key) != len(cases):
        raise ValueError('Duplicate replay case identity')
    unknown = set(names) - {c['name'] for c in cases}
    if unknown:
        raise ValueError(f'Unknown review cases: {sorted(unknown)}')
    if not audit and (not names or not notes or not notes.strip()):
        raise ValueError('Recording requires case names and inspection notes')
    if audit and not receipt_path.exists():
        raise ValueError('No review receipt to audit')
    for c in cases:
        if c['geometryFailures'] or c['failures']:
            raise ValueError('Failing comparisons need an explicit failure-review receipt')
        for kind in ('browser', 'native'):
            if sha(c['prefix'] + '.' + kind + '.png') != c[kind + 'Sha256']:
                raise ValueError(f'Changed {kind} image: {c["name"]}/{c["width"]}')
    direct = {}
    for row in receipt['direct']:
        key = row['name'], row['width']
        if key in direct or key not in by_key:
            raise ValueError(f'Invalid direct review identity: {key}')
        case = by_key[key]
        for kind in ('browser', 'native'):
            if row[kind + 'Sha256'] != case[kind + 'Sha256']:
                raise ValueError(f'Stale direct review: {key}')
        if sha(out / 'review-sheets' / (row['name'] + '.png')) != row['sheetSha256']:
            raise ValueError(f'Changed reviewed sheet: {key}')
        direct[key] = row
    for key, case in by_key.items():
        if case['name'] in names and key not in direct:
            direct[key] = dict(name=case['name'], width=case['width'],
                               browserSha256=case['browserSha256'], nativeSha256=case['nativeSha256'],
                               sheetSha256=sha(out / 'review-sheets' / (case['name'] + '.png')), notes=notes)
    sources = {(r['browserSha256'], r['nativeSha256']): r for r in direct.values()}
    transfers, remaining = [], []
    for key, case in by_key.items():
        if key in direct:
            continue
        source = sources.get((case['browserSha256'], case['nativeSha256']))
        if source:
            transfers.append(dict(name=case['name'], width=case['width'], sourceName=source['name'],
                                  sourceWidth=source['width'], browserSha256=case['browserSha256'],
                                  nativeSha256=case['nativeSha256']))
        else:
            remaining.append(dict(name=case['name'], width=case['width']))
    updated = dict(direct=list(direct.values()), exactImageTransfers=transfers, remaining=remaining,
                   reviewed=len(direct) + len(transfers), status='partial' if remaining else 'complete')
    if audit:
        if updated != receipt:
            raise ValueError('Receipt counts or transfers do not match verified evidence')
    else:
        temp = receipt_path.with_suffix('.json.tmp')
        temp.write_text(json.dumps(updated, indent=2) + '\n')
        temp.replace(receipt_path)
    return {'reviewed': updated['reviewed'], 'remaining': len(remaining)}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('names', nargs='*')
    parser.add_argument('--notes')
    parser.add_argument('--audit', action='store_true')
    args = parser.parse_args()
    if args.audit and (args.names or args.notes):
        parser.error('--audit cannot add reviews')
    print(json.dumps(record(args.output, args.names, args.notes, args.audit)))
