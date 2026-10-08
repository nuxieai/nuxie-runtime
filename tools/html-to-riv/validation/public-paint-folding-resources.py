"""Replay existing resource boundaries with fresh public CLI/JS-WASM output."""
from pathlib import Path
import hashlib,json,subprocess,shutil,sys,time
M=Path(__file__).resolve().parents[1]
B=Path(sys.argv[1]).resolve();O=Path(sys.argv[2]).resolve();O.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
prior=M/'output/public-wrapping-resources-r1/receipt.json';old=json.loads(prior.read_text());summary=json.loads((B/'summary.json').read_text());F=B/'frozen'
assert sha(F/'html-to-riv')==summary['compilerSha256'] and sha(F/'compiler.wasm')==summary['wasmSha256']
shutil.copy2(__file__,O/'runner.py');rows=[]
for c in old['cases']:
 p=O/c['name'];p.mkdir();request=Path(c['request']['path']);assert sha(request)==c['request']['sha256'];shutil.copy2(request,p/'request.json');runs=[]
 for trial in ['first','repeat']:
  d=p/trial;d.mkdir();cmd=[str(F/'html-to-riv'),str(p/'request.json'),str(d/'scene.riv')];start=time.perf_counter_ns();r=subprocess.run(cmd,capture_output=True,timeout=120);elapsed=time.perf_counter_ns()-start
  (d/'stdout.log').write_bytes(r.stdout);(d/'stderr.log').write_bytes(r.stderr);assert not r.stdout
  assert (r.returncode==0)==c['accepted'],c['name'];artifacts={}
  if c['accepted']:
   assert not r.stderr
   for n in ['scene.riv','scene.map.json']:artifacts[n]=bind(d/n)
   ref=c['runs'][0]['artifacts']['scene.map.json'];assert sha(ref['path'])==ref['sha256'];assert (d/'scene.map.json').read_bytes()==Path(ref['path']).read_bytes()
  else:
   assert json.loads(r.stderr)==json.loads(c['diagnostic']),c['name'];assert not (d/'scene.riv').exists() and not (d/'scene.map.json').exists()
  runs.append(dict(command=cmd,exitCode=r.returncode,elapsedNs=elapsed,stdout=bind(d/'stdout.log'),stderr=bind(d/'stderr.log'),artifacts=artifacts))
 assert (p/'first/stderr.log').read_bytes()==(p/'repeat/stderr.log').read_bytes()
 if c['accepted']:
  for n in ['scene.riv','scene.map.json']:assert (p/'first'/n).read_bytes()==(p/'repeat'/n).read_bytes()
 old_riv=c['runs'][0]['artifacts'].get('scene.riv')
 if old_riv:assert sha(old_riv['path'])==old_riv['sha256']
 rows.append(dict(name=c['name'],owners=c['owners'],profile=c['profile'],request=bind(p/'request.json'),accepted=c['accepted'],diagnostic=c.get('diagnostic'),runs=runs,oldRiv=old_riv,unchangedRiv=old_riv is not None and sha(p/'first/scene.riv')==old_riv['sha256']))
write(O/'cli.json',dict(cases=rows))
worker=O/'parity.mjs';worker.write_text('''import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import {pathToFileURL} from 'node:url';
const [root,build]=process.argv.slice(2);const api=await import(pathToFileURL(path.join(build,'frozen/inputs/js/index.mjs')));const compiler=await api.createCompiler(fs.readFileSync(path.join(build,'frozen/compiler.wasm')));const rows=[];
for(const c of JSON.parse(fs.readFileSync(path.join(root,'cli.json'))).cases){
 const input=JSON.parse(fs.readFileSync(c.request.path));const result=compiler.compile({languageVersion:api.LANGUAGE_VERSION,...input});assert.equal(result.ok,c.accepted,c.name);
 if(result.ok){assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(c.runs[0].artifacts['scene.riv'].path),c.name);assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(c.runs[0].artifacts['scene.map.json'].path)),c.name);}
 else assert.deepEqual(result.diagnostics,JSON.parse(c.diagnostic),c.name);
 rows.push({name:c.name,accepted:result.ok,exact:true});
}
fs.writeFileSync(path.join(root,'js-results.json'),JSON.stringify(rows,null,2));console.log(JSON.stringify({total:rows.length,passed:rows.length}));
''')
r=subprocess.run(['node',str(worker),str(O),str(B)],capture_output=True);(O/'parity.log').write_bytes(r.stdout+r.stderr);assert r.returncode==0,r.stderr
write(O/'receipt.json',dict(scope='Existing12 resource boundaries: two deterministic public CLI runs and JS/WASM parity, maps/diagnostics unchanged. Changed RIVs need separate native validation; no performance qualification from CLI elapsed times.',prior=bind(prior),build=bind(B/'summary.json'),runner=bind(O/'runner.py'),cases=rows,parity=bind(O/'js-results.json'),parityLog=bind(O/'parity.log'),worker=bind(worker)))
print(json.dumps(dict(cases=len(rows),accepted=sum(c['accepted']for c in rows),changed=[c['name']for c in rows if c['accepted']and not c['unchangedRiv']])))
