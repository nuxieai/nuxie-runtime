// Copy validated build artifacts before a long rendering run. Never use links:
// Cargo may overwrite artifacts while later features are being developed.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(moduleDir,'../..');
if(!process.argv[2]) throw new Error('Usage: node snapshot-toolchain.mjs NEW_OUTPUT_DIRECTORY');
const out=path.resolve(process.argv[2]);
if(fs.existsSync(out)) throw new Error(`Refusing to overwrite an existing toolchain: ${out}`);
const target=path.resolve(root,process.env.CARGO_TARGET_DIR||'target');
const inputs={
 'html-to-riv':path.join(target,'debug/html-to-riv'),
 'probe':path.join(target,'debug/examples/probe'),
 'renderer-replay':path.join(target,'debug/renderer-replay'),
 'html-to-riv.wasm':path.join(moduleDir,'dist/html-to-riv.wasm'),
};
for(const input of Object.values(inputs)) fs.accessSync(input,fs.constants.R_OK);
fs.mkdirSync(out,{recursive:true});
const files={};
for(const [name,input] of Object.entries(inputs)) {
 const destination=path.join(out,name);
 fs.copyFileSync(input,destination,fs.constants.COPYFILE_EXCL);
 fs.chmodSync(destination,name.endsWith('.wasm')?0o444:0o555);
 files[name]={source:input,sha256:crypto.createHash('sha256').update(fs.readFileSync(destination)).digest('hex')};
}
fs.writeFileSync(path.join(out,'manifest.json'),JSON.stringify({files},null,2)+'\n',{flag:'wx',mode:0o444});
console.log(JSON.stringify({toolchain:out,environment:'NUXIE_HTML_TOOLCHAIN',files:Object.keys(files)}));
