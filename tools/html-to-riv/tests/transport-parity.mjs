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

test('min/max dimension constraints have exact CLI/WASM corpus parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-minmax-cases.json',import.meta.url)));
 assert(fixtures.length>=10);
 await assertCorpusParity(fixtures);
});

test('selector and specificity corpus has exact CLI/WASM parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-selector-cases.json',import.meta.url)));
 assert(fixtures.length>=16);
 await assertCorpusParity(fixtures);
});

test('custom property substitution has exact CLI/WASM corpus parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-variable-cases.json',import.meta.url)));
 assert(fixtures.length>=12);
 await assertCorpusParity(fixtures);
});

test('lazy variable cycles have exact CLI/WASM parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-variable-cycle-cases.json',import.meta.url)));
 assert.equal(fixtures.length,16);
 await assertCorpusParity(fixtures);
});


test('flex directions and automatic sizing have exact CLI/WASM parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-flex-paint-order-cases.json',import.meta.url)));
 assert.equal(fixtures.length,23);
 await assertCorpusParity(fixtures);
});

test("ordinary sibling paint ordering has exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-flex-paint-regression-cases.json",import.meta.url)));
 assert.equal(fixtures.length,52);
 await assertCorpusParity(fixtures);
});

test("intrinsic flex percentage and minimum contexts have exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-flex-intrinsic-cases.json",import.meta.url)));
 assert.equal(fixtures.length,12);
 await assertCorpusParity(fixtures);
});

test("CSS order has exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-order-cases.json",import.meta.url)));
 assert.equal(fixtures.length,19);
 await assertCorpusParity(fixtures);
});

test("align-self ordinary wrapper corpus has exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-align-self-cases.json",import.meta.url)));
 assert.equal(fixtures.length,33);
 await assertCorpusParity(fixtures);
});

test("align-self edge contexts have exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-align-self-edge-cases.json",import.meta.url)));
 assert.equal(fixtures.length,12);
 await assertCorpusParity(fixtures);
});

test("safe and logical align-self have exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-align-safe-cases.json",import.meta.url)));
 assert.equal(fixtures.length,21);
 await assertCorpusParity(fixtures);
});

test("compiler-derived first baseline has exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-first-baseline-cases.json",import.meta.url)));
 assert.equal(fixtures.length,34);
 await assertCorpusParity(fixtures);
});

test("unresolved baseline contexts have CLI/WASM diagnostic parity",async()=>{
 const compiler=await createCompiler(wasm),dir=fs.mkdtempSync(path.join(os.tmpdir(),'baseline-rejection-parity-'));
 const html='<div id="p"><div id="a"></div></div>';
 try{
  for(const css of ['#a{height:20px;align-self:baseline}','#p{flex-direction:row}#a{height:auto;min-height:auto;align-self:baseline}','#p{flex-direction:row;height:100px}#a{height:50%;max-height:60px;align-self:baseline}','#a{align-self:last baseline}','#a{align-self:safe baseline}', '#p{flex-direction:row;height:100px}#a{height:50%;align-self:last baseline}', '#p{flex-direction:row}#a{height:20px;min-height:auto;align-self:last baseline}', '#p{flex-direction:row}#a{height:20px;align-self:safe last baseline}']){
   const request={html,css,width:240,height:160},inputFile=path.join(dir,'input.json'),outputFile=path.join(dir,'scene.riv');
   fs.writeFileSync(inputFile,JSON.stringify(request));
   const cli=spawnSync(binary,[inputFile,outputFile],{encoding:'utf8'}),result=compiler.compile({languageVersion:LANGUAGE_VERSION,...request});
   assert.notEqual(cli.status,0);assert.equal(result.ok,false);assert.deepEqual(result.diagnostics,JSON.parse(cli.stderr));assert(!fs.existsSync(outputFile));
  }
  assert.equal(compiler.compile(document).ok,true);
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test("intrinsic first-baseline metrics have exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-baseline-intrinsic-cases.json",import.meta.url)));
 assert.equal(fixtures.length,16);
 await assertCorpusParity(fixtures);
});


test("last baseline has exact public CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-last-baseline-cases.json",import.meta.url)));
 assert.equal(fixtures.length,62);
 await assertCorpusParity(fixtures);
});


test("nested first/last baseline constraints have exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-last-baseline-nested-cases.json",import.meta.url)));
 assert.equal(fixtures.length,4);
 await assertCorpusParity(fixtures);
});


test("public around/evenly spacing has exact CLI/WASM parity",async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL("../validation/public-spacing-cases.json",import.meta.url)));
 assert.equal(fixtures.length,112);
 await assertCorpusParity(fixtures);
});


