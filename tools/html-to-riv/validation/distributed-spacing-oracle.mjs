// L05 Chromium-only reference; deliberately independent of compiler lowering.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
const here=path.dirname(fileURLToPath(import.meta.url));
const reset=fs.readFileSync(path.join(here,'../src/reset.css'),'utf8');
const out=path.resolve(process.argv[2]||'output/playwright/html-to-riv/distributed-spacing-oracle.json');
const browser=await chromium.launch();
const cases=[];
try {
 const page=await browser.newPage();
 for(const direction of ['row','row-reverse','column','column-reverse']) {
  for(const property of ['justify-content','align-content']) {
   for(const alignment of ['space-between','space-around','space-evenly']) {
    for(const mode of ['nowrap','wrap','single-item','overflow']) {
     const row=direction.startsWith('row');
     const main=property==='justify-content';
     const overflow=mode==='overflow';
     const height=overflow&&((main&&!row)||(!main&&row))?50:180;
     const maxWidth=overflow&&((main&&row)||(!main&&!row))?'max-width:80px;':'';
     const wrap=mode==='wrap'||(!main&&mode!=='nowrap');
     const html='<div id="root">'+(mode==='single-item'?['a']:['a','b','c','d']).map(id=>`<div id="${id}"></div>`).join('')+'</div>';
     const css=`#root{width:100%;${maxWidth}height:${height}px;padding:10px;gap:10px;flex-direction:${direction};flex-wrap:${wrap?'wrap':'nowrap'};align-items:center;align-content:flex-start;${property}:${alignment}}#root>div{${row?'width:60px;height:20px':'width:20px;height:60px'}}#root>#b{${row?'width:80px;height:35px':'width:35px;height:80px'}}#root>#c{${row?'width:40px;height:50px':'width:50px;height:40px'}}#root>#d{${row?'width:70px':'height:70px'}}`;
     const viewports=[];
     for(const width of [240,390,768]) {
      await page.setViewportSize({width,height:320});
      await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
      const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(el=>{
       const r=el.getBoundingClientRect();return [el.id,{x:r.x,y:r.y,width:r.width,height:r.height}];
      })));
      if(mode!=='single-item') {
       const cross=row?'height':'width',axis=row?'width':'height';
       if(boxes.a[cross]!==20||boxes.b[cross]!==35||boxes.c[cross]!==50||boxes.a[axis]!==60||boxes.b[axis]!==80||boxes.c[axis]!==40) throw new Error(`Invalid mixed-size reference: ${direction} ${property} ${alignment} ${mode}`);
      }
      viewports.push({width,boxes});
     }
     cases.push({direction,property,alignment,mode,html,css,viewports});
    }
   }
  }
 }
 fs.mkdirSync(path.dirname(out),{recursive:true});
 fs.writeFileSync(out,JSON.stringify({browser:browser.version(),purpose:'Independent browser reference; compiler/native qualification pending',cases},null,2)+'\n');
 console.log(JSON.stringify({out,scenes:cases.length,viewports:cases.length*3,browser:browser.version()}));
}finally{await browser.close();}
