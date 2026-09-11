// Ellipsis qualification: diagnostic, checked manifest, and public CSS paths.
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
const dir=path.resolve(process.env.NUXIE_HTML_REVIEW_DIR??path.join(root,'output/playwright/html-to-riv/ellipsis-runtime'));
fs.mkdirSync(dir,{recursive:true});
const fontAsset=process.env.NUXIE_ELLIPSIS_FONT??'Inter-Regular.ttf';
const fontFamily=process.env.NUXIE_ELLIPSIS_FONT_FAMILY??'Inter';
assert.match(fontAsset,/^[A-Za-z0-9.-]+$/);
assert.match(fontFamily,/^[A-Za-z][A-Za-z0-9]*$/);
const font=fs.readFileSync(path.join(moduleDir,'tests/assets',fontAsset));
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const compiler=await createCompiler(fs.readFileSync(path.join(moduleDir,'dist/html-to-riv.wasm')));
const plainF=process.argv.includes('--plain-f-control');
const manifestControl=process.argv.includes('--manifest-control');
const publicCss=process.argv.includes('--public-css');
if(publicCss&&(!process.env.NUXIE_HTML_REVIEW_DIR||manifestControl))throw new Error('Public CSS control requires its own directory and no manifest rewrite');
if(manifestControl&&!process.env.NUXIE_HTML_REVIEW_DIR)throw new Error('Manifest control requires a separate review directory');
const fractionalOnly=process.argv.includes('--fractional-control');
const clipOnly=process.argv.includes('--clip-only');
if(clipOnly&&!process.env.NUXIE_HTML_REVIEW_DIR)throw new Error('Use a separate review directory for clipping controls');
const alignment=process.argv.includes('--alignment-control');
const vertical=process.argv.includes('--vertical-control');
const wordSpacing=process.argv.includes('--word-spacing-control');
const letterSpacing=process.argv.includes('--letter-spacing-control');
if(letterSpacing&&(!process.env.NUXIE_HTML_REVIEW_DIR||process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS!=='1'))throw new Error('Letter-spacing control requires CSS ellipsis experiment and a separate review directory');
const decoration=process.argv.includes('--decoration-control');
if(decoration&&(!process.env.NUXIE_HTML_REVIEW_DIR||process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS!=='1'))throw new Error('Decoration control requires CSS ellipsis experiment and a separate review directory');
if(wordSpacing&&(!process.env.NUXIE_HTML_REVIEW_DIR||process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS!=='1'))throw new Error('Word-spacing control requires CSS ellipsis experiment and a separate review directory');
if(vertical&&(!process.env.NUXIE_HTML_REVIEW_DIR||process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS!=='1'))throw new Error('Vertical control requires CSS ellipsis experiment and a separate review directory');
if(alignment&&(!process.env.NUXIE_HTML_REVIEW_DIR||process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS!=='1'))throw new Error('Alignment control requires CSS ellipsis experiment and a separate review directory');
if(plainF&&!process.env.NUXIE_HTML_REVIEW_DIR)throw new Error('Use a separate review directory for the plain-f control');
const run=(bin,args,dpr=1)=>execFileSync(path.join(root,'target/debug',bin),args,{env:{...process.env,NUXIE_NATIVE_GLYPHS:'1',NUXIE_EXPERIMENTAL_TEXT_ELLIPSIS:publicCss||manifestControl||plainF||clipOnly?'0':'1',NUXIE_EXPERIMENTAL_CSS_ELLIPSIS:publicCss||manifestControl||plainF||clipOnly?'0':process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS??'0',NUXIE_DEVICE_SCALE:String(dpr)},stdio:'pipe'});
const fixtures=(letterSpacing?[0,2,-1].flatMap(spacing=>[
 {name:'letter-'+spacing+'-long',text:'ffi long title with repeated words and more words',style:`letter-spacing:${spacing}px`},
 {name:'letter-'+spacing+'-narrow',text:'ffi long title',style:`width:40px;letter-spacing:${spacing}px`},
 {name:'letter-'+spacing+'-combining',text:'A\u0301 long title',style:`width:40px;letter-spacing:${spacing}px`}
]):decoration?['underline','line-through','underline line-through'].flatMap(line=>[
 {name:line.replaceAll(' ','-')+'-long',text:'ffi long title with agypqj repeated words and more words',style:`text-decoration-line:${line};text-decoration-color:red;text-decoration-thickness:2px`},
 {name:line.replaceAll(' ','-')+'-fits',text:'Short',style:`text-decoration-line:${line};text-decoration-color:red;text-decoration-thickness:2px`},
 {name:line.replaceAll(' ','-')+'-narrow',text:'ffi long title',style:`width:40px;text-decoration-line:${line};text-decoration-color:red;text-decoration-thickness:2px`}
]):wordSpacing?[0,4,-2].flatMap(spacing=>[
 {name:'spacing-'+spacing+'-long',text:'ffi long title with repeated words and more words',style:`word-spacing:${spacing}px`},
 {name:'spacing-'+spacing+'-narrow',text:'ffi long title with repeated words',style:`width:72px;word-spacing:${spacing}px`}
]):vertical?[8,20,39,80].map(height=>({name:'height-'+height,text:'A long title with agypqj and words that must disappear as the container becomes narrow',style:`height:${height}px`})) :alignment?['left','center','right'].flatMap(align=>[
 {name:align+'-long',text:'A long title with agypqj and words that must disappear as the container becomes narrow',style:`text-align:${align}`},
 {name:align+'-fits',text:'Short',style:`text-align:${align}`},
 {name:align+'-narrow',text:'ffi long title',style:`width:24px;text-align:${align}`}
]):plainF?[{name:'plain-f-8',text:'f',style:'width:8px'}]:[
 {name:'long',text:'A long title with agypqj and words that must disappear as the container becomes narrow',style:''},
 {name:'fits',text:'Short',style:''},
 {name:'too-narrow',text:'A long title',style:'width:8px'},
 ...(process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS==='1'?[
  {name:'ligature-8',text:'ffi long title',style:'width:8px'},
  {name:'ligature-24',text:'ffi long title',style:'width:24px'},
  {name:'ligature-40',text:'ffi long title',style:'width:40px'},
  {name:'combining-8',text:'A\u0301 long title',style:'width:8px'}
 ]:[]),
 {name:'fractional',text:'A long title with agypqj and words that must disappear as the container becomes narrow',style:'width:73.5%;padding:.25px'}
]).filter(f=>!fractionalOnly||f.name==='fractional').map(f=>({...f,html:`<div id=root><p id=clip>${f.text}</p></div>`,css:`#root{width:100%;padding:12px}#clip{display:block;font:24px/40px ${fontFamily};white-space:nowrap;overflow:hidden;width:100%;height:40px;${f.style}}`}));
const dprControl=process.argv.includes('--dpr-control');
if(dprControl&&(!process.env.NUXIE_HTML_REVIEW_DIR||process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS!=='1'))throw new Error('DPR control requires CSS ellipsis experiment and a separate review directory');
const dprs=dprControl?[1,2,3]:[1],widths=[240,390,768];
const browser=await chromium.launch(),results=[];
try {
 for(const fixture of fixtures){
  const thickness=fixture.name;
  const prefix=path.join(dir,thickness);
  const request={html:fixture.html,css:fixture.css+(publicCss?'#clip{text-overflow:ellipsis}':''),width:390,height:200,assets:{font:{kind:'font',family:fontFamily,weight:400,bytes:[...font]}}};
  const referenceCss=request.css+(plainF||clipOnly?'':'#clip{text-overflow:ellipsis}');
  fs.writeFileSync(prefix+'.json',JSON.stringify(request));run('html-to-riv',[prefix+'.json',prefix+'.riv']);
  const wasm=compiler.compile({languageVersion:'nuxie-html-v1',...request});assert.equal(wasm.ok,true);
  assert.deepEqual(Buffer.from(wasm.riv),fs.readFileSync(prefix+'.riv'));
  assert.deepEqual(wasm.sourceMap,JSON.parse(fs.readFileSync(prefix+'.map.json')));
  assert.deepEqual(wasm.runtimeRequirements,JSON.parse(fs.readFileSync(prefix+'.requirements.json')));
  if(publicCss)assert.ok(wasm.runtimeRequirements.text_policies.some(p=>p.policy==='css-single-line-ellipsis-v1'));
  if(manifestControl){
   assert.ok(!plainF&&!clipOnly,'Manifest ellipsis cannot be combined with clipping-only controls');
   const requirements=JSON.parse(fs.readFileSync(prefix+'.requirements.json'));
   assert.ok(requirements.text_policies.every(p=>p.policy==='css-nowrap-alignment-v1'));
   requirements.capabilities=requirements.capabilities.filter(c=>c!=='text-css-nowrap-alignment-v1');
   requirements.capabilities.push('text-css-single-line-ellipsis-v1');
   requirements.text_policies=requirements.text_policies.map(p=>({...p,policy:'css-single-line-ellipsis-v1'}));
   fs.writeFileSync(prefix+'.requirements.json',JSON.stringify(requirements));
  }
  for(const dpr of dprs){
   const page=await browser.newPage({viewport:{width:390,height:200},deviceScaleFactor:dpr});
   for(const width of widths){
    const frame=prefix+'-'+dpr+'-'+width;
    run('examples/probe',[prefix+'.riv',prefix+'.map.json',String(width),'200',frame],dpr);
    const glyphCache=fs.readFileSync(frame+'.glyph-cache.txt','utf8');
    if(process.env.NUXIE_EXPERIMENTAL_CSS_ELLIPSIS==='1'||plainF){
     assert.ok(Number(glyphCache.match(/entries: (\d+)/)?.[1])>0,'Requested native glyph profile fell back without populating the glyph cache');
    }
    run('renderer-replay',['--stream',frame+'.stream','--output',frame+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']);
    await page.setViewportSize({width,height:200});
    await page.setContent(`<style>@font-face{font-family:${fontFamily};src:url(data:font/ttf;base64,${font.toString('base64')})}${reset}${referenceCss}</style>${request.html}`);
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
    if(decoration){
     metrics.redDecoration=compareRedDecoration(reference,actual,{x:0,y:0,width:actual.width,height:actual.height});
     if(!metrics.redDecoration.passed)failures.push('red decoration support');
    }
    fs.writeFileSync(frame+'.diff.png',PNG.sync.write(diff));
    results.push({thickness,dpr,width,boxes,glyphCache,metrics,failures:[...geometry,...failures]});
   }
   await page.close();
  }
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify({browser:browser.version(),fontAsset,fontFamily,cases:results},null,2));
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
