// DPR is a host raster scale; native layout remains in CSS pixels.
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createCompiler} from '../js/index.mjs';
import {comparePixels,compareRedDecoration} from './pixels.mjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(moduleDir,'../..');
const dir=path.resolve(process.env.NUXIE_HTML_REVIEW_DIR??path.join(root,'output/playwright/html-to-riv/strikethrough-dpr'));
fs.mkdirSync(dir,{recursive:true});
const font=fs.readFileSync(path.join(moduleDir,'tests/assets/NuxieJapaneseFixture-Regular.otf'));
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const compiler=await createCompiler(fs.readFileSync(path.join(moduleDir,'dist/html-to-riv.wasm')));
const run=(bin,args,dpr=1)=>execFileSync(path.join(root,'target/debug',bin),args,{env:{...process.env,NUXIE_NATIVE_GLYPHS:'1',NUXIE_DEVICE_SCALE:String(dpr)},stdio:'pipe'});
const focus=process.argv.includes('--focus');
const fontSize=Number(process.env.NUXIE_STRIKETHROUGH_SIZE??24);
assert.ok(Number.isFinite(fontSize)&&fontSize>0);
const thicknesses=focus?['2px']:['auto','from-font','2px'];
const dprs=focus?[1]:[1,2,3];
const widths=focus?[768]:[240,390,768];
const browser=await chromium.launch(),results=[];
try {
 for(const thickness of thicknesses){
  const prefix=path.join(dir,thickness);
  const request={html:'<div id="root"><p id="a">カタカナ がぎぐげご ／＿ agypqj</p><p id="b">agypqj gap agypqj gap agypqj gap</p></div>',css:`#root{width:100%;padding:8.75px 8px;font-family:Fixture}p{font-size:24px;line-height:40px;text-decoration:line-through solid red ${thickness}}`,width:390,height:200,assets:{font:{kind:'font',family:'Fixture',weight:400,bytes:[...font]}}};
  if(process.argv.includes('--transparent-glyphs'))request.css+='p{color:transparent}';
  request.css+=`p{font-size:${fontSize}px}`;
  if(process.argv.includes('--without-decoration'))request.css+='p{text-decoration:none}';
  if(process.argv.includes('--latin-only'))request.html='<div id="root"><p id="b">agypqj gap agypqj gap agypqj gap</p></div>';
  if(process.env.NUXIE_DPR_TEXT!==undefined){const text=process.env.NUXIE_DPR_TEXT.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;');request.html=`<div id="root"><p id="b">${text}</p></div>`;}
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
    await page.setContent(`<style>@font-face{font-family:Fixture;src:url(data:font/otf;base64,${font.toString('base64')})}${reset}${request.css}</style>${request.html}`);
    await page.evaluate(()=>document.fonts.ready);
    assert.equal(await page.evaluate(()=>[...document.fonts].every(f=>f.status==='loaded')),true);
    await page.screenshot({path:frame+'.browser.png'});
    const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(n=>{const b=n.getBoundingClientRect();return[n.id,{x:b.x,y:b.y,width:b.width,height:b.height}];})));
    const native=JSON.parse(fs.readFileSync(frame+'.bounds.json'));
    const geometry=[];
    for(const [id,b] of Object.entries(boxes))for(const key of ['x','y','width','height'])if(Math.abs(native[id][key]-b[key])>.1)geometry.push('geometry: '+id+'.'+key);
    const deviceBoxes=Object.fromEntries(Object.entries(boxes).map(([id,b])=>[id,Object.fromEntries(Object.entries(b).map(([k,v])=>[k,v*dpr]))]));
    const reference=PNG.sync.read(fs.readFileSync(frame+'.browser.png')),actual=PNG.sync.read(fs.readFileSync(frame+'.native.png'));
    assert.equal(actual.width,width*dpr);assert.equal(actual.height,200*dpr);
    const {metrics,failures,diff}=comparePixels(reference,actual,deviceBoxes,true);
    metrics.redDecoration=compareRedDecoration(reference,actual,{x:0,y:0,width:actual.width,height:actual.height});
    if(process.argv.includes('--without-decoration')){
     assert.equal(metrics.redDecoration.referencePixels,0);
     assert.equal(metrics.redDecoration.actualPixels,0);
    }else if(!metrics.redDecoration.passed)failures.push('red decoration support');
    fs.writeFileSync(frame+'.diff.png',PNG.sync.write(diff));
    results.push({thickness,dpr,width,boxes,metrics,failures:[...geometry,...failures]});
   }
   await page.close();
  }
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify({browser:browser.version(),cases:results},null,2));
 const page=await browser.newPage({viewport:{width:2400,height:800}});
 for(const thickness of thicknesses)for(const dpr of dprs){
  const rows=results.filter(r=>r.thickness===thickness&&r.dpr===dpr);
  const html='<style>body{font:14px system-ui;background:#ddd}section{padding:8.75px 8px;margin:8px;background:white;width:max-content}article{display:flex;gap:12px}img{display:block;max-width:768px}</style>'+rows.map(r=>{
   const prefix=path.join(dir,`${r.thickness}-${r.dpr}-${r.width}`);
   return `<section><b>${r.thickness}, DPR ${r.dpr}, ${r.width}px: ${r.failures.join(', ')||'pass'}</b><article>`+['browser','native','diff'].map(k=>`<div>${k}<img src="data:image/png;base64,${fs.readFileSync(prefix+'.'+k+'.png').toString('base64')}"></div>`).join('')+'</article></section>';
  }).join('');
  await page.setContent(html);await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  fs.writeFileSync(path.join(dir,`review-${thickness}-${dpr}.html`),html);
  await page.screenshot({path:path.join(dir,`review-${thickness}-${dpr}.png`),fullPage:true});
 }
 console.log(JSON.stringify({passed:results.filter(r=>!r.failures.length).length,total:results.length,failures:results.filter(r=>r.failures.length).map(({thickness,dpr,width,failures})=>({thickness,dpr,width,failures}))},null,2));
 if(results.some(r=>r.failures.length))process.exitCode=1;
}finally{await browser.close();}
