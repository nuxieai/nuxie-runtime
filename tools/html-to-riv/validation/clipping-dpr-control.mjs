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
const dir=path.resolve(process.env.NUXIE_HTML_REVIEW_DIR??path.join(root,'output/playwright/html-to-riv/clipping-dpr'));
fs.mkdirSync(dir,{recursive:true});
const font=fs.readFileSync(path.join(moduleDir,'tests/assets/Inter-Regular.ttf'));
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const compiler=await createCompiler(fs.readFileSync(path.join(moduleDir,'dist/html-to-riv.wasm')));
const run=(bin,args,dpr=1)=>execFileSync(path.join(root,'target/debug',bin),args,{env:{...process.env,NUXIE_NATIVE_GLYPHS:'1',NUXIE_DEVICE_SCALE:String(dpr)},stdio:'pipe'});
const referenceHidden=process.argv.includes('--reference-hidden');
if(referenceHidden)assert.ok(process.env.NUXIE_HTML_REVIEW_DIR,'Hidden reference needs a separate output directory');
const unclipped=process.argv.includes('--unclipped');
const transparent=process.argv.includes('--transparent-glyphs');
const transparentClipped=process.argv.includes('--transparent-clipped-text');
if(unclipped||transparent||transparentClipped)assert.ok(process.env.NUXIE_HTML_REVIEW_DIR,'Diagnostic control needs a separate output directory');
const negativeControl=process.argv.includes('--negative-control');
assert.ok(!(unclipped&&negativeControl),'Unclipped and negative controls are distinct experiments');
if(negativeControl)assert.ok(process.env.NUXIE_HTML_REVIEW_DIR,'Negative control needs a separate output directory');
const fixtures=JSON.parse(fs.readFileSync(path.join(moduleDir,'validation/cases.json'))).filter(f=>['text-clip-fractional','text-clip-rounded-edge','text-clip-decorations'].includes(f.name)&&(!(negativeControl||unclipped||transparent||transparentClipped)||f.name==='text-clip-fractional'));
const dprs=[1,2,3],widths=[240,390,768];
const browser=await chromium.launch(),results=[];
try {
 for(const fixture of fixtures){
  const thickness=fixture.name;
  const prefix=path.join(dir,thickness);
  const request={html:fixture.html,css:fixture.css,width:390,height:200,assets:{font:{kind:'font',family:'Inter',weight:400,bytes:[...font]}}};
  if(unclipped)request.css+='#clip{overflow:visible}';
  if(transparentClipped)request.css+='#clip{color:transparent}';
  if(transparent)request.css+='#clip,#after{color:transparent}';
  const referenceCss=request.css+(referenceHidden?'#clip{overflow:hidden}':'');
  if(negativeControl)request.css+='#clip{overflow:visible}';
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
    await page.setContent(`<style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${font.toString('base64')})}${reset}${referenceCss}</style>${request.html}`);
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
    if(fixture.name==='text-clip-decorations'){
     metrics.redDecoration=compareRedDecoration(reference,actual,deviceBoxes.clip);
     if(!metrics.redDecoration.passed)failures.push('red decoration support');
    }
    // The gap below the clip is empty in these fixtures. Require it to stay
    // white outside a one-device-pixel edge neighborhood, independently of
    // broad image averages. The following sibling is excluded by its bounds.
    const clip=deviceBoxes.clip,after=deviceBoxes.after;
    let leaked=0,referenceInk=0;
    for(let y=Math.ceil(clip.y+clip.height)+1;y<Math.floor(after.y)-1;y++)
     for(let x=Math.max(0,Math.floor(clip.x));x<Math.min(actual.width,Math.ceil(clip.x+clip.width));x++){
      const i=(y*actual.width+x)*4;
      if(Math.min(...reference.data.subarray(i,i+3))<249)referenceInk++;
      if(Math.min(...actual.data.subarray(i,i+3))<249)leaked++;
     }
    if(!unclipped){
     assert.equal(referenceInk,0,'Leakage control requires a white reference gap');
     metrics.clipLeakagePixels=leaked;
     if(leaked)failures.push('clip leakage');
    }
    fs.writeFileSync(frame+'.diff.png',PNG.sync.write(diff));
    results.push({thickness,dpr,width,boxes,metrics,failures:[...geometry,...failures]});
   }
   await page.close();
  }
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify({browser:browser.version(),cases:results},null,2));
 const page=await browser.newPage({viewport:{width:2400,height:800}});
 for(const {name:thickness} of fixtures)for(const dpr of dprs){
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
