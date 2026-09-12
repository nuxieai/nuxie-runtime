#!/usr/bin/env python3
"""Strict identity/translation-profile observer of HISTORICAL wrapping streams.

Not a GPU-state observer or a qualification of current compiler output.
"""
import copy
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

MODULE = Path(__file__).resolve().parents[1]
SOURCE = MODULE / 'output/wrapped-sizing-snapped-r1/render'
OUT = MODULE / 'output/wrapped-clip-stream-r1'
NUMBER = r'-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?'
PATH = re.compile(r'clipPath path=\{id=\d+,fillRule=\d+,path=\{verbs=\[([^]]*)\],points=\[([^]]*)\]\}\}')
PAIR = re.compile(r'\((' + NUMBER + r'),(' + NUMBER + r')\)')

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def f32(v):
    return struct.unpack('f', struct.pack('f', float(v)))[0]

def require(value, message):
    if not value:
        raise ValueError(message)

def aabb(line):
    match = PATH.fullmatch(line)
    require(match is not None, 'unrecognized clip serialization')
    verbs = match[1].split(',')
    points = [(f32(m[1]), f32(m[2])) for m in PAIR.finditer(match[2])]
    require(','.join(m[0] for m in PAIR.finditer(match[2])) == match[2], 'unrecognized points')
    require(verbs[:4] == ['move', 'line', 'line', 'line'] and len(points) >= 4, 'not AABB: initial verbs/points')
    # This profile additionally rejects any unknown verbs. Native IsAABB itself
    # checks initial verbs and all trailing points, not arbitrary trailing verbs.
    require(all(v in ['move', 'line', 'close'] for v in verbs), 'unsupported path verb')
    require(all(p == points[0] for p in points[4:]), 'not AABB: extra point')
    p, q, r, s = points[:4]
    require((p[0] == q[0] and r[0] == s[0] and p[1] == s[1] and q[1] == r[1]) or
            (p[0] == s[0] and q[0] == r[0] and p[1] == q[1] and r[1] == s[1]), 'not AABB: edges')
    return [min(p[0], r[0]), min(p[1], r[1]), max(p[0], r[0]), max(p[1], r[1])]

def observe(lines, width, height):
    state = {'translation': [0., 0.], 'clip': None}
    stack = []
    clips = []
    draws = []
    artboard = [0., 0., f32(width), f32(height)]
    for index, line in enumerate(lines):
        token = line.split(' ', 1)[0]
        if line == 'save':
            stack.append(copy.deepcopy(state))
        elif line == 'restore':
            require(stack, 'restore underflow')
            state = stack.pop()
        elif token == 'transform':
            m = re.fullmatch(r'transform matrix=\[(' + NUMBER + r'(?:,' + NUMBER + r'){5})\]', line)
            require(m is not None, 'invalid transform')
            values = [f32(v) for v in m[1].split(',')]
            require(values[:4] == [1, 0, 0, 1], 'unsupported linear matrix')
            state['translation'] = [f32(a + b) for a, b in zip(state['translation'], values[4:])]
        elif token == 'clipPath':
            bounds = aabb(line)
            require(state['translation'] == [0, 0], 'clip matrix is not outer identity')
            before = state['clip']
            if not clips:
                require(bounds == artboard and before is None, 'incorrect initial artboard clip')
                kind = 'artboard'
            else:
                require(before is not None, 'mask escaped artboard clip')
                if bounds == [0, 0, 32768, 32768]:
                    kind = 'active'
                elif bounds in ([65536, 0, 98304, 32768], [0, 65536, 32768, 98304]):
                    kind = 'inactive'
                else:
                    raise ValueError('unexpected mask rectangle')
            after = bounds if before is None else [max(before[0], bounds[0]), max(before[1], bounds[1]), min(before[2], bounds[2]), min(before[3], bounds[3])]
            empty_before = before is not None and (before[0] >= before[2] or before[1] >= before[3])
            if kind == 'active' and not empty_before:
                require(after == artboard, 'active clip does not preserve artboard')
            if kind == 'inactive':
                require(after[0] >= after[2] or after[1] >= after[3], 'inactive clip not empty')
            state['clip'] = after
            clips.append({'command': index, 'kind': kind, 'bounds': bounds, 'matrix': [1, 0, 0, 1, 0, 0], 'intersection': after, 'alreadyEmpty': empty_before})
        elif token == 'drawPath':
            require(state['clip'] is not None, 'draw escaped artboard clip')
            require(re.fullmatch(r'drawPath path=\{.*\} paint=\{.*\}', line) is not None, 'unrecognized draw')
            b = state['clip']
            draws.append({'command': index, 'empty': b[0] >= b[2] or b[1] >= b[3]})
        elif token in ('makeRenderPaint', 'makeEmptyRenderPath'):
            require(re.fullmatch(token + r' \{.*\}', line) is not None, 'unrecognized resource')
        elif token == 'sample':
            require(re.fullmatch(r'sample seconds=' + NUMBER, line) is not None, 'unrecognized sample')
        elif token == 'frameSize':
            require(re.fullmatch(r'frameSize width=\d+ height=\d+', line) is not None, 'unrecognized frame size')
        elif token == 'clearColor':
            require(re.fullmatch(r'clearColor value=0x[0-9a-fA-F]{8}', line) is not None, 'unrecognized clear')
        else:
            raise ValueError('unsupported command: ' + token)
    require(not stack and state == {'translation': [0., 0.], 'clip': None}, 'incomplete stack/state restoration')
    require(clips and any(c['kind'] != 'artboard' for c in clips), 'missing masks')
    return {'clips': clips, 'draws': draws}

