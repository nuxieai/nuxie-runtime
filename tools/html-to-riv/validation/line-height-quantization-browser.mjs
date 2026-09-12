// Independent browser characterization. Observations never enter Rive emission.
// Usage: CASES.json FONT.ttf NEW_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';

const [casesPath,fontPath,out] = process.argv.slice(2).map(p=>path.resolve(p));
assert(!fs.existsSync(out)); fs.mkdirSync(out,{recursive:true});
const hash = p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const write=(p,value)=>fs.writeFileSync(p,JSON.stringify(value,null,2)+'\n');
const bindings=[casesPath,fontPath,fileURLToPath(import.meta.url)].map(p=>({path:p,sha256:hash(p)}));
const cases=JSON.parse(fs.readFileSync(casesPath));
const fontData=fs.readFileSync(fontPath).toString('base64');
const browser=await chromium.launch(); assert.equal(browser.version(),'153.0.8010.12');
const rows=[];
try {
  const page=await browser.newPage({viewport:{width:390,height:200},deviceScaleFactor:1,colorScheme:'light',locale:'en-US'});
  for(const fixture of cases) {
    const prefix=path.join(out,fixture.name);
    const reset=`@font-face{font-family:Probe;src:url(data:font/ttf;base64,${fontData})}html,body{margin:0;background:white}body{padding:0 20px;box-sizing:border-box}#text{font-family:Probe;font-size:${fixture.fontSize}px;line-height:${fixture.factor};white-space:pre;position:relative;top:${fixture.y}px}`;
    const html=`<!doctype html><style>${reset}</style><div id="text"></div>`;
    fs.writeFileSync(prefix+'.html',html);write(prefix+'.source.json',fixture);
    await page.setContent(html);
    await page.locator('#text').evaluate((e,text)=>e.textContent=text,fixture.text);
    const readiness=await page.evaluate(async ({fontSize,text})=>{
      const query=`${fontSize}px Probe`,faces=await document.fonts.load(query,text);
      await document.fonts.ready;
      return {check:document.fonts.check(query,text),faces:faces.map(f=>({family:f.family,status:f.status}))};
    },fixture);
    assert(readiness.check && readiness.faces.length===1 && readiness.faces[0].family==='Probe' && readiness.faces[0].status==='loaded');
    const observed=await page.locator('#text').evaluate(e=>{
      const s=getComputedStyle(e),box=e.getBoundingClientRect(),node=e.firstChild;
      const ctx=document.createElement('canvas').getContext('2d');ctx.font=`${s.fontSize} Probe`;
      const font=ctx.measureText('Agjp'),fragments=[];
      for(let i=0;i<node.length;i++) {
        if(node.textContent[i]==='\n') continue;
        const range=document.createRange();range.setStart(node,i);range.setEnd(node,i+1);
        const r=range.getBoundingClientRect();
        if(r.width>0&&!fragments.some(f=>f.y===r.y))fragments.push({y:r.y,height:r.height,baseline:r.y+font.fontBoundingBoxAscent});
      }
      return {box:{x:box.x,y:box.y,width:box.width,height:box.height},fontSize:s.fontSize,lineHeight:s.lineHeight,
        factor:e.computedStyleMap().get('line-height').toString(),fontAscent:font.fontBoundingBoxAscent,fontDescent:font.fontBoundingBoxDescent,fragments};
    });
    await page.screenshot({path:prefix+'.chrome.png'});
    const marker=await page.locator('#text').evaluate(e=>{
      const parts=e.textContent.split('\n');e.textContent='';
      for(const [i,part] of parts.entries()) {
        if(i)e.append(document.createTextNode('\n'));
        const marker=document.createElement('span');marker.dataset.marker='';
        marker.style.cssText='display:inline-block;width:0;height:0;margin:0;padding:0;border:0;vertical-align:baseline';
        e.append(marker,document.createTextNode(part));
      }
      const r=e.getBoundingClientRect();
      return {box:{x:r.x,y:r.y,width:r.width,height:r.height},baselines:[...e.querySelectorAll('[data-marker]')].map(m=>m.getBoundingClientRect().y)};
    });
    await page.screenshot({path:prefix+'.marked.png'});
    const markerUsable=JSON.stringify(observed.box)===JSON.stringify(marker.box) &&
      PNG.sync.read(fs.readFileSync(prefix+'.chrome.png')).data.equals(PNG.sync.read(fs.readFileSync(prefix+'.marked.png')).data);
    const rawHalf=(Number.parseFloat(observed.lineHeight)-observed.fontAscent-observed.fontDescent)/2;
    const predictions={symmetric:observed.fontAscent+rawHalf,integerLeading:observed.fontAscent+Math.floor(rawHalf)};
    // These predictions use observed metrics to characterize the browser only.
    // A compiler candidate must independently derive metrics from source/font.
    rows.push({name:fixture.name,fixture,readiness,observed,marker,markerUsable,predictions,
      rangeMarkerAgree:markerUsable&&marker.baselines.every((b,i)=>Math.abs(b-observed.fragments[i]?.baseline)<1e-6),
      residuals:Object.fromEntries(Object.entries(predictions).map(([name,b])=>[name,observed.fragments.map((f,i)=>f.baseline-observed.box.y-i*Number.parseFloat(observed.lineHeight)-b)])),
      sourceSha256:hash(prefix+'.source.json'),htmlSha256:hash(prefix+'.html'),chromeSha256:hash(prefix+'.chrome.png'),markedSha256:hash(prefix+'.marked.png')});
  }
} finally {await browser.close();}
for(const b of bindings) assert.equal(hash(b.path),b.sha256);
const receipt={scope:'Finite pinned-Chrome baseline characterization; no native output or compiler qualification.',browser:'153.0.8010.12',bindings,rows,
  summary:{cases:rows.length,fontReady:rows.filter(r=>r.readiness.check).length,markerUsable:rows.filter(r=>r.markerUsable).length,rangeMarkerAgree:rows.filter(r=>r.rangeMarkerAgree).length,
    symmetricExact:rows.filter(r=>r.residuals.symmetric.every(x=>Math.abs(x)<1e-6)).length,integerLeadingExact:rows.filter(r=>r.residuals.integerLeading.every(x=>Math.abs(x)<1e-6)).length}};
write(path.join(out,'receipt.json'),receipt);console.log(JSON.stringify(receipt.summary));
