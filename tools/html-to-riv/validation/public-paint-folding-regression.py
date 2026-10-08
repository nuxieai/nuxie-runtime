"""Replay 1148 accepted inputs through frozen CLI/raw WASM/public JS; no rendering.
Usage: python3 validation/public-paint-folding-regression.py BUILD FRESH_OUTPUT
Changed wrapping files require new native evidence; old files are never replaced.
"""
from pathlib import Path
import hashlib, json, shutil, subprocess, sys
M = Path(__file__).resolve().parents[1]
def read(p): return json.loads(Path(p).read_text())
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def bind(p): return {'path': str(Path(p).resolve()), 'sha256': sha(p)}
def write(p, value): Path(p).write_text(json.dumps(value, indent=2)+'\n')

WORKER = r'''import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';import {pathToFileURL} from 'node:url';
const root=process.argv[2],hash=b=>createHash('sha256').update(b).digest('hex');
const read=p=>JSON.parse(fs.readFileSync(p)),write=(p,v)=>fs.writeFileSync(p,JSON.stringify(v,null,2)+'\n');
const {createCompiler,LANGUAGE_VERSION}=await import(pathToFileURL(path.join(root,'index.mjs')));
assert.equal(LANGUAGE_VERSION,'nuxie-html-immutable-v1');
const bytes=fs.readFileSync(path.join(root,'compiler.wasm')),compiler=await createCompiler(bytes);
const {instance}=await WebAssembly.instantiate(bytes,{}),a=instance.exports;
assert.equal(a.html_compiler_abi_version(),2);
const copy=(ptr,len)=>Buffer.from(new Uint8Array(a.memory.buffer,ptr>>>0,len>>>0));
function raw(request){
 const input=Buffer.from(JSON.stringify({languageVersion:LANGUAGE_VERSION,input:request}));
 const ptr=a.html_compiler_request_alloc(input.length)>>>0;assert(ptr);new Uint8Array(a.memory.buffer,ptr,input.length).set(input);
 const status=a.html_compiler_compile(),metadata=JSON.parse(copy(a.html_compiler_metadata_ptr(),a.html_compiler_metadata_len()).toString('utf8'));
 const riv=copy(a.html_compiler_riv_ptr(),a.html_compiler_riv_len());
 a.html_compiler_reset();assert.equal(a.html_compiler_riv_len(),0);assert.equal(a.html_compiler_metadata_len(),0);
 assert.equal(status,metadata.ok?0:1);return {...metadata,riv};
}
const rows=[];
for(const r of read(path.join(root,'references.json'))){
 const request=read(path.join(r.directory,'request.json')),row={group:r.group,index:r.index,transports:[]};
 for(const [transport,run]of [['wasm',()=>raw(request)],['js',()=>compiler.compile({languageVersion:LANGUAGE_VERSION,...request})]]){
  const result={transport,passed:false};
  try{
   const answer=run(),{riv,...metadata}=answer;
   const metadataPath=path.join(r.directory,transport+'-metadata.json');write(metadataPath,metadata);result.metadata={path:metadataPath,sha256:hash(fs.readFileSync(metadataPath))};
   result.ok=answer.ok;
   if(riv){const p=path.join(r.directory,transport+'-scene.riv');fs.writeFileSync(p,riv);result.riv={path:p,sha256:hash(riv)};}
   assert.equal(answer.ok,true);assert.equal(answer.languageVersion,LANGUAGE_VERSION);
   assert.deepEqual(Object.keys(answer).sort(),['languageVersion','ok','riv','sourceMap']);
   assert.deepEqual(answer.sourceMap,read(r.references['scene.map.json'].path));result.oldMapExact=true;
   assert.deepEqual(answer.sourceMap,read(path.join(r.directory,'scene.map.json')));result.cliMapExact=true;
   assert.equal(result.riv.sha256,hash(fs.readFileSync(path.join(r.directory,'scene.riv'))));result.cliRivExact=true;
   result.oldRivExact=result.riv.sha256===r.references['scene.riv'].sha256;
   assert(r.wrapping||result.oldRivExact);result.passed=true;
  }catch(e){result.error=String(e.stack??e);}
  row.transports.push(result);
 }
 row.passed=row.transports.every(t=>t.passed);rows.push(row);
}
write(path.join(root,'transport-results.json'),rows);
console.log(JSON.stringify({total:rows.length,passed:rows.filter(r=>r.passed).length}));
if(rows.some(r=>!r.passed))process.exitCode=1;
'''

