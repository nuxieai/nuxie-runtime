// Diagnostic: native alpha modulation is not CSS group opacity. Does not waive
// host-state failures or claim support for authored opacity.
import {chromium} from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {PNG} from 'pngjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const dir=path.join(root,'output/playwright/html-to-riv/opacity-overlap-reference');
fs.mkdirSync(dir,{recursive:true});
const rectangle=(l,t,r,b)=>`{id=1,fillRule=0,path={verbs=[move,line,line,line,close],points=[(${l},${t}),(${r},${t}),(${r},${b}),(${l},${b})]}}`;
const paint=color=>`{id=1,style=fill,color=${color},thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}`;
const stream=['rive-golden-stream-v1','frameSize width=32 height=32','clearColor value=0xffffffff','save','modulateOpacity opacity=0.5',`drawPath path=${rectangle(4,4,24,20)} paint=${paint('0xffff0000')}`,`drawPath path=${rectangle(12,12,28,28)} paint=${paint('0xff000000')}`,'restore','frame',''].join('\n');
fs.writeFileSync(path.join(dir,'native.stream'),stream);
execFileSync(path.join(root,'target/debug/renderer-replay'),['--stream',path.join(dir,'native.stream'),'--output',path.join(dir,'native.png'),'--backend','rust-metal','--mode','clockwise-atomic'],{stdio:'pipe'});
const browser=await chromium.launch();
try {
 const page=await browser.newPage({viewport:{width:32,height:32},deviceScaleFactor:1});
 for(const mode of ['group','per-draw']){
  await page.setContent(`<style>body{margin:0;background:white}.group{${mode==='group'?'opacity:.5':''}}i{position:absolute;display:block;${mode==='per-draw'?'opacity:.5':''}}.red{left:4px;top:4px;width:20px;height:16px;background:red}.black{left:12px;top:12px;width:16px;height:16px;background:black}</style><div class="group"><i class="red"></i><i class="black"></i></div>`);
  await page.screenshot({path:path.join(dir,mode+'.png')});
 }
 const images=Object.fromEntries(['native','group','per-draw'].map(k=>[k,PNG.sync.read(fs.readFileSync(path.join(dir,k+'.png')))]));
 const sample=(img,x,y)=>[...img.data.subarray((y*32+x)*4,(y*32+x)*4+4)];
 const samples=Object.fromEntries(Object.entries(images).map(([k,img])=>[k,{red:sample(img,8,8),black:sample(img,26,26),overlap:sample(img,16,16)}]));
 const maxDifference=(a,b)=>Math.max(...images[a].data.map((v,i)=>Math.abs(v-images[b].data[i])));
 const result={browser:browser.version(),samples,nativeVersusPerDrawMax:maxDifference('native','per-draw'),nativeVersusGroupMax:maxDifference('native','group'),qualification:false};
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify(result,null,2));console.log(JSON.stringify(result,null,2));
 if(result.nativeVersusPerDrawMax>1 || result.nativeVersusGroupMax<60)throw new Error('Expected overlap distinction was not observed');
}finally{await browser.close();}
