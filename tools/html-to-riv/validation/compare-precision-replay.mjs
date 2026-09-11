// Replay existing compiled scenes at default precision. Does not overwrite source
// artifacts or substitute a fresh browser capture. Byte identity is strong
// evidence that an observed failure is independent of the precision option.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const [output,...directories]=process.argv.slice(2);
if(!output||!directories.length)throw new Error('Expected output directory and scene artifact directories');
fs.mkdirSync(output,{recursive:true});
const env={...process.env,NUXIE_NATIVE_GLYPHS:'0'};delete env.NUXIE_CSS_SHAPING_PRECISION;
const results=[];
for(const directory of directories){
 const source=path.join(directory,'scene');
 if(JSON.parse(fs.readFileSync(source+'.requirements.json')).capabilities.includes('text-css-shaping-precision-v1'))throw new Error('Scene requires CSS precision; this legacy replay comparison cannot disable a required capability');
 const manifest=JSON.parse(fs.readFileSync(source+'.review.json'));
 const dest=path.resolve(output,`${manifest.name}-${manifest.width}`);
 execFileSync(path.join(root,'target/debug/examples/probe'),[source+'.riv',source+'.map.json',String(manifest.width),String(manifest.height),dest],{env,stdio:'pipe'});
 execFileSync(path.join(root,'target/debug/renderer-replay'),['--stream',dest+'.stream','--output',dest+'.native.png','--backend','rust-metal','--mode','clockwise-atomic'],{env,stdio:'pipe'});
 results.push({name:manifest.name,width:manifest.width,source:path.resolve(source),baseline:dest,
  nativePngIdentical:fs.readFileSync(source+'.native.png').equals(fs.readFileSync(dest+'.native.png')),
  boundsIdentical:fs.readFileSync(source+'.bounds.json').equals(fs.readFileSync(dest+'.bounds.json'))});
}
fs.writeFileSync(path.join(output,'results.json'),JSON.stringify(results,null,2));console.log(JSON.stringify(results,null,2));
