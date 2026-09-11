// L06: Chromium computes all expected bounds independently of the compiler.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
const here=path.dirname(fileURLToPath(import.meta.url));
const reset=fs.readFileSync(path.join(here,'../src/reset.css'),'utf8');
const out=path.resolve(process.argv[2]||'output/playwright/html-to-riv/wrap-reverse-oracle.json');
const browser=await chromium.launch();
const cases=[];
try {
 const page=await browser.newPage();
 for(const direction of ['row','row-reverse','column','column-reverse']) {
  for(const alignment of ['flex-start','center','flex-end','stretch','space-between','space-around','space-evenly']) {
   for(const mode of ['mixed','overflow','single-item','self-overrides']) {
    const row=direction.startsWith('row'),overflow=mode==='overflow';
    const html='<div id="root">'+(mode==='single-item'?['a']:['a','b','c','d']).map(id=>`<div id="${id}"></div>`).join('')+'</div>';
    const itemAlignment={mixed:'flex-start',overflow:'flex-end','single-item':'center','self-overrides':'stretch'}[mode];
    const css=`#root{width:100%;${overflow&&!row?'max-width:80px;':''}height:${overflow&&row?50:160}px;padding:10px;gap:10px;flex-direction:${direction};flex-wrap:wrap-reverse;align-content:${alignment};align-items:${itemAlignment};justify-content:space-between}#root>div{${row?'width:90px;height:20px':'height:60px;width:20px'}}#root>#b{${row?'height':'width'}:35px}#root>#c{${row?'height':'width'}:50px}`+(mode==='self-overrides'?`#root>#b{align-self:flex-start}#root>#c{align-self:center}#root>#d{${row?'height':'width'}:auto}`:'');
    const viewports=[];
    for(const width of [240,390,768]) {
     await page.setViewportSize({width,height:320});
     await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
     const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(el=>{
      const r=el.getBoundingClientRect();return [el.id,{x:r.x,y:r.y,width:r.width,height:r.height}];
     })));
     if(mode!=='single-item') {
      const cross=row?'height':'width',main=row?'width':'height';
      for(const [id,size] of [['a',20],['b',35],['c',50]]) {
       if(boxes[id][cross]!==size||boxes[id][main]!== (row?90:60)) throw new Error(`Invalid mixed-size fixture: ${direction} ${alignment} ${mode} ${id}`);
      }
     }
     viewports.push({width,boxes});
    }
    cases.push({direction,alignment,mode,html,css,viewports});
   }
  }
 }
 fs.mkdirSync(path.dirname(out),{recursive:true});
 fs.writeFileSync(out,JSON.stringify({browser:browser.version(),generator:'validation/wrap-reverse-oracle.mjs',purpose:'Independent browser reference; compiler/native qualification pending',cases},null,2)+'\n');
 console.log(JSON.stringify({out,scenes:cases.length,viewports:cases.length*3,browser:browser.version()}));
}finally{await browser.close();}
