"""Transfer reviewed pixels from diagnostic gradients to public compilation.
Usage: SCRIPT SOURCE_REPLAY TARGET_REPLAY RECORDING COMPILER TOOLCHAIN [--audit]
Only authored HTML/CSS and identical complete PNG pairs inherit visual review.
Public compiler qualification is established independently by its artifact audit.
"""
import argparse
import hashlib
import json
from pathlib import Path
import runpy


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def transfer(source, target, recording, compiler, toolchain, audit=False):
    source, target, recording, compiler, toolchain = [Path(p).resolve() for p in (source, target, recording, compiler, toolchain)]
    assert source != target
    scripts = Path(__file__).parent
    coverage = runpy.run_path(str(scripts/'audit-combined-review.py'))['audit'](source)
    assert coverage['status'] == 'complete' and coverage['remaining'] == []
    source_proof_path = source/'gradient-artifact-audit.json'
    source_proof = read(source_proof_path)
    assert source_proof['status'] == 'passed'
    for binding in source_proof['evidence']:
        assert sha(binding['path']) == binding['sha256']
    assert any(Path(binding['path']).resolve() == source/'replay.json' for binding in source_proof['evidence'])
    # Recompile every public fixture now; do not inherit diagnostic transport.
    target_proof_path = target/'gradient-public-artifact-audit.json'
    fresh = runpy.run_path(str(scripts/'audit-public-gradient.py'))['audit'](recording, target, compiler, toolchain)
    assert fresh == read(target_proof_path), 'Public artifact audit is stale'
    old_rows, new_rows = read(source/'replay.json')['cases'], read(target/'replay.json')['cases']
    by_key = {(row['name'], row['width']): row for row in old_rows}
    assert len(by_key) == len(old_rows) == coverage['total'] == coverage['reviewed'] == 128
    assert len({(row['name'], row['width']) for row in new_rows}) == len(new_rows) == 128
    transferred, remaining = [], []
    for row in new_rows:
        assert row['qualification'] == 'public-compiler-lifecycle'
        assert not row['geometryFailures'] and not row['failures']
        old = by_key.get((row['name'], row['width']))
        assert old is not None and old['qualification'] == 'runtime-experiment-only'
        assert not old['geometryFailures'] and not old['failures']
        for candidate in (old, row):
            for kind in ('browser', 'native'):
                assert sha(candidate['prefix']+'.'+kind+'.png') == candidate[kind+'Sha256']
        fields = ('html', 'css', 'browserSha256', 'nativeSha256')
        if any(row[field] != old[field] for field in fields):
            remaining.append(dict(name=row['name'], width=row['width'], reason='Authored source or full image pair changed'))
            continue
        authored = json.dumps(dict(html=row['html'], css=row['css']), sort_keys=True, separators=(',', ':')).encode()
        transferred.append(dict(name=row['name'], width=row['width'], authoredSourceSha256=hashlib.sha256(authored).hexdigest(),
                                browserSha256=row['browserSha256'], nativeSha256=row['nativeSha256']))
    bindings = [source/'replay.json', source_proof_path, source/'visual-inspection.json', source/'visual-transfer.json', source/'visual-coverage.json',
                target/'replay.json', target_proof_path, recording/'lifecycle.json', toolchain/'manifest.json']
    result = dict(status='complete' if not remaining else 'partial', scope='Visual review transfer only; public compiler provenance independently audited',
                  newVisualInspections=0, sourceReplay=str(source), targetReplay=str(target), total=len(new_rows), reviewed=len(transferred),
                  exactAuthoredSourceAndImageTransfers=transferred, remaining=remaining,
                  evidence=[dict(path=str(path), sha256=sha(path)) for path in bindings])
    output = target/'visual-public-gradient-transfer.json'
    if audit:
        assert result == read(output), 'Visual transfer receipt is stale'
    else:
        output.write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('source', 'target', 'recording', 'compiler', 'toolchain'):
        parser.add_argument(name, type=Path)
    parser.add_argument('--audit', action='store_true')
    args = parser.parse_args()
    result = transfer(args.source, args.target, args.recording, args.compiler, args.toolchain, args.audit)
    print(json.dumps(dict(status=result['status'], transferred=result['reviewed'], remaining=len(result['remaining']), newVisualInspections=0)))