def main():
 P=Path(sys.argv[1]).resolve(); F=P/'frozen'; O=Path(sys.argv[2]).resolve()
 assert not O.exists(); O.mkdir(parents=True)
 summary=read(P/'summary.json'); assert summary['sourceUnchanged'] and summary['status']=='public-build-and-transport-pass'
 sources=[bind(P/'summary.json'),bind(F/'source-bindings.json'),bind(__file__)]
 source_bindings=read(F/'source-bindings.json')['files']
 for b in source_bindings: assert sha(b.get('snapshot',b['path']))==b['sha256']
 for key,name in [('compilerSha256','html-to-riv'),('wasmSha256','compiler.wasm')]:
  assert sha(F/name)==summary[key];shutil.copy2(F/name,O/name)
 wrapper=F/'inputs/js/index.mjs'; assert any(sha(wrapper)==b['sha256'] and b['path'].endswith('/js/index.mjs') for b in source_bindings)
 shutil.copy2(wrapper,O/'index.mjs');shutil.copy2(__file__,O/'regression.py')
 sources.extend([bind(F/'html-to-riv'),bind(F/'compiler.wasm'),bind(wrapper)])
 prior=M/'output/public-wrapping-remainder-regression-r1/references.json';sources.append(bind(prior));refs=read(prior);assert len(refs)==1116
 capture=M/'output/playwright/public-wrapping-remainder-r1';sources.append(bind(capture/'receipt.json'))
 artifacts=read(capture/'receipt.json')['artifacts'];assert len(artifacts)==32
 for index,case in enumerate(artifacts):
  entries={}
  for name,key in [('request.json','requestSha256'),('scene.riv','rivSha256'),('scene.map.json','mapSha256')]:
   p=capture/case['name']/name;assert sha(p)==case[key];entries[name]=bind(p)
  refs.append({'group':'public-remainder','index':index,'references':entries})
 references=[]
 for ref in refs:
  wrapping=ref['group'] not in ['images','accepted-transport']
  d=O/'cases'/f"{ref['group']}-{ref['index']}";d.mkdir(parents=True)
  for name,b in ref['references'].items():
   assert sha(b['path'])==b['sha256'];shutil.copy2(b['path'],d/('old-'+name))
  shutil.copy2(ref['references']['request.json']['path'],d/'request.json')
  references.append(dict(group=ref['group'],index=ref['index'],directory=str(d),wrapping=wrapping,references=ref['references']))
 assert len(references)==1148 and sum(r['wrapping'] for r in references)==72
 write(O/'references.json',references);results=[]
 for ref in references:
  d=Path(ref['directory']);command=[str(O/'html-to-riv'),str(d/'request.json'),str(d/'scene.riv')]
  try:
   p=subprocess.run(command,capture_output=True,timeout=30);code=p.returncode;stdout=p.stdout;stderr=p.stderr;termination=None
  except subprocess.TimeoutExpired as e:code=None;stdout=e.stdout or b'';stderr=e.stderr or b'';termination='timeout'
  (d/'stdout.log').write_bytes(stdout);(d/'stderr.log').write_bytes(stderr)
  outputs={name:bind(d/name) if (d/name).exists() else None for name in ['scene.riv','scene.map.json']}
  same={name:bool(b and b['sha256']==ref['references'][name]['sha256']) for name,b in outputs.items()}
  passed=code==0 and not stdout and not stderr and same['scene.map.json'] and (ref['wrapping'] or same['scene.riv']) and outputs['scene.riv'] is not None
  results.append(dict(group=ref['group'],index=ref['index'],wrapping=ref['wrapping'],command=command,exitCode=code,termination=termination,passed=passed,oldRivExact=same['scene.riv'],oldMapExact=same['scene.map.json'],artifacts=outputs,stdout=bind(d/'stdout.log'),stderr=bind(d/'stderr.log')))
 write(O/'cli-results.json',results)
 worker=O/'transports.mjs';worker.write_text(WORKER)
 p=subprocess.run(['node',str(worker),str(O)],capture_output=True,timeout=1800);(O/'transports.log').write_bytes(p.stdout+p.stderr)
 transports=read(O/'transport-results.json') if (O/'transport-results.json').exists() else []
 for b in source_bindings:assert sha(b.get('snapshot',b['path']))==b['sha256']
 for b in sources:assert sha(b['path'])==b['sha256']
 for ref in references:
  for name,b in ref['references'].items():assert sha(b['path'])==sha(Path(ref['directory'])/('old-'+name))==b['sha256']
  assert sha(Path(ref['directory'])/'request.json')==ref['references']['request.json']['sha256']
 changed=[{'group':r['group'],'index':r['index'],'new':r['artifacts']['scene.riv']} for r in results if not r['oldRivExact']]
 write(O/'changed-riv.json',changed)
 receipt=dict(scope='1148 accepted public requests replayed through frozen CLI, raw WASM ABI, and public JavaScript. All old source maps exact; 1076 nonwrapping RIV files exact. Wrapping RIV files may change only with three-transport agreement. No rendering or native evidence transfer is claimed for changed files.',sources=sources,sourceBindingsVerified=len(source_bindings),compiler=bind(O/'html-to-riv'),wasm=bind(O/'compiler.wasm'),wrapper=bind(O/'index.mjs'),references=bind(O/'references.json'),cliResults=bind(O/'cli-results.json'),transportResults=bind(O/'transport-results.json') if transports else None,worker=bind(worker),transportLog=bind(O/'transports.log'),transportExitCode=p.returncode,total=len(results),nonwrapping=sum(not r['wrapping'] for r in references),wrapping=sum(r['wrapping'] for r in references),cliPassed=sum(r['passed'] for r in results),transportPassed=sum(r['passed'] for r in transports),changedRiv=len(changed),changedRivManifest=bind(O/'changed-riv.json'))
 receipt['passed']=receipt['total']==receipt['cliPassed']==receipt['transportPassed']==1148 and p.returncode==0
 write(O/'receipt.json',receipt);print(json.dumps(receipt));assert receipt['passed']
if __name__=='__main__':main()
