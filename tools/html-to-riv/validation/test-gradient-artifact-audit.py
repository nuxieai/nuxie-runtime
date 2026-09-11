"""Mutation controls for gradient artifact audit; original evidence stays untouched.
Usage: SCRIPT RECORDING REPLAY COMPILER TOOLCHAIN OUTPUT
"""
import importlib.util
import json
from pathlib import Path
import sys
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('gradient_audit', Path(__file__).with_name('audit-gradient-diagnostic.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
recording,replay,compiler,toolchain,output = [Path(p).resolve() for p in sys.argv[1:]]
original_read = module.read
controls = [
    ('missing-frame', replay/'replay.json', lambda d: d['cases'].pop()),
    ('duplicate-frame', replay/'replay.json', lambda d: d['cases'].__setitem__(1,d['cases'][0])),
    ('stale-renderer', replay/'replay.json', lambda d: d.__setitem__('rendererSha256','0'*64)),
    ('changed-image', replay/'replay.json', lambda d: d['cases'][0].__setitem__('nativeSha256','0'*64)),
    ('changed-stream', replay/'replay.json', lambda d: d['cases'][0].__setitem__('streamSha256','0'*64)),
    ('false-public-claim', replay/'replay.json', lambda d: d['cases'][0].__setitem__('qualification','public-compiler-lifecycle')),
    ('changed-paint', recording/'lifecycle.json', lambda d: d['cases'][0]['injectedGradient']['colors'].__setitem__(0,0)),
    ('changed-compiler-css', recording/'lifecycle.json', lambda d: d['cases'][0].__setitem__('compilerRequestCss','')),
    ('missing-pixel-policy', recording/'lifecycle.json', lambda d: d['cases'][0]['injectedPixelBounds'].pop()),
    ('wrong-resize', recording/'lifecycle.json', lambda d: d['cases'][0]['views'][0].__setitem__('width',241)),
    ('wrong-geometry', recording/'lifecycle.json', lambda d: d['cases'][0]['views'][0]['bounds']['gradient'].__setitem__('width',1)),
    ('missing-clone', recording/'lifecycle.json', lambda d: d['cases'][0]['views'][4].__setitem__('instance',0)),
]
rows=[]
for name,target,mutate in controls:
    def altered(path):
        value=original_read(path)
        if path == target:
            mutate(value)
        return value
    with patch.object(module,'read',side_effect=altered):
        try:
            module.audit(recording,replay,compiler,toolchain)
        except AssertionError:
            rows.append(dict(name=name,rejected=True))
        else:
            raise AssertionError('Invalid evidence accepted: '+name)
output.write_text(json.dumps(dict(status='passed',controls=rows),indent=2)+'\n')
print(json.dumps(dict(status='passed',rejected=len(rows))))
