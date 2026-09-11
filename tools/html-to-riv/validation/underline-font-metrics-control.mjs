// Actual Chrome/native comparison for unusual post-table underline metrics.
import assert from 'node:assert/strict';
import {createCompiler} from '../js/index.mjs';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {comparePixels} from './pixels.mjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(moduleDir,'../..');
const dir=process.env.NUXIE_HTML_REVIEW_DIR??path.join(root,'output/playwright/html-to-riv/underline-font-metrics');
fs.mkdirSync(dir,{recursive:true});
const original=fs.readFileSync(path.join(moduleDir,'tests/assets/NuxieJapaneseFixture-Regular.otf'));
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
function variant(thickness){
 const bytes=Buffer.from(original),tables={};
 for(let i=0;i<bytes.readUInt16BE(4);i++){
  const p=12+i*16;tables[bytes.toString('ascii',p,p+4)]={p,offset:bytes.readUInt32BE(p+8),length:bytes.readUInt32BE(p+12)};
 }
 const checksum=(start,length)=>{let sum=0;for(let i=0;i<length;i+=4){let word=0;for(let j=0;j<4;j++)word=(word*256+(i+j<length?bytes[start+i+j]:0))>>>0;sum=(sum+word)>>>0;}return sum;};
 if(thickness==='absent'){
  const count=bytes.readUInt16BE(4)-1;
  bytes.copyWithin(tables.post.p,tables.post.p+16,12+(count+1)*16);
  bytes.fill(0,12+count*16,12+(count+1)*16);
  bytes.writeUInt16BE(count,4);
  const exponent=Math.floor(Math.log2(count)),searchRange=16*2**exponent;
  bytes.writeUInt16BE(searchRange,6);bytes.writeUInt16BE(exponent,8);bytes.writeUInt16BE(count*16-searchRange,10);
  if(tables.head.p>tables.post.p)tables.head.p-=16;
 }else bytes.writeInt16BE(thickness,tables.post.offset+10);
 bytes.writeUInt32BE(0,tables.head.offset+8);
 for(const tag of (thickness==='absent'?['head']:['post','head'])){const t=tables[tag];bytes.writeUInt32BE(checksum(t.offset,t.length),t.p+4);}
 bytes.writeUInt32BE((0xb1b0afba-checksum(0,bytes.length))>>>0,tables.head.offset+8);
 return bytes;
}
const run=(bin,args)=>execFileSync(path.join(root,'target/debug',bin),args,{env:{...process.env,NUXIE_NATIVE_GLYPHS:'1'},stdio:'pipe'});
const compiler=await createCompiler(fs.readFileSync(path.join(moduleDir,'dist/html-to-riv.wasm')));
const browser=await chromium.launch(),results=[];
try{
 const page=await browser.newPage({viewport:{width:240,height:80},deviceScaleFactor:1});
 const fontErrors=[];page.on('console',message=>{const text=message.text();if(text.startsWith('OTS parsing error:'))fontErrors.push(text);});
 for(const thickness of (process.argv.includes('--absent-only')?['absent']:[-200,-20,0,1,50]))for(const mode of ['auto','from-font']){
  const font=variant(thickness),prefix=path.join(dir,`${thickness}-${mode}`);
  const request={html:'<p id="text">agypqj test</p>',css:`p{font-family:Fixture;font-size:24px;line-height:40px;color:transparent;text-decoration:underline red ${mode};text-decoration-skip-ink:none}`,width:390,height:80,assets:{font:{kind:'font',family:'Fixture',weight:400,bytes:[...font]}}};
  fs.writeFileSync(prefix+'.json',JSON.stringify(request));run('html-to-riv',[prefix+'.json',prefix+'.riv']);
  const wasm=compiler.compile({languageVersion:'nuxie-html-v1',...request});assert.equal(wasm.ok,true);
  assert.deepEqual(Buffer.from(wasm.riv),fs.readFileSync(prefix+'.riv'));
  assert.deepEqual(wasm.sourceMap,JSON.parse(fs.readFileSync(prefix+'.map.json')));
  assert.deepEqual(wasm.runtimeRequirements,JSON.parse(fs.readFileSync(prefix+'.requirements.json')));
  for(const width of [240,390,768]){
  const frame=prefix+'-'+width;
  run('examples/probe',[prefix+'.riv',prefix+'.map.json',String(width),'80',frame]);
  await page.setViewportSize({width,height:80});
  run('renderer-replay',['--stream',frame+'.stream','--output',frame+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']);
  await page.setContent(`<style>@font-face{font-family:Fixture;src:url(data:font/otf;base64,${font.toString('base64')})}${reset}${request.css}</style>${request.html}`);
  await page.evaluate(()=>document.fonts.ready);
  const loaded=await page.evaluate(()=>[...document.fonts].every(f=>f.status==='loaded'));
  if(!loaded){
   fs.writeFileSync(path.join(dir,'font-rejection.json'),JSON.stringify({browser:browser.version(),thickness,mode,width,reason:'Chrome rejected font variant',fontErrors},null,2));
   throw new Error('Chrome rejected font variant');
  }
  await page.screenshot({path:frame+'.browser.png'});
  const boxes=await page.evaluate(()=>{const r=document.querySelector('p').getBoundingClientRect();return {text:{x:r.x,y:r.y,width:r.width,height:r.height}};});
  const comparison=comparePixels(PNG.sync.read(fs.readFileSync(frame+'.browser.png')),PNG.sync.read(fs.readFileSync(frame+'.native.png')),boxes,true);
  fs.writeFileSync(frame+'.diff.png',PNG.sync.write(comparison.diff));
  const native=JSON.parse(fs.readFileSync(frame+'.bounds.json'));
  for(const key of ['x','y','width','height'])if(Math.abs(native.text[key]-boxes.text[key])>.1)comparison.failures.push('geometry: '+key);
  results.push({width,thickness,mode,loaded,requirements:JSON.parse(fs.readFileSync(prefix+'.requirements.json')),metrics:comparison.metrics,failures:comparison.failures});
 }
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify({browser:browser.version(),cases:results},null,2));
 await page.setViewportSize({width:2400,height:1000});
 for(let start=0;start<results.length;start+=6){
  const html='<style>body{font:14px system-ui;background:#ddd}section{padding:8px;margin:8px;background:white;width:max-content}article{display:flex;gap:12px}img{display:block}</style>'+results.slice(start,start+6).map(r=>{
   const prefix=path.join(dir,`${r.thickness}-${r.mode}-${r.width}`);
   return `<section><b>post=${r.thickness}, ${r.mode}, ${r.width}px</b><article>`+['browser','native','diff'].map(k=>`<div>${k}<img src="data:image/png;base64,${fs.readFileSync(prefix+'.'+k+'.png').toString('base64')}"></div>`).join('')+'</article></section>';
  }).join('');
  fs.writeFileSync(path.join(dir,`review-${start/6}.html`),html);
  await page.setContent(html);await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  await page.screenshot({path:path.join(dir,`review-${start/6}.png`),fullPage:true});
 }
 console.log(JSON.stringify(results.map(r=>({width:r.width,thickness:r.thickness,mode:r.mode,resolved:r.requirements.text_underlines[0].lines[0].thickness,failures:r.failures})),null,2));
 if(results.some(r=>r.failures.length))process.exitCode=1;
}finally{await browser.close();}
