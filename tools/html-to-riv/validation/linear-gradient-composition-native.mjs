// Public compiler static composition qualification. Reuses existing geometry/pixel gates.
// Usage: node SCRIPT ORACLE_JSON PUBLIC_TOOLCHAIN PROBE NEW_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const [oracleArg, toolchainArg, probeArg, outputArg] = process.argv.slice(2);
assert(outputArg, 'Expected ORACLE_JSON PUBLIC_TOOLCHAIN PROBE NEW_OUTPUT');
const oraclePath=path.resolve(oracleArg), source=path.resolve(toolchainArg), output=path.resolve(outputArg);
const frozen=output+'-toolchain';
assert(!fs.existsSync(output) && !fs.existsSync(frozen), 'Refusing to overwrite evidence');
const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const oracle=JSON.parse(fs.readFileSync(oraclePath));
assert.equal(oracle.browser,'153.0.8010.12');
assert.equal(oracle.cases.length,18);
for(const fixture of oracle.cases) {
 assert.equal(fixture.expectedCompile.status,'accepted');
 assert.deepEqual(fixture.viewports.map(view=>view.width),[240,390,768]);
}
const original=JSON.parse(fs.readFileSync(path.join(source,'manifest.json')));
fs.mkdirSync(frozen,{recursive:true});
const manifest={scope:'Frozen public compiler, native recording adapter and renderer for static composition comparisons; no clone/lifecycle qualification',files:{}};
for(const name of ['html-to-riv','html-to-riv.wasm','renderer-replay','probe']) {
 const src=name==='probe'?path.resolve(probeArg):path.join(source,name);
 if(name!=='probe')assert.equal(sha(src),original.files[name].sha256,name);
 const dst=path.join(frozen,name);fs.copyFileSync(src,dst);
 manifest.files[name]={source:src,sha256:sha(dst)};
}
fs.writeFileSync(path.join(frozen,'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
const run=spawnSync(process.execPath,[fileURLToPath(new URL('./replay-oracle.mjs',import.meta.url)),oraclePath,frozen,output],{
 encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0'},maxBuffer:16*1024*1024});
assert.ifError(run.error);
fs.writeFileSync(path.join(output,'run.log'),run.stdout+run.stderr);
fs.writeFileSync(path.join(output,'execution.json'),JSON.stringify({exitCode:run.status,signal:run.signal,oracle:oraclePath,oracleSha256:sha(oraclePath),toolchain:frozen,driverSha256:sha(fileURLToPath(import.meta.url)),scope:manifest.scope},null,2)+'\n');
process.stdout.write(run.stdout);process.stderr.write(run.stderr);
process.exitCode=run.status??1;
