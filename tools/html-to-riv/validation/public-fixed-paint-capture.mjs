// Direct authored rectangle primitive references, not public compiler/CSS admission.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import crypto from 'node:crypto';
import{chromium}from'@playwright/test';import{PNG}from'pngjs';import{comparePixels}from'./pixels.mjs';
const out=path.resolve(process.argv[2]);const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const cases=JSON.parse(fs.readFileSync(path.join(out,'cases.json')));const rows=[];
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');
try {const page=await browser.newPage({deviceScaleFactor:1});
for(const c of cases){const dir=path.join(out,'cases',c.name);const frames=JSON.parse(fs.readFileSync(path.join(dir,'probe/frames.json'))).frames;assert.equal(frames.length,8);
let html='<!doctype html><style>html,body{margin:0;padding:0;background:white;}div{position:absolute;}</style>';
for(const[r,rect]of c.rects.entries()){if(rect.right<=rect.left||rect.bottom<=rect.top||(rect.color>>>24)===0)continue;const color=rect.color>>>0;html+=`<div id="p${r}" style="left:${rect.left}px;top:${rect.top}px;width:${rect.right-rect.left}px;height:${rect.bottom-rect.top}px;background:rgba(${color>>>16&255},${color>>>8&255},${color&255},${(color>>>24)/255})"></div>`;}
fs.writeFileSync(path.join(dir,'reference.html'),html);
for(const f of frames){await page.setViewportSize({width:f.width,height:f.height});await page.setContent(html);const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('div')].map(e=>{const{x,y,width,height}=e.getBoundingClientRect();return[e.id,{x,y,width,height}]})));
const prefix=path.join(dir,`frame-${f.frame}`);await page.screenshot({path:prefix+'.chrome.png'});const native=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));const comparison=comparePixels(PNG.sync.read(fs.readFileSync(prefix+'.chrome.png')),native,boxes,false);fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(comparison.diff));
for(const clear of ['cyan','transparent'])assert(native.data.equals(PNG.sync.read(fs.readFileSync(prefix+'.'+clear+'.png')).data),'host background must be ordinary paint');
rows.push({name:c.name,...f,boxes,pixelFailures:comparison.failures,metrics:comparison.metrics,chromeSha256:sha(prefix+'.chrome.png'),nativeSha256:sha(prefix+'.native.png'),diffSha256:sha(prefix+'.diff.png')});}}
}finally{await browser.close();}
fs.writeFileSync(path.join(out,'visual-receipt.json'),JSON.stringify({scope:'Direct fixed_paint primitive only; no source-folding proof or authored geometry comparison',browser:'153.0.8010.12',rows},null,2));assert(rows.every(r=>r.pixelFailures.length===0),'pixel gate failure');console.log(JSON.stringify({frames:rows.length,pixelPass:rows.filter(r=>!r.pixelFailures.length).length}));
