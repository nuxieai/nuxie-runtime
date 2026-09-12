// Finite public asset resource/parser campaign; no renderer or production imports.
// Prepare only: node validation/public-image-assets.mjs --prepare --output output/public-image-assets-prepared-r1
// Execute: node validation/public-image-assets.mjs --frozen output/BUILD/frozen --output output/public-image-assets-r1
// The same file runs bounded worker checks. Requests, encoded files, outputs and
// exact diagnostics are retained. No claim about pre-serialization process memory.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {Worker,isMainThread,parentPort,workerData} from 'node:worker_threads';
import {deflateSync,inflateSync} from 'node:zlib';

const LANGUAGE='nuxie-html-immutable-v1',ENCODED_LIMIT=16*1024*1024,AXIS_LIMIT=8192,RGBA_LIMIT=16*1024*1024;
const TIMEOUT=60000,WORKER_HEAP_MB=768,MAX_STDIO=8*1024*1024;
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const hash=file=>sha(fs.readFileSync(file));
const json=(file,value)=>fs.writeFileSync(file,JSON.stringify(value,null,2)+'\n');
const read=file=>JSON.parse(fs.readFileSync(file,'utf8'));
const plain={html:'<div id="box"></div>',css:'#box{width:20px;height:10px;background:red}',width:240,height:160};
const imageInput={html:'<img id="image" src="image">',css:'#image{width:96px;height:64px}',width:240,height:160};
const diagnosticClass=result=>result.ok?'success':result.diagnostics[0].code;
const parserReasons=result=>result.ok?result:{...result,diagnostics:result.diagnostics.map(({message,...other})=>({...other,message:message.replace(/ at line \d+ column \d+$/,'')}))};
const brief=result=>result.ok?{ok:true,rivBytes:result.riv.length,rivSha256:sha(result.riv),sourceMap:result.sourceMap}:{ok:false,diagnostics:result.diagnostics};
function writeResult(prefix,result) {
  if(result.ok){fs.writeFileSync(prefix+'.riv',result.riv);json(prefix+'.map.json',result.sourceMap);}
  json(prefix+'.response.json',brief(result));return brief(result);
}
function packed(result) {
  if(result.ok){assert.equal(result.languageVersion,LANGUAGE);assert.deepEqual(Object.keys(result).sort(),['languageVersion','ok','riv','sourceMap']);return {ok:true,riv:Buffer.from(result.riv),sourceMap:result.sourceMap};}
  assert.deepEqual(Object.keys(result).sort(),['diagnostics','ok']);assert(Array.isArray(result.diagnostics)&&result.diagnostics.length);return result;
}

