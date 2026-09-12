// Independent artifact check; compiles and renders nothing.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
assert(process.argv.length===3,'usage: node validation/public-resource-boundaries-verify.mjs output/public-resource-boundaries-r1');
const output=path.resolve(process.argv[2]),target=path.join(output,'artifact-verification.json');
assert(output.startsWith(path.join(root,'output/public-resource-boundaries-')));assert(!fs.existsSync(target),'refusing to overwrite verification');
const hash=file=>createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const read=file=>JSON.parse(fs.readFileSync(file,'utf8'));
const receipt=read(path.join(output,'receipt.json')),rows=read(path.join(output,'cases.json'));
const helper=await import(pathToFileURL(path.join(output,'frozen/public-resource-boundaries-cases.mjs')));
assert.equal(receipt.passed,true);assert.equal(receipt.completed,40);assert.equal(rows.length,40);
assert.equal(receipt.caseManifestSha256,hash(path.join(output,'cases.json')));
assert.deepEqual(rows,JSON.parse(JSON.stringify(helper.cases())),'case manifest differs from frozen generator');
let artifactCount=0,sourceBindingCount=0,nativeCommands=0,successes=0,failures=0,exactMapFiles=0;
const outcomes={};
for(const input of receipt.inputs)assert.equal(hash(path.join(output,input.snapshot)),input.sha256);
for(const binding of read(path.join(output,'frozen/source-bindings.json')).files){assert.equal(hash(binding.snapshot??binding.path),binding.sha256);sourceBindingCount++;}
for(const row of rows){
  const result=receipt.results.find(result=>result.id===row.id),dir=path.join(output,'cases',row.id);
  assert(result?.passed);assert.equal(result.issues.length,0);assert.deepEqual(read(path.join(dir,'result.json')),result);
  for(const artifact of result.artifacts){const file=path.join(output,artifact.path);assert.equal(hash(file),artifact.sha256);assert.equal(fs.statSync(file).size,artifact.bytes);artifactCount++;}
  const nativeJson=fs.readFileSync(path.join(dir,'cli-input.json'),'utf8'),abiJson=fs.readFileSync(path.join(dir,'abi-input.json'),'utf8');
  assert.deepEqual(JSON.parse(nativeJson),row.request);assert.deepEqual(JSON.parse(abiJson),{languageVersion:helper.languageVersion,input:row.request});
  assert.equal(Buffer.byteLength(nativeJson),row.serializedCliBytes);assert.equal(Buffer.byteLength(abiJson),row.serializedAbiBytes);
  assert.deepEqual(helper.stats(row.request),row.stats);
  if(row.sourceBoundaryOffset!==undefined)assert.equal(row.stats.sourceUtf8Bytes,helper.SOURCE_LIMIT+row.sourceBoundaryOffset);
  if(row.escapedJson){assert(!/[^\x00-\x7f]/.test(nativeJson));assert(nativeJson.includes('\\ud83d\\ude00'));assert(row.serializedCliBytes>row.stats.sourceUtf8Bytes*2);}
  for(const observation of Object.values(result.observations))assert.deepEqual(observation.response,result.observations.cli.response);
  const response=result.observations.cli.response;
  assert.equal(response.ok?'success':response.diagnostics[0].code,row.expect);outcomes[row.expect]=(outcomes[row.expect]??0)+1;
  for(const name of fs.readdirSync(dir).filter(name=>name.endsWith('.command.json'))){const command=read(path.join(dir,name));assert.equal(command.error,null);assert.equal(command.signal,null);assert([0,1].includes(command.status));nativeCommands++;}
  const worker=read(path.join(dir,'worker.json'));assert.equal(worker.kind,'result');assert(Object.values(worker.checks).every(value=>value===true));
  const metadata=JSON.parse(Buffer.from(worker.abi.metadataHex,'hex').toString('utf8'));
  assert.equal(metadata.ok,response.ok);
  if(response.ok){
    successes++;helper.identities(response.sourceMap,row.identities);
    assert.equal(fs.readFileSync(path.join(dir,'native-first.riv')).toString('hex'),response.rivHex);
    assert.deepEqual(read(path.join(dir,'native-first.map.json')),response.sourceMap);
    for(const name of ['native-repeat','native-noop-control'])for(const ext of ['riv','map.json']){
      assert.equal(hash(path.join(dir,'native-first.'+ext)),hash(path.join(dir,name+'.'+ext)),'native repeat and no-op file bytes differ');
      if(ext==='map.json')exactMapFiles++;
    }
    assert.deepEqual(metadata.sourceMap,response.sourceMap);assert.equal(metadata.languageVersion,helper.languageVersion);
    assert.equal(worker.abi.rivLength,response.rivHex.length/2);
    assert.deepEqual(worker.control,response);assert.deepEqual(result.observations.cli.control,response);
  }else{
    failures++;assert.equal(worker.abi.rivLength,0);assert.deepEqual(metadata.diagnostics,response.diagnostics);
    for(const name of ['native-first','native-repeat'])for(const ext of ['riv','map.json','requirements.json'])assert(!fs.existsSync(path.join(dir,name+'.'+ext)));
    const retained=result.observations.cli.retainedHashes;
    assert.equal(hash(path.join(dir,'retained-success.riv')),retained.riv);assert.equal(hash(path.join(dir,'retained-success.map.json')),retained.map);
    const source=receipt.results.find(result=>result.id===retained.sourceCase).observations.cli.response;
    assert.equal(fs.readFileSync(path.join(dir,'retained-success.riv')).toString('hex'),source.rivHex);
    assert.deepEqual(read(path.join(dir,'retained-success.map.json')),source.sourceMap);
    assert.equal(hash(path.join(dir,'native-recovery.riv')),hash(path.join(output,'control/native.riv')));
    assert.equal(hash(path.join(dir,'native-recovery.map.json')),hash(path.join(output,'control/native.map.json')));
  }
}
assert.equal(nativeCommands+1,receipt.nativeCalls);assert.equal(receipt.nativeCalls,136);
assert.deepEqual(receipt.lastWorkerCounts,{jsCalls:146,abiCalls:121});assert.equal(receipt.workerStarts,1);
assert.deepEqual(outcomes,{success:25,'input-limit':9,'depth-limit':3,'object-limit':3});
const script=fileURLToPath(import.meta.url),snapshot=path.join(output,'frozen/public-resource-boundaries-verify.mjs');fs.copyFileSync(script,snapshot);
fs.writeFileSync(target,JSON.stringify({passed:true,checkedAt:new Date().toISOString(),argv:process.argv,cwd:process.cwd(),
  script:{source:script,snapshot:path.relative(output,snapshot),sha256:hash(snapshot)},receiptSha256:hash(path.join(output,'receipt.json')),
  artifactCount,sourceBindingCount,caseCount:rows.length,boundaryObservations:rows.length*3,nativeCalls:receipt.nativeCalls,
  jsCalls:receipt.lastWorkerCounts.jsCalls,abiCalls:receipt.lastWorkerCounts.abiCalls,successes,failures,outcomes,exactMapFileComparisons:exactMapFiles,
  assertions:['All frozen binding and recorded case artifact hashes match','Exact byte lengths, Unicode scalar counts, UTF-16 counts, JSON decoding and source boundary offsets',
    'Frozen generator reproduces every complete request and expectation','Exact CLI/raw ABI/public JS responses','Native repeat/no-op Rive and map files byte-for-byte identical',
    'Rejected native requests publish no fresh files and retain preceding successful outputs','Recovery files match initial successful control','Authored identities and paths preserved at admitted limits']},null,2)+'\n');
console.log(JSON.stringify({passed:true,caseCount:rows.length,artifactCount,sourceBindingCount,verification:target}));
