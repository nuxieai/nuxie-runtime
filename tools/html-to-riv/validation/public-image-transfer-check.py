#!/usr/bin/env python3
"""Finite provenance controls over retained captures; no compile/browser/render.

Fixtures copy existing requests and Rive files. Empty fresh receipts are test
inputs only and never assert that a fresh render occurred. Negative controls
modify only receipt metadata inside this campaign's output directory.
"""
from pathlib import Path
import copy
import hashlib
import json
import shutil
import subprocess
import sys

MODULE = Path(__file__).resolve().parents[1]
OUT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else MODULE / 'output/public-image-transfer-check-r1'
assert OUT.parent == MODULE / 'output' and OUT.name.startswith('public-image-transfer-check-')
HELPER = MODULE / 'validation/public-image-transfer.py'
PRIOR = MODULE / 'output/public-image-layout-r2/combined-receipt.json'
RAW = MODULE / 'output/public-image-layout-r1/render-reference-r2-receipt.json'
FRESH = MODULE / 'output/public-image-layout-r2/native-receipt.json'
COMPILED = MODULE / 'output/public-image-jpeg-regression-r2'


def read(p):
    return json.loads(Path(p).read_text())


def write(p, value):
    Path(p).write_text(json.dumps(value, indent=2) + '\n')


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def binding(p):
    return dict(path=str(Path(p).resolve()), sha256=sha(p))


OUT.mkdir(exist_ok=True)
assert not (OUT / 'receipt.json').exists(), 'Preserve previous campaign outputs'
originals = {p: sha(p) for p in [PRIOR, RAW, FRESH, COMPILED / 'compile-receipt.json', HELPER]}
results = []


def run(name, prior, mode='empty', names=None, mutation=None, expected=True, includes=None):
    root = OUT / name
    root.mkdir()
    rows = read(COMPILED / 'compile-receipt.json')
    if names is not None:
        rows = [r for r in rows if r['name'] in names]
    admitted = [r for r in rows if r['compiled']]
    for row in admitted:
        destination = root / row['name']; destination.mkdir()
        for file in ['request.json', 'scene.riv', 'scene.map.json']:
            shutil.copy2(COMPILED / row['name'] / file, destination / file)
    (root / 'frozen/inputs/src').mkdir(parents=True)
    shutil.copy2(COMPILED / 'frozen/inputs/src/reset.css', root / 'frozen/inputs/src/reset.css')
    shutil.copy2(COMPILED / 'build-receipt.json', root / 'build-receipt.json')
    write(root / 'compile-receipt.json', rows)
    fresh = copy.deepcopy(read(FRESH))
    if mode == 'empty':
        fresh['rows'] = []; fresh['counts'] = []
    write(root / 'native-receipt.json', fresh)
    previous = Path(prior)
    if mutation:
        previous = mutation(root, copy.deepcopy(read(prior)))
    command = [sys.executable, str(HELPER), str(root), str(previous)]
    process = subprocess.run(command, capture_output=True, timeout=60)
    (root / 'stdout.log').write_bytes(process.stdout)
    (root / 'stderr.log').write_bytes(process.stderr)
    output = root / 'combined-receipt.json'
    if expected:
        assert process.returncode == 0, (name, process.stderr.decode())
        data = read(output)
        helper_binding = next(b for b in data['sources'] if b['path'] == str(HELPER))
        assert sha(helper_binding['snapshot']) == helper_binding['sha256'] == sha(HELPER)
        assert data['summary']['frames'] == len(admitted) * 8
        assert {r['name'] for r in data['rows']} == {r['name'] for r in admitted}
        # Provenance is flattened to real raw receipts and their exact results.
        for row in data['rows']:
            e = row['evidence']; source = read(e['receipt']); actual = read(e['result'])
            assert 'outputLabel' in source and 'evidence' not in actual
            assert sha(e['receipt']) == e['receiptSha256'] and sha(e['result']) == e['resultSha256']
            assert actual == {k: v for k, v in row.items() if k != 'evidence'}
            assert actual in source['rows']
        summary = data['summary']
    else:
        assert process.returncode != 0 and not output.exists() and not (root / 'validation-inputs-transfer').exists(), name
        assert includes in process.stderr.decode(), (name, process.stderr.decode())
        summary = None
    result = dict(name=name, expected='transfer' if expected else 'reject-before-output', exitCode=process.returncode,
                  passed=True, command=command, prior=binding(previous), summary=summary,
                  fresh=binding(root / 'native-receipt.json'), stderr=binding(root / 'stderr.log'))
    if output.exists():
        result['output'] = binding(output)
    results.append(result)
    return root


# Two real raw sources contribute216frames:184 original +32 corrected stretch.
full = run('combined-216', PRIOR)
assert read(full / 'combined-receipt.json')['summary'] == dict(frames=216, freshFrames=0, transferredFrames=216,
    geometryPass=216, pixelPass=198, presencePass=216)