async function workerMain() {
  const {wrapper,wasmFile,cliInput,abiInput,directory,jsRepresentation}=workerData;
  const {createCompiler}=await import(pathToFileURL(wrapper));
  const wasm=fs.readFileSync(wasmFile),compiler=await createCompiler(wasm);
  const {instance}=await WebAssembly.instantiate(wasm,{}),abi=instance.exports;
  assert.equal(abi.html_compiler_abi_version(),2);
  let abiCalls=0,jsCalls=0,peakWasmBytes=abi.memory.buffer.byteLength;
  const js=request=>{jsCalls++;return compiler.compile({languageVersion:LANGUAGE,...request});};
  const copy=(ptr,len)=>{ptr>>>=0;len>>>=0;assert(ptr+len<=abi.memory.buffer.byteLength);return Buffer.from(new Uint8Array(abi.memory.buffer,ptr,len));};
  function raw(bytes,prefix) {
    const ptr=abi.html_compiler_request_alloc(bytes.length)>>>0;assert(ptr);
    assert.equal(abi.html_compiler_riv_len(),0);assert.equal(abi.html_compiler_metadata_len(),0);
    new Uint8Array(abi.memory.buffer,ptr,bytes.length).set(bytes);abiCalls++;
    const status=abi.html_compiler_compile();assert([0,1].includes(status));
    const metadata=copy(abi.html_compiler_metadata_ptr(),abi.html_compiler_metadata_len());
    const parsed=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(metadata));assert.equal(parsed.ok,status===0);
    const riv=copy(abi.html_compiler_riv_ptr(),abi.html_compiler_riv_len());if(!parsed.ok)assert.equal(riv.length,0);
    const response=packed(parsed.ok?{...parsed,riv}:parsed);
    peakWasmBytes=Math.max(peakWasmBytes,abi.memory.buffer.byteLength);
    abi.html_compiler_reset();assert.equal(abi.html_compiler_riv_len(),0);assert.equal(abi.html_compiler_metadata_len(),0);
    if(prefix){fs.writeFileSync(prefix+'.metadata.json',metadata);writeResult(prefix,response);}
    return {response,metadataSha256:sha(metadata),status};
  }
  const controlBytes=Buffer.from(JSON.stringify({languageVersion:LANGUAGE,input:plain}));
  const retainedJs=js(plain),retainedJsSnapshot=packed(retainedJs);assert(retainedJsSnapshot.ok);
  retainedJsSnapshot.sourceMap=structuredClone(retainedJsSnapshot.sourceMap);
  const retainedAbi=raw(controlBytes),retainedAbiSnapshot=structuredClone(brief(retainedAbi.response));
  assert.deepEqual(brief(retainedJsSnapshot),retainedAbiSnapshot);
  const requestBytes=fs.readFileSync(abiInput),requestHash=sha(requestBytes);
  const first=raw(requestBytes,path.join(directory,'abi-first'));
  const repeat=raw(requestBytes,path.join(directory,'abi-repeat'));
  assert.deepEqual(repeat,first,'raw ABI exact response and metadata determinism');
  assert.deepEqual(raw(controlBytes,path.join(directory,'abi-recovery')).response,retainedAbi.response,'raw ABI recovery');
  assert.deepEqual(brief(retainedAbi.response),retainedAbiSnapshot,'retained raw ABI copy changed');
  assert.equal(sha(requestBytes),requestHash,'raw request bytes changed');
  const observations={abi:{response:brief(first.response),status:first.status,metadataSha256:first.metadataSha256,
    checks:{deterministic:true,noStaleRive:true,resetCleared:true,retainedCopyPreserved:true,recovered:true}}};
  if(jsRepresentation!==null) {
    const request=JSON.parse(fs.readFileSync(cliInput,'utf8'));
    if(jsRepresentation==='typed-arrays'&&request.assets&&typeof request.assets==='object'&&!Array.isArray(request.assets)) {
      for(const value of Object.values(request.assets))if(value&&Array.isArray(value.bytes))value.bytes=new Uint8Array(value.bytes);
    }
    const owned=js(request),snapshot=packed(owned),before=structuredClone(brief(snapshot));
    writeResult(path.join(directory,'js-first'),snapshot);
    const second=packed(js(request));writeResult(path.join(directory,'js-repeat'),second);
    assert.deepEqual(second,snapshot,'JS deterministic response');
    const recovered=packed(js(plain));writeResult(path.join(directory,'js-recovery'),recovered);
    assert.deepEqual(recovered,retainedJsSnapshot,'JS recovery');
    assert.deepEqual(packed(retainedJs),retainedJsSnapshot,'retained JS result changed');
    assert.deepEqual(brief(packed(owned)),before,'original JS result changed after later calls');
    observations.js={response:before,checks:{deterministic:true,retainedOutputPreserved:true,currentOutputOwned:true,recovered:true}};
  }
  parentPort.postMessage({kind:'result',observations,counts:{abiCalls,jsCalls},rawAbiPeakMemoryBytes:peakWasmBytes,
    processMemoryObservation:process.memoryUsage()});
}

// PNG fixtures have explicit CRCs and filtered rows. Exact encoded sizes use
// legal empty stored DEFLATE blocks and contiguous empty IDAT chunks; no junk
// after zlib/IEND and no ancillary metadata. This makes aggregate limits testable
// with valid files while exact-byte aliases keep emitted output small.
function crc32(bytes) {let crc=0xffffffff;for(const byte of bytes){crc^=byte;for(let bit=0;bit<8;bit++)crc=(crc>>>1)^(0xedb88320&-(crc&1));}return (crc^0xffffffff)>>>0;}
const be32=value=>{const bytes=Buffer.alloc(4);bytes.writeUInt32BE(value);return bytes;};
const le32=value=>{const bytes=Buffer.alloc(4);bytes.writeUInt32LE(value);return bytes;};
function pngChunk(kind,data=Buffer.alloc(0)) {const body=Buffer.concat([Buffer.from(kind),data]);return Buffer.concat([be32(data.length),body,be32(crc32(body))]);}
function pngHeader(width,height) {const data=Buffer.alloc(13);data.writeUInt32BE(width);data.writeUInt32BE(height,4);data[8]=8;data[9]=6;return pngChunk('IHDR',data);}
const pngSignature=Buffer.from('89504e470d0a1a0a','hex');
function png(width,height,options={}) {
  const rows=options.rows??Buffer.alloc((width*4+1)*height),idat=options.idat??deflateSync(rows,{level:9});
  return Buffer.concat([pngSignature,pngHeader(width,height),pngChunk('IDAT',idat),...(options.extra??[]),pngChunk('IEND')]);
}
function exactPng(length) {
  const rows=Buffer.alloc(5);let selected;
  for(let extraBlocks=0;extraBlocks<12;extraBlocks++) {
    const remaining=length-(73+5*extraBlocks);
    if(remaining>=0&&remaining%12===0){selected={extraBlocks,emptyIdats:remaining/12};break;}
  }
  assert(selected);const chunks=[Buffer.from([0x78,0x01])];
  for(let i=0;i<selected.extraBlocks;i++)chunks.push(Buffer.from([0,0,0,255,255]));
  chunks.push(Buffer.from([1,5,0,250,255]),rows,Buffer.from([0,5,0,1]));
  const zlib=Buffer.concat(chunks);assert.deepEqual(inflateSync(zlib),rows);
  const encoded=Buffer.concat([pngSignature,pngHeader(1,1),pngChunk('IDAT',zlib),...Array.from({length:selected.emptyIdats},()=>pngChunk('IDAT')),pngChunk('IEND')]);
  assert.equal(encoded.length,length);return {encoded,construction:{width:1,height:1,filteredRows:5,...selected}};
}
function riff(chunks) {const body=Buffer.concat(chunks);return Buffer.concat([Buffer.from('RIFF'),le32(body.length+4),Buffer.from('WEBP'),body]);}
function webpChunk(kind,payload) {return Buffer.concat([Buffer.from(kind),le32(payload.length),payload,...(payload.length%2?[Buffer.from([0])]:[])]);}
function jpegSegment(marker,data) {const length=Buffer.alloc(2);length.writeUInt16BE(data.length+2);return Buffer.concat([Buffer.from([255,marker]),length,data]);}
function jpegScan(bytes) {const start=bytes.indexOf(Buffer.from([255,218]));assert(start>=0);const headerEnd=start+2+bytes.readUInt16BE(start+2);let end=headerEnd;while(end<bytes.length){if(bytes[end]!==255){end++;continue;}const marker=bytes[end+1];if(marker===0||(marker>=208&&marker<=215)){end+=2;continue;}break;}return {headerEnd,end};}

