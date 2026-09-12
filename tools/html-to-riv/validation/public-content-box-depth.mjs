// Compare old, initial and repaired default-build WASM depth behavior.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import crypto from 'node:crypto';
import{createCompiler,LANGUAGE_VERSION}from'../js/index.mjs';
const[out,...files]=process.argv.slice(2).map(p=>path.resolve(p));assert.equal(files.length,3);assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const hash=buffer=>crypto.createHash('sha256').update(buffer).digest('hex');
const bindings=files.map((file,index)=>({file,label:['prior-variable-recovery','initial-content-box','repaired-content-box'][index],sha256:hash(fs.readFileSync(file))}));
const rows=[];
for(const binding of bindings)for(const boxSizing of ['border-box','content-box'])for(const depth of [64,128,130]){
 const input={html:'<div>'.repeat(depth)+'</div>'.repeat(depth),css:`div{${boxSizing==='content-box'?'box-sizing:content-box;':''}width:1px;height:1px;padding:1px}`,width:240,height:160};
 const name=`${binding.label}-${boxSizing}-${depth}`,requestFile=path.join(out,name+'.json');fs.writeFileSync(requestFile,JSON.stringify(input,null,2));
 const result={...binding,boxSizing,depth,requestFile,requestSha256:hash(fs.readFileSync(requestFile))};
 try{const compiler=await createCompiler(fs.readFileSync(binding.file)),r=compiler.compile({languageVersion:LANGUAGE_VERSION,...input});result.js={ok:r.ok,...(r.ok?{nodes:r.sourceMap.length,rivSha256:hash(r.riv)}:{diagnostics:r.diagnostics})};}
 catch(e){result.js={trap:String(e),stack:e.stack};}
 // Observe the first raw trap without allowing a cleanup trap to mask it.
 try{const{instance}=await WebAssembly.instantiate(fs.readFileSync(binding.file),{}),w=instance.exports;
  const request=new TextEncoder().encode(JSON.stringify({languageVersion:LANGUAGE_VERSION,input}));const ptr=w.html_compiler_request_alloc(request.length)>>>0;
  new Uint8Array(w.memory.buffer,ptr,request.length).set(request);const status=w.html_compiler_compile();
  const metadata=JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer,w.html_compiler_metadata_ptr()>>>0,w.html_compiler_metadata_len()>>>0)));
  result.raw={status,metadata,rivLength:w.html_compiler_riv_len()>>>0};w.html_compiler_reset();
 }catch(e){result.raw={trap:String(e),stack:e.stack};}
 rows.push(result);
}
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({bindings,rows},null,2)+'\n');
const final=rows.filter(r=>r.label==='repaired-content-box');
for(const r of final){assert.equal(r.js.ok,r.depth<=128);assert(!r.raw.trap);if(r.depth<=128){assert.equal(r.js.nodes,r.depth);assert.equal(r.raw.status,0);}else{assert.equal(r.js.diagnostics[0].code,'depth-limit');assert.equal(r.raw.status,1);assert.equal(r.raw.rivLength,0);}}
console.log(JSON.stringify(rows.map(r=>({label:r.label,boxSizing:r.boxSizing,depth:r.depth,result:r.js.ok?'compiled':r.js.trap?'trapped':r.js.diagnostics[0].code}))));