mixed = run('combined-plus-fresh-216', PRIOR, mode='real')
assert read(mixed / 'combined-receipt.json')['summary'] == dict(frames=216, freshFrames=32, transferredFrames=184,
    geometryPass=216, pixelPass=198, presencePass=216)
raw_names = {r['name'] for r in read(RAW)['rows']} - {r['name'] for r in read(FRESH)['rows']}
raw = run('raw-184', RAW, names=raw_names)
assert read(raw / 'combined-receipt.json')['summary']['frames'] == 184
one = {'root-intrinsic-opaque-png'}


def nested(root, previous):
    for row in previous['rows']:
        row['evidence']['receipt'] = str(PRIOR)
        row['evidence']['receiptSha256'] = sha(PRIOR)
    path = root / 'nested-prior.json'; write(path, previous); return path


run('nested-combined-8', PRIOR, names=one, mutation=nested)


def mutate_evidence(field):
    def change(root, previous):
        row = next(r for r in previous['rows'] if r['name'] in one)
        row['evidence'][field] = '0' * 64
        path = root / 'bad-prior.json'; write(path, previous); return path
    return change


run('corrupt-receipt-hash', PRIOR, names=one, mutation=mutate_evidence('receiptSha256'), expected=False, includes='Receipt binding mismatch')
run('corrupt-result-hash', PRIOR, names=one, mutation=mutate_evidence('resultSha256'), expected=False, includes='Result binding mismatch')


def nested_mutation(change):
    def mutate(root, previous):
        rows = [r for r in previous['rows'] if r['name'] in one]
        source = read(rows[0]['evidence']['receipt'])
        change(root, source)
        source_path = root / 'changed-raw.json'; write(source_path, source)
        for row in rows:
            row['evidence']['receipt'] = str(source_path)
            row['evidence']['receiptSha256'] = sha(source_path)
        path = root / 'bad-prior.json'; write(path, previous); return path
    return mutate


def alter_row(root, source):
    row = next(r for r in source['rows'] if r['name'] in one)
    row['pixelFailures'] = ['invented failure']


def duplicate_row(root, source):
    source['rows'].append(copy.deepcopy(next(r for r in source['rows'] if r['name'] in one)))


def alter_label(root, source):
    source['outputLabel'] = 'unbound-label'


def alter_tool(root, source):
    row = next(b for b in source['bindings'] if Path(b['path']).name == 'baseline-probe')
    fake = root / 'different-tool/baseline-probe'; fake.parent.mkdir(); fake.write_bytes(b'different executable')
    row.update(path=str(fake), sha256=sha(fake))


def alter_reset(root, source):
    row = next(b for b in source['bindings'] if Path(b['path']).name == 'reset.css')
    fake = root / 'different-reset/reset.css'; fake.parent.mkdir(); fake.write_bytes(b'body{padding:1px}')
    row.update(path=str(fake), sha256=sha(fake))


run('nested-row-mismatch', PRIOR, names=one, mutation=nested_mutation(alter_row), expected=False, includes='Nested measurement differs')
run('nested-duplicate-row', PRIOR, names=one, mutation=nested_mutation(duplicate_row), expected=False, includes='Missing or duplicated nested source row')
run('nested-wrong-label', PRIOR, names=one, mutation=nested_mutation(alter_label), expected=False, includes='AssertionError')
run('nested-wrong-tool', PRIOR, names=one, mutation=nested_mutation(alter_tool), expected=False, includes='Native source tools/reset differ')
run('nested-wrong-reset', PRIOR, names=one, mutation=nested_mutation(alter_reset), expected=False, includes='Native source tools/reset differ')


def wrong_result_path(root, previous):
    row = next(r for r in previous['rows'] if r['name'] in one)
    copy_path = root / 'unrelated-result.json'; shutil.copy2(row['evidence']['result'], copy_path)
    row['evidence']['result'] = str(copy_path)
    assert sha(copy_path) == row['evidence']['resultSha256']
    path = root / 'bad-prior.json'; write(path, previous); return path


run('unrelated-result-path', PRIOR, names=one, mutation=wrong_result_path, expected=False, includes='Result does not belong')
assert all(sha(p) == value for p, value in originals.items())
shutil.copy2(HELPER, OUT / 'transfer-tested.py')
shutil.copy2(Path(__file__), OUT / 'transfer-check-tested.py')
write(OUT / 'receipt.json', dict(scope='Read-only transfer verification using retained captures; no compilers, browsers or renderers invoked.',
    passed=len(results), planned=12, positive=4, negative=8, existingPixelFailuresRetained=18,
    source=binding(HELPER), testedSnapshot=binding(OUT / 'transfer-tested.py'), driver=dict(**binding(Path(__file__)), snapshot=str(OUT / 'transfer-check-tested.py')),
    inputs=[binding(p) for p in originals if p != HELPER], originalInputsUnchanged=True, rows=results))
assert len(results) == 12
print(json.dumps(dict(passed=len(results), receipt=str(OUT / 'receipt.json'), sha256=sha(OUT / 'receipt.json'))))
