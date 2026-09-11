// Exact native renderer controls for the optional hard-difference operation.
// These establish backend semantics; Chrome text qualification is separate.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {PNG} from 'pngjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const dir=path.join(root,'output/playwright/html-to-riv/hard-clip-native');
fs.mkdirSync(dir,{recursive:true});
const rectangle=(l,t,r,b)=>`{id=1,fillRule=0,path={verbs=[move,line,line,line,close],points=[(${l},${t}),(${r},${t}),(${r},${b}),(${l},${b})]}}`;
const paint=color=>`{id=1,style=fill,color=${color},thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}`;
const controls=[
 {name:'fractional',matrix:[1,0,0,1,0,0],inverse:[1,0,0,1,0,0],rects:[[4.25,5.5,12.25,15.5]],excluded:(x,y)=>x>=4&&x<12&&y>=6&&y<16},
 {name:'scaled',matrix:[2,0,0,1.5,.25,.5],inverse:[.5,0,0,2/3,-.125,-1/3],rects:[[2,2,6,6]],excluded:(x,y)=>x>=4&&x<12&&y>=4&&y<10},
 {name:'reflected',matrix:[-2,0,0,1,24,0],inverse:[-.5,0,0,1,12,0],rects:[[2,2,6,6]],excluded:(x,y)=>x>=12&&x<20&&y>=2&&y<6},
 {name:'rotated',matrix:[1,1,-1,1,16,0],inverse:[.5,-.5,.5,.5,-8,8],rects:[[2,2,6,6]],excluded:(x,y)=>{const spans=[[16,17],[15,18],[14,19],[13,20],[13,20],[14,19],[15,18],[16,17]];return y>=4&&y<12&&x>=spans[y-4][0]&&x<spans[y-4][1];}},
 {name:'overlapping',matrix:[1,0,0,1,0,0],inverse:[1,0,0,1,0,0],rects:[[4,4,14,14],[8,8,18,18]],excluded:(x,y)=>(x>=4&&x<14&&y>=4&&y<14)||(x>=8&&x<18&&y>=8&&y<18)},
];
const results=[];
for(const c of controls){
 const prefix=path.join(dir,c.name);
 const stream=['rive-golden-stream-v1','frameSize width=32 height=32','clearColor value=0xffffffff','save',`clipPath path=${rectangle(2,2,30,24)}`,`transform matrix=[${c.matrix}]`,...c.rects.map(r=>`clipOutRect rect=[${r}]`),`transform matrix=[${c.inverse}]`,`drawPath path=${rectangle(0,0,32,32)} paint=${paint('0xffff0000')}`,'restore',`drawPath path=${rectangle(0,28,32,32)} paint=${paint('0xff0000ff')}`,'frame',''].join('\n');
 fs.writeFileSync(prefix+'.stream',stream);
 execFileSync(path.join(root,'target/debug/renderer-replay'),['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic'],{stdio:'pipe'});
 const actual=PNG.sync.read(fs.readFileSync(prefix+'.native.png')),expected=new PNG({width:32,height:32});
 let mismatched=0;
 for(let y=0;y<32;y++)for(let x=0;x<32;x++){
  const color=y>=28?[0,0,255,255]:x>=2&&x<30&&y>=2&&y<24&&!c.excluded(x,y)?[255,0,0,255]:[255,255,255,255];
  const i=(y*32+x)*4;expected.data.set(color,i);
  if(color.some((v,j)=>actual.data[i+j]!==v))mismatched++;
 }
 fs.writeFileSync(prefix+'.expected.png',PNG.sync.write(expected));
 results.push({name:c.name,mismatchedPixels:mismatched});
}
fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify(results,null,2));
console.log(JSON.stringify(results,null,2));
if(results.some(r=>r.mismatchedPixels))process.exitCode=1;
