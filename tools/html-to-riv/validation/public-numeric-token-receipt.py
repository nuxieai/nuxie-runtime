"""Bind the numeric compiler repair and its retained native/Chrome failures."""
import hashlib
import json
from pathlib import Path
import re

module = Path(__file__).resolve().parents[1]
build = module / 'output/public-numeric-token-r3'
native = module / 'output/public-numeric-native-r1'

def read(path):
    return json.loads(Path(path).read_text())

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

freeze = read(build / 'frozen/source-bindings.json')
for item in freeze['files']:
    assert sha(item['path']) == item['sha256']
    if 'snapshot' in item:
        assert sha(item['snapshot']) == item['sha256']
checks = read(build / 'build-checks.json')
assert len(checks) == 4 and all(check['exitCode'] == 0 for check in checks)
for check in checks:
    assert sha(check['log']) == check['logSha256']
assert sum(map(int, re.findall(r'test result: ok\. (\d+) passed;',
    (build / 'full-rust.log').read_text()))) == 247
transport = read(build / 'transport-check.json')
assert transport['exitCode'] == 0
node = (build / 'full-node.log').read_text()
assert re.search(r'(?:#|ℹ) pass 41\b', node) and re.search(r'(?:#|ℹ) fail 0\b', node)
assert read(build / 'runtime-guard.json')['status'] == 'pass'
red = module / 'output/public-numeric-token-red-r1'
recorded_red = {(row['path'], row['sha256']) for row in read(red / 'command.json')['before']}
recovered_red = read(red / 'verified-source-recovery.json')['files']
assert {(row['originalPath'], row['sha256']) for row in recovered_red} == recorded_red
assert all(sha(row['path']) == row['sha256'] for row in recovered_red)
compiler_hash = sha(build / 'frozen/html-to-riv')
regression_path = module / 'output/numeric-token-prior-regression-r1/manifest.json'
regression = read(regression_path)
assert regression['compilerSha256'] == compiler_hash
assert regression['total'] == 694 and regression['passed'] == 691
changed = [row for row in regression['results'] if not row['exact']]
assert len(changed) == 3 and all(row['exitCode'] == 0 for row in changed)
assert all(row['actualMapSha256'] == row['mapSha256'] for row in changed)
changed_receipt = read(module / 'validation/public-numeric-regression-receipt.json')
assert changed_receipt['compiler']['sha256'] == compiler_hash
assert changed_receipt['geometryPass'] == changed_receipt['pixelPass'] == 24
assert changed_receipt['clearPass'] == 48

counts = {}
for name, expected in [('visible', (208, 208, 204, 416)), ('large', (80, 48, 80, 160)),
                       ('fractional', (16, 16, 4, 32))]:
    run = read(native / name / 'render/receipt.json')
    assert run['toolHashes']['compiler'] == compiler_hash
    rows = run['rows']
    actual = (len(rows), sum(not row['geometryFailures'] for row in rows),
        sum(not row['pixelFailures'] for row in rows),
        sum(check['samePixels'] for row in rows for check in row['clearChecks']))
    assert actual == expected, (name, actual)
    for row in rows:
        assert (row['instance'], row['step']) == (row['frame'] // 4, row['frame'] % 4)
        assert sha(row['prefix'] + '.native.png') == row['nativeSha256']
        assert sha(row['prefix'] + '.chrome.png') == row['chromeSha256']
        for clear in row['clearChecks']:
            assert sha(clear['path']) == clear['sha256']
    assert all(row['nativeIdentical'] and row['chromeIdentical'] for row in run['repeated'])
    counts[name] = dict(zip(('frames', 'geometryPass', 'pixelPass', 'clearPass'), actual))

paths = [build / 'frozen/source-bindings.json', build / 'build-checks.json',
    build / 'transport-check.json', build / 'runtime-guard.json', regression_path,
    module / 'output/public-numeric-token-red-r1/command.json',
    module / 'output/public-numeric-token-red-r1/result.json',
    module / 'output/public-numeric-token-red-r1/test.log',
    module / 'output/public-numeric-token-red-r1/verified-source-recovery.json',
    module / 'output/numeric-token-minimal-red-r1/receipt.json',
    module / 'output/numeric-token-minimal-green-r1/receipt.json',
    module / 'output/public-numeric-token-r1/stale-provenance-tests.log',
    module / 'output/public-numeric-token-r1/stale-provenance-tests-command.json',
    module / 'output/public-numeric-token-r1/build-checks.json',
    module / 'output/public-numeric-token-r1/full-rust.log',
    module / 'output/public-numeric-token-r2/build-checks.json',
    module / 'output/public-numeric-token-r2/full-rust.log',
    module / 'output/numeric-token-flex-bridge-build-r1/build-receipt.json',
    native / 'visible/render/receipt.json', native / 'large/render/receipt.json',
    native / 'fractional/render/receipt.json',
    module / 'validation/public-numeric-native-receipt.json',
    module / 'validation/public-numeric-regression-receipt.json',
    module / 'validation/public-numeric-token-review.md', Path(__file__).resolve()]
paths += [module / 'validation' / name for name in ['public-numeric-token-cases.json',
    'public-numeric-token-rejections.json', 'public-numeric-token-fixtures.py',
    'check-flex-proof-bridge.py']]
receipt = dict(status='compiler-numeric-token-repair-with-retained-layout-and-paint-failures',
    baseline='6c7ac16617835b5f581784ff08a9e779bb52faf3',
    scope='Authored numeric serialization only; no runtime changes or general numeric-domain qualification.',
    rustTests=247, nodeTests=41, publicNumericCases=31, strictRejectionCases=37,
    cliWasmOutputPairs=93, cliWasmDiagnosticPairs=38,
    priorOutputs=694, exactPriorOutputs=691, correctedPriorOutputs=3,
    changedOutputGeometryAndPixelPass=24, native=counts,
    compilerSha256=compiler_hash, wasmSha256=sha(build / 'frozen/compiler.wasm'),
    retained='Four visible and12 supplemental fractional pixel failures;32 large geometry failures; small exact color differences within gates. Nonempty grammar and Unicode normalization remain separate compiler work.',
    bindings=[dict(path=str(path), sha256=sha(path)) for path in paths])
(module / 'validation/public-numeric-token-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({key: receipt[key] for key in ['status', 'rustTests', 'nodeTests', 'native']}))
