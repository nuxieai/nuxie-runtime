"""Negative controls for public gradient provenance; preserve original evidence.
Usage: SCRIPT RECORDING REPLAY COMPILER TOOLCHAIN OUTPUT
JSON reads return fresh copies; mutations are isolated to one audit invocation.
"""
import importlib.util
import json
from pathlib import Path
import sys
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('public_gradient_audit', Path(__file__).with_name('audit-public-gradient.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def controls(recording, replay, compiler, toolchain):
    source = recording/'lifecycle.json'
    result = replay/'replay.json'
    manifest = toolchain/'manifest.json'
    return [
        ('missing-frame', result, lambda d: d['cases'].pop()),
        ('duplicate-frame', result, lambda d: d['cases'].__setitem__(1, d['cases'][0])),
        ('stale-renderer', result, lambda d: d.__setitem__('rendererSha256', '0'*64)),
        ('stale-compiler', manifest, lambda d: d['files']['html-to-riv'].__setitem__('sha256', '0'*64)),
        ('stale-lifecycle', result, lambda d: d.__setitem__('lifecycleSha256', '0'*64)),
        ('changed-native-image', result, lambda d: d['cases'][0].__setitem__('nativeSha256', '0'*64)),
        ('changed-browser-image', result, lambda d: d['cases'][0].__setitem__('browserSha256', '0'*64)),
        ('changed-stream', result, lambda d: d['cases'][0].__setitem__('streamSha256', '0'*64)),
        ('diagnostic-qualification', result, lambda d: d['cases'][0].__setitem__('qualification', 'runtime-experiment-only')),
        ('injected-replay-paint', result, lambda d: d['cases'][0].__setitem__('injectedGradient', {})),
        ('injected-source-paint', source, lambda d: d['cases'][0].__setitem__('injectedGradient', {})),
        ('changed-authored-css', source, lambda d: d['cases'][0].__setitem__('css', '')),
        ('changed-request-css', recording/'linear-gradient-default.request.json', lambda d: d.__setitem__('css', '')),
        ('changed-source-map', source, lambda d: d['cases'][0]['sourceMap'][0].__setitem__('object_id', 999999)),
        ('missing-pixel-policy', source, lambda d: d['cases'][0]['runtimeRequirements']['layout_pixel_bounds'].clear()),
        ('changed-emitted-paint', source, lambda d: d['cases'][0]['runtimeRequirements']['layout_linear_gradients'][0]['stops'][0].__setitem__('color', 0)),
        ('changed-compile-viewport', source, lambda d: d['cases'][0]['compileViewport'].__setitem__('width', 240)),
        ('wrong-resize', source, lambda d: d['cases'][0]['views'][0].__setitem__('width', 241)),
        ('wrong-geometry', source, lambda d: d['cases'][0]['views'][0]['bounds']['gradient'].__setitem__('width', 1)),
        ('missing-clone', source, lambda d: d['cases'][0]['views'][4].__setitem__('instance', 0)),
        ('out-of-order-views', source, lambda d: d['cases'][0]['views'].reverse()),
        ('pixel-failure-hidden', result, lambda d: d['cases'][0]['failures'].append('control')),
        ('geometry-failure-hidden', result, lambda d: d['cases'][0]['geometryFailures'].append('control')),
    ]


def run(recording, replay, compiler, toolchain):
    # A passing baseline is essential: unrelated pre-existing failures cannot
    # stand in for rejection of a mutation.
    baseline = module.audit(recording, replay, compiler, toolchain)
    assert baseline['status'] == 'passed'
    original_read = module.read
    rows = []
    for name, target, mutate in controls(recording, replay, compiler, toolchain):
        touched = []
        def altered(path):
            value = original_read(path)
            if path == target:
                mutate(value)
                touched.append(path)
            return value
        with patch.object(module, 'read', side_effect=altered):
            try:
                module.audit(recording, replay, compiler, toolchain)
            except AssertionError:
                assert touched, f'Mutation never reached: {name}'
                rows.append(dict(name=name, rejected=True))
            else:
                raise AssertionError('Invalid evidence accepted: '+name)
    return dict(status='passed', baseline=dict(scenes=baseline['scenes'], frames=baseline['frames']), controls=rows)


if __name__ == '__main__':
    recording, replay, compiler, toolchain, output = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(recording, replay, compiler, toolchain)
    output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(status=result['status'], rejected=len(result['controls']))))
