"""Bind public gradient composition compilation, original/clone recordings and native replay.
Usage: SCRIPT RECORDING REPLAY COMPILER TOOLCHAIN
This checks artifact provenance and numerical gates, not visual inspection.
"""
import hashlib
import json
import math
from pathlib import Path
import re
import struct
import subprocess
import sys
import tempfile


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def png_size(path):
    data = path.read_bytes()
    assert data[:16] == b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR'
    return struct.unpack('>II', data[16:24])


def audit(recording, replay, compiler, toolchain):
    oracle_path = Path(__file__).parent.parent/'tests/assets/linear-gradient-composition-oracle.json'
    test_path = Path(__file__).parent.parent/'tests/gradient_composition_runtime.rs'
    oracle, source, result, manifest = [read(p) for p in [oracle_path, recording/'lifecycle.json', replay/'replay.json', toolchain/'manifest.json']]
    assert source['kind'] == 'gradient-public'
    assert source['corpus'] == 'linear-gradient-composition'
    assert result['browser'] == source['browser'] == oracle['browser'] == '153.0.8010.12'
    assert result['lifecycleSha256'] == sha(recording/'lifecycle.json')
    assert result['rendererSha256'] == manifest['files']['renderer-replay']['sha256'] == sha(toolchain/'renderer-replay')
    assert sha(compiler) == manifest['files']['html-to-riv']['sha256'] == sha(toolchain/'html-to-riv')
    known = {c['name']: c for c in oracle['cases']}
    fixtures = {c['name']: c for c in source['cases']}
    frames = {c['name']: c for c in result['cases']}
    assert len(known) == len(oracle['cases']) == len(source['cases']) == len(fixtures) == 18
    assert fixtures.keys() == known.keys()
    assert len(frames) == len(result['cases']) == 144
    seen, artifacts = set(), []
    with tempfile.TemporaryDirectory() as directory:
        tmp = Path(directory)
        for name, fixture in fixtures.items():
            expected = known[name]
            assert expected['expectedCompile']['status'] == 'accepted'
            assert fixture['html'] == expected['html'] and fixture['css'] == expected['css']
            assert not any(key.startswith('injected') for key in fixture)
            assert not fixture['font'] and not fixture['image']
            assert fixture['compileViewport'] == dict(width=390, height=320)
            request = dict(html=expected['html'], css=expected['css'], width=390, height=320)
            saved_request = read(recording/f'{name}.request.json')
            for key, value in request.items():
                assert saved_request[key] == value
            assert not saved_request.get('assets', {})
            (tmp/'input.json').write_text(json.dumps(request))
            subprocess.run([str(compiler), str(tmp/'input.json'), str(tmp/'scene.riv')], check=True, stdout=subprocess.DEVNULL)
            assert sha(tmp/'scene.riv') == sha(recording/f'{name}.riv')
            requirements, source_map = read(tmp/'scene.requirements.json'), read(tmp/'scene.map.json')
            assert requirements == fixture['runtimeRequirements']
            assert source_map == fixture['sourceMap']
            ids = {node['id']: node['object_id'] for node in source_map}
            assert len(ids) == len(source_map)
            assert requirements['version'] == 26
            allowed = {
                'layout-css-linear-gradient-v1', 'layout-css-pixel-bounds-v1',
                'layout-css-paint-order-v1', 'layout-css-solid-borders-v1',
                'layout-css-border-sides-v1', 'layout-css-corner-radii-v1',
                'layout-css-group-opacity-v1', 'layout-css-content-box-v1',
                'layout-css-overflow-clip-margin-v1',
            }
            assert set(requirements['capabilities']) <= allowed
            assert 'layout-css-linear-gradient-v1' in requirements['capabilities']
            policies = {
                'layout_pixel_bounds': 'layout-css-pixel-bounds-v1',
                'layout_borders': 'layout-css-solid-borders-v1',
                'layout_border_sides': 'layout-css-border-sides-v1',
                'layout_corner_radii': 'layout-css-corner-radii-v1',
                'layout_group_opacity': 'layout-css-group-opacity-v1',
                'layout_content_box': 'layout-css-content-box-v1',
                'layout_overflow_clip_margins': 'layout-css-overflow-clip-margin-v1',
                'layout_linear_gradients': 'layout-css-linear-gradient-v1',
            }
            for field, capability in policies.items():
                values = requirements.get(field, [])
                targets = [entry['object_id'] if isinstance(entry, dict) else entry for entry in values]
                assert len(set(targets)) == len(targets)
                assert set(targets) <= set(ids.values())
                if values:
                    assert capability in requirements['capabilities']
            assert all(not value for field, value in requirements.items()
                       if field not in {'version', 'capabilities'} | policies.keys())
            gradients = requirements['layout_linear_gradients']
            assert gradients and all(g['object_id'] in requirements['layout_pixel_bounds'] for g in gradients)
            views = fixture['views']
            assert [(v['instance'], v['frame'], v['width'], v['height']) for v in views] == [
                (i, i*4+f, w, 320) for i in [0, 1] for f, w in enumerate([240, 390, 768, 240])]
            previous_stream = b''
            native_states = {}
            for view in views:
                key = f"{name}-{view['instance']}-step{view['frame']}"
                assert key not in seen
                seen.add(key)
                row = frames[key]
                assert row['qualification'] == 'public-compiler-lifecycle'
                assert not any(field.startswith('injected') for field in row)
                assert not row['failures'] and not row['geometryFailures']
                for field in ['html', 'css', 'runtimeRequirements']:
                    assert row[field] == fixture[field], (key, field)
                for field in ['frame', 'instance', 'width']:
                    assert row[field] == view[field]
                stream = Path(view['stream'])
                assert stream.resolve() == (recording/f"{name}-{view['frame']}.stream").resolve()
                assert row['streamSha256'] == sha(stream)
                data = stream.read_bytes()
                assert data.startswith(previous_stream)
                delta = data[len(previous_stream):].decode()
                previous_stream = data
                assert data.decode().splitlines().count('frame') == view['frame']+1
                assert delta.splitlines().count('frame') == 1
                assert f"frameSize width={int(view['width'])} height=320" in delta
                commands = [line for line in delta.splitlines() if line.startswith(
                    ('makePremultipliedLinearGradient ', 'makeTiledPremultipliedLinearGradient '))]
                assert len(commands) == len(gradients)
                assert 'makeLinearGradient ' not in delta
                unmatched = list(gradients)
                for command in commands:
                    colors = [int(value, 16) for value in re.findall(r'color=0x([0-9a-fA-F]{8})', command)]
                    matching = []
                    for gradient in unmatched:
                        authored = [stop['color'] for stop in gradient['stops']]
                        if colors in [authored, [authored[0]]+authored, authored+[authored[-1]], [authored[0]]+authored+[authored[-1]]]:
                            matching.append(gradient)
                    assert matching, (name, command)
                    unmatched.remove(matching[0])
                    if command.startswith('makeTiled'):
                        tile = re.search(r'tile=\(([^)]+)\)', command)
                        assert tile is not None
                        values = [float(value) for value in tile.group(1).split(',')]
                        assert len(values) == 4 and all(math.isfinite(value) for value in values)
                        assert values[2] > 0 and values[3] > 0
                assert not unmatched
                alphas = [float(line.split('opacity=')[1]) for line in delta.splitlines() if line.startswith('beginOpacity opacity=')]
                assert sorted(alphas) == sorted(entry['opacity'] for entry in requirements.get('layout_group_opacity', []))
                assert delta.splitlines().count('endOpacity') == len(alphas)
                reference = next(v for v in expected['viewports'] if v['width'] == view['width'])
                assert view['bounds'].keys() == reference['boxes'].keys()
                for node, box in view['bounds'].items():
                    for axis in ['x', 'y', 'width', 'height']:
                        assert math.isfinite(box[axis])
                        assert abs(box[axis]-reference['boxes'][node][axis]) < .1
                for kind in ['browser', 'native']:
                    image = Path(row['prefix']+'.'+kind+'.png')
                    assert row[kind+'Sha256'] == sha(image)
                    assert png_size(image) == (view['width'], view['height'])
                state = view['width']
                assert native_states.setdefault(state, row['nativeSha256']) == row['nativeSha256']
            artifacts.append(dict(name=name, rivSha256=sha(tmp/'scene.riv'), requestSha256=sha(recording/f'{name}.request.json'), requirementsSha256=sha(tmp/'scene.requirements.json'), sourceMapSha256=sha(tmp/'scene.map.json')))
    assert seen == frames.keys()
    return dict(status='passed', scope='Public compiler artifact and lifecycle provenance; visual coverage separate', scenes=18, frames=144, artifacts=artifacts,
                evidence=[dict(path=str(p), sha256=sha(p)) for p in [oracle_path, test_path, recording/'lifecycle.json', replay/'replay.json', compiler, toolchain/'manifest.json']])


if __name__ == '__main__':
    recording, replay, compiler, toolchain = [Path(p).resolve() for p in sys.argv[1:]]
    result = audit(recording, replay, compiler, toolchain)
    (replay/'gradient-composition-lifecycle-audit.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(status=result['status'], scenes=result['scenes'], frames=result['frames'])))