function prepare(root,output) {
  const cases=[],assets=new Map(),inputs=[];
  const put=(id,bytes,construction)=>{assert(!assets.has(id));const file=path.join(output,'assets',id);fs.writeFileSync(file,bytes);assets.set(id,{file,bytes,construction});return id;};
  const fixtureDirectory=path.join(root,'fixtures/images/ordinary-r1');
  const fixtures=['opaque.png','alpha.png','baseline.jpg','progressive.jpg','lossless.webp','lossy.webp','alpha.webp'];
  const fixtureManifest=read(path.join(fixtureDirectory,'manifest.json'));
  for(const name of fixtures){const source=path.join(fixtureDirectory,name),bytes=fs.readFileSync(source);assert.equal(sha(bytes),fixtureManifest.files.find(row=>row.name===name).sha256);put(name,bytes,{source});inputs.push({source,sha256:sha(bytes)});}
  const entry=(name,file)=>({name,file});
  const add=(id,family,entries,expect='success',extra={})=>cases.push({id,family,entries,expect,parity:'exact',jsRepresentation:'typed-arrays',input:imageInput,...extra});
  const one=(id,family,file,expect='success',extra={})=>add(id,family,[entry('image',file)],expect,extra);
  for(const name of fixtures)one('valid-'+name.replace('.','-'),'format-control',name);
  add('omitted-assets','empty-map-control',null,'success',{input:plain});
  add('empty-assets','empty-map-control',[],'success',{input:plain,equalTo:'omitted-assets'});
  one('numeric-array-input','representation-control','opaque.png','success',{jsRepresentation:'numeric-arrays',equalTo:'valid-opaque-png'});
  for(const count of [255,256,257])add(`names-${count}`,'name-count',Array.from({length:count},(_,i)=>entry(i===0?'image':`alias${String(i).padStart(3,'0')}`,'opaque.png')),count<=256?'success':'asset-limit',{equalTo:count<=256?'valid-opaque-png':undefined,expectedSource:count>256?'assets':undefined,stats:{suppliedNames:count}});
  for(const length of [65535,65536,65537]){const {encoded,construction}=exactPng(length);const file=put(`exact-${length}.png`,encoded,construction);one(`encoded-unit-${length}`,'encoded-valid-control',file);}
  for(const delta of [-1,0,1]) {
    const entries=Array.from({length:256},(_,i)=>entry(`a${String(i).padStart(3,'0')}`,`exact-${i===255?65536+delta:65536}.png`));
    add(`encoded-total-${delta<0?'below':delta>0?'above':'at'}`,'encoded-aggregate',entries,delta<=0?'success':'asset-limit',{
      input:{...imageInput,html:'<img id="image" src="a000">'},equalTo:delta===0?'encoded-unit-65536':undefined,
      expectedSource:delta>0?'assets["a255"]':undefined,stats:{suppliedNames:256,encodedBytes:ENCODED_LIMIT+delta},
      note:'All supplied files are valid independently; encoded aliases count before deduplication.'});
  }
  for(const axis of ['width','height'])for(const size of [8191,8192,8193]) {
    const width=axis==='width'?size:1,height=axis==='height'?size:1;
    one(`axis-${axis}-${size}`,'axis-boundary',put(`axis-${width}x${height}.png`,png(width,height),{width,height}),size<=AXIS_LIMIT?'success':'asset-limit',{stats:{width,height,decodedRgbaBytes:width*height*4}});
  }
  for(const [label,width,height] of [['below',2047,2049],['at',2048,2048],['above',1985,2113]]) {
    const decodedRgbaBytes=width*height*4;assert.equal(decodedRgbaBytes,RGBA_LIMIT+({below:-4,at:0,above:4})[label]);
    one(`decoded-rgba-${label}`,'decoded-boundary',put(`decoded-${label}.png`,png(width,height),{width,height}),label==='above'?'asset-limit':'success',{stats:{width,height,decodedRgbaBytes},note:'Nearest representable RGBA pixel immediately below/at/above 16 MiB; every axis remains below 8192.'});
  }
  for(const [label,width,height,expect] of [['zero',0,1,'invalid-image'],['u32-max',0xffffffff,0xffffffff,'asset-limit']])one(`png-dimensions-${label}`,'dimension-header-guard',put(`dimensions-${label}.png`,Buffer.concat([pngSignature,pngHeader(width,height),pngChunk('IDAT',deflateSync(Buffer.alloc(5))),pngChunk('IEND')]),{width,height,completePixels:false}),expect);
  const valid=png(1,1),idat=deflateSync(Buffer.alloc(5));
  for(const [label,offset] of [['header',29],['idat',valid.length-16],['end',valid.length-4]]){const bad=Buffer.from(valid);bad[offset]^=1;one(`png-crc-${label}`,'png-integrity',put(`crc-${label}.png`,bad,{mutation:'flip CRC byte',offset}),'invalid-image');}
  for(const [label,bytes] of [
    ['missing-iend',valid.subarray(0,valid.length-12)],['trailing-byte',Buffer.concat([valid,Buffer.from([0])])],
    ['missing-adler',png(1,1,{idat:idat.subarray(0,idat.length-1)})],
    ['trailing-zlib',png(1,1,{idat:Buffer.concat([idat,Buffer.from([0])])})],
    ['short-rows',png(1,1,{rows:Buffer.alloc(4)})],['long-rows',png(1,1,{rows:Buffer.alloc(6)})],
    ['invalid-filter',png(1,1,{rows:Buffer.from([5,0,0,0,0])})],
  ])one(`png-${label}`,'png-integrity',put(`malformed-${label}.png`,bytes,{mutation:label}),'invalid-image');
  {const bad=Buffer.from(idat);bad[bad.length-1]^=1;one('png-wrong-adler','png-integrity',put('wrong-adler.png',png(1,1,{idat:bad}),{mutation:'change Adler32, repair IDAT CRC'}),'invalid-image');}
  {const bad=Buffer.from(valid);bad.writeUInt32BE(0xffffffff,33);one('png-overflow-chunk-length','png-integrity',put('overflow-chunk.png',bad,{mutation:'IDAT length u32::MAX'}),'invalid-image');}
  for(const kind of ['eXIf','iCCP','acTL'])one(`png-late-${kind}`,'unsupported-metadata',put(`late-${kind}.png`,png(1,1,{extra:[pngChunk(kind,Buffer.alloc(8))]}),{mutation:'late metadata with valid CRC',kind}),'unsupported-image');
  for(const name of ['baseline.jpg','progressive.jpg']) {
    const bytes=assets.get(name).bytes,{headerEnd,end}=jpegScan(bytes),stem=name.replace('.jpg','');
    for(const [label,changed] of [
      ['empty-entropy',Buffer.concat([bytes.subarray(0,headerEnd),Buffer.from([255,217])])],
      ['scan-last-byte-removed',Buffer.concat([bytes.subarray(0,end-1),bytes.subarray(end)])],
      ['missing-eoi',bytes.subarray(0,bytes.length-1)],
      ['trailing-byte',Buffer.concat([bytes,Buffer.from([0])])],
    ])one(`${stem}-${label}`,'jpeg-entropy-container',put(`${stem}-${label}.jpg`,changed,{source:name,mutation:label,headerEnd,entropyEnd:end}),'invalid-image');
    one(`${stem}-late-app1`,'unsupported-metadata',put(`${stem}-late-app1.jpg`,Buffer.concat([bytes.subarray(0,bytes.length-2),jpegSegment(0xe1,Buffer.from('Exif\0\0')),Buffer.from([255,217])]),{source:name,mutation:'APP1 after entropy'}),'unsupported-image');
    for(const [label,width,height] of [['axis',8193,1],['rgba',1985,2113]]) {
      const changed=Buffer.from(bytes),sof=changed.indexOf(Buffer.from([255,stem==='baseline'?192:194]));assert(sof>=0);
      changed.writeUInt16BE(height,sof+5);changed.writeUInt16BE(width,sof+7);
      one(`${stem}-dimension-${label}`,'dimension-header-guard',put(`${stem}-${label}.jpg`,changed,{source:name,width,height,completePixels:false}),'asset-limit');
    }
  }
  const webp=assets.get('lossless.webp').bytes;
  const webpPayload=webp.subarray(20,20+webp.readUInt32LE(16));assert.equal(webpPayload.length%2,1);
  for(const [label,bytes] of [
    ['truncated',webp.subarray(0,webp.length-1)],['trailing-byte',Buffer.concat([webp,Buffer.from([0])])],
    ['duplicate-payload',riff([webp.subarray(12),webp.subarray(12)])],
    ['header-only',riff([webpChunk('VP8L',webpPayload.subarray(0,5))])],
  ])one(`webp-${label}`,'webp-integrity',put(`${label}.webp`,bytes,{mutation:label}),'invalid-image');
  for(const [label,offset,value] of [['nonzero-pad',webp.length-1,1],['overflow-length',16,0xffffffff]]) {
    const bytes=Buffer.from(webp);if(label==='nonzero-pad')bytes[offset]=value;else bytes.writeUInt32LE(value,offset);
    one(`webp-${label}`,'webp-integrity',put(`${label}.webp`,bytes,{mutation:label}),'invalid-image');
  }
  for(const kind of ['EXIF','ICCP','ANIM'])one(`webp-late-${kind}`,'unsupported-metadata',put(`late-${kind}.webp`,riff([webp.subarray(12),webpChunk(kind,Buffer.alloc(10))]),{mutation:'late metadata',kind}),'unsupported-image');
  for(const [label,width,height] of [['axis',8193,1],['rgba',1985,2113]]) {
    const payload=Buffer.from(webpPayload);payload.writeUInt32LE(((width-1)|((height-1)<<14))>>>0,1);
    one(`webp-dimension-${label}`,'dimension-header-guard',put(`dimensions-${label}.webp`,riff([webpChunk('VP8L',payload)]),{width,height,completePixels:false}),'asset-limit');
  }
  put('unknown.bin',Buffer.from([1,2,3]),{mutation:'unrecognized format'});
  one('empty-key','key-validation','opaque.png','invalid-asset',{entries:[entry('','opaque.png')],expectedSource:'assets[""]'});
  add('unused-invalid-asset','unused-validation',[entry('image','opaque.png'),entry('unused"key','unknown.bin')],'unsupported-image',{expectedSource:'assets["unused\\"key"]'});
  add('unicode-key-diagnostic-order','key-order',[entry('\u{10000}','unknown.bin'),entry('\ue000','unknown.bin'),entry('image','opaque.png')],'unsupported-image',{expectedSource:'assets["\ue000"]',note:'Rust scalar/UTF-8 ordering differs from JS UTF-16 sorting for these names.'});
  const rawAsset=JSON.stringify({kind:'image',bytes:[...assets.get('opaque.png').bytes]});
  for(const [id,rawAssets,message] of [
    ['duplicate-name',`{"image":${rawAsset},"image":${rawAsset}}`,'duplicate asset key "image"'],
    ['duplicate-escaped-name',`{"image":${rawAsset},"\\u0069mage":${rawAsset}}`,'duplicate asset key "image"'],
    ['duplicate-before-bad-value',`{"image":${rawAsset},"image":null}`,'duplicate asset key "image"'],
    ['duplicate-asset-kind','{"image":{"kind":"image","kind":"image","bytes":[]}}','duplicate field `kind`'],
    ['duplicate-asset-bytes','{"image":{"kind":"image","bytes":[],"bytes":[]}}','duplicate field `bytes`'],
    ['float-byte-spelling','{"image":{"kind":"image","bytes":[1.0]}}','invalid type: floating point'],
  ])add(id,'raw-json-parser',[],'invalid-request',{rawAssets,jsRepresentation:null,parity:'parser-reason',expectedMessageIncludes:message,note:'Raw JSON lexical/duplicate shape is not representable by a JS object; JS boundary intentionally omitted.'});
  for(const [id,value] of [['null-map',null],['array-map',[]],['array-asset',{image:[]}],['wrong-kind',{image:{kind:'font',bytes:[]}}],['negative-byte',{image:{kind:'image',bytes:[-1]}}],['overflow-byte',{image:{kind:'image',bytes:[256]}}],['unknown-asset-field',{image:{kind:'image',bytes:[],width:1}}]])add(id,'typed-request-parser',[],'invalid-request',{rawAssets:JSON.stringify(value),jsRepresentation:'numeric-arrays',parity:'parser-reason-js-class'});
  fs.mkdirSync(path.join(output,'cases'),{recursive:true});
  for(const row of cases) {
    const directory=path.join(output,'cases',row.id);fs.mkdirSync(directory);
    const body=JSON.stringify(row.input).slice(0,-1);
    const encodedAssets=row.rawAssets??(row.entries===null?null:`{${row.entries.map(({name,file})=>JSON.stringify(name)+':{"kind":"image","bytes":['+assets.get(file).bytes.join(',')+']}').join(',')}}`);
    const request=body+(encodedAssets===null?'':',"assets":'+encodedAssets)+'}';
    fs.writeFileSync(path.join(directory,'cli-input.json'),request);
    fs.writeFileSync(path.join(directory,'abi-input.json'),`{"languageVersion":${JSON.stringify(LANGUAGE)},"input":${request}}`);
    if(row.entries!==null&&!row.rawAssets){const total=row.entries.reduce((sum,entry)=>sum+assets.get(entry.file).bytes.length,0);if(row.stats?.encodedBytes!==undefined)assert.equal(total,row.stats.encodedBytes);row.stats={...row.stats,suppliedNames:row.entries.length,encodedBytes:total,uniqueEncodedFiles:new Set(row.entries.map(entry=>sha(assets.get(entry.file).bytes))).size};}
    row.stats={...row.stats,serializedCliBytes:Buffer.byteLength(request),serializedAbiBytes:fs.statSync(path.join(directory,'abi-input.json')).size};
    assert(row.stats.serializedAbiBytes<192*1024*1024);
    json(path.join(directory,'case.json'),row);
  }
  assert.equal(new Set(cases.map(row=>row.id)).size,cases.length);
  const expectationSources=['src/assets.rs','src/jpeg_validation.rs','src/request.rs','src/main.rs','src/wasm.rs','js/index.mjs'].map(name=>({path:name,sha256:hash(path.join(root,name))}));
  const manifest={cases,assets:[...assets].map(([id,entry])=>({id,path:path.relative(output,entry.file),sha256:sha(entry.bytes),bytes:entry.bytes.length,construction:entry.construction})),inputs,expectationSources,
    expectationBasis:{
      resource:'assets.rs validates 256 names and 16 MiB encoded aggregate before exact-byte deduplication, then positive dimensions, 8192 axes and width*height*4 <=16 MiB before full decode.',
      malformed:'assets.rs requires exact PNG zlib/chunk/CRC, JPEG framing plus jpeg_validation.rs entropy, and WebP RIFF/chunk/pixel decode; recognized malformed files are invalid-image and unqualified metadata is unsupported-image.',
      parser:'Asset and AssetMap use typed named-object visitors with duplicate-key rejection; raw CLI and ABI serde envelopes differ only in parser position for the included identical payloads. JS normalizes values earlier with its own documented diagnostics.',
      allocation:'main.rs reads the whole input and serde builds byte vectors before semantic limits. js/index.mjs copies and serializes inputs before its 192 MiB envelope check; no whole-process allocation bound is inferred.'}};
  json(path.join(output,'cases.json'),manifest);return manifest;
}

