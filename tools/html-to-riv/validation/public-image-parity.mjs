// Bind all public image admission fixtures across the frozen CLI/raw ABI/JS.
// COMPILED_ROOT FROZEN_BUILD FRESH_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {fileURLToPath,pathToFileURL} from 'node:url';
const [root,frozen,out] = process.argv.slice(2).map(p=>path.resolve(p));
const read=p=>JSON.parse(fs.readFileSync(p));
const sha=p=>createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const hash=b=>createHash('sha256').update(b).digest('hex');
assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const driver=fileURLToPath(import.meta.url);fs.copyFileSync(driver,path.join(out,'driver.mjs'));
const sourceBindings=read(path.join(frozen,'source-bindings.json')).files;
for(const b of sourceBindings)assert.equal(sha(b.snapshot??b.path),b.sha256);
const js=path.join(frozen,'inputs/js/index.mjs'),wasmPath=path.join(frozen,'compiler.wasm');
const bindings=[driver,js,wasmPath,path.join(root,'build-receipt.json'),path.join(root,'compile-receipt.json')].map(p=>({path:p,sha256:sha(p)}));
const {createCompiler,LANGUAGE_VERSION}=await import(pathToFileURL(js));
const wasm=fs.readFileSync(wasmPath),compiler=await createCompiler(wasm);
const {instance}=await WebAssembly.instantiate(wasm,{}),w=instance.exports;
function raw(input){
  const bytes=new TextEncoder().encode(JSON.stringify({languageVersion:LANGUAGE_VERSION,input}));
  const ptr=w.html_compiler_request_alloc(bytes.length)>>>0;assert(ptr);
  new Uint8Array(w.memory.buffer,ptr,bytes.length).set(bytes);
  const status=w.html_compiler_compile();
  const result=JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer,w.html_compiler_metadata_ptr()>>>0,w.html_compiler_metadata_len()>>>0)));
  const riv=new Uint8Array(w.memory.buffer,w.html_compiler_riv_ptr()>>>0,w.html_compiler_riv_len()>>>0).slice();
  assert.equal(status,result.ok?0:1);if(!result.ok)assert.equal(riv.length,0);
  w.html_compiler_reset();return result.ok?{...result,riv}:result;
}
const rows=[];
for(const expected of read(path.join(root,'compile-receipt.json'))){
  const dir=path.join(root,expected.name),request=read(path.join(dir,'request.json'));
  assert.equal(sha(path.join(dir,'request.json')),expected.requestSha256);
  const results=[['raw-abi',raw(request)],['public-js',compiler.compile({languageVersion:LANGUAGE_VERSION,...request})]];
  for(const [transport,result]of results){
    assert.equal(result.ok,expected.compiled,`${expected.name} ${transport}`);
    if(result.ok){
      assert.equal(hash(result.riv),expected.rivSha256);
      assert.deepEqual(result.sourceMap,read(path.join(dir,'scene.map.json')));
      assert.deepEqual(Object.keys(result).sort(),['languageVersion','ok','riv','sourceMap']);
    }else assert.deepEqual(result.diagnostics,expected.diagnostics);
    rows.push({name:expected.name,transport,ok:result.ok,requestSha256:expected.requestSha256,
      ...(result.ok?{rivSha256:hash(result.riv),mapSha256:expected.mapSha256}:{diagnostics:result.diagnostics}),matchesCli:true});
  }
}
for(const b of bindings)assert.equal(sha(b.path),b.sha256);
const receipt={scope:'Exact public image corpus CLI/raw ABI/JavaScript admission and output parity; native rendering evidence is separate',bindings,sourceBindingsVerified:sourceBindings.length,cases:rows.length/2,observations:rows.length,passed:rows.length,rows};
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
console.log(JSON.stringify({cases:receipt.cases,observations:receipt.observations,passed:receipt.passed}));
