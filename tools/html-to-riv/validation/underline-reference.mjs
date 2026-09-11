// Independent Chromium decoration measurements. This is not native qualification.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..'), dir=process.env.NUXIE_UNDERLINE_REFERENCE_DIR || root+'/output/playwright/html-to-riv/underline-reference';fs.mkdirSync(dir,{recursive:true});
const fonts=['Inter','OpenSans'].map(name=>({name,bytes:fs.readFileSync(root+'/tools/html-to-riv/tests/assets/'+name+'-Regular.ttf')}));
const b=await chromium.launch();const results=[];
try {
 const p=await b.newPage({viewport:{width:320,height:80},deviceScaleFactor:1});
 await p.setContent(`<style>${fonts.map(f=>`@font-face{font-family:${f.name};src:url(data:font/ttf;base64,${f.bytes.toString('base64')})}`).join('')}body{margin:0;background:white}p{margin:0;width:320px;height:80px;line-height:60px;color:transparent;text-decoration-line:underline;text-decoration-color:red}span{display:inline-block;width:0;height:0}</style><p id=a>agypqj M<span id=baseline></span></p>`);
 for(const font of fonts)await p.evaluate(name=>document.fonts.load(`24px ${name}`),font.name);
 for(const font of fonts)for(const size of [16,20,24,32])for(const thickness of ['auto','from-font','2px'])for(const skip of ['none','auto']) {
  const name=`${font.name}-${size}-${thickness}-${skip}`;
  const baseline=await p.evaluate(({font,size,thickness,skip})=>{const e=document.querySelector('#a');e.style.fontFamily=font;e.style.fontSize=size+'px';e.style.textDecorationThickness=thickness;e.style.textDecorationSkipInk=skip;return document.querySelector('#baseline').getBoundingClientRect().top;},{font:font.name,size,thickness,skip});
  const data=await p.screenshot();fs.writeFileSync(dir+'/'+name+'.png',data);const png=PNG.sync.read(data);const rows=[];
  for(let y=0;y<png.height;y++){let n=0,x0=Infinity,x1=-1,start=null;const spans=[];for(let x=0;x<=png.width;x++){const i=(y*png.width+x)*4;const ink=x<png.width&&png.data[i]>200&&png.data[i+1]<200&&png.data[i+2]<200;if(ink){n++;x0=Math.min(x0,x);x1=x;if(start===null)start=x;}else if(start!==null){spans.push([start,x]);start=null;}}if(n)rows.push({y,pixels:n,x0,x1,spans});}
  results.push({name,font:font.name,size,thickness,skip,baseline,rows});
 }
 fs.writeFileSync(dir+'/report.json',JSON.stringify({browser:b.version(),cases:results},null,2));const reference=JSON.parse(fs.readFileSync(new URL('./underline-reference.json',import.meta.url)));assert.deepEqual(results,reference.cases);console.log(`${results.length} underline metric/skip-ink references match Chromium ${b.version()}`);
}finally{await b.close();}
