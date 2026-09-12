"""Bind the completed source, signed-padding, quantized and precision evidence.
Run only after source/visual reviews are complete. Does not rebuild or rerender.
"""
from pathlib import Path
import hashlib,json,subprocess,sys
module=Path(__file__).resolve().parents[1]
repo=module.parents[1]
root=module/'output/quantized-line-height-r1'
precision=module/'output/quantized-line-height-precision-r1'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
summary=read(root/'analysis-receipt.json')['summary']
assert summary['candidateMetricPass']==summary['candidateFrames']==344
assert summary['candidateBaselineSkipped']==16
assert summary['originalPixelPass']==290 and summary['supplementalInkPass']==16
assert summary['independentBrowserJoins']==32 and summary['exactHistoricalControlFrames']==16
coverage=read(root/'visual/coverage.json')
assert coverage['reviewCompleted'] and coverage['directPairs']==135 and coverage['exactTransferredPairs']==225
# Historical build input identities are checked against their frozen snapshots;
# subsequent public request-code changes do not rewrite a private experiment.
for binding in read(root/'build-receipt.json')['bindings']:
 target=Path(binding.get('snapshot',binding['path']))
 assert sha(target)==binding['sha256'],target
for receipt in [root/'native-receipt.json',root/'ink-region-receipt.json',root/'analysis-receipt.json']:
 for b in read(receipt)['bindings']:
  assert sha(Path(b['path']))==b['sha256'],b['path']
source=module/'validation/quantized-line-height-source-review.md'
assert source.exists()
assert (precision/'receipt.json').exists()
owned=[module/'examples/reduced-line-height.rs',module/'examples/quantized-line-height.rs']
for pattern in ['line-height-quantization-*','reduced-line-height-*','quantized-line-height-*']:
 owned.extend(p for p in (module/'validation').glob(pattern) if p.is_file() and p.name!='quantized-line-height-receipt.json')
owned.extend([module/'fixtures/fonts/roboto-regular.ttf',module/'src/wire.rs',module/'validation/unitless-line-height-probe.rs',module/'output/immutable-baseline-toolchain-r2/baseline-probe',module/'output/immutable-baseline-toolchain-r2/renderer-replay'])
for dirname in ['line-height-quantization-source-r1','line-height-quantization-browser-r1','reduced-line-height-r1','quantized-line-height-r1','quantized-line-height-precision-r1']:
 owned.extend(p for p in (module/'output'/dirname).rglob('*') if p.is_file() and p!=root/'checkpoint-receipt.json')
paths=sorted(set(owned))
bindings=[dict(path=str(p),sha256=sha(p)) for p in paths]
for b in bindings:assert sha(Path(b['path']))==b['sha256']
guard=subprocess.run(['python3',str(module/'validation/check-target-runtime.py')],cwd=repo,capture_output=True,text=True)
assert guard.returncode==0,guard.stdout+guard.stderr
receipt=dict(scope='Private source-backed line-height and signed-padding evidence; not public text/font admission or general paint qualification.',baseline='6c7ac16617835b5f581784ff08a9e779bb52faf3',main=summary,precisionReceipt=str(precision/'receipt.json'),visual=dict(directPairs=135,exactTransferredPairs=225,complete=True),immutableSourceGuard=json.loads(guard.stdout),artifactCount=len(bindings),bindings=bindings)
content=json.dumps(receipt,indent=2)+'\n'
(root/'checkpoint-receipt.json').write_text(content)
(module/'validation/quantized-line-height-receipt.json').write_text(content)
print(json.dumps(dict(status='pass',files=len(bindings),receiptSha256=sha(root/'checkpoint-receipt.json'),main=summary),indent=2))
