"""Transfer exact full static composition image reviews to clone/resize frames.
Usage: SCRIPT STATIC_REPLAY LIFECYCLE_REPLAY RECORDING COMPILER TOOLCHAIN
Rebuilds static evidence once and independently audits the lifecycle. No new inspections.
"""
import argparse
import hashlib
import json
from pathlib import Path
import runpy
from unittest.mock import patch


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(source, target, recording, compiler, toolchain, static_proof):
    scripts = Path(__file__).parent
    assert static_proof == read(source/'composition-artifact-audit.json')
    assert static_proof['replaySha256'] == sha(source/'replay.json')
    assert static_proof['visualInspectionSha256'] == sha(source/'composition-visual-inspection.json')
    lifecycle = runpy.run_path(str(scripts/'audit-gradient-composition-lifecycle.py'))['audit'](recording, target, compiler, toolchain)
    assert lifecycle == read(target/'gradient-composition-lifecycle-audit.json')
    old = {(r['name'], r['width']): r for r in read(source/'replay.json')['cases']}
    rows = read(target/'replay.json')['cases']
    assert len(old) == 54 and len(rows) == 144
    transfers = []
    seen = set()
    for row in rows:
        suffix = f"-{row['instance']}-step{row['frame']}"
        assert row['name'].endswith(suffix)
        source_name = row['name'][:-len(suffix)]
        prior = old[(source_name, row['width'])]
        assert row['qualification'] == 'public-compiler-lifecycle'
        assert not row['failures'] and not row['geometryFailures']
        for field in ('html', 'css', 'browserSha256', 'nativeSha256'):
            assert row[field] == prior[field], (row['name'], field)
        for candidate in (prior, row):
            for kind in ('browser', 'native'):
                assert sha(Path(candidate['prefix']+'.'+kind+'.png')) == candidate[kind+'Sha256']
        key = (row['name'], row['width'])
        assert key not in seen
        seen.add(key)
        transfers.append(dict(name=row['name'], width=row['width'], sourceName=source_name,
                              browserSha256=row['browserSha256'], nativeSha256=row['nativeSha256']))
    bindings = [source/'replay.json', source/'composition-artifact-audit.json', source/'composition-visual-inspection.json',
                target/'replay.json', target/'gradient-composition-lifecycle-audit.json', recording/'lifecycle.json', toolchain/'manifest.json']
    return dict(status='complete', scope='Visual review transfer only: exact authored HTML/CSS and complete PNG pairs',
                total=144, reviewed=144, remaining=[], newVisualInspections=0, exactImageTransfers=transfers,
                evidence=[dict(path=str(p), sha256=sha(p)) for p in bindings])


def run(source, target, recording, compiler, toolchain):
    source, target, recording, compiler, toolchain = [Path(p).resolve() for p in (source, target, recording, compiler, toolchain)]
    scripts = Path(__file__).parent
    static_proof = runpy.run_path(str(scripts/'audit-gradient-composition.py'))['audit'](source)
    result = verify(source, target, recording, compiler, toolchain, static_proof)
    (target/'visual-composition-lifecycle-transfer.json').write_text(json.dumps(result, indent=2)+'\n')
    # Rebuild/review all54 static views only once. Each negative control still
    # runs the fresh lifecycle auditor; source-proof tampering fails upfront.
    controls = [
        ('stale-review-source', source/'composition-artifact-audit.json', lambda d: d.__setitem__('visualInspectionSha256', '0'*64)),
        ('stale-lifecycle', target/'gradient-composition-lifecycle-audit.json', lambda d: d['evidence'][2].__setitem__('sha256', '0'*64)),
    ]
    rejected = []
    original = read
    for name, path, mutate in controls:
        touched = []
        def changed(p):
            value = original(p)
            if p == path:
                mutate(value); touched.append(p)
            return value
        with patch.dict(verify.__globals__, read=changed):
            try:
                verify(source, target, recording, compiler, toolchain, static_proof)
            except AssertionError:
                assert touched
                rejected.append(dict(name=name, rejected=True))
            else:
                raise AssertionError('Mutation accepted: '+name)
    module = runpy.run_path(str(scripts/'audit-gradient-composition-lifecycle.py'))
    audit = module['audit']
    original = audit.__globals__['read']
    for name, path, mutate in [
        ('changed-emitted-paint', recording/'lifecycle.json', lambda d: d['cases'][0]['runtimeRequirements']['layout_linear_gradients'][0]['stops'][0].__setitem__('color', 0)),
        ('changed-stream', target/'replay.json', lambda d: d['cases'][0].__setitem__('streamSha256', '0'*64)),
    ]:
        touched = []
        def changed(p):
            value = original(p)
            if p == path:
                mutate(value); touched.append(p)
            return value
        with patch.dict(audit.__globals__, read=changed):
            try:
                audit(recording, target, compiler, toolchain)
            except AssertionError:
                assert touched
                rejected.append(dict(name=name, rejected=True))
            else:
                raise AssertionError('Mutation accepted: '+name)
    (target/'visual-composition-lifecycle-transfer-controls.json').write_text(json.dumps(dict(status='passed', controls=rejected), indent=2)+'\n')
    return dict(status=result['status'], transferred=result['reviewed'], controls=len(rejected), newVisualInspections=0)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('source', 'target', 'recording', 'compiler', 'toolchain'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    print(json.dumps(run(args.source, args.target, args.recording, args.compiler, args.toolchain)))
