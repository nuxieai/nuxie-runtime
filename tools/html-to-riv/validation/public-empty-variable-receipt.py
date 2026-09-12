"""Bind completed empty-variable qualification without changing its acceptance gates."""
import hashlib
import json
import pathlib
import re
import sys

out=pathlib.Path(sys.argv[1]).resolve()
node_command=pathlib.Path(sys.argv[2]).resolve()
root=pathlib.Path(__file__).resolve().parent.parent
def read(path):return json.loads(pathlib.Path(path).read_text())
def sha(path):return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()
render=read(out/'render/receipt.json');visual=read(out/'visual-coverage.json')
forms=read(root/'validation/public-empty-variable-forms.json')
cases=read(root/'validation/public-empty-variable-cases.json')
rejections=read(root/'validation/public-empty-variable-rejections.json')
browser=read(out/'browser-r2/receipt.json')
regression_path=root/'output/empty-variable-prior-regression-r1/manifest.json'
regression=read(regression_path)
freeze=read(out/'frozen/source-bindings.json')
for item in freeze['files']:
    assert sha(item['path'])==sha(item['snapshot'])==item['sha256']
for check in read(out/'build-checks.json'):assert check['exitCode']==0
assert read(out/'runtime-guard.json')['status']=='pass'
rust=sum(int(n) for n in re.findall(r'test result: ok\. (\d+) passed;', (out/'full-rust.log').read_text()))
assert rust==237
node=(out/'full-node.log').read_text();assert re.search(r'(?:#|ℹ) (?:tests|pass) 40\b',node)
assert re.search(r'(?:#|ℹ) fail 0\b',node)
assert len(forms)==231 and sum(row['accepted'] for row in forms)==210
assert len(cases)==26 and len(rejections)==27
assert len(browser['rows'])==231 and all(row['exactComputedAndGeometry'] for row in browser['rows'])
assert regression['passed']==regression['total']==668
assert regression['compilerSha256']==sha(out/'frozen/html-to-riv')==render['toolHashes']['compiler']
assert len(render['rows'])==208 and visual['directPairs']==28 and visual['transferredPairs']==180
assert visual['renderReceiptSha256']==sha(out/'render/receipt.json')
paints=0
for row in render['rows']:
    assert not row['geometryFailures'] and not row['pixelFailures']
    assert row['instance']==row['frame']//4 and row['step']==row['frame']%4
    assert all(check['samePixels'] for check in row['clearChecks'])
    for box in row['boxes'].values():
        assert box['width']>0 and box['height']>0
        assert box['x']>=0 and box['y']>=0
        assert box['x']+box['width']<=row['width'] and box['y']+box['height']<=row['height']
        paints+=1
assert paints==624
assert all(row['nativeIdentical'] and row['chromeIdentical'] for row in render['repeated'])

paths=[out/'frozen/source-bindings.json',out/'build-checks.json',node_command,
       out/'red-rust.log',out/'red-command.json',out/'full-rust.log',out/'full-node.log',
       out/'cli-build.log',out/'wasm-build.log',out/'types.log',out/'render-command.json',
       out/'render/receipt.json',out/'visual-coverage.json',out/'runtime-guard.json',
       out/'browser-r2/receipt.json',out/'browser-r2-command.json',out/'nonempty-boundary/receipt.json',regression_path,
       root/'tests/empty-variable.rs',root/'tests/transport-parity.mjs']
paths += [root/'validation'/name for name in ['public-empty-variable-fixtures.py','public-empty-variable-forms.json',
          'public-empty-variable-cases.json','public-empty-variable-rejections.json','public-empty-variable-browser.mjs',
          'public-empty-variable-visual.py','public-empty-variable-nonempty-boundary.mjs','public-empty-variable-review.md',
          'public-empty-variable-receipt.py']]
receipt=dict(status='qualified-bounded-empty-ordinary-variable-recovery',baseline='6c7ac16617835b5f581784ff08a9e779bb52faf3',
             scope='Successful token-empty ordinary variable values compute as unset at existing cascade priority. Seven forms across33 property names have parser/transport/browser-control evidence;26 representative scenes have native/Chrome qualification. Nonempty grammar and Unicode-boundary residuals remain.',
             rustTests=rust,nodeTests=40,propertyNames=33,emptyForms=7,tokenFormCases=231,acceptedTokenFormCases=210,
             unsupportedInitialValueCases=21,additionalStrictRejectionCases=27,resourceDiagnosticPairs=1,
             acceptedCliWasmPairs=708,rejectedCliWasmPairs=49,priorExactPublicOutputs=668,
             browserComputedControlComparisons=231,cases=26,frames=208,geometryPass=208,pixelPass=208,clearPass=416,
             expectedPaintAssertions=624,everyAuthoredBoxEntirelyInsideEveryViewport=True,
             directVisualPairs=28,exactFullRgbaTransferredPairs=180,
             transferScope='Complete decoded RGBA equality after explicit white-canvas extension; responsive widths only match identical viewports.',
             browser=render['browser'],backend=render['backend'],effectiveMode=render['effectiveMode'],
             toolHashes=render['toolHashes'],wasmSha256=sha(out/'frozen/compiler.wasm'),
             fixturesSha256=render['fixturesSha256'],resetSha256=render['resetSha256'],pixelGateSha256=render['pixelGateSha256'],
             preservedResidual='NBSP-wrapped nonempty color emits literal-red bytes in prior/current compilers while Chrome inherits navy; no runtime limitation claim.',
             bindings=[dict(path=str(path),sha256=sha(path)) for path in paths])
(root/'validation/public-empty-variable-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({key:receipt[key] for key in ['status','rustTests','nodeTests','frames','directVisualPairs','priorExactPublicOutputs']}))
