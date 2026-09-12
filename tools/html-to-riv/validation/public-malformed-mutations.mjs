// Deterministic, finite source mutation campaign. This is not coverage-guided fuzzing.
// Run from tools/html-to-riv: node validation/public-malformed-mutations.mjs
// Replay one retained case: add --only mutation-0000 --output output/public-malformed-mutations-r1/replay-0000
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {Worker} from 'node:worker_threads';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';

const moduleRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const option = (name, fallback) => args.includes(name) ? args[args.indexOf(name)+1] : fallback;
for (let i=0;i<args.length;i+=2) assert(['--frozen','--output','--only'].includes(args[i]) && args[i+1], 'unknown/missing argument');
const frozen = path.resolve(option('--frozen', path.join(moduleRoot,'output/public-content-owner-build-r1/frozen')));
const output = path.resolve(option('--output', path.join(moduleRoot,'output/public-malformed-mutations-r1')));
const allowedOutput = path.join(moduleRoot, 'output/public-malformed-mutations-r1');
assert(output === allowedOutput || output.startsWith(allowedOutput + path.sep));
assert(!fs.existsSync(output), `refusing to overwrite ${output}`);
fs.mkdirSync(output,{recursive:true});
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const hashFile = file => sha(fs.readFileSync(file));
const json = (file, value) => fs.writeFileSync(file, JSON.stringify(value,null,2)+'\n');
const read = file => JSON.parse(fs.readFileSync(file,'utf8'));
const rel = file => path.relative(output,file);
const log = text => {process.stdout.write(text+'\n');fs.appendFileSync(path.join(output,'run.log'),text+'\n');};
const started = new Date().toISOString();
const TIMEOUT_MS = 10000, MAX_BUFFER = 8*1024*1024, PRNG_SEED = 0x514f0531;
const sourceManifest = path.join(frozen,'source-bindings.json');
const bindings = read(sourceManifest).files;
for (const binding of bindings) assert.equal(hashFile(binding.snapshot ?? binding.path), binding.sha256, `frozen input changed: ${binding.path}`);
const saved = path.join(output,'frozen'); fs.mkdirSync(saved);
const inputs = [
  ['cli',path.join(frozen,'html-to-riv'),'html-to-riv'],
  ['wasm',path.join(frozen,'compiler.wasm'),'compiler.wasm'],
  ['wrapper',path.join(frozen,'inputs/js/index.mjs'),'index.mjs'],
  ['sourceBindings',sourceManifest,'source-bindings.json'],
  ['campaign',fileURLToPath(import.meta.url),'public-malformed-mutations.mjs'],
  ['worker',path.join(moduleRoot,'validation/public-malformed-worker.mjs'),'public-malformed-worker.mjs'],
].map(([kind,source,name]) => {
  const destination=path.join(saved,name); fs.copyFileSync(source,destination);
  if(kind==='cli') fs.chmodSync(destination,0o755);
  return {kind,source,sha256:hashFile(source),snapshot:rel(destination)};
});
const location = kind => path.join(output,inputs.find(input=>input.kind===kind).snapshot);
const fixtureSelections = [
  ['validation/public-baseline-cases.json',[0,10]],
  ['validation/public-selector-cases.json',[1,14]],
  ['validation/public-variable-cases.json',[1,4]],
  ['validation/public-variable-cycle-cases.json',[0,2]],
  ['validation/public-numeric-token-cases.json',[1,2]],
  ['validation/public-content-box-cases.json',[0,5]],
  ['validation/public-data-attributes-cases.json',[0,10]],
  ['validation/public-value-native-cases.json',[0,2]],
  ['validation/public-variable-recovery-cases.json',[0,4]],
  ['tests/fixtures/content-owner/scenes.json',[0,6]],
];
const seeds=[];
fs.mkdirSync(path.join(saved,'seed-fixtures'));
for (const [file,indices] of fixtureSelections) {
  const binding=bindings.find(binding=>binding.path===path.join(moduleRoot,file));
  // Source fixtures must come from the public compiler's own frozen source set.
  assert(binding, `seed not bound by public freeze: ${file}`);
  const fixture=read(binding.snapshot);
  fs.copyFileSync(binding.snapshot,path.join(saved,'seed-fixtures',path.basename(file)));
  for (const index of indices) {
    const row=fixture[index]; assert(row);
    const request=row.request ?? {html:row.html,css:row.css,width:240,height:160};
    seeds.push({id:`seed-${String(seeds.length).padStart(2,'0')}`,name:row.name,
      source:{file,index,sha256:binding.sha256},request});
  }
}
assert.equal(seeds.length,20);
let state=PRNG_SEED;
function random(n) { state^=state<<13;state^=state>>>17;state^=state<<5;return (state>>>0)%n; }
// These spans are mutation locations, not a parser or validity oracle. /u keeps
// whole Unicode scalars; escape prefixes and ASCII words receive useful spans.
function spans(text) {
  return [...text.matchAll(/\\[0-9a-fA-F]{1,6}[ \t\n\f\r]?|\\[^\r\n\f]|[-_a-zA-Z0-9]+|[ \t\n\f\r]+|[^]/gu)]
    .map(match=>({start:match.index,end:match.index+match[0].length,text:match[0]}));
}
const payloads={
  html:['<','</','>','"',"'",'=','<!--','-->','<div','&','&#0;','\0','\u00a0','\u2003','\ufeff','\\','é','😀',' data-x="a"',' style="width:var(--x,)"'],
  css:['\\','\\31 ','\\65 2px','(',')','{','}',':',';','#',':is(',':not(','[data-x=','/*','*/','var(--m,','--x:var(--x);','1e999px','1e-50px','\0','\u00a0','\u2003','\ufeff','\u000b','\n','calc(1px + 2px)','@media','--é','😀','1000001px'],
};
const cases=seeds.map(seed=>({id:seed.id,kind:'seed-control',seed:seed.id,expect:'success',request:seed.request}));
let mutationIndex=0;
for (const [seedIndex,seed] of seeds.entries()) for(const field of ['html','css']) {
  const original=seed.request[field], tokens=spans(original); assert(tokens.length>0);
  for(const operator of ['insert','delete','duplicate','truncate']) for(let variant=0;variant<6;variant++) {
    const selection=[0,tokens.length-1,Math.floor(tokens.length/2),random(tokens.length),random(tokens.length),random(tokens.length)][variant];
    const token=tokens[selection];
    const at=variant===1 ? token.end : token.start;
    let value, inserted='', removed='', start=at, end=at;
    if(operator==='insert') {
      inserted=payloads[field][(seedIndex*6+variant)%payloads[field].length];
      value=original.slice(0,at)+inserted+original.slice(at);
    } else if(operator==='delete') {
      start=token.start;end=tokens[Math.min(tokens.length-1,selection+(variant%3))].end;
      removed=original.slice(start,end);value=original.slice(0,start)+original.slice(end);
    } else if(operator==='duplicate') {
      start=token.end;end=start;inserted=token.text.repeat(variant%3+1);
      value=original.slice(0,start)+inserted+original.slice(start);
    } else {
      start=token.start;end=original.length;removed=original.slice(start);value=original.slice(0,start);
    }
    assert.notEqual(value,original);
    assert.equal(original.slice(start,end),removed);
    assert.equal(original.slice(0,start)+inserted+original.slice(end),value);
    assert.equal([...value].join(''),value);
    const request={...seed.request,[field]:value};
    cases.push({id:`mutation-${String(mutationIndex++).padStart(4,'0')}`,kind:'mutation',seed:seed.id,
      mutation:{field,operator,variant,spanIndex:selection,startUtf16:start,endUtf16:end,inserted,removed},request});
  }
}
assert.equal(mutationIndex,960);
const control={html:'<div id="box"></div>',css:'#box{width:20px;height:10px;background:teal}',width:240,height:160};
const controls=[
 // SUPPORT.md HTML contract and compiler.rs empty-document require one box.
 ['empty','rejection',{html:'',css:''}],
 ['ordinary','success',{}],
 ['cycle-fallback','success',{css:'#box{--a:var(--b);--b:var(--a);width:var(--a,20px);height:10px}'}],
 ['invalid-variable-recovery','success',{css:'#box{--a:red;width:var(--a);height:10px}'}],
 ['escaped-unit','success',{css:'#box{width:20p\\78 ;height:10px}'}],
 ['unicode-custom-property','success',{css:'#box{--é:20px;width:var(--é);height:10px}'}],
 ['css-whitespace','success',{css:'#box{width:\t\n\f\r 20px\t\n\f\r ;height:10px}'}],
 ['viewport-zero','rejection',{width:0}],
 ['viewport-too-large','rejection',{width:16385}],
 ['viewport-negative','rejection',{height:-1}],
 ['coefficient-limit','rejection',{css:'#box{width:1000001px}'}],
 ['nonfinite-number','rejection',{css:'#box{width:1e999px}'}],
 ['grid','rejection',{css:'#box{display:grid}'}],
 ['absolute-position','rejection',{css:'#box{position:absolute}'}],
 ['text','rejection',{html:'<div id="box">hello</div>'}],
 ['image-element','rejection',{html:'<img id="box">'}],
 ['transform','rejection',{css:'#box{transform:rotate(30deg)}'}],
 ['depth-129','rejection',{html:'<div>'.repeat(129)+'</div>'.repeat(129),css:'div{width:1px;height:1px}'}],
 ['non-css-whitespace','rejection',{css:'#box{width:\u00a020px}'}],
 ['escaped-exponent-unit','rejection',{css:'#box{width:1\\65 2px}'}],
];
for(const [name,expect,change] of controls) cases.push({id:`boundary-${name}`,kind:'boundary-control',expect,request:{...control,...change}});
assert.equal(cases.length,1000);
assert.equal(new Set(cases.map(row=>row.id)).size,cases.length);
for(const row of cases) {
  assert(typeof row.request.html==='string' && typeof row.request.css==='string');
  assert(Number.isFinite(row.request.width) && Number.isFinite(row.request.height));
  for(const field of ['html','css']) for(const scalar of row.request[field]) {
    const cp=scalar.codePointAt(0); assert(!(cp>=0xd800 && cp<=0xdfff),'unpaired surrogate');
  }
}
json(path.join(output,'seeds.json'),seeds);
json(path.join(output,'cases.json'),cases);
json(path.join(output,'command.json'),{started,cwd:process.cwd(),argv:process.argv,node:process.version,
  platform:process.platform,arch:process.arch,prng:'xorshift32',prngSeed:PRNG_SEED,timeoutMs:TIMEOUT_MS,
  maxCliBufferBytes:MAX_BUFFER,caseCount:cases.length,inputs,sourceSnapshotsVerified:bindings.filter(b=>b.snapshot).length,
  binaryBindingsVerified:bindings.filter(b=>!b.snapshot).length});
