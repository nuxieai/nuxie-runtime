// Chrome-only semantic control; this does not qualify compiler or native output.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';

const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const dir=path.resolve(process.env.NUXIE_HTML_REVIEW_DIR??path.join(moduleDir,'../../output/playwright/html-to-riv/ellipsis-narrow-reference'));
fs.mkdirSync(dir,{recursive:true});
const font=fs.readFileSync(path.join(moduleDir,'tests/assets/Inter-Regular.ttf')).toString('base64');
const fixtures=[['latin','A long title'],['ligature','ffi long title'],['combining','A\u0301 long title']];
const widths=[0,8,16,20,24,30,40,48];
const browser=await chromium.launch();
const results=[],cards=[];
try {
 const page=await browser.newPage({viewport:{width:160,height:64},deviceScaleFactor:1});
 for(const [name,text] of fixtures)for(const width of widths){
  const first=new Intl.Segmenter('en',{granularity:'grapheme'}).segment(text)[Symbol.iterator]().next().value.segment;
  await page.setContent(`<style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${font})}#measure{font:24px/40px Inter;white-space:nowrap}</style><span id=measure>${text}</span>`);
  await page.evaluate(()=>document.fonts.ready);
  const firstRangeWidth=await page.evaluate(length=>{const range=document.createRange();range.setStart(document.querySelector('#measure').firstChild,0);range.setEnd(document.querySelector('#measure').firstChild,length);return range.getBoundingClientRect().width;},first.length);
  const separateMarker=prefix=>`<span style="display:inline-block">${prefix}</span><span style="display:inline-block">…</span>`;
  const rangeMarker=`<span style="display:inline-block;width:${firstRangeWidth}px">${first}</span><span style="display:inline-block">…</span>`;
  const variants={ellipsis:text,clip:text,first,firstMarker:first+'…',separateMarker:separateMarker(first),rangeMarker,...(name==='ligature'?{twoMarker:separateMarker('ff'),threeMarker:separateMarker('ffi')}:{}),marker:'…',dots:'...'};
  const images={};
  for(const [kind,content] of Object.entries(variants)){
   const html=`<style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${font})}body{margin:0;background:white}#box{margin:12px;width:${width}px;height:40px;overflow:hidden;white-space:nowrap;font:24px/40px Inter;text-overflow:${kind==='ellipsis'?'ellipsis':'clip'}}</style><div id=box>${content}</div>`;
   await page.setContent(html);await page.evaluate(()=>document.fonts.ready);
   assert.equal(await page.evaluate(()=>document.fonts.check('24px Inter')),true);
   const stem=path.join(dir,`${name}-${width}-${kind}`);
   fs.writeFileSync(stem+'.html',html);
   images[kind]=await page.screenshot({path:stem+'.png'});
  }
  const actual=PNG.sync.read(images.ellipsis);
  const identical=Object.entries(images).filter(([kind,bytes])=>kind!=='ellipsis'&&actual.data.equals(PNG.sync.read(bytes).data)).map(([kind])=>kind);
  // The existing Latin reproducer clips A; ligature/combining cases are observations.
  if(width===8&&name==='latin'){assert.ok(identical.includes('clip'));assert.ok(identical.includes('first'));assert.ok(!identical.includes('marker'));assert.ok(!identical.includes('dots'));}
  if(width===8&&name==='ligature'){assert.ok(identical.includes('first'));assert.ok(!identical.includes('clip'));}
  if(name==='ligature'&&[16,20,24,30].includes(width)){
   assert.ok(identical.includes('rangeMarker'));
   assert.ok(!identical.includes('separateMarker'));
  }
  if(name==='ligature'&&width===40)assert.ok(identical.includes('twoMarker'));
  if(name==='ligature'&&width===48)assert.ok(identical.includes('threeMarker'));
  results.push({name,text,width,firstRangeWidth,identical});
  cards.push(`<section><b>${name} ${width}px; exact matches: ${identical.join(', ')||'none'}</b><article>${Object.entries(images).map(([kind,bytes])=>`<div>${kind}<img src="data:image/png;base64,${bytes.toString('base64')}"></div>`).join('')}</article></section>`);
 }
 const html='<style>body{font:13px system-ui;background:#ddd}section{background:white;padding:4px;margin:4px}article{display:flex}img{display:block}</style>'+cards.join('');
 fs.writeFileSync(path.join(dir,'gallery.html'),html);
 await page.setViewportSize({width:1650,height:800});await page.setContent(html);await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
 await page.screenshot({path:path.join(dir,'gallery.png'),fullPage:true});
 for(let index=0;index<fixtures.length;index++){
  await page.setContent('<style>body{font:13px system-ui;background:#ddd}section{background:white;padding:4px;margin:4px}article{display:flex}img{display:block}</style>'+cards.slice(index*widths.length,(index+1)*widths.length).join(''));
  await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  await page.screenshot({path:path.join(dir,`review-${fixtures[index][0]}.png`),fullPage:true});
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify({browser:browser.version(),scope:'Chrome-only; no native or compiler qualification',cases:results},null,2));
 console.log(JSON.stringify(results));
} finally {await browser.close();}
