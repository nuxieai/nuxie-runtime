// Public native/WASM parity for the standard Inter/quadrant fixture format.
// Usage: node parity-fixtures.mjs FIXTURES TOOLCHAIN NEW_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createCompiler} from '../js/index.mjs';

const [fixturesArg,toolchainArg,outArg]=process.argv.slice(2);
assert(outArg,'Expected FIXTURES TOOLCHAIN NEW_OUTPUT');
const toolchain=path.resolve(toolchainArg),out=path.resolve(outArg);
assert(!fs.existsSync(out),'Refusing to overwrite parity evidence');
const sha=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const manifest=JSON.parse(fs.readFileSync(path.join(toolchain,'manifest.json')));
for(const name of ['html-to-riv','html-to-riv.wasm']) {
  assert.equal(sha(fs.readFileSync(path.join(toolchain,name))),manifest.files[name].sha256);
}
const fixtures=fs.readFileSync(fixturesArg),cases=JSON.parse(fixtures);
assert.equal(new Set(cases.map(c=>c.name)).size,cases.length,'Duplicate fixture names');
const compiler=await createCompiler(fs.readFileSync(path.join(toolchain,'html-to-riv.wasm')));
fs.mkdirSync(out,{recursive:true});
const rows=[];
for(const fixture of cases) {
  assert.equal(path.basename(fixture.name),fixture.name);
  assert(!fixture.fontAsset && !fixture.fontFamily && !fixture.imageAsset,'Fixture needs richer asset handling');
  const request={html:fixture.html,css:fixture.css,width:390,height:320};
  if(fixture.font)request.assets={inter:{kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('../tests/assets/Inter-Regular.ttf',import.meta.url))]}};
  if(fixture.image)request.assets={...request.assets,photo:{kind:'image',bytes:[...fs.readFileSync(new URL('../tests/assets/quadrants.png',import.meta.url))]}};
  const prefix=path.join(out,fixture.name);
  fs.writeFileSync(prefix+'.json',JSON.stringify(request));
  const wasm=compiler.compile({languageVersion:'nuxie-html-v1',...request});
  fs.writeFileSync(prefix+'.wasm-result.json',JSON.stringify(wasm.ok ? {ok:true,sourceMap:wasm.sourceMap,runtimeRequirements:wasm.runtimeRequirements} : wasm,null,2));
  assert.equal(wasm.ok,true,wasm.ok ? fixture.name : JSON.stringify(wasm));
  execFileSync(path.join(toolchain,'html-to-riv'),[prefix+'.json',prefix+'.riv'],{stdio:'pipe'});
  assert.deepEqual(Buffer.from(wasm.riv),fs.readFileSync(prefix+'.riv'),fixture.name+' Rive bytes');
  assert.deepEqual(wasm.sourceMap,JSON.parse(fs.readFileSync(prefix+'.map.json')),fixture.name+' source map');
  assert.deepEqual(wasm.runtimeRequirements,JSON.parse(fs.readFileSync(prefix+'.requirements.json')),fixture.name+' requirements');
  rows.push({name:fixture.name,status:'passed',inputSha256:sha(fs.readFileSync(prefix+'.json')),rivSha256:sha(wasm.riv)});
}
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({status:'passed',cases:rows,fixturesSha256:sha(fixtures),compilerSha256:manifest.files['html-to-riv'].sha256,wasmSha256:manifest.files['html-to-riv.wasm'].sha256},null,2)+'\n');
console.log(`${rows.length} public native/WASM parity cases passed`);
