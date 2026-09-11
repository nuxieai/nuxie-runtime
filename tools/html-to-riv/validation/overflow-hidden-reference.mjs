// Browser differential control for initial overflow position; not native qualification.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const dir=path.resolve(process.env.NUXIE_HTML_REVIEW_DIR??path.join(moduleDir,'../../output/playwright/html-to-riv/overflow-hidden-alignment'));
fs.mkdirSync(dir,{recursive:true});
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const browser=await chromium.launch();const cases=[];
try{
 const page=await browser.newPage();
 for(const direction of ['row','column'])for(const align of ['flex-start','center','flex-end'])for(const width of [240,390,768]){
  const name=`${direction}-${align}-${width}`;
  const html='<div id=clip><div id=child><div id=marker></div></div></div><div id=after></div>';
  const css=`#clip{width:80%;height:60px;margin:12px;padding:4px;border-radius:12px;flex-direction:${direction};justify-content:${align};background:#edf3fa}#child{width:900px;height:100px;background:#2969b8}#marker{width:30px;height:30px;background:#ed701d}#after{width:100%;height:10px;background:#169454}`;
  fs.writeFileSync(path.join(dir,name+'.json'),JSON.stringify({html,css,width,height:160}));
  await page.setViewportSize({width,height:160});const states={};
  for(const overflow of ['clip','hidden']){
   await page.setContent(`<style>${reset}${css}#clip{overflow:${overflow}}</style>${html}`);
   states[overflow]=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(n=>{const b=n.getBoundingClientRect();return[n.id,{x:b.x,y:b.y,width:b.width,height:b.height,scrollLeft:n.scrollLeft,scrollTop:n.scrollTop}];})));
   await page.screenshot({path:path.join(dir,`${name}-${overflow}.png`)});
  }
  const a=PNG.sync.read(fs.readFileSync(path.join(dir,name+'-clip.png'))),b=PNG.sync.read(fs.readFileSync(path.join(dir,name+'-hidden.png')));
  cases.push({name,states,identicalPixels:a.data.equals(b.data),identicalGeometry:JSON.stringify(states.clip)===JSON.stringify(states.hidden)});
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify({browser:browser.version(),cases},null,2));
 console.log(JSON.stringify({total:cases.length,identical:cases.filter(c=>c.identicalPixels&&c.identicalGeometry).length,differences:cases.filter(c=>!c.identicalPixels||!c.identicalGeometry)},null,2));
}finally{await browser.close();}
