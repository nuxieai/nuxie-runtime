import fs from 'node:fs';
import assert from 'node:assert/strict';
import {parentPort,workerData} from 'node:worker_threads';
import {pathToFileURL} from 'node:url';
const {control,languageVersion,serialize,identities}=await import(pathToFileURL(workerData.cases));
const {createCompiler}=await import(pathToFileURL(workerData.wrapper));
const wasm=fs.readFileSync(workerData.wasm),compiler=await createCompiler(wasm);
const {instance}=await WebAssembly.instantiate(wasm,{}),abi=instance.exports;
assert.equal(abi.html_compiler_abi_version(),2);
let jsCalls=0,abiCalls=0;
const js=request=>{jsCalls++;return compiler.compile({languageVersion,...request});};
function packed(result) {
  if(result.ok){assert.equal(result.languageVersion,languageVersion);return {ok:true,rivHex:Buffer.from(result.riv).toString('hex'),sourceMap:result.sourceMap};}
  assert(Array.isArray(result.diagnostics)&&result.diagnostics.length);return {ok:false,diagnostics:result.diagnostics};
}
function raw(request,escaped=false) {
  const bytes=Buffer.from(serialize({languageVersion,input:request},escaped)),ptr=abi.html_compiler_request_alloc(bytes.length)>>>0;
  assert(ptr);assert.equal(abi.html_compiler_riv_len(),0);assert.equal(abi.html_compiler_metadata_len(),0);
  new Uint8Array(abi.memory.buffer,ptr,bytes.length).set(bytes);abiCalls++;
  const status=abi.html_compiler_compile();assert([0,1].includes(status));
  const copy=(pointer,length)=>{pointer>>>=0;length>>>=0;assert(pointer+length<=abi.memory.buffer.byteLength);return new Uint8Array(abi.memory.buffer,pointer,length).slice();};
  const metadataBytes=copy(abi.html_compiler_metadata_ptr(),abi.html_compiler_metadata_len());
  const metadata=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(metadataBytes));assert.equal(metadata.ok,status===0);
  const riv=copy(abi.html_compiler_riv_ptr(),abi.html_compiler_riv_len());if(!metadata.ok)assert.equal(riv.length,0);
  const result={response:packed(metadata.ok?{...metadata,riv}:metadata),status,metadataHex:Buffer.from(metadataBytes).toString('hex'),rivLength:riv.length};
  abi.html_compiler_reset();assert.equal(abi.html_compiler_metadata_len(),0);assert.equal(abi.html_compiler_riv_len(),0);return result;
}
let retained=js(control),retainedSnapshot=structuredClone(packed(retained));const expected=structuredClone(retainedSnapshot);
assert.equal(expected.ok,true);assert.deepEqual(raw(control).response,expected);
parentPort.postMessage({kind:'ready',control:expected,counts:{jsCalls,abiCalls}});
parentPort.on('message',({row})=>{
  const started=performance.now();let stage='raw';
  try {
    const abiFirst=raw(row.request,row.escapedJson);assert.deepEqual(raw(row.request,row.escapedJson),abiFirst,'raw deterministic metadata and Rive');
    assert.deepEqual(raw(control).response,expected,'raw recovery after resource request');
    stage='js';const first=js(row.request),response=structuredClone(packed(first));
    assert.deepEqual(packed(js(row.request)),response,'public JS deterministic result');
    assert.deepEqual(packed(js(control)),expected,'public JS recovery');
    assert.deepEqual(packed(retained),retainedSnapshot,'previous successful result changed across calls');
    assert.deepEqual(packed(first),response,'current output not owned across later calls');
    assert.deepEqual(response,abiFirst.response,'public JS/raw ABI exact parity');
    stage='control';
    let controlResult;
    if(row.controlRequest) {
      controlResult=packed(js(row.controlRequest));assert.deepEqual(response,controlResult,'inert source padding / zero CSS padding changed bytes or source map');
      assert.deepEqual(packed(first),response,'no-op control mutated original output');
    }
    if(response.ok){identities(response.sourceMap,row.identities);retained=first;retainedSnapshot=structuredClone(response);}
    parentPort.postMessage({kind:'result',id:row.id,response,abi:abiFirst,control:controlResult,
      checks:{deterministic:true,rawNoStaleRive:true,rawResetCleared:true,publicOutputOwned:true,previousOutputOwned:true,recovered:true,exactTransportParity:true},
      counts:{jsCalls,abiCalls},elapsedMs:performance.now()-started});
  }catch(error){parentPort.postMessage({kind:'failure',id:row.id,stage,error:{name:error.name,message:error.message,stack:error.stack},counts:{jsCalls,abiCalls},elapsedMs:performance.now()-started});}
});
