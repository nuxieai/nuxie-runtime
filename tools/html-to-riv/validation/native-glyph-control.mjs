// A single-font, white-background diagnostic. Not a shipping renderer or an
// acceptance reference. Requires text-diagnosis.mjs and native-glyph-probe.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const out=path.join(root,'output/playwright/html-to-riv');
const width=Number(process.argv[2] || 390);
if(!Number.isInteger(width) || width<1 || width>8192) throw new Error('Invalid diagnostic viewport');
const dir=path.join(out,'text-diagnosis'+(width===390?'':'-'+width));
const results=[];
for(const name of ['minimal-1','minimal-2','normal-line-height-cascade','glyph-color-alpha']){
 const request=JSON.parse(fs.readFileSync(path.join(dir,name+'.json')));
 const assets=Object.values(request.assets);
 if(assets.length!==1 || assets[0].kind!=='font' || /background/i.test(request.css)) throw new Error('Only single-font white-background controls are supported');
 const fontFile=path.join(dir,'control-font.ttf');fs.writeFileSync(fontFile,Buffer.from(assets[0].bytes));
 const reference=PNG.sync.read(fs.readFileSync(path.join(dir,name+'.browser.png')));
 const boxes=JSON.parse(fs.readFileSync(path.join(dir,name+'.bounds.json')));
 for(const mode of ['plain','smooth']){
  const output=path.join(dir,`${name}.coretext-${mode}.png`);
  execFileSync(path.join(out,'native-glyph-probe'),[fontFile,path.join(dir,name+'.glyphs.json'),String(reference.width),String(reference.height),mode,output],{encoding:'utf8'});
  const {metrics,failures,diff}=comparePixels(reference,PNG.sync.read(fs.readFileSync(output)),boxes,true);
  fs.writeFileSync(path.join(dir,`${name}.coretext-${mode}.diff.png`),PNG.sync.write(diff));
  results.push({name,mode,metrics,failures});
 }
}
fs.writeFileSync(path.join(dir,'native-glyph-results.json'),JSON.stringify(results,null,2));
console.log(JSON.stringify(results));
