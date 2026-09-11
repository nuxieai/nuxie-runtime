// Capability experiments on ordinary RIV files; not public compiler qualification.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';

const [inputArg,rendererArg,outArg]=process.argv.slice(2);
assert(outArg,'usage: node check-ordinary-layout.mjs PROBE_EVIDENCE BASELINE_RENDERER NEW_OUTPUT');
const input=path.resolve(inputArg), renderer=path.resolve(rendererArg), out=path.resolve(outArg);
assert(!fs.existsSync(out),'fresh output required');fs.mkdirSync(out,{recursive:true});
const hash=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const rendererHash=hash(renderer),rows=[];
const browser=await chromium.launch();
try {
  assert.equal(browser.version(),'153.0.8010.12');
  const page=await browser.newPage({deviceScaleFactor:1,colorScheme:'light',locale:'en-US',reducedMotion:'reduce'});
  for(const [mode,color] of [['solid','rebeccapurple'],['fractional','hsl(120 100% 25%)'],['nested','coral'],['corners','hsl(200 100% 40%)'],['corners-limit','hsl(200 100% 40%)'],['alpha','hsla(270,50%,40%,0.5)'],['corners-wrapper','hsl(200 100% 40%)']]) {
    const frames=JSON.parse(fs.readFileSync(path.join(input,mode,'frames.json'))).frames;
    assert.equal(frames.length,8);
    const child=mode==='nested'||mode==='fractional';
    const css=`html,body{margin:0;width:100%;height:100%;background:white}div{display:flex;flex-direction:column;box-sizing:border-box}#outer{width:100%;height:100%;background:${color};${mode==='corners'?'border-radius:8px 20px 32px 4px;':mode==='corners-limit'?'border-radius:100px 100px 0 0;':mode==='corners-wrapper'?'border-radius:100px 100px 0 0;min-height:100px;':''}}#inner{width:${mode==='fractional'?'80.5px':'50%'};height:50%;background:rebeccapurple}`;
    const html=`<!doctype html><style>${css}</style><div id="outer">${child?'<div id="inner"></div>':''}</div>`;
    fs.writeFileSync(path.join(out,mode+'.html'),html);
    for(const f of frames) {
      const prefix=path.join(out,`${mode}-${f.frame}`);
      await page.setViewportSize({width:f.width,height:f.height});await page.setContent(html);
      const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{const{x,y,width,height}=e.getBoundingClientRect();return[e.id,{x,y,width,height}]})));
      await page.screenshot({path:prefix+'.chrome.png',animations:'disabled'});
      const geometryFile=path.join(input,mode,f.geometry),streamFile=path.join(input,mode,f.stream);
      const geometry=JSON.parse(fs.readFileSync(geometryFile));const geometryFailures=[];
      for(const [name,id] of [['outer',2],...(child?[['inner',6]]:[])]) {
        const actual=geometry.find(g=>g.objectId===id);assert(actual,`missing object ${id}`);
        const matrix=actual.worldMatrix;assert.deepEqual(matrix.slice(0,4),[1,0,0,1]);
        const observed={x:matrix[4],y:matrix[5],width:actual.width,height:actual.height};
        for(const key of Object.keys(observed)) if(Math.abs(observed[key]-boxes[name][key])>0.1)geometryFailures.push(`${name}.${key}: ${observed[key]} vs ${boxes[name][key]}`);
      }
      const command=[renderer,'--stream',streamFile,'--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic','--frame',String(f.frame)];
      // rust-metal selects existing raster-order; mode is the baseline CLI's required non-MSAA token.
      const native=spawnSync(command[0],command.slice(1),{encoding:'utf8'});
      fs.writeFileSync(prefix+'.native.log',native.stdout+native.stderr);assert.equal(native.status,0,native.stderr);
      const reference=PNG.sync.read(fs.readFileSync(prefix+'.chrome.png')),actual=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));
      const comparison=comparePixels(reference,actual,boxes,false);fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(comparison.diff));
      rows.push({mode,...f,boxes,geometry,geometryFailures,pixelFailures:comparison.failures,metrics:comparison.metrics,command,sourceSha256:hash(path.join(out,mode+'.html')),rivSha256:hash(path.join(input,mode,'scene.riv')),streamSha256:hash(streamFile),geometrySha256:hash(geometryFile),chromeSha256:hash(prefix+'.chrome.png'),nativeSha256:hash(prefix+'.native.png')});
    }
  }
} finally {await browser.close();}
assert.equal(hash(renderer),rendererHash);
const receipt={status:rows.every(r=>!r.geometryFailures.length&&!r.pixelFailures.length)?'passed-experiments':'failed-experiments',scope:'Seven ordinary file capability experiments, original/clone resize; no public HTML/CSS admission or visual-review claim',browser:'153.0.8010.12',renderer,rendererSha256:rendererHash,driverSha256:hash(fileURLToPath(import.meta.url)),pixelGateSha256:hash(fileURLToPath(new URL('./pixels.mjs',import.meta.url))),rows};
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
const cells=rows.map(r=>`<section><h2>${r.mode} · ${r.width}×${r.height} · instance ${r.instance} step ${r.step}</h2><p>${[...r.geometryFailures,...r.pixelFailures].join('; ')||'metric gates pass'}</p><div><img src="${r.mode}-${r.frame}.chrome.png" alt="Chrome"><img src="${r.mode}-${r.frame}.native.png" alt="Baseline native"></div></section>`).join('');
fs.writeFileSync(path.join(out,'gallery.html'),`<!doctype html><meta charset="utf-8"><title>Immutable Rive experiments</title><style>body{font:15px system-ui;margin:24px}section{margin:32px 0}section div{display:flex;gap:16px}img{max-width:48%;object-fit:contain;object-position:left top;border:1px solid #aaa}</style><h1>Chrome / unchanged native Rive</h1><p>Capability experiments only. Public compiler and full feature qualification remain pending.</p>${cells}`);
console.log(JSON.stringify({status:receipt.status,rows:rows.length,geometryPass:rows.filter(r=>!r.geometryFailures.length).length,pixelPass:rows.filter(r=>!r.pixelFailures.length).length,failures:rows.filter(r=>r.geometryFailures.length||r.pixelFailures.length).map(r=>({mode:r.mode,frame:r.frame,geometry:r.geometryFailures,pixels:r.pixelFailures}))}));