async function main() {
  const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..'),argv=process.argv.slice(2),args={};
  for(let i=0;i<argv.length;i++){const key=argv[i];assert(['--prepare','--frozen','--output','--only','--expect-cli','--expect-wasm'].includes(key),key);args[key]=key==='--prepare'?true:argv[++i];assert(args[key]);}
  assert(args['--output'],'--output is required');assert(args['--prepare']||args['--frozen'],'--frozen is required for execution');
  const output=path.resolve(args['--output']);assert(output.startsWith(path.join(root,'output/public-image-assets-')));assert(!fs.existsSync(output),'refusing to overwrite evidence');
  fs.mkdirSync(path.join(output,'assets'),{recursive:true});
  const started=new Date().toISOString(),self=fileURLToPath(import.meta.url),selfHash=hash(self);
  fs.copyFileSync(self,path.join(output,'prepared-driver.mjs'));
  const log=message=>{fs.appendFileSync(path.join(output,'run.log'),message+'\n');process.stdout.write(message+'\n');};
  const manifest=prepare(root,output),selected=args['--only']?manifest.cases.filter(row=>row.id===args['--only']):manifest.cases;assert(selected.length);
  json(path.join(output,'command.json'),{started,argv:process.argv,cwd:process.cwd(),node:process.version,zlib:process.versions.zlib,platform:process.platform,arch:process.arch,driver:{source:self,sha256:selfHash},limits:{timeoutMs:TIMEOUT,workerOldGenerationMb:WORKER_HEAP_MB,nativeStdioBytes:MAX_STDIO}});
  if(args['--prepare']){json(path.join(output,'receipt.json'),{prepared:true,executed:false,cases:manifest.cases.length,driverSha256:selfHash,driverSnapshot:'prepared-driver.mjs',manifestSha256:hash(path.join(output,'cases.json')),limits:{suppliedNames:256,encodedBytes:ENCODED_LIMIT,axis:AXIS_LIMIT,rgbaBytes:RGBA_LIMIT},scope:'Source-backed expected outcomes and independent fixture construction only; no compiler invoked.'});log(`Prepared ${manifest.cases.length} cases; no compiler invoked.`);return;}
  const sourceFrozen=path.resolve(args['--frozen']),bindingsFile=path.join(sourceFrozen,'source-bindings.json'),bindings=read(bindingsFile).files;
  for(const binding of bindings)assert.equal(hash(binding.snapshot??binding.path),binding.sha256);
  const frozen=path.join(output,'frozen');fs.mkdirSync(frozen);
  const inputs=[['cli',path.join(sourceFrozen,'html-to-riv'),'html-to-riv'],['wasm',path.join(sourceFrozen,'compiler.wasm'),'compiler.wasm'],['wrapper',path.join(sourceFrozen,'inputs/js/index.mjs'),'index.mjs'],['bindings',bindingsFile,'source-bindings.json'],['driver',self,'public-image-assets.mjs']].map(([kind,source,name])=>{const snapshot=path.join(frozen,name);fs.copyFileSync(source,snapshot);if(kind==='cli')fs.chmodSync(snapshot,0o755);return {kind,source,snapshot,sha256:hash(source)};});
  const location=kind=>inputs.find(input=>input.kind===kind).snapshot;
  if(args['--expect-cli'])assert.equal(hash(location('cli')),args['--expect-cli']);if(args['--expect-wasm'])assert.equal(hash(location('wasm')),args['--expect-wasm']);
  const results=[],knownSuccess=new Map();let nativeCalls=0,workerCalls={abiCalls:0,jsCalls:0},failures=0,setupFailure;
  const native=(input,prefix)=>{
    nativeCalls++;const start=performance.now(),child=spawnSync(location('cli'),[input,prefix+'.riv'],{encoding:'utf8',timeout:TIMEOUT,killSignal:'SIGKILL',maxBuffer:MAX_STDIO});
    json(prefix+'.command.json',{argv:[location('cli'),input,prefix+'.riv'],status:child.status,signal:child.signal,error:child.error?{name:child.error.name,message:child.error.message,code:child.error.code}:null,stdout:child.stdout,stderr:child.stderr,elapsedMs:performance.now()-start});
    assert(!child.error&&child.signal===null&&[0,1].includes(child.status),'CLI terminated unexpectedly');assert.equal(child.stdout,'');
    assert(!fs.existsSync(prefix+'.requirements.json'));
    if(child.status===0){assert.equal(child.stderr,'');return {ok:true,riv:fs.readFileSync(prefix+'.riv'),sourceMap:read(prefix+'.map.json')};}
    const diagnostics=JSON.parse(child.stderr);assert(Array.isArray(diagnostics)&&diagnostics.length);return {ok:false,diagnostics};
  };
  const runWorker=(row,directory)=>new Promise((resolve,reject)=>{
    const worker=new Worker(location('driver'),{workerData:{wrapper:location('wrapper'),wasmFile:location('wasm'),cliInput:path.join(directory,'cli-input.json'),abiInput:path.join(directory,'abi-input.json'),directory,jsRepresentation:row.jsRepresentation},resourceLimits:{maxOldGenerationSizeMb:WORKER_HEAP_MB},stdout:true,stderr:true});
    let settled=false;const finish=(error,value)=>{if(settled)return;settled=true;clearTimeout(timer);worker.terminate();error?reject(error):resolve(value);};
    const timer=setTimeout(()=>finish(new Error('bounded worker timed out')),TIMEOUT);
    worker.stdout.on('data',bytes=>fs.appendFileSync(path.join(directory,'worker.stdout.log'),bytes));worker.stderr.on('data',bytes=>fs.appendFileSync(path.join(directory,'worker.stderr.log'),bytes));
    worker.on('message',value=>value.kind==='result'?finish(null,value):finish(new Error(JSON.stringify(value))));worker.on('error',error=>finish(error));worker.on('exit',code=>{if(!settled)finish(new Error(`worker exited ${code}`));});
  });
  try {
    const control=path.join(output,'control');fs.mkdirSync(control);const controlInput=path.join(control,'input.json');json(controlInput,plain);
    const expected=native(controlInput,path.join(control,'native'));assert(expected.ok);let retained=expected;
    for(const [index,row] of selected.entries()) {
      const directory=path.join(output,'cases',row.id),request=path.join(directory,'cli-input.json'),observations={},issues=[];const start=performance.now();
      try {
        const first=native(request,path.join(directory,'cli-first')),repeat=native(request,path.join(directory,'cli-repeat'));assert.deepEqual(repeat,first,'CLI deterministic result');
        if(!first.ok)for(const label of ['cli-first','cli-repeat'])for(const suffix of ['riv','map.json'])assert(!fs.existsSync(path.join(directory,`${label}.${suffix}`)),'rejection published new output');
        const retainedPrefix=path.join(directory,'cli-retained');fs.writeFileSync(retainedPrefix+'.riv',retained.riv);json(retainedPrefix+'.map.json',retained.sourceMap);
        const before=[hash(retainedPrefix+'.riv'),hash(retainedPrefix+'.map.json')];
        if(!first.ok){assert.deepEqual(native(request,retainedPrefix),first);assert.deepEqual([hash(retainedPrefix+'.riv'),hash(retainedPrefix+'.map.json')],before,'CLI failure modified prior successful files');}
        assert.deepEqual(native(controlInput,path.join(directory,'cli-recovery')),expected,'CLI recovery');
        if(first.ok){retained=first;knownSuccess.set(row.id,brief(first));if(row.equalTo&&knownSuccess.has(row.equalTo))assert.deepEqual(brief(first),knownSuccess.get(row.equalTo),'alias/empty-map output equality');}
        observations.cli={response:brief(first),checks:{deterministic:true,rejectionNoNewOutput:!first.ok?true:null,retainedFilesPreserved:!first.ok?true:null,recovered:true},retainedHashes:before};
        const worker=await runWorker(row,directory);json(path.join(directory,'worker.json'),worker);Object.assign(observations,worker.observations);for(const [key,value] of Object.entries(worker.counts))workerCalls[key]+=value;
        for(const [boundary,observation] of Object.entries(observations)) {
          assert.equal(diagnosticClass(observation.response),row.expect,`${boundary} contract classification`);
          if(!observation.response.ok&&row.expectedSource)assert.equal(observation.response.diagnostics[0].source,row.expectedSource);
          if(!observation.response.ok&&row.expectedMessageIncludes)assert(observation.response.diagnostics[0].message.includes(row.expectedMessageIncludes));
          if(row.parity==='exact')assert.deepEqual(observation.response,observations.cli.response,`${boundary} exact CLI parity`);
          else if(boundary==='abi')assert.deepEqual(parserReasons(observation.response),parserReasons(observations.cli.response),'CLI/ABI typed reason parity (envelope column omitted)');
        }
        // Hashes in observations index evidence; compare the complete successful
        // output files as well so parity does not depend only on a digest oracle.
        if(first.ok)for(const boundary of ['abi','js'])if(observations[boundary])assert.deepEqual(fs.readFileSync(path.join(directory,boundary+'-first.riv')),first.riv,'complete successful Rive bytes');
      }catch(error){failures++;issues.push({name:error.name,message:error.message,stack:error.stack});json(path.join(directory,'failure.json'),issues);}
      const result={id:row.id,family:row.family,expect:row.expect,passed:issues.length===0,observations,issues,elapsedMs:performance.now()-start};
      result.artifacts=fs.readdirSync(directory).sort().map(name=>({path:path.relative(output,path.join(directory,name)),sha256:hash(path.join(directory,name)),bytes:fs.statSync(path.join(directory,name)).size}));
      results.push(result);json(path.join(directory,'result.json'),result);log(`${index+1}/${selected.length} ${row.id}: ${result.passed?'PASS':'FAIL '+issues[0].message}`);
      if(failures>=5)break;
    }
  }catch(error){setupFailure={name:error.name,message:error.message,stack:error.stack};json(path.join(output,'setup-failure.json'),setupFailure);}
  const unchanged=inputs.map(input=>({kind:input.kind,original:hash(input.source)===input.sha256,snapshot:hash(input.snapshot)===input.sha256}));
  const sourceBindingsUnchanged=bindings.every(binding=>hash(binding.snapshot??binding.path)===binding.sha256);
  const fixtureInputsUnchanged=manifest.inputs.every(input=>hash(input.source)===input.sha256);
  const passed=!setupFailure&&failures===0&&results.length===selected.length&&sourceBindingsUnchanged&&fixtureInputsUnchanged&&unchanged.every(item=>item.original&&item.snapshot);
  const receipt={started,finished:new Date().toISOString(),passed,planned:manifest.cases.length,selected:selected.length,completed:results.length,failures,setupFailure,nativeCalls,workerCalls,inputs,unchanged,sourceBindingsUnchanged,fixtureInputsUnchanged,
    caseManifestSha256:hash(path.join(output,'cases.json')),results,limits:{timeoutMs:TIMEOUT,workerOldGenerationMb:WORKER_HEAP_MB,nativeStdioBytes:MAX_STDIO,suppliedNames:256,encodedBytes:ENCODED_LIMIT,axis:AXIS_LIMIT,rgbaBytes:RGBA_LIMIT,maxSerializedAbiBytes:Math.max(...selected.map(row=>row.stats.serializedAbiBytes))},
    coverageLimits:['Finite compiler/transport checks; no native or browser rendering.','No arbitrary pointers or 192 MiB boundary allocation.','JS normalization/serialization and CLI read/serde parsing allocate before semantic asset guards; this campaign does not establish pre-serialization or whole-process memory bounds.','Raw JSON duplicate keys and numeric-token spelling have no equivalent JS object representation; JS is intentionally omitted for those cases.','Fresh bounded worker per case; output ownership and recovery are tested within each reused ABI/JS instance, not long-duration memory growth.','JPEG strict entropy and pixel parity have separate evidence; this campaign repeats only small public rejection controls.']};
  json(path.join(output,'receipt.json'),receipt);log(JSON.stringify({passed,completed:results.length,failures,receipt:path.join(output,'receipt.json')}));if(!passed)process.exitCode=1;
}

if(isMainThread)await main();else workerMain().catch(error=>parentPort.postMessage({kind:'failure',error:{name:error.name,message:error.message,stack:error.stack}}));
