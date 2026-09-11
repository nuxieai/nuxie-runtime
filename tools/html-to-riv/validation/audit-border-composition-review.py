"""Verify reviewed public composition frames and reproduce embedded-asset scenes.

Usage: SCRIPT STATIC_REVIEW RECORDING REPLAY TOOLCHAIN
The receipt qualifies this corpus only, not the entire border feature.
"""
import hashlib
import json
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(review_dir, recording_dir, replay_dir, toolchain, expected_scenes=48):
    validation = Path(__file__).resolve().parent
    record = runpy.run_path(str(validation / 'record-visual-review.py'))['record']
    assert record(review_dir, audit=True)['remaining'] == 0
    projection = read(review_dir / 'replay.json')
    replay_path = replay_dir / 'replay.json'
    assert Path(projection['sourceReplay']).resolve() == replay_path.resolve()
    assert projection['sourceReplaySha256'] == sha(replay_path)
    replay = read(replay_path)
    lifecycle_path = recording_dir / 'lifecycle.json'
    lifecycle = read(lifecycle_path)
    assert lifecycle['kind'] == 'border-public'
    assert replay['browser'] == '153.0.8010.12'
    assert replay['lifecycleSha256'] == sha(lifecycle_path)
    manifest = read(toolchain / 'manifest.json')
    for binary in ('html-to-riv', 'renderer-replay'):
        assert sha(toolchain / binary) == manifest['files'][binary]['sha256']
    assert replay['rendererSha256'] == sha(toolchain / 'renderer-replay')
    frames = {r['name']: r for r in replay['cases']}
    assert expected_scenes > 0
    assert len(frames) == len(replay['cases']) == expected_scenes * 8
    assert len(lifecycle['cases']) == expected_scenes
    assert len(projection['cases']) == expected_scenes * 3
    for row in projection['cases']:
        original = dict(row)
        original['name'] = original.pop('originalName')
        assert original == frames[original['name']]
    reviewed = read(review_dir / 'visual-inspection.json')
    known = {(r['browserSha256'], r['nativeSha256']) for r in reviewed['direct']}
    seen, names, artifacts = set(), set(), []
    with tempfile.TemporaryDirectory(prefix='border-composition-audit-') as tmp:
        tmp = Path(tmp)
        for fixture in lifecycle['cases']:
            name = fixture['name']
            assert name not in names
            names.add(name)
            assets = {}
            if fixture['font']:
                assets['inter'] = dict(kind='font', family='Inter', weight=400,
                    bytes=list((validation.parent / 'tests/assets/Inter-Regular.ttf').read_bytes()))
            if fixture['image']:
                assets['photo'] = dict(kind='image',
                    bytes=list((validation.parent / 'tests/assets/quadrants.png').read_bytes()))
            request = dict(html=fixture['html'], css=fixture['css'], width=390, height=320, assets=assets)
            (tmp / 'input.json').write_text(json.dumps(request))
            subprocess.run([str(toolchain / 'html-to-riv'), str(tmp / 'input.json'), str(tmp / 'scene.riv')],
                           check=True, stdout=subprocess.DEVNULL)
            riv = recording_dir / (name + '.riv')
            requirements = recording_dir / (name + '.requirements.json')
            assert sha(tmp / 'scene.riv') == sha(riv)
            assert read(tmp / 'scene.requirements.json') == read(requirements) == fixture['runtimeRequirements']
            artifacts.append(dict(name=name, rivSha256=sha(riv), requirementsSha256=sha(requirements),
                                  requestSha256=sha(tmp / 'input.json')))
            assert {(v['instance'], v['frame'], v['width'], v['height']) for v in fixture['views']} == {
                (i, i * 4 + f, w, 320) for i in (0, 1) for f, w in enumerate((240, 390, 768, 240))}
            assert len(fixture['views']) == 8
            for view in fixture['views']:
                key = f"{name}-{view['instance']}-step{view['frame']}"
                assert key not in seen
                seen.add(key)
                row = frames[key]
                assert row['qualification'] == 'public-compiler-lifecycle'
                assert not row['failures'] and not row['geometryFailures']
                for field in ('html', 'css', 'runtimeRequirements'):
                    assert row[field] == fixture[field]
                for field in ('width', 'instance', 'frame'):
                    assert row[field] == view[field]
                assert Path(view['stream']).is_absolute()
                assert sha(Path(view['stream'])) == row['streamSha256']
                assert (row['browserSha256'], row['nativeSha256']) in known
                for kind in ('browser', 'native'):
                    assert sha(Path(row['prefix'] + '.' + kind + '.png')) == row[kind + 'Sha256']
    assert seen == set(frames)
    return dict(status='complete', qualification='public-border-composition-lifecycle',
                framesReviewed=len(seen), scenesReproduced=len(artifacts),
                compilerSha256=sha(toolchain / 'html-to-riv'), replaySha256=sha(replay_path),
                lifecycleSha256=sha(lifecycle_path), reviewSha256=sha(review_dir / 'visual-inspection.json'),
                artifacts=artifacts, limitation='Focused composition corpus only; full P02 qualification remains open')


if __name__ == '__main__':
    args = [Path(p).resolve() for p in sys.argv[1:]]
    assert len(args) == 4
    result = audit(*args)
    (args[2] / 'public-composition-audit.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: result[k] for k in ('status', 'framesReviewed', 'scenesReproduced')}))
