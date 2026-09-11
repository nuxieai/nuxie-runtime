"""Audit reviewed diagnostic clip margins promoted to the public compiler.

Unlike ordinary exact-input transfer, this proves the one intentional request
change: removing diagnostic injection and carrying it in the checked manifest.
Usage: python3 promote-clip-margin-review.py EXPERIMENT PUBLIC [--audit]
"""
import hashlib
import json
from pathlib import Path
import runpy
import sys


def read(path):
    return json.loads(Path(path).read_text())


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def promote(source, target, audit=False):
    source, target = Path(source).resolve(), Path(target).resolve()
    review = runpy.run_path(str(Path(__file__).with_name('record-visual-review.py')))['record'](source, audit=True)
    assert not review['remaining'], 'Diagnostic visual review must be complete'
    old_report, new_report = read(source / 'replay.json'), read(target / 'replay.json')
    old = {(c['name'], c['width']): c for c in old_report['cases']}
    assert len(old) == len(old_report['cases'])
    rows = []
    for case in new_report['cases']:
        prior = old[(case['name'], case['width'])]
        assert prior['qualification'] == 'runtime-experiment-only'
        assert 'qualification' not in case and 'runtimeClipMarginExperiment' not in case
        assert not case['failures'] and not case['geometryFailures']
        assert case['html'] == prior['html'] and case['css'] == prior['css']
        request = read(target / (case['name'] + '.json'))
        experiment_request = read(source / (case['name'] + '.json'))
        assert experiment_request['css'] == prior['compilerRequestCss']
        assert request['css'] == case['css']
        experiment_request['css'] = prior['css']
        assert request == experiment_request, 'Only the recorded CSS declaration may change'
        requirements = read(target / (case['name'] + '.requirements.json'))
        old_requirements = read(source / (case['name'] + '.requirements.json'))
        injection = prior['runtimeClipMarginExperiment']
        expected = []
        if injection is not None:
            source_id, origin, pixels = injection
            # Explicit padding-box zero has the ordinary default clip semantics.
            if origin != 'padding-box' or pixels != 0:
                mapping = read(target / (case['name'] + '.map.json'))
                ids = [node['object_id'] for node in mapping if node['id'] == source_id]
                assert len(ids) == 1
                expected = [{'object_id': ids[0], 'origin': origin, 'pixels': pixels}]
        assert requirements.get('layout_overflow_clip_margins', []) == expected
        capability = 'layout-css-overflow-clip-margin-v1'
        assert (capability in requirements['capabilities']) == bool(expected)
        if expected:
            assert requirements['version'] == 21
            requirements.pop('layout_overflow_clip_margins')
            requirements['capabilities'].remove(capability)
            requirements['version'] = old_requirements['version']
        assert requirements == old_requirements, 'No other runtime policy may change'
        hashes = {}
        for kind in ['browser', 'native']:
            current_hash = sha(case['prefix'] + '.' + kind + '.png')
            old_hash = sha(prior['prefix'] + '.' + kind + '.png')
            assert current_hash == case[kind + 'Sha256'] == old_hash == prior[kind + 'Sha256']
            hashes[kind + 'Sha256'] = current_hash
        rows.append({'name': case['name'], 'width': case['width'], **hashes})
    assert len({(r['name'], r['width']) for r in rows}) == len(rows)
    assert len(rows) == len(old), 'Promotion must cover the complete diagnostic corpus'
    result = {'status': 'complete', 'reviewed': len(rows), 'source': str(source),
              'sourceReplaySha256': sha(source / 'replay.json'),
              'sourceVisualSha256': sha(source / 'visual-inspection.json'),
              'targetReplaySha256': sha(target / 'replay.json'),
              'method': 'Exact browser CSS and full PNG pairs; diagnostic policy matches public manifest; all other compiler inputs and runtime policies unchanged.',
              'rows': rows}
    output = target / 'visual-promotion.json'
    if audit:
        assert read(output) == result
    else:
        assert not output.exists(), 'Refusing to overwrite evidence'
        output.write_text(json.dumps(result, indent=2) + '\n')
    return {'status': result['status'], 'reviewed': len(rows)}


if __name__ == '__main__':
    print(json.dumps(promote(sys.argv[1], sys.argv[2], '--audit' in sys.argv[3:])))