def extract(path, frame):
    lines = path.read_text().splitlines()
    require(lines.pop(0) == 'rive-golden-stream-v1', 'wrong stream version')
    frames, current = [], []
    for line in lines:
        if line == 'frame':
            frames.append(current)
            current = []
        else:
            current.append(line)
    require(not current and len(frames) == frame + 1, 'incorrect cumulative frame count')
    return frames[frame]

def controls(lines, width, height):
    results = []
    first = next(i for i, s in enumerate(lines) if s.startswith('clipPath '))
    mutations = {
        'clip-matrix': lines[:first] + ['transform matrix=[1,0,0,1,1,0]'] + lines[first:],
        'not-aabb': lines[:first] + [lines[first].replace('verbs=[move,line', 'verbs=[move,cubic', 1)] + lines[first+1:],
        'incomplete-stack': lines + ['save'],
        'unknown-command': lines + ['unknownClipPolicy value=1'],
    }
    for name, mutated in mutations.items():
        try:
            observe(mutated, width, height)
        except ValueError as error:
            results.append({'name': name, 'rejected': str(error)})
        else:
            raise ValueError('negative control passed: ' + name)
    return results

def main():
    require(not OUT.exists(), 'fresh output required')
    reports = []
    control_results = None
    for manifest in sorted(SOURCE.glob('*/probe/frames.json')):
        for frame in json.loads(manifest.read_text())['frames']:
            stream = manifest.parent / frame['stream']
            lines = extract(stream, frame['frame'])
            observation = observe(lines, frame['width'], frame['height'])
            if control_results is None:
                control_results = controls(lines, frame['width'], frame['height'])
            reports.append({'stream': str(stream.relative_to(MODULE)), 'streamSha256': sha(stream), 'framesSha256': sha(manifest), 'frame': frame['frame'], 'instance': frame['instance'], 'step': frame['step'], **observation})
    require(len(reports) == 384, 'expected 384 historical frames')
    OUT.mkdir()
    receipt = {'scope': 'Historical experimental snapped candidate only; offline runtime command observation and source-derived clip intersections, not direct GPU state or current candidate qualification', 'scriptSha256': sha(Path(__file__)), 'frames': reports, 'negativeControls': control_results, 'frameCount': len(reports), 'clipCount': sum(len(r['clips']) for r in reports), 'emptyDrawCount': sum(d['empty'] for r in reports for d in r['draws'])}
    (OUT / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({k: v for k, v in receipt.items() if k not in ('frames', 'negativeControls')}))

if __name__ == '__main__':
    main()
