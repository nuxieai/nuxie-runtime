// Tests transparent native font images through actual Metal image composition.
// Default Swift mode uses whole-frame diagnostic masks. The optional `rust`
// mode exercises bounded masks from the opt-in native Rust rasterizer.
// Neither mode is compiler output or the integrated runtime glyph adapter.
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {comparePixels} from './pixels.mjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(moduleDir,'../..');
const output=path.join(root,'output/playwright/html-to-riv');
const width=Number(process.argv[2]||390);
const rust=process.argv[3]==='rust';
if(!Number.isInteger(width)||width<1||width>8192) throw new Error('Invalid viewport');
const source=path.join(output,'text-diagnosis'+(width===390?'':'-'+width));
const dir=path.join(output,(rust?'rust-glyph-mask-':'glyph-mask-')+width);fs.mkdirSync(dir,{recursive:true});
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const backgrounds=['ffffff','111827','eeeeff','369966'];
const names=['minimal-1','minimal-2','normal-line-height-cascade','glyph-color-alpha'];
const run=(bin,args)=>execFileSync(bin,args,{encoding:'utf8',stdio:'pipe'});
const browser=await chromium.launch();
try {
 const page=await browser.newPage({viewport:{width,height:320},deviceScaleFactor:1});
 const results=[];
 for(const name of names){
  const request=JSON.parse(fs.readFileSync(path.join(source,name+'.json')));
  const assets=Object.values(request.assets);
  if(assets.length!==1||assets[0].kind!=='font'||/background/i.test(request.css)) throw new Error('Only single-font scenes with no authored background are supported');
  const font=Buffer.from(assets[0].bytes),fontFile=path.join(dir,'font.ttf');fs.writeFileSync(fontFile,font);
  const mask=path.join(dir,name+'.mask.png');
  if(!rust) {
  run(path.join(output,'native-glyph-probe'),[fontFile,path.join(source,name+'.glyphs.json'),String(width),'320','mask-smooth',mask]);
  const decoded=PNG.sync.read(fs.readFileSync(mask));
  if(decoded.width!==width||decoded.height!==320||decoded.data[3]!==0) throw new Error('Expected a transparent full-viewport mask');
  }
  const boxes=JSON.parse(fs.readFileSync(path.join(source,name+'.bounds.json')));
  for(const background of backgrounds){
   const prefix=path.join(dir,name+'-'+background);
   if(rust) run(path.join(root,'target/debug/examples/native_glyph_composite'),[fontFile,path.join(source,name+'.glyphs.json'),String(width),'320','ff'+background,prefix+'.stream',prefix+'.masks.json']);
   else run(path.join(root,'target/debug/examples/glyph_mask_composite'),[mask,String(width),'320','ff'+background,prefix+'.stream']);
   run(path.join(root,'target/debug/renderer-replay'),['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']);
   await page.setContent(`<!doctype html><style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${font.toString('base64')})}${reset}body{background:#${background}}${request.css}</style>${request.html}`);
   await page.evaluate(()=>document.fonts.ready);await page.screenshot({path:prefix+'.browser.png'});
   const {metrics,failures,diff}=comparePixels(PNG.sync.read(fs.readFileSync(prefix+'.browser.png')),PNG.sync.read(fs.readFileSync(prefix+'.native.png')),boxes,true);
   fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
   results.push({name,width,background,metrics,failures});
  }
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify(results,null,2));
 // A durable, reproducible visual review accompanies the numerical result.
 // Embed the exact outputs from this run so a later rerun cannot mix receipts.
 const selected=['minimal-2-ffffff','normal-line-height-cascade-eeeeff','glyph-color-alpha-111827','glyph-color-alpha-369966'];
 const title=`${rust?'Bounded Rust/CoreText':'Swift full-frame'} glyph masks — ${width}px`;
 const review=`<!doctype html><meta charset="utf-8"><title>${title}</title><style>body{font:14px system-ui;background:#ddd}section{background:white;padding:8px;margin-bottom:16px;width:max-content}article{display:flex;gap:8px}img{max-width:500px}</style><h2>${title}</h2>`+
  selected.map(name=>`<section><h3>${name}</h3><article>`+
   [['browser','Chromium'],['native','Native mask → Metal'],['diff','Difference']].map(([suffix,label])=>
    `<div>${label}<br><img src="data:image/png;base64,${fs.readFileSync(path.join(dir,`${name}.${suffix}.png`)).toString('base64')}"></div>`).join('')+
   '</article></section>').join('');
 fs.writeFileSync(path.join(dir,'review.html'),review);
 await page.setViewportSize({width:1700,height:1000});
 await page.setContent(review);
 await page.evaluate(()=>Promise.all([...document.images].map(image=>image.decode())));
 await page.screenshot({path:path.join(dir,'review.png'),fullPage:true});
 const failed=results.filter(r=>r.failures.length);
 console.log(JSON.stringify({width,passed:results.length-failed.length,total:results.length,failures:failed.map(r=>({name:r.name,background:r.background,failures:r.failures}))}));
 if(failed.length) process.exitCode=1;
}finally{await browser.close()}
