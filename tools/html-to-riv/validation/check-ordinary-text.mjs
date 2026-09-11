// Ordinary-file text experiment. No public HTML/CSS text admission is claimed.
// Usage: RIV FONT BASELINE_PROBE BASELINE_RENDERER NEW_OUTPUT [REFERENCE.json]
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {comparePixels,compareInkPresence} from './pixels.mjs';
const args=process.argv.slice(2);assert([5,6].includes(args.length));
const [riv,font,probe,renderer,out]=args.slice(0,5).map(p=>path.resolve(p));
const ref=args[5]?JSON.parse(fs.readFileSync(path.resolve(args[5]))):{fontSize:32,x:20,y:20,text:'aaaa',lineHeight:'normal'};
assert(Object.keys(ref).every(k=>['fontSize','x','y','text','lineHeight'].includes(k)));
for(const k of ['fontSize','x','y'])assert(Number.isFinite(ref[k])&&ref[k]>=0&&ref[k]<=16384);
assert(ref.fontSize>0&&typeof ref.text==='string'&&ref.text.length>0&&ref.text.length<=8192);
assert(ref.lineHeight==='normal'||(Number.isFinite(ref.lineHeight)&&ref.lineHeight>0&&ref.lineHeight<=16384));
assert(!fs.existsSync(out),'fresh output required');fs.mkdirSync(out,{recursive:true});
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const bindings=[riv,font,probe,renderer,fileURLToPath(import.meta.url),fileURLToPath(new URL('./pixels.mjs',import.meta.url))].map(p=>({path:p,sha256:hash(p)}));
if(args[5])bindings.push({path:path.resolve(args[5]),sha256:hash(path.resolve(args[5]))});
fs.writeFileSync(path.join(out,'reference-input.json'),JSON.stringify(ref,null,2)+'\n');
fs.copyFileSync(riv,path.join(out,'scene.riv'));fs.copyFileSync(font,path.join(out,'font.ttf'));
fs.copyFileSync(fileURLToPath(import.meta.url),path.join(out,'check-ordinary-text.mjs'));
fs.copyFileSync(fileURLToPath(new URL('./pixels.mjs',import.meta.url)),path.join(out,'pixels.mjs'));
const run=(command,log)=>{const r=spawnSync(command[0],command.slice(1),{encoding:'utf8'});fs.writeFileSync(log,(r.stdout??'')+(r.stderr??''));assert.ifError(r.error);assert.equal(r.status,0,r.stderr);};
const observed=path.join(out,'probe');
run([probe,riv,observed,'240x160','390x200','768x120','240x160'],path.join(out,'probe.log'));
const frames=JSON.parse(fs.readFileSync(path.join(observed,'frames.json'))).frames;assert.equal(frames.length,8);
const escapedText=ref.text.replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));
const html=`<!doctype html><style>@font-face{font-family:Probe;src:url(data:font/ttf;base64,${fs.readFileSync(font).toString('base64')})}html,body{margin:0;background:white}#text{position:absolute;left:${ref.x}px;top:${ref.y}px;font:${ref.fontSize}px Probe;line-height:${ref.lineHeight==='normal'?'normal':ref.lineHeight+'px'};white-space:pre}</style><div id="text">${escapedText}</div>`;
fs.writeFileSync(path.join(out,'reference.html'),html);
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');const rows=[];
try{
 const page=await browser.newPage({deviceScaleFactor:1,colorScheme:'light',locale:'en-US'});
 for(const f of frames){
  const prefix=path.join(out,`frame-${f.frame}`);await page.setViewportSize({width:f.width,height:f.height});await page.setContent(html);await page.locator('#text').evaluate((e,text)=>{e.textContent=text},ref.text);
  const metrics=await page.evaluate(async ref=>{await document.fonts.load(`${ref.fontSize}px Probe`,ref.text);await document.fonts.ready;const font=[...document.fonts][0];if(font.status!=='loaded')throw Error('font did not load');const e=document.getElementById('text'),b=e.getBoundingClientRect(),s=getComputedStyle(e);const c=document.createElement('canvas').getContext('2d');c.font=`${ref.fontSize}px Probe`;const t=c.measureText(ref.text);return{box:{x:b.x,y:b.y,width:b.width,height:b.height},lineHeight:s.lineHeight,advance:t.width,ascent:t.actualBoundingBoxAscent,descent:t.actualBoundingBoxDescent,fontAscent:t.fontBoundingBoxAscent,fontDescent:t.fontBoundingBoxDescent};},ref);
  await page.screenshot({path:prefix+'.chrome.png'});
  const command=[renderer,'--stream',path.join(observed,f.stream),'--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic','--frame',String(f.frame)];run(command,prefix+'.native.log');
  const expected=PNG.sync.read(fs.readFileSync(prefix+'.chrome.png')),actual=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));
  const comparison=comparePixels(expected,actual,{text:metrics.box},true),ink=compareInkPresence(expected,actual,{x:0,y:0,width:f.width,height:f.height},[255,255,255]);
  fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(comparison.diff));
  rows.push({...f,prefix,browserMetrics:metrics,pixelMetrics:comparison.metrics,pixelFailures:comparison.failures,ink,command,streamSha256:hash(path.join(observed,f.stream)),chromeSha256:hash(prefix+'.chrome.png'),nativeSha256:hash(prefix+'.native.png')});
 }
}finally{await browser.close();}
for(const b of bindings)assert.equal(hash(b.path),b.sha256,'input changed during experiment');
const receipt={scope:'Ordinary embedded-font text experiment; no public compiler text admission or native geometry qualification. Reference JSON describes independent CSS expectations, not assumed native mappings.',browser:'153.0.8010.12',reference:ref,bindings,rows,anyNativeInk:rows.every(r=>r.ink.actualInk>0),pixelPass:rows.filter(r=>!r.pixelFailures.length&&r.ink.passed).length};
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
fs.writeFileSync(path.join(out,'gallery.html'),`<!doctype html><title>Ordinary text / Chrome</title><style>body{font:16px system-ui}section{margin:20px}img{border:1px solid #ddd;max-width:47%}</style><h1>Chrome / ordinary Rive text</h1>${rows.map(r=>`<section><h2>Frame ${r.frame}</h2><p>${r.pixelFailures.join(', ')}; ink aligned: ${r.ink.passed}</p><img src="frame-${r.frame}.chrome.png"><img src="frame-${r.frame}.native.png"></section>`).join('')}`);
console.log(JSON.stringify({nativeInk:receipt.anyNativeInk,pixelPass:receipt.pixelPass,frames:rows.length,first:rows[0].ink,browser:rows[0].browserMetrics}));
if(receipt.pixelPass!==rows.length)process.exitCode=1;
