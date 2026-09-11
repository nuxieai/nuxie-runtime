// Browser-only semantic reference for L04. Does not qualify compiler output.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
const here=path.dirname(fileURLToPath(import.meta.url));
const reset=fs.readFileSync(path.join(here,'../src/reset.css'),'utf8');
const out=path.resolve(process.argv[2]||'output/playwright/html-to-riv/align-content-oracle.json');
const browser=await chromium.launch();
const cases=[];
try {
 const page=await browser.newPage();
 for(const direction of ['row','row-reverse','column','column-reverse']) {
  for(const alignment of ['flex-start','center','flex-end','stretch','space-between','space-around']) {
   for(const mode of ['wrap','nowrap','single-item','overflow']) {
    const row=direction.startsWith('row');
    const html='<div id="root">'+(mode==='single-item'?['a']:['a','b','c','d']).map(id=>`<div id="${id}"></div>`).join('')+'</div>';
    const css=`#root{width:100%;height:${mode==='overflow'&&row?50:160}px;${mode==='overflow'&&!row?'max-width:80px;':''}padding:10px;gap:10px;flex-direction:${direction};flex-wrap:${mode==='nowrap'?'nowrap':'wrap'};align-items:center;align-content:${alignment}}#root>div{${row?'width:90px;height:20px':'height:60px;width:20px'}}#root>#b{${row?'height':'width'}:35px}#root>#c{${row?'height':'width'}:50px}`;
    const viewports=[];
    for(const width of [240,390,768]) {
     await page.setViewportSize({width,height:320});
     await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
     const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(el=>{
      const r=el.getBoundingClientRect();return [el.id,{x:r.x,y:r.y,width:r.width,height:r.height}];
     })));
     if(mode !== 'single-item') {
      const cross=row?'height':'width';
      if(boxes.a[cross]!==20 || boxes.b[cross]!==35 || boxes.c[cross]!==50) {
       throw new Error(`Mixed cross-size fixture invalid: ${direction} ${alignment} ${mode} at ${width}`);
      }
     }
     viewports.push({width,boxes});
    }
    cases.push({direction,alignment,mode,html,css,viewports});
   }
  }
 }
 fs.mkdirSync(path.dirname(out),{recursive:true});
 fs.writeFileSync(out,JSON.stringify({browser:browser.version(),purpose:'Independent Chromium reference; no compiler support claimed',cases},null,2)+'\n');
 console.log(JSON.stringify({out,scenes:cases.length,viewports:cases.length*3,browser:browser.version()}));
}finally{await browser.close();}
