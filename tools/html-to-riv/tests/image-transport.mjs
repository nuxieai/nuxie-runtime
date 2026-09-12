// Run after the coordinated public compiler native/WASM build, from repository root.
// HTML_TO_RIV_BIN=... HTML_TO_RIV_WASM=... node --test tools/html-to-riv/tests/image-transport.mjs
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createCompiler, LANGUAGE_VERSION} from '../js/index.mjs';

const binary=path.resolve(process.env.HTML_TO_RIV_BIN ?? 'tools/html-to-riv/target/debug/html-to-riv');
const wasm=fs.readFileSync(path.resolve(process.env.HTML_TO_RIV_WASM ?? 'tools/html-to-riv/target/wasm32-unknown-unknown/debug/nuxie_html_to_riv.wasm'));
const fixtures=new URL('../fixtures/images/ordinary-r1/',import.meta.url);
const names=['opaque.png','alpha.png','baseline.jpg','progressive.jpg','lossless.webp','lossy.webp','alpha.webp'];
const png=fs.readFileSync(new URL('opaque.png',fixtures));
const input={html:'<img id="image" src="logo">',css:'#image{width:96px;height:64px}',width:240,height:160};
const source=(assets,other={})=>({languageVersion:LANGUAGE_VERSION,...input,...other,assets});
const asset=bytes=>({kind:'image',bytes});
const plain=()=>({languageVersion:LANGUAGE_VERSION,html:'<div id="box"></div>',css:'#box{width:20px;height:10px;background:red}',width:240,height:160});
function success(result) { assert.equal(result.ok,true,JSON.stringify(result));return result; }
function rejection(result,code='invalid-request') {
  assert.equal(result.ok,false,JSON.stringify(result));assert.equal(result.diagnostics[0].code,code);
  assert(!Object.hasOwn(result,'riv'));assert(!Object.hasOwn(result,'sourceMap'));
}
function cli(request) {
  const dir=fs.mkdtempSync(path.join(os.tmpdir(),'immutable-image-transport-'));
  try {
    const inputFile=path.join(dir,'request.json'),riv=path.join(dir,'scene.riv');
    fs.writeFileSync(inputFile,typeof request==='string'?request:JSON.stringify(request));
    const process=spawnSync(binary,[inputFile,riv],{encoding:'utf8',maxBuffer:8*1024*1024});
    if(process.status!==0) {
      assert(!fs.existsSync(riv));assert(!fs.existsSync(path.join(dir,'scene.map.json')));
      return {ok:false,diagnostics:JSON.parse(process.stderr.trim())};
    }
    assert(!fs.existsSync(path.join(dir,'scene.requirements.json')));
    return {ok:true,riv:fs.readFileSync(riv),sourceMap:JSON.parse(fs.readFileSync(path.join(dir,'scene.map.json')))};
  }finally{fs.rmSync(dir,{recursive:true,force:true});}
}
async function rawCompiler() {
  const {instance}=await WebAssembly.instantiate(wasm,{}),w=instance.exports;
  return request=>{
    const bytes=new TextEncoder().encode(typeof request==='string'?request:JSON.stringify(request));
    const ptr=w.html_compiler_request_alloc(bytes.length)>>>0;assert(ptr);
    new Uint8Array(w.memory.buffer,ptr,bytes.length).set(bytes);
    const status=w.html_compiler_compile();
    const result=JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer,w.html_compiler_metadata_ptr()>>>0,w.html_compiler_metadata_len()>>>0)));
    const riv=new Uint8Array(w.memory.buffer,w.html_compiler_riv_ptr()>>>0,w.html_compiler_riv_len()>>>0).slice();
    assert.equal(status,result.ok?0:1);if(!result.ok)assert.equal(riv.length,0);
    w.html_compiler_reset();
    return result.ok?{...result,riv}:result;
  };
}

test('all seven authored image formats have exact CLI/raw WASM/JS parity',async()=>{
  const compiler=await createCompiler(wasm),raw=await rawCompiler();
  for(const name of names)for(const [width,height]of[[240,160],[390,200],[768,120]]) {
    const encoded=fs.readFileSync(new URL(name,fixtures));
    const request={...input,width,height,assets:{logo:asset([...encoded])}};
    const a=success(cli(request)),b=success(raw({languageVersion:LANGUAGE_VERSION,input:request})),c=success(compiler.compile(source({logo:asset(encoded)},{width,height})));
    assert.deepEqual(Buffer.from(b.riv),a.riv,name);assert.deepEqual(Buffer.from(c.riv),a.riv,name);
    assert.deepEqual(b.sourceMap,a.sourceMap,name);assert.deepEqual(c.sourceMap,a.sourceMap,name);
    assert.deepEqual(Object.keys(c).sort(),['languageVersion','ok','riv','sourceMap']);
  }
});

test('empty and omitted assets retain the old ordinary request output',async()=>{
  const compiler=await createCompiler(wasm),doc=plain(),expected=success(compiler.compile(doc));
  for(const assets of[{},Object.create(null),undefined])assert.deepEqual(compiler.compile({...doc,assets}),expected);
  const {languageVersion,...request}=doc;
  const actual=success(cli({...request,assets:{}}));assert.deepEqual(actual.riv,Buffer.from(expected.riv));assert.deepEqual(actual.sourceMap,expected.sourceMap);
});

