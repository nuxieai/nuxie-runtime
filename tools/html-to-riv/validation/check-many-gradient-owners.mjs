// Public 64-owner/256-stop resource correctness, not performance or measured flush inventory.
// Usage: node SCRIPT FROZEN_TOOLCHAIN FRESH_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';
const [toolchainArg,outputArg]=process.argv.slice(2);
assert(outputArg);const toolchain=path.resolve(toolchainArg),out=path.resolve(outputArg);
assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const manifest=JSON.parse(fs.readFileSync(path.join(toolchain,'manifest.json')));
for(const name of ['html-to-riv','probe','renderer-replay'])assert.equal(sha(path.join(toolchain,name)),manifest.files[name].sha256);
const html='<div id="host">'+Array.from({length:64},(_,i)=>`<div id="cell${i}"></div>`).join('')+'</div>';
const css='#host{display:flex;flex-direction:row;flex-wrap:wrap;width:512px;}#host>div{width:64px;height:64px;flex-shrink:0;}'+Array.from({length:64},(_,i)=>`#cell${i}{background-image:linear-gradient(to right,${Array.from({length:256},(_,j)=>`rgb(${j},${i*4},${255-j}) ${j/255*100}%`).join(',')});}`).join('');
const request={html,css,width:512,height:512};fs.writeFileSync(path.join(out,'scene.json'),JSON.stringify(request));
const reset=fs.readFileSync(new URL('../src/reset.css',import.meta.url),'utf8');fs.writeFileSync(path.join(out,'reset.css'),reset);
const browser=await chromium.launch();let boxes;
try{assert.equal(browser.version(),'153.0.8010.12');const page=await browser.newPage({viewport:{width:512,height:512},deviceScaleFactor:1,colorScheme:'light',locale:'en-US',reducedMotion:'reduce'});await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{const {x,y,width,height}=e.getBoundingClientRect();return[e.id,{x,y,width,height}]})));await page.screenshot({path:path.join(out,'browser.png'),animations:'disabled'});}finally{await browser.close();}
for(let i=0;i<64;i++)assert.deepEqual(boxes[`cell${i}`],{x:(i%8)*64,y:Math.floor(i/8)*64,width:64,height:64});
fs.writeFileSync(path.join(out,'browser-boxes.json'),JSON.stringify(boxes,null,2));
const run=(name,args)=>{const log=execFileSync(path.join(toolchain,name),args,{env:{...process.env,NUXIE_NATIVE_GLYPHS:'0'},encoding:'utf8',maxBuffer:32*1024*1024});fs.writeFileSync(path.join(out,name+'.log'),log);};
run('html-to-riv',[path.join(out,'scene.json'),path.join(out,'scene.riv')]);
run('probe',[path.join(out,'scene.riv'),path.join(out,'scene.map.json'),'512','512',path.join(out,'scene')]);
const bounds=JSON.parse(fs.readFileSync(path.join(out,'scene.bounds.json')));assert.deepEqual(Object.keys(bounds).sort(),Object.keys(boxes).sort());
for(const [id,b] of Object.entries(boxes))for(const axis of ['x','y','width','height'])assert(Math.abs(bounds[id][axis]-b[axis])<=.1,`${id}.${axis}`);
const stream=fs.readFileSync(path.join(out,'scene.stream'),'utf8');const tables=stream.split('\n').filter(l=>l.includes('makePremultipliedLinearGradient'));
assert.equal(tables.length,64);assert.equal(new Set(tables.map(l=>l.split('stops=')[1])).size,64);assert(tables.every(l=>(l.match(/color=/g)||[]).length===256));
const reference=PNG.sync.read(fs.readFileSync(path.join(out,'browser.png'))),rows=[];
for(const backend of ['rust-metal','rust-metal-atomic']){
 const png=path.join(out,backend+'.png');run('renderer-replay',['--stream',path.join(out,'scene.stream'),'--output',png,'--backend',backend,'--mode',backend==='rust-metal-atomic'?'atomics':'clockwise-atomic']);
 const actual=PNG.sync.read(fs.readFileSync(png));const {metrics,failures,diff}=comparePixels(reference,actual,boxes,false);fs.writeFileSync(path.join(out,backend+'.diff.png'),PNG.sync.write(diff));rows.push({backend,metrics,failures,nativeSha256:sha(png)});
}
for(const name of ['html-to-riv','probe','renderer-replay'])assert.equal(sha(path.join(toolchain,name)),manifest.files[name].sha256);
const receipt={status:rows.every(r=>!r.failures.length)?'passed':'failed',scope:'64 distinct owners × 256 authored opaque stops, public compiler/import/render; no exact flush count claim',browser:'153.0.8010.12',owners:64,stopsPerOwner:256,geometryTolerance:.1,toolchain,manifestSha256:sha(path.join(toolchain,'manifest.json')),toolchainFiles:manifest.files,driverSha256:sha(fileURLToPath(import.meta.url)),pixelGateSha256:sha(fileURLToPath(new URL('./pixels.mjs',import.meta.url))),rows,files:Object.fromEntries(fs.readdirSync(out).map(n=>[n,sha(path.join(out,n))]))};fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');console.log(JSON.stringify({status:receipt.status,owners:64,rows:rows.map(r=>({backend:r.backend,failures:r.failures,meanChannelError:r.metrics.meanChannelError}))}));assert.equal(receipt.status,'passed');
