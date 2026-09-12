"""Bind the finite content-box experiment, source audit and exact repeats."""
from pathlib import Path
import hashlib
import json
import re
import subprocess

module=Path(__file__).resolve().parent.parent
repo=module.parent.parent
root=module/'output/content-box-candidate-r2'
baseline='6c7ac16617835b5f581784ff08a9e779bb52faf3'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
binding=lambda p:dict(path=str(p),sha256=sha(p))
receipt=json.loads((root/'render/receipt.json').read_text())
cases=json.loads((module/'validation/content-box-candidate-cases.json').read_text())
assert receipt['browser']=='153.0.8010.12'
assert receipt['toolHashes']['probe']=='7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a'
assert receipt['toolHashes']['renderer']=='276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f'
compiler=module/'output/flex-proof-checkpoint-r2/html-to-riv'
assert sha(compiler)==json.loads((module/'output/flex-proof-checkpoint-r2/controls.json').read_text())['compilerSha256']
paths=[
 'crates/nuxie-schema/src/generated/schema.rs',
 'crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs',
 'crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs',
 'crates/nuxie-runtime/src/mechanical_port/source/layout/layout_sizing_style.rs',
 'crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs',
 'crates/nuxie-runtime/src/mechanical_port/source/generated/layout/layout_component_style_base.rs',
 'crates/nuxie-runtime/src/mechanical_port/source/generated/layout/layout_sizing_style_base.rs',
 'vendor/taffy-0.12.1-rive-yoga-order/src/style/mod.rs',
 'vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs',
 'vendor/taffy-0.12.1-rive-yoga-order/src/compute/leaf.rs',
]
audit=[]
for path in paths:
 before=subprocess.run(['git','show',baseline+':'+path],cwd=repo,capture_output=True,check=True).stdout
 assert (repo/path).read_bytes()==before,path
 audit.append(dict(path=path,sha256=sha(repo/path),baselineByteIdentical=True))
schema=(repo/paths[0]).read_text()
assert not re.search(r'box[_-]?sizing|content[_-]?box',schema,re.I)
definitions={}
for name,body in re.findall(r'    Definition \{\n        name: "([^"]+)",(.*?)\n    \},',schema,re.S):
 p=re.search(r'properties: (DEF_\d+_PROPERTIES)',body)
 parent=re.search(r'runtime_parent: Some\("([^"]+)"\)',body)
 if p:
  block=re.search(r'static '+p[1]+r': &\[Property\] = &\[(.*?)\n\];',schema,re.S)
  assert block
  definitions[name]=dict(parent=parent[1] if parent else None,properties=re.findall(r'Property \{\n        name: "([^"]+)"',block[1]))
hierarchies={}
for name in ['LayoutComponent','LayoutComponentStyle']:
 chain=[];current=name
 while current:
  d=definitions[current];chain.append(dict(name=current,properties=d['properties']));current=d['parent']
 hierarchies[name]=chain
(root/'source-audit.json').write_text(json.dumps(dict(baseline=baseline,byteIdenticalFiles=audit,schemaHasBoxSizingProperty=False,hierarchies=hierarchies),indent=2)+'\n')
repeat=root/'repeated';repeat.mkdir(exist_ok=False)
reproduced=[]
for case in cases:
 directory=root/'render'/case['name'];output=repeat/(case['name']+'.riv')
 result=subprocess.run([str(module/'validation/content-box-candidate.py'),str(directory/'request.json'),str(output)],capture_output=True,text=True)
 assert result.returncode==0,result.stderr
 assert output.read_bytes()==(directory/'scene.riv').read_bytes()
 assert output.with_suffix('.map.json').read_bytes()==(directory/'scene.map.json').read_bytes()
 reproduced.append(dict(name=case['name'],rivSha256=sha(output),mapSha256=sha(output.with_suffix('.map.json'))))
summaries=[]
for case in cases:
 rows=[r for r in receipt['rows'] if r['name']==case['name']]
 summaries.append(dict(name=case['name'],kind=case['kind'],frames=len(rows),geometryPass=sum(not r['geometryFailures'] for r in rows),pixelPass=sum(not r['pixelFailures'] for r in rows),clearPass=sum(c['samePixels'] for r in rows for c in r['clearChecks'])))
assert [sum(s[k] for s in summaries) for k in ['frames','geometryPass','pixelPass','clearPass']]==[96,80,74,192]
visual=json.loads((root/'visual-evidence.json').read_text())
assert len(visual['placements'])==72 and len(visual['exactRepeatedTransfers'])==120
result=dict(status='private-composition-candidate-with-preserved-negative-controls',scope='Finite authored arithmetic translated to existing public border-box fields. Not public content-box admission, exhaustive numeric proof, or runtime enhancement.',baseline=baseline,browser=receipt['browser'],backend=receipt['backend'],effectiveMode=receipt['effectiveMode'],frames=96,geometryPass=80,pixelPass=74,clearPass=192,cases=summaries,
 sourceFiles=[binding(module/'validation'/name)for name in ['content-box-candidate.py','content-box-candidate-cases.json','content-box-visual.py','content-box-receipt.py','content-box-review.md']],
 sourceAudit=binding(root/'source-audit.json'),compiler=binding(compiler),compilerPriorBinding=binding(module/'output/flex-proof-checkpoint-r2/controls.json'),nativeRun=binding(root/'render/receipt.json'),visualEvidence=binding(root/'visual-evidence.json'),reproduced=reproduced,
 directVisualPairs=36,exactRepeatedPairTransfers=60,initialSetupFailure='output/content-box-candidate-r1/render/point-dimensions-asymmetric/compile.log: two fixtures initially had the same authored HTML/CSS identity. Unique inert CSS comments disambiguate candidate selection; no comparison frames existed in the failed setup.',
 command=['node','tools/html-to-riv/validation/check-public-baseline.mjs','tools/html-to-riv/validation/content-box-candidate-cases.json','tools/html-to-riv/validation/content-box-candidate.py','tools/html-to-riv/output/immutable-baseline-toolchain-r2/baseline-probe','tools/html-to-riv/output/immutable-baseline-toolchain-r2/renderer-replay','tools/html-to-riv/output/content-box-candidate-r2/render'],commandExitStatus=1)
(module/'validation/content-box-candidate-receipt.json').write_text(json.dumps(result,indent=2)+'\n')
