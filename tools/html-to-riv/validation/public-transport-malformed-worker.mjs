import fs from 'node:fs';
import assert from 'node:assert/strict';
import {parentPort,workerData} from 'node:worker_threads';
import {pathToFileURL} from 'node:url';
const {control,languageVersion,jsValue}=await import(pathToFileURL(workerData.cases));
const {createCompiler}=await import(pathToFileURL(workerData.wrapper));
const bytes=fs.readFileSync(workerData.wasm);
const compiler=await createCompiler(bytes);
const {instance}=await WebAssembly.instantiate(bytes,{}),abi=instance.exports;
assert.equal(abi.html_compiler_abi_version(),2);
const encoder=new TextEncoder(),decoder=new TextDecoder('utf-8',{fatal:true});
const controlBytes=encoder.encode(JSON.stringify({languageVersion,input:control}));
let jsCalls=0,abiCalls=0;
function packed(result) {
  if(result.ok) {
    assert.equal(result.languageVersion,languageVersion);
    assert.deepEqual(Object.keys(result).sort(),['languageVersion','ok','riv','sourceMap']);
    return {ok:true,rivHex:Buffer.from(result.riv).toString('hex'),sourceMap:result.sourceMap};
  }
  assert.deepEqual(Object.keys(result).sort(),['diagnostics','ok']);
  assert(Array.isArray(result.diagnostics) && result.diagnostics.length);
  return {ok:false,diagnostics:result.diagnostics};
}
function errorValue(error) {
  let name='ThrownValue',message='[unprintable thrown value]';
  try {name=String(error?.name??typeof error);}catch{}
  try {message=String(error);}catch{}
  return {threw:{name,message}};
}
const jsCompile=value=>{jsCalls++;return compiler.compile(value);};
const jsControl=()=>jsCompile({languageVersion,...control});
function copy(ptr,length) {
  ptr>>>=0;length>>>=0;assert(ptr+length<=abi.memory.buffer.byteLength);
  return new Uint8Array(abi.memory.buffer,ptr,length).slice();
}
function rawResponse() {
  abiCalls++;const status=abi.html_compiler_compile();
  assert([0,1].includes(status));
  const metadataBytes=copy(abi.html_compiler_metadata_ptr(),abi.html_compiler_metadata_len());
  const metadata=JSON.parse(decoder.decode(metadataBytes));
  assert.equal(metadata.ok,status===0);
  const riv=copy(abi.html_compiler_riv_ptr(),abi.html_compiler_riv_len());
  if(!metadata.ok)assert.equal(riv.length,0,'rejected ABI request exposed stale Rive bytes');
  const response=packed(metadata.ok?{...metadata,riv}:metadata);
  return {response,status,metadataHex:Buffer.from(metadataBytes).toString('hex'),rivLength:riv.length};
}
function rawCompile(input) {
  const ptr=abi.html_compiler_request_alloc(input.length)>>>0;
  assert.equal(abi.html_compiler_riv_len(),0,'allocation must discard stale output');
  assert.equal(abi.html_compiler_metadata_len(),0,'allocation must discard stale metadata');
  if(input.length) {assert(ptr);new Uint8Array(abi.memory.buffer,ptr,input.length).set(input);}
  else assert.equal(ptr,0);
  return rawResponse();
}
const retainedJs=jsControl(),retainedJsCopy=JSON.parse(JSON.stringify(packed(retainedJs)));
assert.equal(retainedJsCopy.ok,true);
const expected=rawCompile(controlBytes);assert.deepEqual(expected.response,retainedJsCopy);
const retainedAbi=JSON.parse(JSON.stringify(expected));
const mutable=jsControl();mutable.riv[0]^=255;
assert.deepEqual(packed(jsControl()),retainedJsCopy);
function runAbi(row) {
  if(row.abiHex!==undefined)return rawCompile(Buffer.from(row.abiHex,'hex'));
  const operation=row.abiOperation;
  assert.deepEqual(rawCompile(controlBytes).response,expected.response);
  if(operation.kind==='allocate-rejected') {
    assert.equal(abi.html_compiler_request_alloc(operation.length),0);
    assert.equal(abi.html_compiler_metadata_len(),0);assert.equal(abi.html_compiler_riv_len(),0);
  } else if(operation.kind==='compile-without-request'||operation.kind==='reset-twice') {
    abi.html_compiler_reset();if(operation.kind==='reset-twice')abi.html_compiler_reset();
    assert.equal(abi.html_compiler_metadata_len(),0);assert.equal(abi.html_compiler_riv_len(),0);
  } else assert.equal(operation.kind,'compile-twice');
  return rawResponse();
}
function runJs(row) {
  try {const owned=jsCompile(jsValue(row.js));return {response:packed(owned),owned};}catch(error){return {response:errorValue(error)};}
}
parentPort.postMessage({kind:'ready',control:expected.response,counts:{jsCalls,abiCalls}});
parentPort.on('message',({row})=>{
  const started=performance.now(),results={};
  try {
    if(row.abiHex!==undefined||row.abiOperation) {
      const first=runAbi(row),snapshot=JSON.parse(JSON.stringify(first));
      assert.deepEqual(runAbi(row),snapshot,'ABI response and metadata determinism');
      assert.deepEqual(rawCompile(controlBytes).response,expected.response,'ABI recovery');
      assert.deepEqual(first,snapshot,'ABI owned copy changed');assert.deepEqual(expected,retainedAbi,'retained ABI copied result changed');
      abi.html_compiler_reset();assert.equal(abi.html_compiler_riv_len(),0);assert.equal(abi.html_compiler_metadata_len(),0);
      results.abi={...first,checks:{deterministic:true,noStaleRive:true,recovered:true,ownedCopy:true,resetCleared:true}};
    }
    if(row.js) {
      const first=runJs(row),snapshot=JSON.parse(JSON.stringify(first.response));
      assert.deepEqual(runJs(row).response,snapshot,'JS deterministic result/exception');
      assert.deepEqual(packed(jsControl()),retainedJsCopy,'JS recovery');
      assert.deepEqual(packed(retainedJs),retainedJsCopy,'retained JS output changed');
      assert.deepEqual(first.response,snapshot,'owned JS response changed');
      if(first.owned)assert.deepEqual(packed(first.owned),snapshot,'original JS output changed after subsequent calls');
      results.js={response:first.response,checks:{deterministic:true,recovered:true,retainedOutputOwned:true}};
    }
    parentPort.postMessage({kind:'result',id:row.id,results,counts:{jsCalls,abiCalls},elapsedMs:performance.now()-started});
  }catch(error){parentPort.postMessage({kind:'harness-failure',id:row.id,results,error:errorValue(error),counts:{jsCalls,abiCalls},elapsedMs:performance.now()-started});}
});
