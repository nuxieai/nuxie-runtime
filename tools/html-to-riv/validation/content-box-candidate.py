#!/usr/bin/env python3
"""Finite authored-arithmetic experiment; not a public compiler implementation.

Run as the compiler argument of check-public-baseline.mjs. Candidate CSS is
written independently in content-box-candidate-cases.json. The output uses the
frozen ordinary public compiler; this script never reads browser geometry.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys

module = Path(__file__).resolve().parent.parent
compiler = module / 'output/flex-proof-checkpoint-r2/html-to-riv'
cases = json.loads((module / 'validation/content-box-candidate-cases.json').read_text())
request = json.loads(Path(sys.argv[1]).read_text())
case, = [c for c in cases if c['html'] == request['html'] and c['css'] == request['css']]
request['css'] = case['nativeCss']
target = Path(sys.argv[2])
lowered = target.with_suffix('.lowered.json')
lowered.write_text(json.dumps(request, indent=2) + '\n')
subprocess.run([str(compiler), str(lowered), str(target)], check=True)
