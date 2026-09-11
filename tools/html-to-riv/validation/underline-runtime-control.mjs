// Runtime transport controls, not acceptance of underline CSS by the compiler.
// The .riv is compiled once, then resized in the runtime by the probe.
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {comparePixels} from './pixels.mjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(moduleDir,'../..');
const dir=process.env.NUXIE_UNDERLINE_RUNTIME_DIR || path.join(root,'output/playwright/html-to-riv/underline-runtime-controls');
fs.mkdirSync(dir,{recursive:true});
const font=fs.readFileSync(path.join(moduleDir,'tests/assets/Inter-Regular.ttf'));
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const source={html:'<p id="a">agypqj M agypqj M agypqj M agypqj M agypqj M agypqj M</p>',css:'p{width:100%;font:20px/30px Inter;white-space:pre-wrap}',width:390,height:240,assets:{inter:{kind:'font',family:'Inter',weight:400,bytes:[...font]}}};
const run=(bin,args,profile='0')=>execFileSync(path.join(root,'target/debug',bin),args,{encoding:'utf8',maxBuffer:8*1024*1024,env:{...process.env,NUXIE_NATIVE_GLYPHS:profile}});
fs.writeFileSync(path.join(dir,'source.json'),JSON.stringify(source));
run('html-to-riv',[path.join(dir,'source.json'),path.join(dir,'source.riv')]);
const manifest=JSON.parse(fs.readFileSync(path.join(dir,'source.requirements.json')));
assert.equal(manifest.text_policies.length,1);
const controls=[{name:'baseline',thickness:0,offset:0,skip:'none'},{name:'solid',thickness:2,offset:1,skip:'none'},{name:'skip-auto',thickness:2,offset:1,skip:'auto'},{name:'offset',thickness:3,offset:4,skip:'auto'}];
const browser=await chromium.launch();
const results=[];
try {
 assert.equal(browser.version(),'153.0.8010.12','Use the pinned Chromium reference');
 for(const control of controls) {
  const resolved=control.name==='baseline'?manifest:{...manifest,version:3,capabilities:[...manifest.capabilities,'text-solid-underlines-v1'],text_underlines:[{object_id:manifest.text_policies[0].object_id,lines:[{color:0xffff0000,thickness:control.thickness,offset:control.offset,skip_ink:control.skip}]}]};
  fs.writeFileSync(path.join(dir,'source.requirements.json'),JSON.stringify(resolved));
  fs.writeFileSync(path.join(dir,control.name+'.requirements.json'),JSON.stringify(resolved));
  for(const width of [240,390,768]) {
   const page=await browser.newPage({viewport:{width,height:240},deviceScaleFactor:1});
   const base=path.join(dir,control.name+'-'+width);
   await page.setContent(`<!doctype html><style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${font.toString('base64')})}${reset}${source.css}#a{text-decoration-line:${control.name==='baseline'?'none':'underline'};text-decoration-color:red;text-decoration-thickness:${control.thickness}px;text-underline-offset:${control.offset}px;text-decoration-skip-ink:${control.skip}}</style>${source.html}`);
   await page.evaluate(()=>document.fonts.ready);
   await page.screenshot({path:base+'.browser.png'});
   const boxes=await page.evaluate(()=>{const b=document.getElementById('a').getBoundingClientRect();return {a:{x:b.x,y:b.y,width:b.width,height:b.height}};});
   for(const profile of ['0','1']) {
    const name=control.name+'-'+(profile==='1'?'glyph':'vector');
    const prefix=path.join(dir,name+'-'+width);
    run('examples/probe',[path.join(dir,'source.riv'),path.join(dir,'source.map.json'),String(width),'240',prefix],profile);
    run('renderer-replay',['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']);
    const bounds=JSON.parse(fs.readFileSync(prefix+'.bounds.json'));
    const reference=PNG.sync.read(fs.readFileSync(base+'.browser.png'));
    const native=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));
    const {metrics,failures,diff}=comparePixels(reference,native,boxes,true);
    for(const key of ['x','y','width','height']) if(Math.abs(bounds.a[key]-boxes.a[key])>.1) failures.push('geometry: a.'+key);
    // Require visible red decoration independently of black text pixels.
    const redCount=image=>{let n=0;for(let i=0;i<image.data.length;i+=4)if(image.data[i]>image.data[i+1]+64 && image.data[i]>image.data[i+2]+64)n++;return n;};
    const expectedRed=redCount(reference),actualRed=redCount(native);
    if(control.name==='baseline' ? actualRed!==0 : (!expectedRed || actualRed<expectedRed*.85 || actualRed>expectedRed*1.15)) failures.push('red decoration coverage');
    metrics.redPixels={browser:expectedRed,native:actualRed};
    fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
    results.push({name,width,metrics,failures,bounds:{browser:boxes,native:bounds},images:{browser:base+'.browser.png',native:prefix+'.native.png',diff:prefix+'.diff.png'}});
   }
   await page.close();
  }
 }
 fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify({browser:browser.version(),cases:results},null,2));
 const review=await browser.newPage({viewport:{width:1700,height:1000}});
 for(const width of [240,390,768])for(const profile of ['glyph','vector']) {
  const cases=results.filter(r=>r.width===width&&r.name.endsWith(profile));
  const html='<!doctype html><style>body{font:14px system-ui;background:#ddd}section{background:white;margin:12px;padding:8px;width:max-content}article{display:flex;gap:8px}img{max-width:500px}</style>'+cases.map(r=>`<section><h3>${r.name} ${width}: ${r.failures.join(', ')||'pass'}</h3><article>`+Object.entries(r.images).map(([kind,file])=>`<div>${kind}<br><img src="data:image/png;base64,${fs.readFileSync(file).toString('base64')}"></div>`).join('')+'</article></section>').join('');
  const name=`review-${profile}-${width}`;fs.writeFileSync(path.join(dir,name+'.html'),html);
  await review.setContent(html);await review.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  await review.screenshot({path:path.join(dir,name+'.png'),fullPage:true});
 }
 const failed=results.filter(r=>r.failures.length);
 console.log(JSON.stringify({passed:results.length-failed.length,total:results.length,failures:failed.map(({name,width,failures})=>({name,width,failures}))},null,2));
 if(failed.length)process.exitCode=1;
}finally{await browser.close();}
