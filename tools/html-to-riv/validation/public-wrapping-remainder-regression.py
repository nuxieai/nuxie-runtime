"""Rerun accepted public requests with new frozen CLI and raw WASM ABI.
No rendering; exact scene bytes preserve prior native input. Maps also checked.
"""
from pathlib import Path
import hashlib,json,shutil,subprocess,time,sys
M=Path(__file__).resolve().parents[1];P=Path(sys.argv[1]).resolve();F=P/'frozen';O=Path(sys.argv[2]).resolve()
assert not O.exists();O.mkdir()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
read=lambda p:json.loads(Path(p).read_text())
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
summary=read(P/'summary.json');assert summary['sourceUnchanged']and summary['status']=='public-build-and-transport-pass'
for key,name in [('compilerSha256','html-to-riv'),('wasmSha256','compiler.wasm')]:assert sha(F/name)==summary[key];shutil.copy2(F/name,O/name)
source_bindings=read(F/'source-bindings.json')['files']
for b in source_bindings:assert sha(b.get('snapshot',b['path']))==b['sha256']
shutil.copy2(__file__,O/'regression.py');sources=[bind(P/'summary.json'),bind(F/'source-bindings.json'),bind(__file__)]
image=M/'output/wrapped-paint-mask-existing-r1/manifest.json';transport=M/'output/public-transport-malformed-wrapped-paint-mask-regression-r1/receipt.json'
sources += [bind(image),bind(transport)];references=[]
for group,path in [('images',image),('accepted-transport',transport)]:
 doc=read(path);assert doc['passed']==doc['total']==len(doc['results'])==(282 if group=='images'else 794)
 for index,r in enumerate(doc['results']):
  assert r['exact']and r['exitCode']==0
  if group=='images':triples=[('request.json',r['request'],r['requestSha256']),('scene.riv',str(Path(r['result'])/'scene.riv'),r['actualRivSha256']),('scene.map.json',str(Path(r['result'])/'scene.map.json'),r['actualMapSha256'])]
  else:triples=[('request.json',str(Path(r['reference']['reference'])/'request.json'),r['reference']['hashes']['request.json'])]+[(name,str(path.parent/'results'/str(r['index'])/name),r['actualHashes'][name])for name in ['scene.riv','scene.map.json']]
  d=O/'cases'/f'{group}-{index}';d.mkdir(parents=True);refs={}
  for name,p,h in triples:
   assert sha(p)==h;refs[name]=dict(path=str(Path(p)),sha256=h)
   if name=='request.json':shutil.copy2(p,d/name)
  references.append(dict(group=group,index=index,directory=str(d),references=refs))
for group,folder,count in [('public-wrapping','public-wrapping-r2',16),('public-stretch','public-wrapping-stretch-r1',24)]:
 capture=M/'output/playwright'/folder
 previous=read(capture/'receipt.json');sources.append(bind(capture/'receipt.json'))
 assert len(previous['artifacts'])==count
 for index,case in enumerate(previous['artifacts']):
  d=O/'cases'/f'{group}-{index}';d.mkdir(parents=True);refs={}
  for name,key in [('request.json','requestSha256'),('scene.riv','rivSha256'),('scene.map.json','mapSha256')]:
   p=capture/case['name']/name;assert sha(p)==case[key];refs[name]=bind(p)
   if name=='request.json':shutil.copy2(p,d/name)
  references.append(dict(group=group,index=index,directory=str(d),references=refs))
(O/'references.json').write_text(json.dumps(references,indent=2)+'\n')
results=[]
for ref in references:
 d=Path(ref['directory']);command=[str(O/'html-to-riv'),str(d/'request.json'),str(d/'scene.riv')];start=time.perf_counter_ns()
 try:p=subprocess.run(command,capture_output=True,timeout=10);code=p.returncode;stdout=p.stdout;stderr=p.stderr;termination=None
 except subprocess.TimeoutExpired as e:code=None;stdout=e.stdout or b'';stderr=e.stderr or b'';termination='timeout'
 elapsed=time.perf_counter_ns()-start;(d/'stdout.log').write_bytes(stdout);(d/'stderr.log').write_bytes(stderr)
 artifacts={name:bind(d/name)if(d/name).exists()else None for name in ['scene.riv','scene.map.json']}
 exact=code==0 and not stdout and not stderr and all(v and v['sha256']==ref['references'][name]['sha256']for name,v in artifacts.items())
 results.append(dict(group=ref['group'],index=ref['index'],command=command,exitCode=code,termination=termination,elapsedNs=elapsed,exact=exact,artifacts=artifacts,stdout=bind(d/'stdout.log'),stderr=bind(d/'stderr.log')))
