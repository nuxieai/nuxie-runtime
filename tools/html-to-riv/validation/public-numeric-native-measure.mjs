// Add precise local/rectangle observations to an existing same-file native run.
// Usage: node SCRIPT LARGE_CASES RENDER_RECEIPT FRESH_OUTPUT
import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import{fileURLToPath}from'node:url';import{chromium}from'@playwright/test';
const[casesPath,renderPath,out]=process.argv.slice(2).map(p=>path.resolve(p));assert(out&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const read=p=>JSON.parse(fs.readFileSync(p)),hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const cases=read(casesPath),render=read(renderPath),resetPath=fileURLToPath(new URL('../src/reset.css',import.meta.url)),reset=fs.readFileSync(resetPath,'utf8');
assert.equal(hash(resetPath),render.resetSha256);assert.equal(hash(casesPath),render.fixturesSha256);
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');const rows=[];
try{const page=await browser.newPage({deviceScaleFactor:1,colorScheme:'light',locale:'en-US'});
 for(const sample of cases){
  const frames=render.rows.filter(r=>r.name===sample.name);assert.equal(frames.length,8);
  for(const frame of frames){
   await page.setViewportSize({width:frame.width,height:frame.height});await page.setContent(`<!doctype html><style>${reset}\n${sample.css}</style>${sample.html}`);
   const measured=await page.evaluate(async()=>{
    const nodes=[...document.querySelectorAll('[id]')];
    const local=await new Promise(resolve=>{const ro=new ResizeObserver(entries=>{ro.disconnect();resolve(Object.fromEntries(entries.map(e=>[e.target.id,{contentWidth:e.contentRect.width,contentHeight:e.contentRect.height,borderWidth:e.borderBoxSize[0].inlineSize,borderHeight:e.borderBoxSize[0].blockSize}])));});nodes.forEach(e=>ro.observe(e));});
    return Object.fromEntries(nodes.map(e=>{const rect=e.getBoundingClientRect(),m=e.computedStyleMap(),s=getComputedStyle(e);const typed=key=>{const v=m.get(key);return{value:v.value??null,unit:v.unit??null,text:String(v)};};return[e.id,{rect:{x:rect.x,y:rect.y,width:rect.width,height:rect.height},local:local[e.id],typed:{width:typed('width'),paddingLeft:typed('padding-left'),paddingRight:typed('padding-right')},cssomWidth:s.width}];}));
   });
   const mapPath=path.join(path.dirname(frame.prefix),'scene.map.json');assert.equal(hash(mapPath),frame.sourceMapSha256);
   const map=read(mapPath),comparisons=[];
   for(const item of map){
    const g=frame.geometry.find(g=>g.objectId===item.object_id);assert(g);
    // Rust's short JSON float spelling must be restored to its original f32.
    const native={x:Math.fround(g.worldMatrix[4]),y:Math.fround(g.worldMatrix[5]),width:Math.fround(g.width),height:Math.fround(g.height)};
    const chrome=measured[item.id];assert.deepEqual(chrome.rect,frame.boxes[item.id]);
    const rectangleDelta=Object.fromEntries(Object.keys(native).map(axis=>[axis,native[axis]-chrome.rect[axis]]));
    comparisons.push({id:item.id,native,chrome,rectangleDelta,localBorderWidthDelta:native.width-chrome.local.borderWidth,
                      geometryFailures:Object.entries(rectangleDelta).filter(([,value])=>Math.abs(value)>.1).map(([axis,value])=>({axis,delta:value}))});
   }
   rows.push({name:sample.name,frame:frame.frame,instance:frame.instance,step:frame.step,width:frame.width,height:frame.height,requestSha256:frame.requestSha256,rivSha256:frame.rivSha256,comparisons});
  }
 }
}finally{await browser.close();}
const failures=rows.flatMap(row=>row.comparisons.flatMap(c=>c.geometryFailures.map(f=>({name:row.name,frame:row.frame,id:c.id,...f}))));
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({scope:'Read-only supplemental measurements from the same native original/clone files already rendered. Restored f32 dimensions distinguish local sizing from Chrome rectangle projection. No inferred runtime fix or general equivalence.',browser:'153.0.8010.12',renderReceiptSha256:hash(renderPath),fixturesSha256:hash(casesPath),driverSha256:hash(fileURLToPath(import.meta.url)),rows,failures},null,2)+'\n');
console.log(JSON.stringify({cases:cases.length,frames:rows.length,frameFailures:rows.filter(r=>r.comparisons.some(c=>c.geometryFailures.length)).length,failureCount:failures.length}));
if(failures.length)process.exitCode=1;