test('typed subarrays and readonly numeric arrays are copied without input mutation or user iterators',async()=>{
  const compiler=await createCompiler(wasm),backing=new Uint8Array(png.length+4);backing.set(png,2);
  const view=backing.subarray(2,png.length+2),before=backing.slice();
  view[Symbol.iterator]=()=>{throw new Error('input iterator must not be called');};
  const numeric=Object.freeze([...png]);
  const expected=success(compiler.compile(source({logo:asset(numeric)})));
  assert.deepEqual(compiler.compile(source({logo:asset(view)})),expected);assert.deepEqual(backing,before);
  const withToJSON=[...png];withToJSON.toJSON=()=>{throw new Error('byte toJSON must not be called');};
  assert.deepEqual(compiler.compile(source({logo:asset(withToJSON)})),expected);
});

test('own special and case-sensitive keys survive transport and exact-byte aliases deduplicate',async()=>{
  const compiler=await createCompiler(wasm),one=success(compiler.compile(source({logo:asset(png)})));
  const withAlias=Object.fromEntries([['alias',asset(png)],['logo',asset([...png])]]);
  assert.deepEqual(compiler.compile(source(withAlias)),one);
  for(const name of['__proto__','constructor','toString','toJSON','Logo','\u{1f680}']) {
    const assets=Object.fromEntries([[name,asset(png)]]),html=`<img id="image" src="${name}">`;
    const actual=success(compiler.compile(source(assets,{html})));assert.deepEqual(actual.riv,one.riv,name);assert.deepEqual(actual.sourceMap,one.sourceMap);
    assert.equal(Object.getPrototypeOf(assets),Object.prototype);
  }
  rejection(compiler.compile(source({Logo:asset(png)})),'missing-image');
  const alpha=fs.readFileSync(new URL('alpha.png',fixtures)),entries=[['z',asset(png)],['a',asset(alpha)]];
  assert.deepEqual(compiler.compile(source(Object.fromEntries(entries),{html:'<img id="image" src="a">'})),compiler.compile(source(Object.fromEntries(entries.toReversed()),{html:'<img id="image" src="a">'})));
});

test('malformed image transport values stay structured and do not publish',async()=>{
  const compiler=await createCompiler(wasm);
  const values=[null,[],3,'image',{logo:null},{logo:[]},{logo:{kind:'font',bytes:[]}},
    {logo:{kind:'image'}},{logo:{bytes:[]}},{logo:{kind:'image',bytes:[],width:96}},
    {logo:asset(new ArrayBuffer(8))},{logo:asset(new Uint16Array(8))},{logo:asset({0:137,length:1})},
    ...[[-1],[256],[.5],[NaN],[Infinity],['1'],[null],[undefined],new Array(1)].map(bytes=>({logo:asset(bytes)}))];
  for(const assets of values) {
    const first=compiler.compile(source(assets));rejection(first);assert.deepEqual(compiler.compile(source(assets)),first);
  }
  success(compiler.compile(source({logo:asset(png)})));
});

test('asset and byte getters preserve safe structured request-error normalization',async()=>{
  const compiler=await createCompiler(wasm),badString={toString(){throw new Error('secondary');}};
  for(const[thrown,message]of[[new Error('getter failed'),'Error: getter failed'],[Object.create(null),'Cannot read or serialize design document'],[badString,'Cannot read or serialize design document']]) {
    const getter=()=>{throw thrown;};
    const named=Object.defineProperty({},'logo',{enumerable:true,get:getter});
    const nested={kind:'image',get bytes(){return getter();}};
    const bytes=[0];Object.defineProperty(bytes,0,{enumerable:true,get:getter});
    for(const assets of[named,{logo:nested},{logo:asset(bytes)}])assert.deepEqual(compiler.compile(source(assets)),{ok:false,diagnostics:[{code:'invalid-request',source:'request',message}]});
  }
  success(compiler.compile(source({logo:asset(png)})));
});

test('asset validation failure releases output and retained successful results survive later calls',async()=>{
  const compiler=await createCompiler(wasm),first=success(compiler.compile(source({logo:asset(png)}))),retained=first.riv.slice();
  const bad={...input,assets:{logo:asset([...png.subarray(0,24)])}};
  const result=compiler.compile({languageVersion:LANGUAGE_VERSION,...bad});rejection(result,'invalid-image');
  assert.deepEqual(cli(bad).diagnostics,result.diagnostics);
  assert.deepEqual(success(compiler.compile(source({logo:asset(png)}))).riv,retained);assert.deepEqual(first.riv,retained);
  first.riv[0]^=255;assert.deepEqual(success(compiler.compile(source({logo:asset(png)}))).riv,retained);
});

test('raw ABI and CLI reject malformed named asset values without stale output',async()=>{
  const raw=await rawCompiler();
  for(const assets of[null,[],{logo:['image',[]]},{logo:{kind:'image',bytes:[-1]}},{logo:{kind:'image',bytes:[],unknown:1}},{logo:{kind:'font',bytes:[]}}]) {
    const request={...input,assets},actual=raw({languageVersion:LANGUAGE_VERSION,input:request});rejection(actual);
    const native=cli(request);rejection(native);
    // The ABI envelope adds languageVersion/input around the CLI input, so
    // serde's byte column differs while its typed reason remains the same.
    const reasons=result=>result.diagnostics.map(({message,...rest})=>({...rest,message:message.replace(/ at line \d+ column \d+$/,'')}));
    assert.deepEqual(reasons(native),reasons(actual));
  }
  success(raw({languageVersion:LANGUAGE_VERSION,input:{...input,assets:{logo:asset([...png])}}}));
});