(O/'cli-results.json').write_text(json.dumps(results,indent=2)+'\n')
worker=O/'wasm.mjs';worker.write_text('''import fs from 'node:fs';import crypto from 'node:crypto';import assert from 'node:assert/strict';import path from 'node:path';
const root=process.argv[2],hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const wasm=fs.readFileSync(path.join(root,'compiler.wasm')),{instance}=await WebAssembly.instantiate(wasm,{}),a=instance.exports;
assert.equal(a.html_compiler_abi_version(),2);const refs=JSON.parse(fs.readFileSync(path.join(root,'references.json'))),rows=[];
const copy=(ptr,len)=>Buffer.from(new Uint8Array(a.memory.buffer,ptr>>>0,len>>>0));
for(const r of refs){const request=JSON.parse(fs.readFileSync(path.join(r.directory,'request.json'))),input=Buffer.from(JSON.stringify({languageVersion:'nuxie-html-immutable-v1',input:request}));
const ptr=a.html_compiler_request_alloc(input.length)>>>0;assert(ptr);new Uint8Array(a.memory.buffer,ptr,input.length).set(input);
const status=a.html_compiler_compile(),metadata=JSON.parse(copy(a.html_compiler_metadata_ptr(),a.html_compiler_metadata_len()).toString('utf8')),riv=copy(a.html_compiler_riv_ptr(),a.html_compiler_riv_len());
fs.writeFileSync(path.join(r.directory,'wasm-metadata.json'),JSON.stringify(metadata,null,2));fs.writeFileSync(path.join(r.directory,'wasm-scene.riv'),riv);
let sameMap=false;try{assert.deepEqual(metadata.sourceMap,JSON.parse(fs.readFileSync(r.references['scene.map.json'].path)));sameMap=true;}catch{}
const exact=status===0&&metadata.ok&&metadata.languageVersion==='nuxie-html-immutable-v1'&&hash(riv)===r.references['scene.riv'].sha256&&sameMap;
rows.push({group:r.group,index:r.index,status,exact,sameMap,rivSha256:hash(riv),metadataSha256:hash(fs.readFileSync(path.join(r.directory,'wasm-metadata.json')))});
a.html_compiler_reset();assert.equal(a.html_compiler_riv_len(),0);assert.equal(a.html_compiler_metadata_len(),0);
}
fs.writeFileSync(path.join(root,'wasm-results.json'),JSON.stringify(rows,null,2));console.log(JSON.stringify({total:rows.length,passed:rows.filter(x=>x.exact).length}));if(rows.some(x=>!x.exact))process.exitCode=1;
''')
wasm=subprocess.run(['node',str(worker),str(O)],capture_output=True);(O/'wasm.log').write_bytes(wasm.stdout+wasm.stderr)
wasm_rows=read(O/'wasm-results.json')if(O/'wasm-results.json').exists()else[]
for b in source_bindings:assert sha(b.get('snapshot',b['path']))==b['sha256']
for key,name in [('compilerSha256','html-to-riv'),('wasmSha256','compiler.wasm')]:assert sha(O/name)==summary[key]
for ref in references:
 for b in ref['references'].values():assert sha(b['path'])==b['sha256']
 assert sha(Path(ref['directory'])/'request.json')==ref['references']['request.json']['sha256']
receipt=dict(scope='1116 accepted public corpus entries (282 image +794 transport +16 public wrapping +24 public stretch). Newly executed frozen CLI and raw WASM ABI. Exact scene bytes, CLI map bytes and WASM map structures. No new rendering, malformed-input corpus rerun or JS-wrapper regression claim.',sources=sources,sourceBindingsVerified=len(source_bindings),compiler=bind(O/'html-to-riv'),wasm=bind(O/'compiler.wasm'),references=bind(O/'references.json'),cliResults=bind(O/'cli-results.json'),wasmResults=bind(O/'wasm-results.json')if wasm_rows else None,wasmWorker=bind(worker),wasmLog=bind(O/'wasm.log'),wasmExitCode=wasm.returncode,total=len(results),cliPassed=sum(r['exact']for r in results),wasmPassed=sum(r['exact']for r in wasm_rows),groups={g:sum(r['exact']for r in results if r['group']==g)for g in ['images','accepted-transport','public-wrapping','public-stretch']})
(O/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt));assert receipt['total']==receipt['cliPassed']==receipt['wasmPassed']==1116
