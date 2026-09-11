// Chrome-only phase measurements; each candidate is diagnostic, not a native gate.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const dir=process.env.NUXIE_STRIKETHROUGH_PHASE_DIR || root+'/output/playwright/html-to-riv/strikethrough-phase';
fs.mkdirSync(dir,{recursive:true});
const browser=await chromium.launch();
const results=[];
try {
 assert.equal(browser.version(),'153.0.8010.12');
 const page=await browser.newPage({viewport:{width:100,height:70},deviceScaleFactor:1});
 for(const font of ['Inter','OpenSans']) {
  const bytes=fs.readFileSync(`${root}/tools/html-to-riv/tests/assets/${font}-Regular.ttf`);
  const tables={};for(let i=0;i<bytes.readUInt16BE(4);i++){const offset=12+i*16;tables[bytes.toString('ascii',offset,offset+4)]=bytes.readUInt32BE(offset+8);}
  const normalizedAscent=bytes.readInt16BE(tables.hhea+4)/bytes.readUInt16BE(tables.head+18);
  await page.setContent(`<style>@font-face{font-family:fixture;src:url(data:font/ttf;base64,${bytes.toString('base64')})}body{margin:0;background:white}p{margin:0;font-family:fixture;color:transparent;text-decoration:line-through red;text-decoration-skip-ink:none}span{display:inline-block;width:0;height:0}</style><div id=wrapper><p id=text>M<span class=baseline></span><br>M<span class=baseline></span></p></div>`);
  await page.evaluate(()=>document.fonts.load('20px fixture'));
  for(const size of [20,24])for(const lineHeight of [30,30.5])for(const padding of [0,.125,.25,.375,.5,.625,.75,.875])for(const thickness of [1,2,3]) {
   const name=`${font}-${size}-${lineHeight}-${padding}-${thickness}`;
   const baselines=await page.evaluate(({size,lineHeight,padding,thickness})=>{
    const p=document.querySelector('#text');p.style.fontSize=size+'px';p.style.lineHeight=lineHeight+'px';p.style.textDecorationThickness=thickness+'px';document.querySelector('#wrapper').style.paddingTop=padding+'px';return [...document.querySelectorAll('.baseline')].map(e=>e.getBoundingClientRect().top);
   },{size,lineHeight,padding,thickness});
   const bytes=await page.screenshot();fs.writeFileSync(`${dir}/${name}.png`,bytes);
   const image=PNG.sync.read(bytes);const rows=[];
   for(let y=0;y<image.height;y++){const i=(y*image.width+5)*4;if(image.data[i+1]<255)rows.push({y,green:image.data[i+1]});}
   for(let lineIndex=0;lineIndex<baselines.length;lineIndex++) {
   const baseline=baselines[lineIndex];
   const lineTop=padding+lineIndex*lineHeight;
   const lineRows=rows.filter(row=>Math.abs(row.y-(baseline-normalizedAscent*size/3))<5);
   const offset=-normalizedAscent*size/3-thickness/2;
   const round=v=>Math.floor(v+.5);
   const predictions={roundGlobal:round(baseline+offset),roundBaselineThenOffset:round(baseline)+round(offset),roundLineThenStripe:round(lineTop)+round(baseline-lineTop+offset),roundedAscentAndLine:round(lineTop)+round(baseline-lineTop-round(normalizedAscent*size)/3-thickness/2)};
   results.push({name:name+'-line'+lineIndex,font,size,lineHeight,padding,thickness,baseline,lineTop,offset,rows:lineRows,predictions});
   }
  }
 }
 const failures=Object.fromEntries(Object.keys(results[0].predictions).map(key=>[key,results.filter(c=>c.rows[0]?.y!==c.predictions[key]).map(c=>c.name)]));
 assert.equal(failures.roundedAscentAndLine.length,0,'candidate placement must match every measured stripe top');
 const report={browser:browser.version(),cases:results,predictionFailures:failures};fs.writeFileSync(`${dir}/report.json`,JSON.stringify(report,null,2));
 const reference=new URL('./strikethrough-phase-reference.json',import.meta.url);
 if(process.argv.includes('--record'))fs.writeFileSync(reference,JSON.stringify(report,null,2));
 else assert.deepEqual(report,JSON.parse(fs.readFileSync(reference)));
 const selected=results.filter(c=>c.size===20&&c.lineHeight===30.5&&c.thickness===2&&[0,.25,.5,.75].includes(c.padding)&&c.name.endsWith('line0'));
 await page.setViewportSize({width:1280,height:620});
 await page.setContent('<style>body{font:14px system-ui;background:#ddd;display:flex;flex-wrap:wrap;gap:8px}section{background:white;padding:8px}img{width:290px;height:203px;image-rendering:pixelated}</style>'+selected.map(c=>`<section><b>${c.font}, padding ${c.padding}, line-height 30.5</b><br><img src="data:image/png;base64,${fs.readFileSync(dir+'/'+c.name.replace(/-line0$/,'')+'.png').toString('base64')}"></section>`).join(''));
 await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
 await page.screenshot({path:dir+'/review.png',fullPage:true});
 console.log(JSON.stringify({cases:results.length,predictionFailures:Object.fromEntries(Object.entries(failures).map(([k,v])=>[k,v.length]))}));
}finally{await browser.close();}
