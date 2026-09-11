// Browser-only L03 semantic reference; does not stand in for compiler/native QA.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
const directory=path.dirname(fileURLToPath(import.meta.url));
const reset=fs.readFileSync(path.join(directory,'../src/reset.css'),'utf8');
const out=path.resolve(process.argv[2]||'output/playwright/html-to-riv/align-self-oracle.json');
const browser=await chromium.launch();
const cases=[];
try {
 const page=await browser.newPage();
 for(const parentAlignment of ['flex-end','stretch']) {
 for(const direction of ['row','column','row-reverse','column-reverse']) {
  for(const alignment of ['auto','flex-start','center','flex-end','stretch','baseline']) {
   for(const sizing of ['fixed','auto','bounded']) {
    const row=direction.startsWith('row');
    const cross=row?'height':'width',main=row?'width':'height';
    const html='<div id="root"><div id="a"><div id="content"></div></div><div id="b"></div></div>';
    const css=`#root{width:100%;height:120px;flex-direction:${direction};align-items:${parentAlignment};padding:10px;gap:8px}#a{${main}:40px;align-self:${alignment};${sizing==='fixed'?`${cross}:30px;`:''}${sizing==='bounded'?`min-${cross}:25px;max-${cross}:60px;`:''}}#content{width:20px;height:20px}#b{width:30px;height:30px}`;
    for(const width of [240,390,768]) {
     await page.setViewportSize({width,height:320});
     await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
     const boxes=await page.evaluate(()=>Object.fromEntries(['root','a','b','content'].map(id=>{
      const element=document.getElementById(id),rect=element.getBoundingClientRect(),style=getComputedStyle(element);
      return [id,{x:rect.x,y:rect.y,width:rect.width,height:rect.height,alignSelf:style.alignSelf}];
     })));
     cases.push({parentAlignment,direction,alignment,sizing,width,html,css,boxes});
    }
   }
  }
 }
 }
 fs.mkdirSync(path.dirname(out),{recursive:true});
 fs.writeFileSync(out,JSON.stringify({browser:browser.version(),purpose:'Browser-only semantic reference, not compiler qualification',cases},null,2)+'\n');
 if(process.argv[3]) {
  const groups=new Map();
  for(const c of cases) {
   const key=JSON.stringify([c.parentAlignment,c.direction,c.alignment,c.sizing]);
   if(!groups.has(key)) groups.set(key,{parentAlignment:c.parentAlignment,direction:c.direction,alignment:c.alignment,sizing:c.sizing,html:c.html,css:c.css,viewports:[]});
   groups.get(key).viewports.push({width:c.width,boxes:c.boxes});
  }
  const fixture=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(fixture),{recursive:true});
  fs.writeFileSync(fixture,JSON.stringify({browser:browser.version(),generator:'validation/align-self-oracle.mjs',cases:[...groups.values()]})+'\n');
 }
 console.log(JSON.stringify({path:out,cases:cases.length,browser:browser.version()}));
}finally{await browser.close();}
