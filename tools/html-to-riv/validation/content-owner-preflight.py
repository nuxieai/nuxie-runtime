#!/usr/bin/env python3
"""Freeze source-identical private experiment requests, admission and repeats."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import shutil
import subprocess

BASE = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('cases', type=Path)
    parser.add_argument('build', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    compiler = args.build.resolve() / 'compiler'
    build_receipt = args.build.resolve() / 'build-receipt.json'
    build = json.loads(build_receipt.read_text())
    assert sha(compiler) == build['compilerSha256']
    source_sha = sha(args.cases)
    shutil.copy2(args.cases, out / 'source-cases.json')
    cases = json.loads(args.cases.read_text())
    rows = []
    selected = []
    for case in cases:
        directory = out / case['name']
        directory.mkdir()
        request = directory / 'request.json'
        write(request, dict(html=case['html'], css=case['css'], width=390, height=160))
        row = dict(name=case['name'], requestSha256=sha(request), outputs=[])
        for attempt in ['scene', 'repeat']:
            riv = directory / (attempt + '.riv')
            command = [str(compiler), str(request), str(riv)]
            result = subprocess.run(command, capture_output=True, text=True)
            log = directory / (attempt + '.log')
            log.write_text(result.stdout + result.stderr)
            sample = dict(command=command, exitCode=result.returncode, logSha256=sha(log),
                          owners=[dict(source=source, outer=int(outer), inner=int(inner)) for source, outer, inner
                                  in re.findall(r'^private-content-owner source=(.*?) outer=(\d+) inner=(\d+)$', result.stderr, re.M)])
            if result.returncode == 0:
                sample.update(rivSha256=sha(riv), mapSha256=sha(riv.with_suffix('.map.json')))
            else:
                assert not riv.exists() and not riv.with_suffix('.map.json').exists()
                sample['diagnostic'] = result.stderr
            row['outputs'].append(sample)
        a, b = row['outputs']
        assert a['exitCode'] == b['exitCode']
        assert a['logSha256'] == b['logSha256']
        assert a['owners'] == b['owners']
        expected = case.get('baselineExpected', 'success')
        assert (a['exitCode'] == 0) == (expected == 'success'), (case['name'], a)
        if a['exitCode'] == 0:
            assert a['rivSha256'] == b['rivSha256'] and a['mapSha256'] == b['mapSha256']
            selected.append(case)
        rows.append(row)
    assert sha(compiler) == build['compilerSha256'] and sha(args.cases) == source_sha
    write(out / 'render-cases.json', selected)
    receipt = dict(scope='Private composition: deterministic finite corpus only; no public or numeric admission proof.',
                   sourceSha256=source_sha, compilerSha256=sha(compiler), buildReceiptSha256=sha(build_receipt),
                   scriptSha256=sha(__file__), renderCasesSha256=sha(out / 'render-cases.json'),
                   cases=len(cases), admitted=len(selected), diagnostic=len(cases) - len(selected), rows=rows)
    write(out / 'receipt.json', receipt)
    print(json.dumps({k: receipt[k] for k in ['cases', 'admitted', 'diagnostic', 'compilerSha256']}))


if __name__ == '__main__':
    main()
