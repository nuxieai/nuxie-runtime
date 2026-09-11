// Rejected proxy control: Chrome SVG clipping remains antialiased even with
// crispEdges, so this font-free comparison cannot qualify text hard clipping.
// Retained to prevent substituting an invalid browser reference.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const dir=path.join(root,'output/playwright/html-to-riv/hard-clip-browser-reference');
fs.mkdirSync(dir,{recursive:true});
const clips=[[107.53456,61.34375,122.89465,64.34375],[123.337524,61.34375,129.52142,64.34375]];
const stripe=[107,61.84375,129.703125,63.84375];
const polygon=([l,t,r,b])=>`M${l} ${t}H${r}V${b}H${l}Z`;
const pathRecord=([l,t,r,b])=>`{id=1,fillRule=0,path={verbs=[move,line,line,line,close],points=[(${l},${t}),(${r},${t}),(${r},${b}),(${l},${b})]}}`;
const browser=await chromium.launch();
const page=await browser.newPage({viewport:{width:240,height:160},deviceScaleFactor:1});
const results=[];
try{
 for(const tx of [2.24,2.245,2.249,2.25,2.251,2.255,2.26]){
  const prefix=path.join(dir,String(tx));
  const stream=['rive-golden-stream-v1','frameSize width=240 height=160','clearColor value=0xffffffff','save',`transform matrix=[1.4,0,0,0.8,${tx},3.5]`,'transform matrix=[1,0,0,1,8,10.15625]',...clips.map(r=>`clipOutRect rect=[${r}]`),`drawPath path=${pathRecord(stripe)} paint={id=1,style=fill,color=0xffff0000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}`,'restore','frame',''].join('\n');
  fs.writeFileSync(prefix+'.stream',stream);
  execFileSync(path.join(root,'target/debug/renderer-replay'),['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic'],{stdio:'pipe'});
  const modes={};
  for(const rendering of ['crispEdges','auto']){
   const svg=`<svg width="240" height="160"><defs>${clips.map((r,i)=>`<clipPath id="c${i}" clipPathUnits="userSpaceOnUse"><path clip-rule="evenodd" shape-rendering="${rendering}" d="${polygon([-1000,-1000,1000,1000])+polygon(r)}"/></clipPath>`).join('')}</defs><g transform="matrix(1.4 0 0 .8 ${tx} 3.5)"><g transform="translate(8 10.15625)"><g clip-path="url(#c0)"><g clip-path="url(#c1)"><path fill="red" d="${polygon(stripe)}"/></g></g></g></g></svg>`;
   const html='<!doctype html><style>body{margin:0;background:white}</style>'+svg;
   fs.writeFileSync(prefix+'.'+rendering+'.html',html);
   await page.setContent(html);await page.screenshot({path:prefix+'.'+rendering+'.png'});
   const png=PNG.sync.read(fs.readFileSync(prefix+'.'+rendering+'.png'));
   modes[rendering]=[61,62].map(y=>[...png.data.subarray((y*240+185)*4,(y*240+185)*4+4)]);
  }
  const native=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));
  results.push({tx,...modes,native:[61,62].map(y=>[...native.data.subarray((y*240+185)*4,(y*240+185)*4+4)])});
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify({kind:'SVG clipping proxy; not text hard-clip qualification',browser:browser.version(),clips,stripe,results},null,2));
 console.log(JSON.stringify(results,null,2));
}finally{await browser.close();}
