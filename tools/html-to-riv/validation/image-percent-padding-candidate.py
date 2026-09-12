#!/usr/bin/env python3
"""Generate source-driven ordinary-file image percentage-padding experiments.

The frozen public compiler still rejects the authored request. Its accepted
point-padding output supplies the existing file structure; this private
generator changes only ordinary sizing/padding fields from explicit source
recipes. No browser measurement, target-runtime change or public support claim.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct
import subprocess

MODULE = Path(__file__).resolve().parents[1]
BASELINE = '6c7ac16617835b5f581784ff08a9e779bb52faf3'
PADDING = {'left': (512, 617), 'right': (513, 618), 'top': (514, 619), 'bottom': (515, 620)}
AXIS = {'width': (7, 607, 655), 'height': (8, 608, 656)}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bind(path):
    return {'path': str(Path(path).resolve()), 'sha256': sha(path)}


def read(path):
    return json.loads(Path(path).read_text())


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def vu(value):
    assert isinstance(value, int) and 0 <= value <= 0xffffffff
    data = bytearray()
    while value >= 128:
        data.append((value & 127) | 128)
        value >>= 7
    data.append(value)
    return data


def decode(data):
    assert data[:7] == b'RIVE\x07\x03\x00'
    pos = 7

    def uint():
        nonlocal pos
        result = shift = 0
        while True:
            byte = data[pos]
            pos += 1
            result |= (byte & 127) << shift
            if not byte & 128:
                return result
            shift += 7
            assert shift < 35

    fields = []
    while key := uint():
        fields.append(key)
    kinds = {}
    for offset in range(0, len(fields), 4):
        word = struct.unpack_from('<I', data, pos)[0]
        pos += 4
        for index, key in enumerate(fields[offset:offset + 4]):
            kinds[key] = (word >> (2 * index)) & 3
    records = []
    while pos < len(data):
        kind, values = uint(), {}
        while key := uint():
            assert key not in values
            if key == 196:
                values[key] = bool(data[pos])
                pos += 1
            elif kinds[key] == 0:
                values[key] = uint()
            elif kinds[key] == 1:
                size = uint()
                values[key] = data[pos:pos + size]
                pos += size
            else:
                values[key] = struct.unpack_from('<f' if kinds[key] == 2 else '<I', data, pos)[0]
                pos += 4
        records.append((kind, values))
    assert pos == len(data)
    return kinds, records


def encode(kinds, records):
    fields = sorted({key for _, values in records for key in values if key != 196})
    data = bytearray(b'RIVE\x07\x03\x00')
    for key in fields:
        data.extend(vu(key))
    data.extend(vu(0))
    for offset in range(0, len(fields), 4):
        word = sum(kinds[key] << (2 * index) for index, key in enumerate(fields[offset:offset + 4]))
        data.extend(struct.pack('<I', word))
    for kind, values in records:
        data.extend(vu(kind))
        for key, value in sorted(values.items()):
            data.extend(vu(key))
            if key == 196:
                data.append(int(value))
            elif kinds[key] == 0:
                data.extend(vu(value))
            elif kinds[key] == 1:
                data.extend(vu(len(value)))
                data.extend(value)
            else:
                data.extend(struct.pack('<f' if kinds[key] == 2 else '<I', value))
        data.extend(vu(0))
    return bytes(data)


def generate(root, cases, frozen, variant):
    result = []
    for case in cases:
        directory = root / case['name']
        directory.mkdir()
        request = dict(case['input'])
        request['assets'] = {key: {'kind': 'image', 'bytes': list((MODULE / file).read_bytes())}
                             for key, file in case['assetFiles'].items()}
        write(directory / 'request.json', request)
        # Preserve the actual public rejection; the generated scene is private.
        rejected = subprocess.run([str(frozen / 'html-to-riv'), str(directory / 'request.json'),
                                   str(directory / 'public.riv')], capture_output=True, timeout=30)
        (directory / 'public.log').write_bytes(rejected.stdout + rejected.stderr)
        assert rejected.returncode != 0 and not (directory / 'public.riv').exists(), case['name']
        diagnostics = json.loads(rejected.stderr)
        image = case['recipe']['image']
        sizes = {axis: image[axis] for axis in AXIS}
        content = image['boxSizing'] == 'content-box'
        assert image['boxSizing'] in ('content-box', 'border-box')
        if case['candidateExpectation']['status'] != 'intended-support':
            result.append({'name': case['name'], 'compiled': False, 'status': rejected.returncode,
                           'diagnostics': diagnostics, 'scope': 'Retained unresolved context; no candidate emitted',
                           'requestSha256': sha(directory / 'request.json')})
            continue
        inset_axis = {'left': 'width', 'right': 'width', 'top': 'height', 'bottom': 'height'}
        for side, padding in image['padding'].items():
            assert padding['unit'] in ('px', '%') and math.isfinite(padding['value']) and padding['value'] >= 0
            if content and sizes[inset_axis[side]]['unit'] == '%':
                assert padding['value'] == 0, 'Same-axis content percentage plus padding is not this composition'
        seed = dict(request)
        # The original CSS is parsed, but these final longhands replace its
        # percentage padding for the accepted structural seed only.
        seed_padding = {side: 0 if content and sizes[inset_axis[side]]['unit'] == '%' else 1 for side in PADDING}
        seed['css'] += '\n#image{' + ''.join(f'padding-{side}:{value}px;' for side, value in seed_padding.items()) + '}'
        write(directory / 'seed-request.json', seed)
        seeded = subprocess.run([str(frozen / 'html-to-riv'), str(directory / 'seed-request.json'),
                                 str(directory / 'seed.riv')], capture_output=True, timeout=30)
        (directory / 'seed.log').write_bytes(seeded.stdout + seeded.stderr)
        assert seeded.returncode == 0, (case['name'], seeded.stderr.decode())
        encoded = (directory / 'seed.riv').read_bytes()
        kinds, records = decode(encoded)
        assert encode(kinds, records) == encoded, 'Seed must round-trip byte-for-byte'
        mapping = read(directory / 'seed.map.json')
        object_id = next(row['object_id'] for row in mapping if row['id'] == 'image')
        artboards = [index for index, (kind, _) in enumerate(records) if kind == 1]
        assert len(artboards) == 1
        artboard = artboards[0]
        assert records[object_id + artboard][0] == 409
        owner = records[object_id + artboard][1]
        assert owner[4] == b'image' and owner[494] == object_id + 1
        style = records[owner[494] + artboard][1]
        inner = [(index, values) for index, (kind, values) in enumerate(records)
                 if kind == 409 and values.get(5) == object_id]
        assert len(inner) == 1, 'Require exactly the existing unpadded content owner'
        changes = []

        def set_field(values, key, value, kind, reason):
            before = values.get(key)
            kinds[key] = kind
            values[key] = value
            changes.append({'record': next(i for i, (_, props) in enumerate(records) if props is values),
                            'field': key, 'before': before, 'after': value, 'reason': reason})

        for side, (field, unit_field) in PADDING.items():
            inset = image['padding'][side]
            set_field(style, field, inset['value'], 2, f'Authored {side} padding coefficient')
            set_field(style, unit_field, 2 if inset['unit'] == '%' else 1, 0,
                      f'Authored {side} padding unit; percentages retain original containing-width basis')
        parent = case['recipe']['parent']
        cross = 'height' if parent['direction'] == 'row' else 'width'
        for axis, (field, unit_field, scale_field) in AXIS.items():
            if content and sizes[axis]['unit'] == 'px':
                set_field(owner, field, 0., 2, 'Outer auto accumulates responsive padding around exact inner points')
                set_field(style, unit_field, 3, 0, 'Synthetic outer auto; authored size stays on inner')
                erroneous_stretch = variant == 'stretch-control' and axis == cross and image['alignSelf'] in ('auto', 'stretch')
                set_field(style, scale_field, 1 if erroneous_stretch else 2, 0,
                          'Deliberate erroneous cross stretch' if erroneous_stretch else 'Hug preserves authored definite size')
        candidate = encode(kinds, records)
        assert encode(*decode(candidate)) == candidate
        assert [v[212] for kind, v in records if kind == 106] == [v[212] for kind, v in decode(encoded)[1] if kind == 106]
        (directory / 'scene.riv').write_bytes(candidate)
        shutil.copyfile(directory / 'seed.map.json', directory / 'scene.map.json')
        write(directory / 'candidate-changes.json', changes)
        row = {'name': case['name'], 'compiled': True, 'status': 0,
               'scope': 'Private source-recipe file composition; actual public request remains rejected',
               'publicDiagnostics': diagnostics, 'browserGeometryConsumed': False,
               'requestSha256': sha(directory / 'request.json'), 'rivSha256': sha(directory / 'scene.riv'),
               'mapSha256': sha(directory / 'scene.map.json'), 'changes': bind(directory / 'candidate-changes.json'),
               'seedRequest': bind(directory / 'seed-request.json'), 'seedScene': bind(directory / 'seed.riv')}
        write(directory / 'compile-result.json', row)
        result.append(row)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    parser.add_argument('--cases', type=Path, default=MODULE / 'validation/image-percent-padding-cases.json')
    parser.add_argument('--build', type=Path, default=MODULE / 'output/public-image-padding-build-r2')
    parser.add_argument('--variant', choices=['candidate', 'stretch-control'], default='candidate')
    parser.add_argument('--only', help='Comma-separated case names')
    args = parser.parse_args()
    root, build, case_path = args.root.resolve(), args.build.resolve(), args.cases.resolve()
    root.mkdir(parents=True, exist_ok=False)
    frozen = build / 'frozen'
    for binding in read(frozen / 'source-bindings.json')['files']:
        assert sha(binding.get('snapshot', binding['path'])) == binding['sha256']
    (root / 'frozen').symlink_to(frozen, target_is_directory=True)
    cases = read(case_path)
    if args.only:
        names = set(args.only.split(','))
        cases = [case for case in cases if case['name'] in names]
        assert {case['name'] for case in cases} == names
    assert cases and len({case['name'] for case in cases}) == len(cases)
    write(root / 'cases.json', cases)
    authoring = [bind(case_path)]
    authoring[-1]['snapshot'] = str(root / 'all-source-cases.json')
    shutil.copyfile(case_path, root / 'all-source-cases.json')
    for file in sorted({file for case in cases for file in case['assetFiles'].values()}):
        source = (MODULE / file).resolve()
        relative = source.relative_to(MODULE)
        target = root / 'authoring-inputs' / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        authoring.append({**bind(source), 'snapshot': str(target)})
    write(root / 'authoring-bindings.json', authoring)
    helper = root / 'candidate-generator.py'
    shutil.copyfile(__file__, helper)
    write(root / 'build-receipt.json', {'scope': 'Private ordinary-file composition; no public admission or native qualification',
          'baseline': BASELINE, 'seedBuild': str(build), 'seedCompiler': bind(frozen / 'html-to-riv'),
          'generator': bind(helper), 'variant': args.variant, 'browserGeometryConsumed': False})
    result = generate(root, cases, frozen, args.variant)
    write(root / 'compile-receipt.json', result)
    print(json.dumps({'cases': len(result), 'privateCandidateFiles': sum(row['compiled'] for row in result),
                      'publicAdmission': False, 'variant': args.variant}))


if __name__ == '__main__':
    main()
