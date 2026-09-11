// Locate failures from the unchanged red support gate in an existing state run.
import fs from 'node:fs';
import path from 'node:path';
import {PNG} from 'pngjs';
import {compareRedDecoration} from './pixels.mjs';
const dir=process.argv[2];
if(!dir)throw new Error('Usage: node red-decoration-locations.mjs <state-review-directory>');
const results=JSON.parse(fs.readFileSync(path.join(dir,'results.json'))),report=[];
for(const result of results){
 const prefix=path.join(dir,`${result.name}-${result.width}`);
 const reference=PNG.sync.read(fs.readFileSync(prefix+'.browser.png'));
 const actual=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));
 const {width,height}=reference;
 const gate=compareRedDecoration(reference,actual,{x:0,y:0,width,height});
 if(gate.passed)continue;
 const red=(image,x,y)=>{if(x<0||y<0||x>=width||y>=height)return false;const i=(y*width+x)*4;return image.data[i]>image.data[i+1]+64&&image.data[i]>image.data[i+2]+64;};
 const locations=(a,b)=>{
  const list=[];
  for(let y=0;y<height;y++)for(let x=0;x<width;x++){
   if(!red(a,x,y))continue;
   let found=false;
   for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++)if(red(b,x+dx,y+dy))found=true;
   if(!found){const i=(y*width+x)*4;list.push({x,y,reference:[...reference.data.subarray(i,i+4)],actual:[...actual.data.subarray(i,i+4)]});}
  }
  return list;
 };
 const missing=locations(reference,actual),extra=locations(actual,reference);
 if(missing.length!==gate.missing||extra.length!==gate.extra)throw new Error('Diagnostic diverged from the red support gate');
 report.push({name:result.name,width,gate,missing,extra});
}
fs.writeFileSync(path.join(dir,'red-locations.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify(report,null,2));
