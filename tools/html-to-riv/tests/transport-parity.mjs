// Run after coordinated native/WASM builds:
// HTML_TO_RIV_BIN=... HTML_TO_RIV_WASM=... node --test tests/transport-parity.mjs
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createCompiler,LANGUAGE_VERSION} from '../js/index.mjs';
const binary=path.resolve(process.env.HTML_TO_RIV_BIN ?? 'tools/html-to-riv/target/debug/html-to-riv');
const wasm=fs.readFileSync(path.resolve(process.env.HTML_TO_RIV_WASM ?? 'tools/html-to-riv/target/wasm32-unknown-unknown/debug/nuxie_html_to_riv.wasm'));
const input={html:'<div id="box"></div>',css:'#box{width:100px;height:40px;background-color:red;}',width:240,height:160};
const document={languageVersion:LANGUAGE_VERSION,...input};

test('CLI and WASM produce exact ordinary Rive bytes and source map',async()=>{
 const compiler=await createCompiler(wasm);const dir=fs.mkdtempSync(path.join(os.tmpdir(),'immutable-parity-'));
 try{
  for(const value of [input,{...input,width:390,html:'<div id="outer"><div id="inner"></div></div>',css:'#outer{width:200px;height:80px;background-color:blue;}#inner{width:30px;height:20px;background-color:red;}'}]){
   fs.writeFileSync(path.join(dir,'input.json'),JSON.stringify(value));
   const cli=spawnSync(binary,[path.join(dir,'input.json'),path.join(dir,'scene.riv')],{encoding:'utf8'});
   assert.equal(cli.status,0,cli.stderr);
   const result=compiler.compile({languageVersion:LANGUAGE_VERSION,...value});assert.equal(result.ok,true,JSON.stringify(result));
   assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(path.join(dir,'scene.riv')));
   assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(path.join(dir,'scene.map.json'))));
   assert.deepEqual(Object.keys(result).sort(),['languageVersion','ok','riv','sourceMap']);
   assert(!fs.existsSync(path.join(dir,'scene.requirements.json')));
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('strict document contract and owned output survive failures and subsequent calls',async()=>{
 const compiler=await createCompiler(new WebAssembly.Module(wasm));const first=compiler.compile(document);assert.equal(first.ok,true);const retained=first.riv.slice();
 for(const bad of [null,[],{...document,languageVersion:'nuxie-html-v1'},{...document,assets:{}},{...document,runtimeRequirements:{}},{...document,width:NaN},{...document,css:undefined}])assert.equal(compiler.compile(bad).ok,false);
 const rejected=compiler.compile({...document,css:'#box{display:grid;}'});assert.equal(rejected.ok,false);
 const next=compiler.compile(document);assert.equal(next.ok,true);assert.deepEqual(first.riv,retained);assert.deepEqual(next.riv,retained);
 first.riv[0]^=255;assert.deepEqual(compiler.compile(document).riv,retained);
});

test('raw ABI version2 rejects old language and unknown assets without stale output',async()=>{
 const {instance}=await WebAssembly.instantiate(wasm,{});const w=instance.exports;assert.equal(w.html_compiler_abi_version(),2);
 const compile=request=>{const bytes=new TextEncoder().encode(JSON.stringify(request));const ptr=w.html_compiler_request_alloc(bytes.length)>>>0;assert(ptr);new Uint8Array(w.memory.buffer,ptr,bytes.length).set(bytes);const status=w.html_compiler_compile();const metadata=JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer,w.html_compiler_metadata_ptr()>>>0,w.html_compiler_metadata_len()>>>0)));return{status,metadata};};
 assert.equal(compile({languageVersion:LANGUAGE_VERSION,input}).status,0);assert(w.html_compiler_riv_len()>0);
 for(const request of [{languageVersion:'nuxie-html-v1',input},{languageVersion:LANGUAGE_VERSION,input:{...input,assets:{}}}]){assert.equal(compile(request).status,1);assert.equal(w.html_compiler_riv_len(),0);}
 assert.equal(w.html_compiler_request_alloc(0),0);assert.equal(w.html_compiler_request_alloc(0xffffffff),0);w.html_compiler_reset();assert.equal(w.html_compiler_metadata_len(),0);assert.equal(w.html_compiler_riv_len(),0);
});

