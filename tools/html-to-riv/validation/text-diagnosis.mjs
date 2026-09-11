// Diagnostic experiments, deliberately separate from the acceptance corpus.
import { chromium } from '@playwright/test';
import { PNG } from 'pngjs';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { comparePixels } from './pixels.mjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const root=path.resolve(moduleDir,'../..');
const width=Number(process.argv[2] || 390);
if(!Number.isInteger(width) || width<1 || width>8192) throw new Error('Invalid diagnostic viewport');
const out=path.join(root,'output/playwright/html-to-riv/text-diagnosis'+(width===390?'':'-'+width));
fs.mkdirSync(out,{recursive:true});
const font=fs.readFileSync(path.join(moduleDir,'tests/assets/Inter-Regular.ttf'));
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const original=JSON.parse(fs.readFileSync(path.join(moduleDir,'validation/cases.json'))).find(c=>c.name==='font-shorthand-resets');
const variants=[{...original,name:'original'},
 {...original,name:'longhands',css:original.css.replace('font:normal 16px/1.5 "Inter"','font-size:16px;line-height:1.5;font-weight:400;font-family:Inter')},
 ...['Larger text retains the multiplier.','H','Hg'].map((text,i)=>({name:`minimal-${i}`,html:`<p id="text">${text}</p>`,css:`#text {font-size:20px;line-height:30px;${i===1?'width:20px':i===2?'width:30px':''}}`}))];
variants.push({name:'tight-line',html:'<p id="text">H</p>',css:'#text {font-size:22px;line-height:27px;width:22px}'});
variants.push({name:'fractional',html:'<p id="text">Larger text retains the multiplier.</p>',css:'#text {font-size:20.5px;line-height:30.75px}'});
variants.push({name:'antialiased-control',html:'<p id="text">Hg</p>',css:'#text {font-size:20px;line-height:30px;width:30px}',browserCss:'* {-webkit-font-smoothing:antialiased}'});
for (const fixture of JSON.parse(fs.readFileSync(path.join(moduleDir,'validation/cases.json'))).filter(c=>['font-shorthand-normal-resets','normal-line-height-cascade'].includes(c.name))) {
 variants.push(fixture, {...fixture,name:fixture.name+'-antialiased-control',browserCss:'* {-webkit-font-smoothing:antialiased}'});
}
variants.push({name:'glyph-color-alpha',html:'<div id="root"><p id="colored">Hg color sample</p><p id="alpha">Hg alpha sample</p></div>',css:'#root {padding:0.25px;gap:8px} #colored,#alpha {font-size:20px;line-height:30px} #colored {color:#369} #alpha {color:rgb(0 0 0 / .5)}'});
const browser=await chromium.launch();
try {
 const page=await browser.newPage({viewport:{width,height:320},deviceScaleFactor:1});
 const results=[];
 for(const fixture of variants) {
  const prefix=path.join(out,fixture.name);
  const request={html:fixture.html,css:fixture.css,width:390,height:320,assets:{inter:{kind:'font',family:'Inter',weight:400,bytes:[...font]}}};
  fs.writeFileSync(prefix+'.json',JSON.stringify(request));
  for(const [bin,args] of [['html-to-riv',[prefix+'.json',prefix+'.riv']],['examples/probe',[prefix+'.riv',prefix+'.map.json',String(width),'320',prefix]],['renderer-replay',['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']]]) execFileSync(path.join(root,'target/debug',bin),args,{stdio:'pipe',encoding:'utf8'});
  await page.setContent(`<!doctype html><style>@font-face {font-family:Inter;src:url(data:font/ttf;base64,${font.toString('base64')})} ${reset} ${fixture.css} ${fixture.browserCss || ""}</style>${fixture.html}`);
  await page.evaluate(()=>document.fonts.ready);
  await page.screenshot({path:prefix+'.browser.png'});
  const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{const r=e.getBoundingClientRect();return [e.id,{x:r.x,y:r.y,width:r.width,height:r.height}]})));
  const native=PNG.sync.read(fs.readFileSync(prefix+'.native.png')), reference=PNG.sync.read(fs.readFileSync(prefix+'.browser.png'));
  const {metrics,failures,diff}=comparePixels(reference,native,boxes,true);
  fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
  const typography=await page.evaluate(()=>[...document.querySelectorAll('p')].map(e=>{const s=getComputedStyle(e),range=document.createRange();range.selectNodeContents(e);const r=range.getBoundingClientRect();const canvas=document.createElement('canvas'),c=canvas.getContext('2d');c.font=`${s.fontSize} ${s.fontFamily}`;const m=c.measureText(e.textContent);return {font:s.font,lineHeight:s.lineHeight,range:{y:r.y,height:r.height},ascent:m.fontBoundingBoxAscent,descent:m.fontBoundingBoxDescent}}));
  results.push({name:fixture.name,metrics,failures,typography});
 }
 fs.writeFileSync(path.join(out,'results.json'),JSON.stringify(results,null,2));
 console.log(JSON.stringify(results));
} finally {await browser.close()}
