"""Transfer prior visual review only for identical compiler inputs and full PNG pairs.

Usage: python3 transfer-visual-review.py PREVIOUS_REPLAY CURRENT_REPLAY [--audit]
The source must have a complete, independently audited direct/within-run review.
Changed cases remain unreviewed; this script never declares a new inspection.
"""
import argparse
import hashlib
import json
from pathlib import Path
import runpy


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def canonical_input(folder, name, case=None):
    if case is not None and 'injectedGradient' in case:
        # Runtime diagnostics do not have per-view compiler request files.
        # Require their independently recompiled artifact audit, bound to this
        # exact replay, before comparing the full compile/installation identity.
        proof = json.loads((folder / 'gradient-artifact-audit.json').read_text())
        assert proof['status'] == 'passed'
        replay_path = folder / 'replay.json'
        assert any(Path(e['path']).resolve() == replay_path.resolve()
                   and e['sha256'] == sha(replay_path) for e in proof['evidence'])
        suffix = f"-{case['instance']}-step{case['frame']}"
        assert name.endswith(suffix)
        fixture_name = name[:-len(suffix)]
        artifact = next(a for a in proof['artifacts'] if a['name'] == fixture_name)
        value = {field: case[field] for field in [
            'html', 'css', 'compilerRequestCss', 'qualification',
            'injectedGradient', 'injectedPixelBounds', 'runtimeRequirements']}
        value['rivSha256'] = artifact['rivSha256']
        value['compileViewport'] = [390, 320]
    else:
        value = json.loads((folder / (name + '.json')).read_text())
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def transfer(previous, current, audit=False, combined_source=False):
    previous, current = Path(previous).resolve(), Path(current).resolve()
    if previous == current:
        raise ValueError('Use the within-run review tool for one replay')
    record = runpy.run_path(str(Path(__file__).with_name('record-visual-review.py')))['record']
    if combined_source:
        source_audit = runpy.run_path(str(Path(__file__).with_name('audit-combined-review.py')))['audit'](previous)
        source_receipt = previous / 'visual-coverage.json'
    else:
        source_audit = record(previous, audit=True)
        source_receipt = previous / 'visual-inspection.json'
    if source_audit['remaining']:
        raise ValueError('Source review must be complete')
    source = json.loads((previous / 'replay.json').read_text())['cases']
    target = json.loads((current / 'replay.json').read_text())['cases']
    by_key = {(c['name'], c['width']): c for c in source}
    if len({(c['name'], c['width']) for c in target}) != len(target):
        raise ValueError('Duplicate target case identity')
    transfers, remaining = [], []
    for case in target:
        if case['geometryFailures'] or case['failures']:
            raise ValueError('Failing comparisons require an explicit failure review')
        for kind in ('browser', 'native'):
            if sha(case['prefix'] + '.' + kind + '.png') != case[kind + 'Sha256']:
                raise ValueError('Changed target PNG: ' + case['name'])
        old = by_key.get((case['name'], case['width']))
        reason = None
        request = canonical_input(current, case['name'], case)
        if old is None:
            reason = 'no matching prior case'
        elif canonical_input(previous, case['name'], old) != request:
            reason = 'compiler input changed'
        elif any(old.get(key) != case.get(key) for key in
                 ('qualification', 'runtimeClipMarginExperiment', 'compilerRequestCss', 'css')):
            reason = 'browser CSS or runtime experiment changed'
        elif any(old[kind + 'Sha256'] != case[kind + 'Sha256'] for kind in ('browser', 'native')):
            reason = 'image pair changed'
        if reason:
            remaining.append(dict(name=case['name'], width=case['width'], reason=reason))
        else:
            transfers.append(dict(name=case['name'], width=case['width'],
                compilerInputSha256=hashlib.sha256(request).hexdigest(),
                browserSha256=case['browserSha256'], nativeSha256=case['nativeSha256']))
    result = dict(status='partial' if remaining else 'complete', sourceReplay=str(previous),
        sourceReplaySha256=sha(previous / 'replay.json'),
        sourceReceiptSha256=sha(source_receipt),
        targetReplaySha256=sha(current / 'replay.json'),
        exactSourceAndImageTransfers=transfers, remaining=remaining)
    if combined_source:
        result['sourceReceiptKind'] = 'combined'
    output = current / 'visual-transfer.json'
    if audit:
        if json.loads(output.read_text()) != result:
            raise ValueError('Stale transfer receipt')
    else:
        output.write_text(json.dumps(result, indent=2) + '\n')
    return dict(transferred=len(transfers), remaining=len(remaining))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('previous')
    parser.add_argument('current')
    parser.add_argument('--audit', action='store_true')
    parser.add_argument('--combined-source', action='store_true',
                        help='Revalidate a source whose coverage combines direct and transferred reviews')
    args = parser.parse_args()
    print(json.dumps(transfer(args.previous, args.current, args.audit, args.combined_source)))
