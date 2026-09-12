// Finite documented resource-boundary checks. No renderer or unbounded ABI allocation.
// node validation/public-resource-boundaries-campaign.mjs --output output/public-resource-boundaries-r1
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {Worker} from 'node:worker_threads';
import {cases,control,languageVersion,serialize,identities,SOURCE_LIMIT} from './public-resource-boundaries-cases.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const args=process.argv.slice(2),option=(name,fallback)=>args.includes(name)?args[args.indexOf(name)+1]:fallback;
for(let i=0;i<args.length;i+=2)assert(['--frozen','--output','--only'].includes(args[i])&&args[i+1]);
const frozen=path.resolve(option('--frozen',path.join(root,'output/public-transport-malformed-build-r1/frozen')));
const output=path.resolve(option('--output',path.join(root,'output/public-resource-boundaries-r1')));
assert(output.startsWith(path.join(root,'output/public-resource-boundaries-')));assert(!fs.existsSync(output),'refusing to overwrite evidence');
fs.mkdirSync(output,{recursive:true});
const sha=value=>createHash('sha256').update(value).digest('hex'),hash=file=>sha(fs.readFileSync(file));
const json=(file,value)=>fs.writeFileSync(file,JSON.stringify(value,null,2)+'\n'),read=file=>JSON.parse(fs.readFileSync(file,'utf8'));
const relative=file=>path.relative(output,file),log=text=>{fs.appendFileSync(path.join(output,'run.log'),text+'\n');process.stdout.write(text+'\n');};
const manifest=path.join(frozen,'source-bindings.json'),bindings=read(manifest).files;
for(const binding of bindings)assert.equal(hash(binding.snapshot??binding.path),binding.sha256);
assert.equal(hash(path.join(frozen,'html-to-riv')),'ca8f561f470dffa80a287aa5cb4d0d6866e7e77475d1cce8631136386fa6061a');
assert.equal(hash(path.join(frozen,'compiler.wasm')),'1383b476e655423130f3450a4f3803593ca4f80bef0fe97c9dd2ac12637681f9');
const destination=path.join(output,'frozen');fs.mkdirSync(destination);
const inputs=[['cli',path.join(frozen,'html-to-riv'),'html-to-riv'],['wasm',path.join(frozen,'compiler.wasm'),'compiler.wasm'],
  ['wrapper',path.join(frozen,'inputs/js/index.mjs'),'index.mjs'],['bindings',manifest,'source-bindings.json'],
  ...['campaign','cases','worker'].map(name=>[name,path.join(root,`validation/public-resource-boundaries-${name}.mjs`),`public-resource-boundaries-${name}.mjs`])]
  .map(([kind,source,name])=>{const target=path.join(destination,name);fs.copyFileSync(source,target);if(kind==='cli')fs.chmodSync(target,0o755);return {kind,source,sha256:hash(source),snapshot:relative(target)};});
