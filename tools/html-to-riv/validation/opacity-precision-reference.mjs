// Compare public opacity requirements with Chrome's observable stacking boundary.
// Usage: node SCRIPT COMPILER NEW_OUTPUT [PROBE RENDERER]
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import crypto from 'node:crypto';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';
const [compilerArg,outArg,probeArg,rendererArg]=process.argv.slice(2);
assert(Boolean(probeArg)===Boolean(rendererArg));
const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const compiler=path.resolve(compilerArg),out=path.resolve(outArg);
assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const values=['0','.5','1','-1e100','1e100','1e100%','99.99999%','99.999994%','99.999997%','99.999999%','.99999994','.99999997','.99999999','100.000001%'];
const browser=await chromium.launch();const rows=[];
try {
 assert.equal(browser.version(),'153.0.8010.12');
 const page=await browser.newPage({viewport:{width:240,height:320},deviceScaleFactor:1});
 for(const value of values)for(const substituted of [false,true]) {
  const authored=substituted?`var(--alpha, ${value})`:value;
  const request={html:'<div id=frame><div id=group><div id=child></div></div><div id=sibling></div></div>',css:`#frame{position:relative;width:240px;height:320px;background:white}#group{width:130px;height:140px;background:blue;opacity:${authored}}#child{position:absolute;left:30px;top:30px;width:110px;height:100px;background:red;z-index:2}#sibling{position:absolute;left:60px;top:60px;width:100px;height:100px;background:lime;z-index:1}`,width:240,height:320};
  const prefix=path.join(out,`case-${rows.length}`);
  fs.writeFileSync(prefix+'.json',JSON.stringify(request));
  execFileSync(compiler,[prefix+'.json',prefix+'.riv']);
  const req=JSON.parse(fs.readFileSync(prefix+'.requirements.json'));
  await page.setContent(`<style>body{margin:0}${request.css}</style>${request.html}`);
  const observed=await page.evaluate(()=>({computed:getComputedStyle(document.getElementById('group')).opacity,top:document.elementFromPoint(100,100).id}));
  const groups=req.layout_group_opacity||[];
  const expectedTop=groups.length?'sibling':'child';
  rows.push({value,substituted,...observed,groups,expectedTop,pass:observed.top===expectedTop});
  await page.screenshot({path:prefix+'.browser.png'});
  if(probeArg) {
   execFileSync(path.resolve(probeArg),[prefix+'.riv',prefix+'.map.json','240','320',prefix],{env:{...process.env,NUXIE_NATIVE_GLYPHS:'0'}});
   execFileSync(path.resolve(rendererArg),['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']);
   const bounds=JSON.parse(fs.readFileSync(prefix+'.bounds.json'));
   const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(el=>{const b=el.getBoundingClientRect();return [el.id,{x:b.x,y:b.y,width:b.width,height:b.height}]})));
   const geometryFailures=[];
   for(const id of Object.keys(boxes))for(const axis of ['x','y','width','height'])if(Math.abs(boxes[id][axis]-bounds[id][axis])>.1)geometryFailures.push({id,axis,browser:boxes[id][axis],native:bounds[id][axis]});
   const native=PNG.sync.read(fs.readFileSync(prefix+'.native.png')),reference=PNG.sync.read(fs.readFileSync(prefix+'.browser.png'));
   const {metrics,failures,diff}=comparePixels(reference,native,boxes,false);
   fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
   const sample=image=>[...image.data.slice((100*image.width+100)*4,(100*image.width+100)*4+4)];
   const browserSample=sample(reference),nativeSample=sample(native);
   const samplePass=JSON.stringify(browserSample)===JSON.stringify(nativeSample);
   Object.assign(rows.at(-1),{prefix,geometryFailures,metrics,failures,browserSample,nativeSample,samplePass,
     browserSha256:sha(prefix+'.browser.png'),nativeSha256:sha(prefix+'.native.png'),streamSha256:sha(prefix+'.stream')});
   rows.at(-1).pass &&= geometryFailures.length===0 && failures.length===0 && samplePass;
  }

 }
}finally{await browser.close();}
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({browser:'153.0.8010.12',compilerSha256:sha(compiler),...(probeArg?{probeSha256:sha(probeArg),rendererSha256:sha(rendererArg)}:{}),rows},null,2)+'\n');
console.log(JSON.stringify({total:rows.length,failed:rows.filter(r=>!r.pass)}));
assert(rows.every(r=>r.pass),'Opacity stacking boundary differs from Chrome');
