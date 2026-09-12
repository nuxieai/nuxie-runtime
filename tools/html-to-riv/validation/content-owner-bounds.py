#!/usr/bin/env python3
"""Compile-only falsification of padding-cancelled descendant overflow bounds.

No emitted scene from this experiment is imported or rendered. The old private
candidate can emit nonfinite descendant geometry; that is the retained failure.
"""
from pathlib import Path
import argparse
import hashlib
import json
import math
import struct
import subprocess

BASE = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')


def f32(value):
    try:
        return struct.unpack('<f', struct.pack('<f', value))[0]
    except OverflowError:
        return math.copysign(math.inf, value)


def observed_number(value):
    return value if math.isfinite(value) else '+Infinity'


def cases():
    result = []
    for axis in ['width', 'height']:
        other = 'height' if axis == 'width' else 'width'
        inset = '0 .03125px 0 1000000px' if axis == 'width' else '1000000px 0 .03125px 0'
        for variant, dimension, initial, padding in [
            ('cancelled-point', f'{axis}:.03125px', .03125, inset),
            ('cancelled-minimum', f'{axis}:0;min-{axis}:.03125px', .03125, inset),
            ('cancelled-maximum', f'{axis}:1px;max-{axis}:.03125px', .03125, inset),
            ('zero-control', f'{axis}:0', 0., inset),
            ('no-padding-control', f'{axis}:.03125px', .03125, '0'),
        ]:
            for depth in [9, 10]:
                ids = [f'n{i}' for i in range(depth)]
                html = '<div id="p">' + ''.join(f'<div id="{i}">' for i in ids) + '</div>' * (depth + 1)
                css = f'#p{{box-sizing:content-box;{dimension};{other}:20px;padding:{padding}}}'
                css += ','.join('#' + i for i in ids) + f'{{{axis}:1000000%;{other}:1px}}'
                stages = []
                value = initial
                for index in range(depth):
                    product = f32(1_000_000. * value)
                    value = f32(product * f32(.01))
                    stages.append(dict(depth=index + 1, intermediate=observed_number(product), content=observed_number(value)))
                overflow = not math.isfinite(value)
                old_rejects = variant == 'no-padding-control' and overflow
                result.append(dict(name=f'{axis}-{variant}-depth-{depth}', html=html, css=css,
                                   expected={'public': not old_rejects, 'r2': not old_rejects, 'r3': not overflow},
                                   arithmetic=stages, renderAllowed=False))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    compilers = {label: BASE / 'output' / relative for label, relative in {
        'public': 'public-value-build-r1/frozen/html-to-riv',
        'r2': 'content-owner-candidate-build-r2/compiler',
        'r3': 'content-owner-candidate-build-r3/compiler',
    }.items()}
    initial_hashes = {k: sha(v) for k, v in compilers.items()}
    corpus = cases()
    write(out / 'cases.json', corpus)
    rows = []
    for case in corpus:
        directory = out / case['name']
        directory.mkdir()
        request = directory / 'request.json'
        write(request, dict(html=case['html'], css=case['css'], width=390, height=160))
        outputs = {}
        for label, compiler in compilers.items():
            riv = directory / f'{label}.riv'
            command = list(map(str, [compiler, request, riv]))
            result = subprocess.run(command, capture_output=True, text=True)
            log = directory / f'{label}.log'
            log.write_text(result.stdout + result.stderr)
            outputs[label] = dict(exitCode=result.returncode, command=command, logSha256=sha(log))
            assert (result.returncode == 0) == case['expected'][label], (case['name'], label, result.stderr)
            if result.returncode == 0:
                outputs[label].update(rivSha256=sha(riv), mapSha256=sha(riv.with_suffix('.map.json')))
            else:
                assert 'Resolved percentage size may exceed finite binary32 geometry' in result.stderr
                assert not riv.exists() and not riv.with_suffix('.map.json').exists()
            if label == 'r3' and result.returncode == 0:
                assert outputs[label]['rivSha256'] == outputs['r2']['rivSha256']
                assert outputs[label]['mapSha256'] == outputs['r2']['mapSha256']
        rows.append(dict(name=case['name'], requestSha256=sha(request), outputs=outputs))
    assert {k: sha(v) for k, v in compilers.items()} == initial_hashes
    summary = {label: dict(accepted=sum(r['outputs'][label]['exitCode'] == 0 for r in rows),
                           rejected=sum(r['outputs'][label]['exitCode'] != 0 for r in rows)) for label in compilers}
    receipt = dict(scope=__doc__, scriptSha256=sha(__file__), casesSha256=sha(out / 'cases.json'),
                   compilerHashes=initial_hashes, summary=summary, cases=len(rows), rows=rows,
                   limitation='Finite compilation experiment, not a complete numeric proof or public admission. No import or rendering performed.')
    write(out / 'receipt.json', receipt)
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
