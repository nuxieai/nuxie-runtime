// Actual Chrome geometry and native replay; extreme hard-stop divergence is explicit.
// Usage: node SCRIPT RECORDING FROZEN_RENDERER NEW_OUTPUT
import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import{spawnSync}from'node:child_process';
import{chromium}from'@playwright/test';import{PNG}from'pngjs';import{comparePixels}from'./pixels.mjs';
const [recording,renderer,out]=process.argv.slice(2).map(p=>path.resolve(p));assert(!fs.existsSync(out));fs.mkdirSync(out);
const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');const read=p=>JSON.parse(fs.readFileSync(p));
const lifecycle=read(path.join(recording,'lifecycle.json'));assert.equal(lifecycle.cases.length,4);
const reset=fs.readFileSync(new URL('../src/reset.css',import.meta.url),'utf8');fs.writeFileSync(path.join(out,'reset.css'),reset);
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');const page=await browser.newPage({deviceScaleFactor:1,colorScheme:'light',locale:'en-US',reducedMotion:'reduce'});
const rows=[];const rendererHash=sha(renderer);
try{for(const fixture of lifecycle.cases){
 const request=read(path.join(recording,fixture.name+'.request.json'));assert.equal(request.html,fixture.html);assert.equal(request.css,fixture.css);
 for(const view of fixture.views){
  const prefix=path.join(out,`${fixture.name}-${view.frame}`);await page.setViewportSize({width:view.width,height:view.height});await page.setContent(`<!doctype html><style>${reset}\n${request.css}</style>${request.html}`);
  const measured=await page.evaluate(()=>({computed:getComputedStyle(document.querySelector('#gradient')).backgroundImage,boxes:Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{const{x,y,width,height}=e.getBoundingClientRect();return[e.id,{x,y,width,height}]}))}));
  for(const[id,box]of Object.entries(measured.boxes))for(const axis of ['x','y','width','height'])assert(Math.abs(box[axis]-view.bounds[id][axis])<0.1);
  await page.screenshot({path:prefix+'.browser.png',animations:'disabled'});
  const result=spawnSync(renderer,['--stream',view.stream,'--frame',String(view.frame),'--backend','rust-metal-atomic','--mode','atomics','--output',prefix+'.native.png'],{encoding:'utf8'});assert.equal(result.status,0,result.stderr);fs.writeFileSync(prefix+'.log',result.stdout+result.stderr);
  const b=PNG.sync.read(fs.readFileSync(prefix+'.browser.png')),n=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));const{metrics,failures,diff}=comparePixels(b,n,measured.boxes,false);fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
  const sample=(im,x,y)=>[...im.data.subarray((y*im.width+x)*4,(y*im.width+x)*4+4)];
  const hard=fixture.name==='hard-interior';if(!hard)assert.equal(failures.length,0,JSON.stringify({name:fixture.name,failures}));
  if(hard){assert.deepEqual(sample(n,Math.floor(view.width/4),70),[255,0,0,255]);assert.deepEqual(sample(n,Math.floor(view.width*3/4),70),[0,0,255,255]);assert.deepEqual(sample(b,Math.floor(view.width/4),70),[0,0,255,255]);}
  rows.push({name:fixture.name,frame:view.frame,instance:view.instance,width:view.width,height:view.height,prefix,requestSha256:sha(path.join(recording,fixture.name+'.request.json')),streamSha256:sha(view.stream),browserSha256:sha(prefix+'.browser.png'),nativeSha256:sha(prefix+'.native.png'),measured,metrics,failures,expectedDivergence:hard});
 }
 const subset=rows.filter(r=>r.name===fixture.name&&r.frame<3);const sheet=new PNG({width:1536,height:960});sheet.data.fill(255);
 for(const[row,r]of subset.entries())for(const[col,kind]of ['browser','native'].entries()){const im=PNG.sync.read(fs.readFileSync(r.prefix+'.'+kind+'.png'));PNG.bitblt(im,sheet,0,0,im.width,im.height,col*768,row*320);}
 fs.writeFileSync(path.join(out,fixture.name+'.review.png'),PNG.sync.write(sheet));
}}finally{await browser.close();}
for(const r of rows){const first=rows.find(p=>p.name===r.name&&p.width===r.width);assert.equal(r.nativeSha256,first.nativeSha256);assert.equal(r.browserSha256,first.browserSha256);}
assert.equal(sha(renderer),rendererHash);assert.equal(rows.length,32);
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({status:'passed-with-explicit-Chrome-divergence',scope:'32 actual native frames and independently captured Chrome geometry/pixels; hard-interior preserves mathematical split instead of Chrome extreme precision collapse; visual review separate',browser:'153.0.8010.12',backend:'rust-metal-atomic',mode:'atomics',renderer,rendererSha256:rendererHash,lifecycleSha256:sha(path.join(recording,'lifecycle.json')),scriptSha256:sha(new URL(import.meta.url)),rows},null,2)+'\n');console.log('32 native frames;24 Chrome pixel passes;8 expected hard-stop divergences;32 geometry passes');
