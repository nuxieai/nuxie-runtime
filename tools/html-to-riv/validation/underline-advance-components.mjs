// Diagnose advance scaling separately from contextual pair positioning.
import {chromium} from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(moduleDir,'../..');
const fontPath=path.resolve(process.env.NUXIE_ADVANCE_FONT??path.join(moduleDir,'tests/assets/NuxieJapaneseFixture-Regular.otf'));
const size=Number(process.env.NUXIE_ADVANCE_SIZE??24);
if(!Number.isFinite(size)||size<=0)throw new Error('Positive font size required');
const texts=process.env.NUXIE_ADVANCE_TEXTS?JSON.parse(process.env.NUXIE_ADVANCE_TEXTS):['a','g','y','p','q','j',' ','ag','gy','yp','pq','qj','j ',' g','ga','ap','p ',' a','agypqj gap agypqj gap agypqj ga'];
const browser=await chromium.launch();
try{
 const page=await browser.newPage();
 await page.setContent(`<style>@font-face{font-family:Fixture;src:url(data:font/otf;base64,${fs.readFileSync(fontPath).toString('base64')})}p{font:${size}px Fixture}</style><p>agypqj</p>`);
 await page.evaluate(()=>document.fonts.ready);
 const chrome=await page.evaluate(({texts,size})=>{const c=document.createElement('canvas').getContext('2d');c.font=`${size}px Fixture`;c.fontKerning='normal';return texts.map(text=>({text,width:c.measureText(text).width}));},{texts,size});
 const results=chrome.map(c=>{
  const native=JSON.parse(execFileSync(path.join(root,'target/debug/examples/outline_probe'),[fontPath,c.text,String(size),'css'],{maxBuffer:8*1024*1024}));
  const width=native.reduce((sum,g)=>sum+g.advance,0);
  return {...c,nativeWidth:width,nativeFloatWidth:Math.fround(width),delta:width-c.width,deltaFixedUnits:(width-c.width)*65536,glyphs:native.map(({scalar,glyph,advance})=>({scalar,glyph,advance}))};
 });
 const singles=new Map(results.filter(r=>[...r.text].length===1).map(r=>[r.text,r.deltaFixedUnits]));
 const full=results.at(-1);
 const predictedFixedUnits=[...full.text].reduce((sum,c)=>sum+(singles.get(c)??NaN),0);
 const predictedWidth=Math.fround(full.nativeWidth-predictedFixedUnits/65536);
 const decomposition={predictedFixedUnits,actualFixedUnits:full.deltaFixedUnits,predictedWidth,explained:predictedWidth===full.width};
 console.log(JSON.stringify({browser:browser.version(),fontPath,size,results,decomposition},null,2));
 if(process.argv.includes('--check')&&results.some(r=>r.nativeFloatWidth!==r.width))process.exitCode=1;
}finally{await browser.close();}