async function assertCorpusParity(fixtures) {
 const compiler=await createCompiler(wasm);
 assert.equal(new Set(fixtures.map(f=>f.name)).size,fixtures.length,'unique fixture identities');
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'immutable-corpus-parity-'));
 try {
  for(const fixture of fixtures) for(const [width,height] of [[240,160],[390,200],[768,120]]) {
   const value={html:fixture.html,css:fixture.css,width,height};
   const prefix=path.join(dir,`${fixture.name}-${width}`);
   fs.writeFileSync(prefix+'.json',JSON.stringify(value));
   const cli=spawnSync(binary,[prefix+'.json',prefix+'.riv'],{encoding:'utf8',maxBuffer:8*1024*1024});
   assert.equal(cli.status,0,`${fixture.name}/${width}: ${cli.stderr}`);
   const result=compiler.compile({languageVersion:LANGUAGE_VERSION,...value});
   assert.equal(result.ok,true,`${fixture.name}/${width}: ${JSON.stringify(result)}`);
   assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(prefix+'.riv'),`${fixture.name}/${width}: complete Rive bytes`);
   assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(prefix+'.map.json')),`${fixture.name}/${width}: complete source map`);
   assert.equal(new Set(result.sourceMap.map(n=>n.object_id)).size,result.sourceMap.length);
   assert(!fs.existsSync(prefix+'.requirements.json'));
  }
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
}

test('every public baseline fixture has exact CLI/WASM parity at three viewports',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-baseline-cases.json',import.meta.url)));
 assert(fixtures.length>=15,'baseline corpus unexpectedly shrank');
 await assertCorpusParity(fixtures);
});

test('rejected styles produce identical native and WASM diagnostics and no output',async()=>{
 const compiler=await createCompiler(wasm);const dir=fs.mkdtempSync(path.join(os.tmpdir(),'immutable-rejection-parity-'));
 try {
  for(const [index,css] of ['#box{display:grid;}','#box{background-image:linear-gradient(red,blue);}','#box{background-color:var(--missing);}','#box{position:fixed;}','#box{transform:rotate(20deg);}','#box{opacity:0.5;}'].entries()) {
   const value={...input,css},prefix=path.join(dir,String(index));fs.writeFileSync(prefix+'.json',JSON.stringify(value));
   const cli=spawnSync(binary,[prefix+'.json',prefix+'.riv'],{encoding:'utf8'});
   const result=compiler.compile({languageVersion:LANGUAGE_VERSION,...value});
   assert.equal(cli.status,1,css);assert.equal(result.ok,false,css);
   assert.deepEqual(result.diagnostics,JSON.parse(cli.stderr),css);
   assert(!fs.existsSync(prefix+'.riv'));assert(!fs.existsSync(prefix+'.map.json'));
  }
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('public color palettes have exact multi-object CLI/WASM parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-color-palettes.json',import.meta.url)));
 assert(fixtures.length>0,'color palette corpus is required');
 assert(fixtures.some(f=>(f.html.match(/id=/g)||[]).length>1),'palette must exercise multiple source objects');
 await assertCorpusParity(fixtures);
});

test('inherited currentColor and literal control have exact CLI/WASM parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-inheritance-cases.json',import.meta.url)));
 assert.equal(fixtures.length,2);
 await assertCorpusParity(fixtures);
});


test('CSS-wide sizing and color corpus has exact CLI/WASM parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-css-wide-cases.json',import.meta.url)));
 assert(fixtures.length>=10,'CSS-wide corpus must retain sizing and color cases');
 await assertCorpusParity(fixtures);
});


test('solid background shorthand has exact CLI/WASM corpus parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-background-cases.json',import.meta.url)));
 assert(fixtures.length>=10,'background corpus must retain cascade, inheritance and literal controls');
 await assertCorpusParity(fixtures);
});

test('font-relative box dimensions have exact CLI/WASM corpus parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-font-relative-cases.json',import.meta.url)));
 assert.equal(fixtures.length,8);
 await assertCorpusParity(fixtures);
});