const location=kind=>path.join(output,inputs.find(input=>input.kind===kind).snapshot);
const all=cases(),selected=option('--only',null)?all.filter(row=>row.id===option('--only')):all;assert(selected.length);
const TIMEOUT=10000,MAX_BUFFER=8*1024*1024,started=new Date().toISOString();
json(path.join(output,'cases.json'),all);
json(path.join(output,'command.json'),{started,argv:process.argv,cwd:process.cwd(),node:process.version,platform:process.platform,arch:process.arch,timeoutMs:TIMEOUT,maxNativeBuffer:MAX_BUFFER,inputs});
let nativeCalls=0;
function native(request,prefix) {
  nativeCalls++;const start=performance.now();
  const child=spawnSync(location('cli'),[request,prefix+'.riv'],{encoding:'utf8',timeout:TIMEOUT,killSignal:'SIGKILL',maxBuffer:MAX_BUFFER});
  const execution={argv:[location('cli'),request,prefix+'.riv'],status:child.status,signal:child.signal,
    error:child.error?{name:child.error.name,message:child.error.message,code:child.error.code}:null,stdout:child.stdout,stderr:child.stderr,elapsedMs:performance.now()-start};
  json(prefix+'.command.json',execution);
  assert(!child.error&&child.signal===null&&[0,1].includes(child.status),JSON.stringify(execution));assert.equal(child.stdout,'');
  if(child.status===0){assert.equal(child.stderr,'');return {ok:true,rivHex:fs.readFileSync(prefix+'.riv').toString('hex'),sourceMap:read(prefix+'.map.json')};}
  const diagnostics=JSON.parse(child.stderr);assert(Array.isArray(diagnostics)&&diagnostics.length);return {ok:false,diagnostics};
}
let retainedNative;
function nativeCase(row,dir,controlFile,expected) {
  const request=path.join(dir,'cli-input.json');fs.writeFileSync(request,serialize(row.request,row.escapedJson));
  assert.equal(fs.statSync(request).size,row.serializedCliBytes);
  const first=native(request,path.join(dir,'native-first')),second=native(request,path.join(dir,'native-repeat'));
  assert.deepEqual(second,first,'native deterministic response');
  let controlResult,retainedHashes;
  if(first.ok) {
    identities(first.sourceMap,row.identities);
    if(row.controlRequest){const file=path.join(dir,'noop-control-input.json');json(file,row.controlRequest);controlResult=native(file,path.join(dir,'native-noop-control'));assert.deepEqual(first,controlResult,'inert source padding / zero CSS padding changed native bytes or map');}
    retainedNative={id:row.id,response:first};
  }else {
    for(const name of ['native-first','native-repeat'])for(const ext of ['riv','map.json','requirements.json'])assert(!fs.existsSync(path.join(dir,`${name}.${ext}`)),'rejection published output');
    const retained=path.join(dir,'retained-success');
    fs.writeFileSync(retained+'.riv',Buffer.from(retainedNative.response.rivHex,'hex'));json(retained+'.map.json',retainedNative.response.sourceMap);
    const before=[hash(retained+'.riv'),hash(retained+'.map.json')];
    assert.deepEqual(native(request,retained),first);assert.deepEqual([hash(retained+'.riv'),hash(retained+'.map.json')],before,'failure changed existing files');
    assert.deepEqual(native(controlFile,path.join(dir,'native-recovery')),expected,'native recovery');
    assert.deepEqual([hash(retained+'.riv'),hash(retained+'.map.json')],before);
    retainedHashes={sourceCase:retainedNative.id,riv:before[0],map:before[1]};
  }
  return {response:first,control:controlResult,retainedHashes,checks:{deterministic:true,rejectionNoOutput:!first.ok?true:null,retainedFilesPreserved:!first.ok?true:null,recovered:!first.ok?true:null}};
}
let worker,pending,workerStarts=0,lastCounts;
const workerEvents=[];
function stop(){if(worker){const old=worker;worker=undefined;old.terminate();}}
function message(send){return new Promise((resolve,reject)=>{assert(!pending);const timer=setTimeout(()=>{pending=undefined;stop();reject(new Error('worker timeout'));},TIMEOUT);
  pending={resolve:value=>{clearTimeout(timer);pending=undefined;resolve(value);},reject:error=>{clearTimeout(timer);pending=undefined;reject(error);}};send?.();});}