test("unresolved spacing contexts have CLI/WASM diagnostic parity",async()=>{
 const compiler=await createCompiler(wasm),dir=fs.mkdtempSync(path.join(os.tmpdir(),'spacing-rejection-parity-'));
 const html='<div id="p"><div id="a"><div id="leaf"></div></div></div>';
 try{
  for(const css of ['#never{justify-content:space-between}', '#never{justify-content:safe space-around}', '#p{flex-direction:row}#a{height:60px;align-self:baseline;justify-content:space-evenly}#leaf{height:10px}', '#p{flex-direction:row}#a{height:60px;align-self:last baseline;justify-content:space-around}#leaf{height:10px}']){
   const request={html,css,width:240,height:160},inputFile=path.join(dir,'input.json'),outputFile=path.join(dir,'scene.riv');
   fs.writeFileSync(inputFile,JSON.stringify(request));
   const cli=spawnSync(binary,[inputFile,outputFile],{encoding:'utf8'}),result=compiler.compile({languageVersion:LANGUAGE_VERSION,...request});
   assert.notEqual(cli.status,0);assert.equal(result.ok,false);assert.deepEqual(result.diagnostics,JSON.parse(cli.stderr));assert(!fs.existsSync(outputFile));
  }
  assert.equal(compiler.compile(document).ok,true);
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('automatic margins preserve CLI/WASM bytes and authored source maps',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-auto-margin-expanded-cases.json',import.meta.url)));
 assert.equal(fixtures.length,64);
 await assertCorpusParity(fixtures);
});

test('unresolved automatic-margin combinations have CLI/WASM diagnostic parity',async()=>{
 const compiler=await createCompiler(wasm);const dir=fs.mkdtempSync(path.join(os.tmpdir(),'immutable-margin-diagnostics-'));
 try {
  const html='<div id="p"><div id="b"></div></div>';
  for(const [index,css] of ['#b{margin:10px}','#p{flex-direction:row;justify-content:space-around}#b{margin-left:auto}','#p{flex-direction:row}#b{align-self:baseline;margin-left:auto}'].entries()) {
   const value={html,css,width:240,height:160},prefix=path.join(dir,String(index));
   fs.writeFileSync(prefix+'.json',JSON.stringify(value));
   const cli=spawnSync(binary,[prefix+'.json',prefix+'.riv'],{encoding:'utf8'});
   const result=compiler.compile({languageVersion:LANGUAGE_VERSION,...value});
   assert.equal(cli.status,1,css);assert.equal(result.ok,false,css);
   assert.deepEqual(result.diagnostics,JSON.parse(cli.stderr),css);
   assert(!fs.existsSync(prefix+'.riv'));assert(!fs.existsSync(prefix+'.map.json'));
  }
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('automatic-margin intrinsic and composition boundaries have CLI/WASM parity',async()=>{
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/public-auto-margin-boundary-cases.json',import.meta.url)));
 assert.equal(fixtures.length,22);
 await assertCorpusParity(fixtures);
});

test('legacy flex declarations preserve public CLI/WASM output',async()=>{
 const html='<div id="p"><div id="a"></div></div>';
 await assertCorpusParity(['div{flex:none}','div{flex:0 0 auto}','div{flex-grow:0;flex-shrink:0;flex-basis:auto}','#p{--f:none}#a{flex:var(--f)}'].map((css,i)=>({name:`legacy-flex-${i}`,html,css:`#p{width:160px;height:120px;flex-direction:row}${css}`})));
});

test('unqualified flex candidates have identical CLI/WASM diagnostics',async()=>{
 const compiler=await createCompiler(wasm),dir=fs.mkdtempSync(path.join(os.tmpdir(),'immutable-flex-diagnostics-'));
 try{
  for(const [index,css] of [...['1 7 0px','1 1 30px','1 1 auto','initial','1','-1 1 0px'].map(value=>`#a{flex:${value}}`),'#a{flex:1 1 30px;flex:none}','#never{flex:1}','#a{--f:1 1 30px;flex:var(--f);flex:none}'].entries()){
   const request={html:'<div id="p"><div id="a"></div></div>',css:`#p{width:160px;height:120px;flex-direction:row}${css}`,width:240,height:160};
   const prefix=path.join(dir,String(index));fs.writeFileSync(prefix+'.json',JSON.stringify(request));
   const cli=spawnSync(binary,[prefix+'.json',prefix+'.riv'],{encoding:'utf8'}),result=compiler.compile({languageVersion:LANGUAGE_VERSION,...request});
   assert.equal(cli.status,1);assert.equal(result.ok,false);assert.deepEqual(result.diagnostics,JSON.parse(cli.stderr));assert(!fs.existsSync(prefix+'.riv'));assert(!fs.existsSync(prefix+'.map.json'));
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('percentage overflow diagnostics and finite clamps agree across CLI/WASM',async()=>{
 const compiler=await createCompiler(wasm),dir=fs.mkdtempSync(path.join(os.tmpdir(),'immutable-numeric-boundaries-'));
 try{
  for(const axis of ['width','height'])for(const [index,[depth,extra,accepted]] of [[8,'',true],[9,'',false],[9,'div{min-width:auto}',false],[9,'div{min-width:auto}#n8{max-width:100px}',true],[9,'div{min-width:1000000%;max-width:100px}',false],[9,'#n0{width:0px}',true]].entries()){
   let css='div{width:1000000%;height:1px}'+extra;
   if(axis==='height')css=css.replaceAll('width','AXIS').replaceAll('height','width').replaceAll('AXIS','height');
   const request={html:Array.from({length:depth},(_,i)=>`<div id="n${i}">`).join('')+'</div>'.repeat(depth),css,width:1,height:1};
   const prefix=path.join(dir,`${axis}-${index}`);fs.writeFileSync(prefix+'.json',JSON.stringify(request));
   const cli=spawnSync(binary,[prefix+'.json',prefix+'.riv'],{encoding:'utf8'}),result=compiler.compile({languageVersion:LANGUAGE_VERSION,...request});
   assert.equal(result.ok,accepted);assert.equal(cli.status,accepted?0:1);
   if(accepted){assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(prefix+'.riv'));assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(prefix+'.map.json','utf8')));}
   else{assert.deepEqual(result.diagnostics,JSON.parse(cli.stderr));assert(!fs.existsSync(prefix+'.riv'));assert(!fs.existsSync(prefix+'.map.json'));}
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});
