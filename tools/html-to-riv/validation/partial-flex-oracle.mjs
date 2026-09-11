// L09: independent Chromium references for partial grow/shrink distribution.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
const here=path.dirname(fileURLToPath(import.meta.url));
const reset=fs.readFileSync(path.join(here,'../src/reset.css'),'utf8');
const out=path.resolve('output/playwright/html-to-riv/partial-flex-oracle.json');
const browser=await chromium.launch();
try {
 const page=await browser.newPage();const cases=[];
 for(const direction of ['row','row-reverse','column','column-reverse']) {
  const row=direction.startsWith('row');
  for(const [grow,shrink] of [[.1,.1],[.25,.25],[.25,.5],[.5,.25],[.25,1],[1,.25]]) {
   for(const gap of [0,8]) {
    for(const mode of ['grow','shrink','freeze','shrink-freeze','mixed']) {
     const html='<div id="root"><div id="a"></div><div id="b"></div><div id="c"></div></div>';
     const basis=['shrink','shrink-freeze','mixed'].includes(mode)?180:20;
     let css=`#root{width:100%;height:280px;padding:10px;gap:${gap}px;flex-direction:${direction}}#root>div{width:${row?basis:40}px;height:${row?40:basis}px;flex:${grow} ${shrink} ${basis}px}`+(mode==='freeze'?`#a{max-${row?'width':'height'}:24px}#b{max-${row?'width':'height'}:50px}`:'');
     if(mode==='shrink-freeze') css+=`#a{min-${row?'width':'height'}:160px}#b{min-${row?'width':'height'}:120px}`;
     if(mode==='mixed') css+=`#root>#b{${row?'width':'height'}:90px;flex:${grow*2} ${shrink*.5} 90px}#root>#c{${row?'width':'height'}:60px;flex-basis:60px}`;
     const viewports=[];
     for(const width of [240,390,768]) {
      await page.setViewportSize({width,height:320});
      await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
      if(mode==='mixed') {
       const actual=await page.evaluate(()=>['a','b','c'].map(id=>{const s=getComputedStyle(document.getElementById(id));return [parseFloat(s.flexBasis),parseFloat(s.flexGrow),parseFloat(s.flexShrink)]}));
       const expected=[[180,grow,shrink],[90,grow*2,shrink*.5],[60,grow,shrink]];
       if(JSON.stringify(actual)!==JSON.stringify(expected)) throw new Error(`Mixed-factor fixture lost its intended cascade: ${JSON.stringify(actual)}`);
      }
      const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(el=>{const r=el.getBoundingClientRect();return[el.id,{x:r.x,y:r.y,width:r.width,height:r.height}]})));
      viewports.push({width,boxes});
     }
     cases.push({direction,grow,shrink,gap,mode,html,css,viewports});
    }
   }
  }
 }
 fs.mkdirSync(path.dirname(out),{recursive:true});
 fs.writeFileSync(out,JSON.stringify({browser:browser.version(),cases},null,2)+'\n');
 console.log(JSON.stringify({scenes:cases.length,viewports:cases.length*3,browser:browser.version()}));
}finally{await browser.close();}
