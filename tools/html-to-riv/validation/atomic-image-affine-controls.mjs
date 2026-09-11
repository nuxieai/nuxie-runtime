// Compare image antialiasing under shear/rotation with the raster backend.
// Usage: node SCRIPT OLD_RENDERER NEW_RENDERER NEW_OUTPUT
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import crypto from 'node:crypto';import {execFileSync} from 'node:child_process';import {PNG} from 'pngjs';import {comparePixels} from './pixels.mjs';
const [oldRenderer,newRenderer,out]=process.argv.slice(2).map(p=>path.resolve(p));assert(!fs.existsSync(out));fs.mkdirSync(out);
const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const white='89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c63f8ffffff7f0009fb03fd2a86e38a0000000049454e44ae426082';
const controls=[['shear-x',[160,0,64,100,20,20]],['rotate90',[0,150,-100,0,150,20]],['rotate-general',[96,72,-72,96,100,30]],['affine-general',[120,35,-25,80,80,40]]];const rows=[];
for(const [name,m] of controls){
 const stream=path.join(out,name+'.stream');fs.writeFileSync(stream,`rive-golden-stream-v1\nframeSize width=256 height=256\nclearColor value=0\ndecodeImage id=1 data=${white}\ntransform matrix=[${m}]\ndrawImage image=1 sampler={wrapX=0,wrapY=0,filter=0} blendMode=3 opacity=1\nframe\n`);
 const images={};for(const [label,binary,backend,mode] of [['raster',oldRenderer,'rust-metal','clockwise-atomic'],['old',oldRenderer,'rust-metal-atomic','atomics'],['new',newRenderer,'rust-metal-atomic','atomics']]){
  const file=path.join(out,`${name}.${label}.png`);execFileSync(binary,['--stream',stream,'--output',file,'--backend',backend,'--mode',mode],{stdio:'pipe'});images[label]={file,png:PNG.sync.read(fs.readFileSync(file))};
 }
 const comparisons={};for(const label of ['old','new']){const result=comparePixels(images.raster.png,images[label].png,{},false);fs.writeFileSync(path.join(out,`${name}.${label}.diff.png`),PNG.sync.write(result.diff));let alphaError=0,maxAlphaError=0;for(let i=3;i<images.raster.png.data.length;i+=4){const e=Math.abs(images.raster.png.data[i]-images[label].png.data[i]);alphaError+=e;maxAlphaError=Math.max(maxAlphaError,e);}comparisons[label]={metrics:result.metrics,failures:result.failures,totalAlphaError:alphaError,maxAlphaError};}
 rows.push({name,matrix:m,streamSha256:sha(stream),images:Object.fromEntries(Object.entries(images).map(([k,v])=>[k,sha(v.file)])),comparisons});
}
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({scope:'Affine native image AA comparison; raster backend reference, not new Chrome qualification',oldRendererSha256:sha(oldRenderer),newRendererSha256:sha(newRenderer),rows},null,2)+'\n');
console.log(JSON.stringify(rows.map(r=>({name:r.name,oldError:r.comparisons.old.totalAlphaError,newError:r.comparisons.new.totalAlphaError,newFailures:r.comparisons.new.failures}))));
assert(rows.every(r=>r.comparisons.new.failures.length===0));assert(rows.every(r=>r.comparisons.new.totalAlphaError<=r.comparisons.old.totalAlphaError));