const selected=option('--only',null) ? cases.filter(row=>row.id===option('--only')) : cases;
assert(selected.length>0,'unknown replay case');

let nativeCalls=0;
function native(requestFile,prefix) {
  nativeCalls++;
  const start=performance.now();
  const result=spawnSync(location('cli'),[requestFile,prefix+'.riv'],{
    encoding:'utf8',timeout:TIMEOUT_MS,killSignal:'SIGKILL',maxBuffer:MAX_BUFFER});
  const command={argv:[location('cli'),requestFile,prefix+'.riv'],status:result.status,signal:result.signal,
    error:result.error ? {name:result.error.name,message:result.error.message,code:result.error.code}:null,
    stdout:result.stdout,stderr:result.stderr,elapsedMs:performance.now()-start};
  json(prefix+'.command.json',command);
  assert(!result.error && result.signal===null && [0,1].includes(result.status),`native termination: ${JSON.stringify(command)}`);
  assert.equal(result.stdout,'','unexpected native stdout');
  if(result.status===0) {
    assert.equal(result.stderr,'','unexpected native success stderr');
    assert(!fs.existsSync(prefix+'.requirements.json'));
    return {ok:true,rivHex:fs.readFileSync(prefix+'.riv').toString('hex'),sourceMap:read(prefix+'.map.json')};
  }
  const diagnostics=JSON.parse(result.stderr);
  assert(Array.isArray(diagnostics) && diagnostics.length>0,'native structured diagnostics');
  return {ok:false,diagnostics};
}
let worker, pending, workerStarts=0;
const workerEvents=[];
function stopWorker(){if(worker){worker.terminate();worker=undefined;}}
function awaitMessage(send) {
  return new Promise((resolve,reject)=>{
    const timer=setTimeout(()=>{pending=undefined;stopWorker();reject(new Error(`WASM worker exceeded ${TIMEOUT_MS}ms`));},TIMEOUT_MS);
    pending={resolve:value=>{clearTimeout(timer);pending=undefined;resolve(value);},reject:error=>{clearTimeout(timer);pending=undefined;reject(error);}};
    send?.();
  });
}
async function startWorker() {
  workerStarts++;
  const ready=awaitMessage();
  const current=new Worker(location('worker'),{workerData:{wrapper:location('wrapper'),wasm:location('wasm'),control},stdout:true,stderr:true,
    resourceLimits:{maxOldGenerationSizeMb:256}});
  worker=current;const serial=workerStarts;
  for(const [stream,name] of [[current.stdout,'stdout'],[current.stderr,'stderr']]) stream.on('data',bytes=>fs.appendFileSync(path.join(output,`worker-${serial}.${name}.log`),bytes));
  current.on('message',message=>{if(worker===current)pending?.resolve(message);});
  current.on('error',error=>{
    workerEvents.push({serial,event:'error',active:worker===current,pending:!!pending,name:error.name,message:error.message});
    if(worker===current)pending?.reject(error);
  });
  current.on('exit',code=>{
    workerEvents.push({serial,event:'exit',active:worker===current,pending:!!pending,code});
    if(worker===current && pending)pending.reject(new Error(`WASM worker exited ${code}`));
  });
  const response=await ready; assert.equal(response.kind,'ready');return response;
}
async function verifyWorkerRestart(expected) {
  const original=worker, retiredSerial=workerStarts;
  const retiredExit=new Promise((resolve,reject)=>{
    const timer=setTimeout(()=>reject(new Error('retired worker did not exit')),TIMEOUT_MS);
    original.once('exit',code=>{clearTimeout(timer);resolve(code);});
  });
  let expectedError;
  try {
    await awaitMessage(()=>original.postMessage({kind:'lifecycle-failure'}));
    assert.fail('deliberate worker exception did not reject the pending call');
  } catch(error) {
    assert.match(error.message,/^intentional malformed-campaign worker lifecycle control$/);
    expectedError={name:error.name,message:error.message};
  }
  stopWorker();
  const restarted=await startWorker();
  assert.deepEqual(restarted.control,expected,'successful isolated worker restart');
  const code=await retiredExit;
  assert.equal(code,1,'intentional exception worker exit');
  const oldExit=workerEvents.find(event=>event.serial===retiredSerial && event.event==='exit');
  assert(oldExit && !oldExit.active,'retired worker must not own current pending requests');
  return {passed:true,kind:'intentional JavaScript worker exception, no compiler call',expectedError,
    retiredSerial,restartedSerial:workerStarts,retiredExitCode:code,
    retiredExitDuringNewPendingCall:oldExit.pending,restartedControlMatches:true};
}
const rows=[];let failureCount=0;let setup,workerLifecycle;
try {
  const initDir=path.join(output,'control');fs.mkdirSync(initDir);
  const controlFile=path.join(initDir,'request.json');json(controlFile,control);
  const expected=native(controlFile,path.join(initDir,'initial'));assert.equal(expected.ok,true);
  setup=await startWorker();assert.deepEqual(setup.control,expected);
  json(path.join(initDir,'worker-initial.json'),setup);
  workerLifecycle=await verifyWorkerRestart(expected);
  json(path.join(initDir,'worker-lifecycle.json'),{...workerLifecycle,events:workerEvents});
  for (const [index,row] of selected.entries()) {
    const dir=path.join(output,'cases',row.id);fs.mkdirSync(dir,{recursive:true});
    const requestFile=path.join(dir,'request.json');json(requestFile,row.request);json(path.join(dir,'mutation.json'),row);
    let first,second,wasm,stage='native-first';const beginning=performance.now();
    try {
      first=native(requestFile,path.join(dir,'native-first'));
      stage='native-repeat';second=native(requestFile,path.join(dir,'native-repeat'));
      assert.deepEqual(second,first,'native deterministic output');
      if(!first.ok) for(const name of ['native-first','native-repeat']) {
        assert(!fs.existsSync(path.join(dir,name+'.riv')),'rejected CLI emitted Rive');
        assert(!fs.existsSync(path.join(dir,name+'.map.json')),'rejected CLI emitted map');
        assert(!fs.existsSync(path.join(dir,name+'.requirements.json')));
      }
      stage='wasm';
      if(!worker){const ready=await startWorker();assert.deepEqual(ready.control,expected);}
      wasm=await awaitMessage(()=>worker.postMessage({id:row.id,request:row.request}));
      json(path.join(dir,'wasm.json'),wasm);
      assert.equal(wasm.kind,'result',`WASM ${JSON.stringify(wasm)}`);
      assert.equal(wasm.id,row.id);
      assert.deepEqual(wasm.response,first,'exact native/WASM output or diagnostic parity');
      stage='expectation';
      if(row.expect)assert.equal(first.ok,row.expect==='success',`${row.id}: preserved ${row.expect}`);
      stage='native-output-preservation';
      if(!first.ok) {
        const prefix=path.join(dir,'retained-control');
        fs.copyFileSync(path.join(initDir,'initial.riv'),prefix+'.riv');
        fs.copyFileSync(path.join(initDir,'initial.map.json'),prefix+'.map.json');
        const hashes=[hashFile(prefix+'.riv'),hashFile(prefix+'.map.json')];
        assert.deepEqual(native(requestFile,prefix),first,'failure when caller output already exists');
        assert.deepEqual([hashFile(prefix+'.riv'),hashFile(prefix+'.map.json')],hashes,'failure changed owned native output');
        assert.deepEqual(native(controlFile,path.join(dir,'native-recovery')),expected,'native successful recovery');
        assert.deepEqual([hashFile(prefix+'.riv'),hashFile(prefix+'.map.json')],hashes,'recovery changed retained files');
      }
      rows.push({id:row.id,kind:row.kind,seed:row.seed,expect:row.expect,ok:true,accepted:first.ok,
        diagnostics:first.ok?undefined:first.diagnostics,calls:first.ok?2:4,elapsedMs:performance.now()-beginning});
    } catch(error) {
      failureCount++;
      const failure={id:row.id,kind:row.kind,ok:false,stage,elapsedMs:performance.now()-beginning,
        error:{name:error.name,message:error.message,stack:error.stack},first,second,wasm};
      json(path.join(dir,'failure.json'),failure);rows.push(failure);
      log(`FAIL ${row.id} at ${stage}: ${error.message.slice(0,200)}`);
      stopWorker();
    }
    rows.at(-1).artifacts=fs.readdirSync(dir).sort().map(name=>({path:rel(path.join(dir,name)),sha256:hashFile(path.join(dir,name)),bytes:fs.statSync(path.join(dir,name)).size}));
    json(path.join(dir,'result.json'),rows.at(-1));
    if((index+1)%100===0)log(`${index+1}/${selected.length}: ${rows.filter(r=>r.ok&&r.accepted).length} accepted, ${rows.filter(r=>r.ok&&!r.accepted).length} rejected, ${failureCount} failures`);
    if(failureCount>=10) break;
  }
} catch(error) {
  failureCount++;json(path.join(output,'setup-failure.json'),{name:error.name,message:error.message,stack:error.stack});
} finally {
  stopWorker();
  const immutableChecks=inputs.map(input=>({kind:input.kind,originalUnchanged:hashFile(input.source)===input.sha256,
    snapshotUnchanged:hashFile(path.join(output,input.snapshot))===input.sha256}));
  const sourcesUnchanged=bindings.every(binding=>hashFile(binding.snapshot ?? binding.path)===binding.sha256);
  const passed=rows.length===selected.length && failureCount===0 && sourcesUnchanged && immutableChecks.every(c=>c.originalUnchanged&&c.snapshotUnchanged);
  const receipt={started,finished:new Date().toISOString(),passed,planned:cases.length,selected:selected.length,completed:rows.length,
    uniqueRequests:new Set(cases.map(row=>sha(JSON.stringify(row.request)))).size,
    seedControls:seeds.length,boundaryControls:controls.length,mutations:mutationIndex,
    mutationCounts:{insert:240,delete:240,duplicate:240,truncate:240,html:480,css:480},
    accepted:rows.filter(row=>row.ok&&row.accepted).length,rejected:rows.filter(row=>row.ok&&!row.accepted).length,
    failures:failureCount,nativeCalls,workerStarts,workerLifecycle,workerEvents,wasmCaseCalls:rows.filter(row=>row.ok).length*3,
    wasmInitialCallsPerWorker:3,timeoutMs:TIMEOUT_MS,inputs,immutableChecks,sourceSnapshotsVerified:bindings.filter(b=>b.snapshot).length,
    binaryBindingsVerified:bindings.filter(b=>!b.snapshot).length,sourcesUnchanged,
    files:{seeds:{path:'seeds.json',sha256:hashFile(path.join(output,'seeds.json'))},cases:{path:'cases.json',sha256:hashFile(path.join(output,'cases.json'))}},
    exclusions:['malformed JSON and ill-typed transport requests','unpaired UTF-16 surrogates','coverage-guided fuzzing','native scene rendering','browser semantic or pixel comparison'],rows};
  json(path.join(output,'receipt.json'),receipt);
  log(JSON.stringify({passed,completed:rows.length,accepted:receipt.accepted,rejected:receipt.rejected,failures:failureCount,nativeCalls,receipt:path.join(output,'receipt.json')}));
  if(!passed)process.exitCode=1;
}