async function startWorker(){
  const ready=message(),serial=++workerStarts;
  const current=new Worker(location('worker'),{workerData:{wrapper:location('wrapper'),wasm:location('wasm'),cases:location('cases')},stdout:true,stderr:true,resourceLimits:{maxOldGenerationSizeMb:256}});worker=current;
  for(const [stream,name] of [[current.stdout,'stdout'],[current.stderr,'stderr']])stream.on('data',bytes=>fs.appendFileSync(path.join(output,`worker-${serial}.${name}.log`),bytes));
  current.on('message',value=>{if(worker===current)pending?.resolve(value);});
  current.on('error',error=>{workerEvents.push({serial,event:'error',active:worker===current,message:error.message});if(worker===current)pending?.reject(error);});
  current.on('exit',code=>{workerEvents.push({serial,event:'exit',active:worker===current,code});if(worker===current)pending?.reject(new Error(`worker exited ${code}`));});
  const response=await ready;assert.equal(response.kind,'ready');lastCounts=response.counts;return response;
}
const classify=response=>response.ok?'success':response.diagnostics[0].code;
const results=[];let setupFailure,unexpected=0,defects=0;
try {
  const dir=path.join(output,'control');fs.mkdirSync(dir);const controlFile=path.join(dir,'input.json');json(controlFile,control);
  const expected=native(controlFile,path.join(dir,'native'));assert.equal(expected.ok,true);retainedNative={id:'initial-control',response:expected};
  const ready=await startWorker();assert.deepEqual(ready.control,expected);json(path.join(dir,'worker.json'),ready);
  for(const [index,row] of selected.entries()) {
    const dir=path.join(output,'cases',row.id);fs.mkdirSync(dir,{recursive:true});json(path.join(dir,'case.json'),row);
    const abiInput=path.join(dir,'abi-input.json');fs.writeFileSync(abiInput,serialize({languageVersion,input:row.request},row.escapedJson));assert.equal(fs.statSync(abiInput).size,row.serializedAbiBytes);
    const observations={},issues=[];const start=performance.now();
    try {
      observations.cli=nativeCase(row,dir,controlFile,expected);
      if(!worker){const ready=await startWorker();assert.deepEqual(ready.control,expected);}
      const response=await message(()=>worker.postMessage({row}));json(path.join(dir,'worker.json'),response);lastCounts=response.counts;
      assert.equal(response.kind,'result',JSON.stringify(response));assert.equal(response.id,row.id);
      observations.abi={response:response.abi.response,status:response.abi.status,metadataHex:response.abi.metadataHex,rivLength:response.abi.rivLength};
      observations.js={response:response.response,control:response.control,checks:response.checks};
      for(const [boundary,observation] of Object.entries(observations)) {
        const actual=classify(observation.response);
        if(actual!==row.expect)issues.push({boundary,wanted:row.expect,actual,kind:'contract-mismatch'});
        assert.deepEqual(observation.response,observations.cli.response,'exact CLI/ABI/JS compiler response parity');
      }
      if(issues.length){defects++;log(`CONTRACT ${row.id}: ${JSON.stringify(issues)}`);}
    }catch(error){unexpected++;issues.push({kind:'harness-or-termination',name:error.name,message:error.message,stack:error.stack});stop();log(`ERROR ${row.id}: ${error.message.slice(0,200)}`);}
    const result={id:row.id,family:row.family,expect:row.expect,passed:issues.length===0,stats:row.stats,issues,observations,elapsedMs:performance.now()-start};
    if(issues.length)json(path.join(dir,'failure.json'),result);
    result.artifacts=fs.readdirSync(dir).sort().map(name=>({path:relative(path.join(dir,name)),sha256:hash(path.join(dir,name)),bytes:fs.statSync(path.join(dir,name)).size}));
    results.push(result);json(path.join(dir,'result.json'),result);
    log(`${index+1}/${selected.length} ${row.id}: ${issues.length?'FAIL':row.expect}`);
    if(unexpected>=5)break;
  }
}catch(error){setupFailure={name:error.name,message:error.message,stack:error.stack};json(path.join(output,'setup-failure.json'),setupFailure);}
finally {
  stop();
  const unchanged=inputs.map(input=>({kind:input.kind,original:hash(input.source)===input.sha256,snapshot:hash(path.join(output,input.snapshot))===input.sha256}));
  const sourceBindingsUnchanged=bindings.every(binding=>hash(binding.snapshot??binding.path)===binding.sha256);
  const passed=results.length===selected.length&&!setupFailure&&defects===0&&unexpected===0&&sourceBindingsUnchanged&&unchanged.every(item=>item.original&&item.snapshot);
  const totals={};for(const result of results)for(const [boundary,value] of Object.entries(result.observations)){const key=boundary+':'+classify(value.response);totals[key]=(totals[key]??0)+1;}
  const receipt={started,finished:new Date().toISOString(),passed,planned:all.length,selected:selected.length,completed:results.length,contractMismatches:defects,harnessOrTerminationFailures:unexpected,
    setupFailure,nativeCalls,workerStarts,lastWorkerCounts:lastCounts,workerEvents,totals,inputs,unchanged,sourceBindingsUnchanged,
    sourceSnapshots:bindings.filter(binding=>binding.snapshot).length,binaryBindings:bindings.filter(binding=>!binding.snapshot).length,
    caseManifestSha256:hash(path.join(output,'cases.json')),limits:{timeoutMs:TIMEOUT,maxNativeBufferBytes:MAX_BUFFER,workerOldGenerationMb:256,
      documentedSourceUtf8Bytes:SOURCE_LIMIT,documentedNesting:128,documentedAuthoredElements:8192,maxTestedSourceUtf8Bytes:Math.max(...selected.map(row=>row.stats.sourceUtf8Bytes)),maxTestedSerializedAbiBytes:Math.max(...selected.map(row=>row.serializedAbiBytes))},results};
  json(path.join(output,'receipt.json'),receipt);log(JSON.stringify({passed,completed:results.length,contractMismatches:defects,harnessOrTerminationFailures:unexpected,receipt:path.join(output,'receipt.json')}));
  if(!passed)process.exitCode=1;
}
