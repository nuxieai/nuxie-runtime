// L08: Chromium reference for independently authored grow/shrink factors.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
const here=path.dirname(fileURLToPath(import.meta.url));
const reset=fs.readFileSync(path.join(here,'../src/reset.css'),'utf8');
const out=path.resolve('output/playwright/html-to-riv/independent-flex-oracle.json');
const browser=await chromium.launch();
try {
 const page=await browser.newPage();const cases=[];
 for(const direction of ['row','row-reverse','column','column-reverse']) {
  const row=direction.startsWith('row');
  for(const [grow,shrink] of [[0,1],[1,0],[2,1],[1,2],[0,2],[3,1]]) {
   for(const mode of ['fixed-basis','percentage-basis','explicit-auto-basis','constrained']) {
    const html='<div id="root"><div id="a"></div><div id="b"></div><div id="c"></div></div>';
    const basis=mode==='percentage-basis'?'30%':mode==='explicit-auto-basis'?'auto':'70px';
    const css=`#root{width:100%;height:${mode==='constrained'?180:280}px;padding:10px;gap:10px;flex-direction:${direction}}#root>div{width:${row?100:40}px;height:${row?40:100}px;flex:${grow} ${shrink} ${basis}}#b{flex-grow:${grow+1};flex-shrink:${shrink+1}}`+(mode==='constrained'?`#a{min-${row?'width':'height'}:80px}#b{max-${row?'width':'height'}:110px}`:'');
    const viewports=[];
    for(const width of [240,390,768]) {
     await page.setViewportSize({width,height:320});
     await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
     const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(el=>{const r=el.getBoundingClientRect();return[el.id,{x:r.x,y:r.y,width:r.width,height:r.height}]})));
     viewports.push({width,boxes});
    }
    cases.push({direction,grow,shrink,mode,html,css,viewports});
   }
  }
 }
 fs.mkdirSync(path.dirname(out),{recursive:true});
 fs.writeFileSync(out,JSON.stringify({browser:browser.version(),cases},null,2)+'\n');
 console.log(JSON.stringify({scenes:cases.length,viewports:cases.length*3,browser:browser.version()}));
}finally{await browser.close();}
