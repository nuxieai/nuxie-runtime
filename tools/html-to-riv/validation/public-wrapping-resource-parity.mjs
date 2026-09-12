import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';
import {createCompiler,LANGUAGE_VERSION} from '../js/index.mjs';
const root=path.resolve(process.argv[2]),build=path.resolve('tools/html-to-riv/output/public-wrapping-product-build-r1/frozen');
const wasm=fs.readFileSync(path.join(build,'compiler.wasm')),compiler=await createCompiler(wasm);
const receipt=JSON.parse(fs.readFileSync(path.join(root,'receipt.json'))),rows=[];
for(const c of receipt.cases){
 const input=JSON.parse(fs.readFileSync(c.request.path)),result=compiler.compile({languageVersion:LANGUAGE_VERSION,...input});
 assert.equal(result.ok,c.accepted,c.name);
 if(result.ok){
  assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(c.runs[0].artifacts['scene.riv'].path),c.name);
  assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(c.runs[0].artifacts['scene.map.json'].path)),c.name);
 }else assert.deepEqual(result.diagnostics,JSON.parse(c.diagnostic),c.name);
 rows.push({name:c.name,accepted:result.ok,exact:true});
}
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
fs.writeFileSync(path.join(root,'wasm-parity.json'),JSON.stringify({wasmSha256:hash(path.join(build,'compiler.wasm')),receiptSha256:hash(path.join(root,'receipt.json')),scriptSha256:hash(new URL(import.meta.url)),rows},null,2));
console.log(JSON.stringify({cases:rows.length,passed:rows.length}));
