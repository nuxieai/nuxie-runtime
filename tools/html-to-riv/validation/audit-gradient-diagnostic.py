"""Reprove experimental gradient input/artifact/lifecycle identity.
Usage: SCRIPT RECORDING REPLAY COMPILER TOOLCHAIN
Visual coverage is reported separately; this never promotes public CSS.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(recording, replay, compiler, toolchain):
    oracle_path = Path(__file__).parent.parent/'tests/assets/linear-gradient-initial-oracle.json'
    oracle = read(oracle_path)
    source = read(recording/'lifecycle.json')
    result = read(replay/'replay.json')
    manifest = read(toolchain/'manifest.json')
    assert source['kind'] == 'gradient-diagnostic'
    assert result['browser'] == source['browser'] == oracle['browser'] == '153.0.8010.12'
    assert result['lifecycleSha256'] == sha(recording/'lifecycle.json')
    assert result['rendererSha256'] == manifest['files']['renderer-replay']['sha256'] == sha(toolchain/'renderer-replay')
    known = {c['name']: c for c in oracle['cases']}
    fixtures = {c['name']: c for c in source['cases']}
    frames = {c['name']: c for c in result['cases']}
    assert len(known) == len(oracle['cases']) == len(source['cases']) == len(fixtures)
    assert fixtures.keys() == known.keys()
    assert len(frames) == len(result['cases']) == len(known)*8
    seen = set()
    artifacts = []
    with tempfile.TemporaryDirectory() as directory:
        tmp = Path(directory)
        for name, fixture in fixtures.items():
            expected = known[name]
            assert fixture['html'] == expected['html'] and fixture['css'] == expected['css']
            assert fixture['injectedGradient'] == expected['diagnosticGradient']
            assert not fixture['font'] and not fixture['image']
            css = expected['css']
            start = css.index('background:linear-gradient(')
            end = css.index(';', start)+1
            stripped = css[:start]+css[end:]
            assert fixture['compilerRequestCss'] == stripped
            request = dict(html=expected['html'], css=stripped, width=390, height=320)
            (tmp/'input.json').write_text(json.dumps(request))
            subprocess.run([str(compiler),str(tmp/'input.json'),str(tmp/'scene.riv')],check=True,stdout=subprocess.DEVNULL)
            assert sha(tmp/'scene.riv') == sha(recording/f'{name}.riv')
            requirements = read(tmp/'scene.requirements.json')
            assert requirements == fixture['runtimeRequirements']
            ids = {n['id']:n['object_id'] for n in read(tmp/'scene.map.json')}
            assert fixture['injectedPixelBounds'] == requirements.get('layout_pixel_bounds',[])+[ids['gradient']]
            views = fixture['views']
            assert len(views) == 8
            assert {(v['instance'],v['frame'],v['width'],v['height']) for v in views} == {
                (i,i*4+f,w,320) for i in [0,1] for f,w in enumerate([240,390,768,240])}
            for view in views:
                key = f"{name}-{view['instance']}-step{view['frame']}"
                assert key not in seen
                seen.add(key)
                row = frames[key]
                assert row['qualification'] == 'runtime-experiment-only'
                assert not row['failures'] and not row['geometryFailures']
                for field in ['html','css','compilerRequestCss','injectedGradient','injectedPixelBounds','runtimeRequirements']:
                    assert row[field] == fixture[field], (key,field)
                for field in ['frame','instance','width']:
                    assert row[field] == view[field]
                stream = Path(view['stream'])
                assert row['streamSha256'] == sha(stream)
                text = stream.read_text()
                assert text.splitlines().count('frame') == view['frame']+1
                assert 'makePremultipliedLinearGradient ' in text
                assert 'makeLinearGradient ' not in text
                reference = next(v for v in expected['viewports'] if v['width']==view['width'])
                assert view['bounds'].keys() == reference['boxes'].keys()
                for node, box in view['bounds'].items():
                    for axis in ['x','y','width','height']:
                        assert abs(box[axis]-reference['boxes'][node][axis]) < .1
                for kind in ['browser','native']:
                    assert row[kind+'Sha256'] == sha(Path(row['prefix']+'.'+kind+'.png'))
            artifacts.append(dict(name=name,rivSha256=sha(tmp/'scene.riv')))
    assert seen == frames.keys()
    return dict(status='passed',scope='Experimental runtime injection only; visual coverage separate',scenes=len(known),frames=len(frames),artifacts=artifacts,
        evidence=[dict(path=str(p),sha256=sha(p)) for p in [oracle_path,recording/'lifecycle.json',replay/'replay.json',compiler,toolchain/'manifest.json']])


if __name__ == '__main__':
    recording,replay,compiler,toolchain = [Path(p).resolve() for p in sys.argv[1:]]
    result = audit(recording,replay,compiler,toolchain)
    (replay/'gradient-artifact-audit.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(status=result['status'],scenes=result['scenes'],frames=result['frames'])))
