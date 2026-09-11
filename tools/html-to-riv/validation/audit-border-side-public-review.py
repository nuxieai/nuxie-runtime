"""Audit public border-side output against reviewed diagnostic images.

This narrow audit supports the asset-free 24-scene side corpus. It proves fresh
public artifact reproduction and exact authored HTML/CSS/full-image identity;
it does not assert that diagnostic injection was public compilation.
Usage: SCRIPT DIAGNOSTIC_STATIC_REVIEW PUBLIC_RECORDING PUBLIC_REPLAY TOOLCHAIN
"""
import hashlib
import json
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile


def read(p):
    return json.loads(p.read_text())


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def audit(review_dir, recording_dir, replay_dir, toolchain):
    record = runpy.run_path(str(Path(__file__).with_name('record-visual-review.py')))['record']
    assert record(review_dir, audit=True)['remaining'] == 0
    projection = read(review_dir / 'replay.json')
    diagnostic_path = Path(projection['sourceReplay'])
    assert sha(diagnostic_path) == projection['sourceReplaySha256']
    diagnostic = read(diagnostic_path)
    old = {r['name']: r for r in diagnostic['cases']}
    assert len(old) == len(diagnostic['cases'])
    reviewed = read(review_dir / 'visual-inspection.json')
    known = {(r['browserSha256'], r['nativeSha256']) for r in reviewed['direct']}
    for row in projection['cases']:
        original = dict(row)
        original['name'] = original.pop('originalName')
        assert original == old[original['name']]
    lifecycle_path = recording_dir / 'lifecycle.json'
    lifecycle = read(lifecycle_path)
    public = read(replay_dir / 'replay.json')
    assert lifecycle['kind'] == 'border-public'
    assert public['browser'] == diagnostic['browser'] == '153.0.8010.12'
    assert public['lifecycleSha256'] == sha(lifecycle_path)
    manifest = read(toolchain / 'manifest.json')
    compiler = toolchain / 'html-to-riv'
    assert sha(compiler) == manifest['files']['html-to-riv']['sha256']
    frames = {r['name']: r for r in public['cases']}
    assert len(frames) == len(public['cases']) == len(old) == 192
    seen, artifacts = set(), []
    with tempfile.TemporaryDirectory(prefix='border-side-public-audit-') as tmp:
        tmp = Path(tmp)
        for fixture in lifecycle['cases']:
            name = fixture['name']
            assert not fixture['font'] and not fixture['image']
            request = dict(html=fixture['html'], css=fixture['css'], width=390, height=320, assets={})
            (tmp / 'input.json').write_text(json.dumps(request))
            subprocess.run([str(compiler), str(tmp / 'input.json'), str(tmp / 'scene.riv')], check=True)
            riv = recording_dir / (name + '.riv')
            requirements = recording_dir / (name + '.requirements.json')
            assert sha(tmp / 'scene.riv') == sha(riv)
            assert read(tmp / 'scene.requirements.json') == read(requirements) == fixture['runtimeRequirements']
            artifacts.append(dict(name=name, rivSha256=sha(riv), requirementsSha256=sha(requirements)))
            for view in fixture['views']:
                key = f"{name}-{view['instance']}-step{view['frame']}"
                assert key not in seen
                seen.add(key)
                row, prior = frames[key], old[key]
                assert row['qualification'] == 'public-compiler-lifecycle'
                assert not row['failures'] and not row['geometryFailures']
                assert row['html'] == prior['html'] == fixture['html']
                assert row['css'] == prior['css'] == fixture['css']
                assert row['runtimeRequirements'] == fixture['runtimeRequirements']
                assert row['width'] == prior['width'] == view['width']
                assert row['frame'] == prior['frame'] == view['frame']
                assert row['instance'] == prior['instance'] == view['instance']
                assert Path(view['stream']).is_absolute()
                assert sha(Path(view['stream'])) == row['streamSha256']
                pair = row['browserSha256'], row['nativeSha256']
                assert pair in known
                for kind in ('browser', 'native'):
                    assert row[kind + 'Sha256'] == prior[kind + 'Sha256']
                    for image in (row, prior):
                        assert sha(Path(image['prefix'] + '.' + kind + '.png')) == image[kind + 'Sha256']
    assert seen == set(frames)
    return dict(status='complete', qualification='public-border-side-focused-review', framesReviewed=len(seen),
                compilerSha256=sha(compiler), diagnosticReplaySha256=sha(diagnostic_path),
                directReviewSha256=sha(review_dir / 'visual-inspection.json'),
                publicReplaySha256=sha(replay_dir / 'replay.json'), lifecycleSha256=sha(lifecycle_path),
                artifacts=artifacts, limitation='Focused corpus only; broader P02 qualification remains open')


if __name__ == '__main__':
    args = [Path(p).resolve() for p in sys.argv[1:]]
    assert len(args) == 4
    result = audit(*args)
    (args[2] / 'public-visual-transfer.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(status=result['status'], framesReviewed=result['framesReviewed'])))
