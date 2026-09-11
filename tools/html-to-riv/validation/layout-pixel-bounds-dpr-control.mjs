// DPR is a host raster scale; native layout remains in CSS pixels.
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createCompiler} from '../js/index.mjs';
import {comparePixels} from './pixels.mjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(moduleDir,'../..');
const dir=path.resolve(process.env.NUXIE_HTML_REVIEW_DIR??path.join(root,'output/playwright/html-to-riv/layout-pixel-bounds-dpr'));
fs.mkdirSync(dir,{recursive:true});
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const compiler=await createCompiler(fs.readFileSync(path.join(moduleDir,'dist/html-to-riv.wasm')));
const run=(bin,args,dpr=1)=>execFileSync(path.join(root,'target/debug',bin),args,{env:{...process.env,NUXIE_NATIVE_GLYPHS:'1',NUXIE_DEVICE_SCALE:String(dpr)},stdio:'pipe'});
const focus=process.argv.includes('--focus');
const skips=['fractional','rounded','clipped'];
const dprs=focus?[1]:[1,2,3];
const widths=focus?[768]:[240,390,768];
const browser=await chromium.launch(),results=[];
try {
 for(const skip of skips){
  const prefix=path.join(dir,skip);
  const css={
   fractional:'#root{width:100%;height:80px;margin-top:.5px;background:coral}',
   rounded:'#root{width:100%;padding:3.75px;background:#eef}#a{width:80%;height:60px;border-radius:7.5px;background:#369}',
   clipped:'#root{width:100%;padding:3.75px;background:#eef}#a{width:80%;height:40.5px;overflow:hidden;border-radius:7.5px;background:#369}#b{width:120%;height:60px;background:coral}'
  }[skip];
  const request={html:skip==='fractional'?'<div id="root"></div>':'<div id="root"><div id="a"><div id="b"></div></div></div>',css,width:390,height:200};
  fs.writeFileSync(prefix+'.json',JSON.stringify(request));run('html-to-riv',[prefix+'.json',prefix+'.riv']);
  const wasm=compiler.compile({languageVersion:'nuxie-html-v1',...request});assert.equal(wasm.ok,true);
  assert.deepEqual(Buffer.from(wasm.riv),fs.readFileSync(prefix+'.riv'));
  assert.deepEqual(wasm.sourceMap,JSON.parse(fs.readFileSync(prefix+'.map.json')));
  assert.deepEqual(wasm.runtimeRequirements,JSON.parse(fs.readFileSync(prefix+'.requirements.json')));
  for(const dpr of dprs){
   const page=await browser.newPage({viewport:{width:390,height:200},deviceScaleFactor:dpr});
   for(const width of widths){
    const frame=prefix+'-'+dpr+'-'+width;
    run('examples/probe',[prefix+'.riv',prefix+'.map.json',String(width),'200',frame],dpr);
    run('renderer-replay',['--stream',frame+'.stream','--output',frame+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']);
    await page.setViewportSize({width,height:200});
    await page.setContent(`<style>${reset}${request.css}</style>${request.html}`);
    await page.screenshot({path:frame+'.browser.png'});
    const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(n=>{const b=n.getBoundingClientRect();return[n.id,{x:b.x,y:b.y,width:b.width,height:b.height}];})));
    const native=JSON.parse(fs.readFileSync(frame+'.bounds.json'));
    const geometry=[];
    for(const [id,b] of Object.entries(boxes))for(const key of ['x','y','width','height'])if(Math.abs(native[id][key]-b[key])>.1)geometry.push('geometry: '+id+'.'+key);
    const deviceBoxes=Object.fromEntries(Object.entries(boxes).map(([id,b])=>[id,Object.fromEntries(Object.entries(b).map(([k,v])=>[k,v*dpr]))]));
    const reference=PNG.sync.read(fs.readFileSync(frame+'.browser.png')),actual=PNG.sync.read(fs.readFileSync(frame+'.native.png'));
    assert.equal(actual.width,width*dpr);assert.equal(actual.height,200*dpr);
    const {metrics,failures,diff}=comparePixels(reference,actual,deviceBoxes,false);
    fs.writeFileSync(frame+'.diff.png',PNG.sync.write(diff));
    results.push({skip,dpr,width,boxes,metrics,failures:[...geometry,...failures]});
   }
   await page.close();
  }
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify({browser:browser.version(),cases:results},null,2));
 const page=await browser.newPage({viewport:{width:2400,height:800}});
 for(const skip of skips)for(const dpr of dprs){
  const rows=results.filter(r=>r.skip===skip&&r.dpr===dpr);
  const html='<style>body{font:14px system-ui;background:#ddd}section{padding:8px;margin:8px;background:white;width:max-content}article{display:flex;gap:12px}img{display:block;max-width:768px}</style>'+rows.map(r=>{
   const prefix=path.join(dir,`${r.skip}-${r.dpr}-${r.width}`);
   return `<section><b>${r.skip}, DPR ${r.dpr}, ${r.width}px: ${r.failures.join(', ')||'pass'}</b><article>`+['browser','native','diff'].map(k=>`<div>${k}<img src="data:image/png;base64,${fs.readFileSync(prefix+'.'+k+'.png').toString('base64')}"></div>`).join('')+'</article></section>';
  }).join('');
  await page.setContent(html);await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  fs.writeFileSync(path.join(dir,`review-${skip}-${dpr}.html`),html);
  await page.screenshot({path:path.join(dir,`review-${skip}-${dpr}.png`),fullPage:true});
 }
 console.log(JSON.stringify({passed:results.filter(r=>!r.failures.length).length,total:results.length,failures:results.filter(r=>r.failures.length).map(({skip,dpr,width,failures})=>({skip,dpr,width,failures}))},null,2));
 if(results.some(r=>r.failures.length))process.exitCode=1;
}finally{await browser.close();}
