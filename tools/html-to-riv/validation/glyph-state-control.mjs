// Native host renderer state, not newly accepted CSS compiler syntax. The same
// compiled scene is resized and drawn through the live runtime glyph adapter.
import assert from 'node:assert/strict';
import {createCompiler} from '../js/index.mjs';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {comparePixels,compareRedDecoration} from './pixels.mjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(moduleDir,'../..');
const profile=process.argv[2]??'glyph';
if(!['glyph','underline','strikethrough','clipping','ellipsis'].includes(profile))throw new Error('Expected glyph, underline, strikethrough, clipping or ellipsis profile');
const clipping=profile==='clipping';
const ellipsis=profile==='ellipsis';
const underline=profile==='underline';
const strike=profile==='strikethrough';
const decorated=underline||strike;
const restoreCopy=!process.argv.includes('--no-restore');
const transparentGlyphs=process.argv.includes('--transparent-glyphs');
const clipPhase=process.argv.includes('--clip-phase');
if(clipPhase&&(!transparentGlyphs||restoreCopy))throw new Error('Clip phase diagnostic requires --no-restore --transparent-glyphs');
if(transparentGlyphs&&!decorated)throw new Error('Transparent glyph diagnostic requires a decoration profile');
const dir=process.env.NUXIE_HTML_REVIEW_DIR??path.join(root,'output/playwright/html-to-riv',`${profile}-state-controls`+(restoreCopy?'':'-isolated')+(transparentGlyphs?'-transparent':'')+(clipPhase?'-phase':''));
fs.mkdirSync(dir,{recursive:true});
const font=fs.readFileSync(path.join(moduleDir,'tests/assets',decorated?'NuxieJapaneseFixture-Regular.otf':'Inter-Regular.ttf'));
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
let request={html:'<div id="root"><p id="a">Glyph phase Hg</p><p id="b">Color / alpha</p></div>',css:'#root{width:100%;padding:12px;gap:8px;font-family:Inter}#a{width:170px;font:20px/30px Inter;color:#369}#b{width:140px;font:16px/24px Inter;color:rgba(51,34,85,.5)}',width:390,height:320,assets:{inter:{kind:'font',family:'Inter',weight:400,bytes:[...font]}}};
if(decorated)request={
 html:'<div id="root"><p id="a">カタカナ がぎぐげご ／＿ agypqj</p><p id="b">agypqj gap</p></div>',
 css:'#root{width:100%;padding:8px;font-family:"Nuxie Japanese Fixture"}p{font-size:24px;line-height:40px;text-decoration:underline solid red 2px;text-underline-offset:-6px}#a{text-decoration-skip-ink:auto}#b{text-decoration-skip-ink:all}',
 width:390,height:320,assets:{japanese:{kind:'font',family:'Nuxie Japanese Fixture',weight:400,bytes:[...font]}}
};
if(clipping)request={...request,html:'<div id=root><p id=a>agypqj fractional clipping across this edge</p><p id=b>MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM</p><p id=after>Restored sibling</p></div>',css:'#root{width:100%;padding:12px;gap:8px}p{font:24px/40px Inter}#a{width:73.5%;height:22.5px;padding:.25px;white-space:nowrap;overflow:clip;background:#edf3fa}#b{width:100%;height:28px;white-space:nowrap;overflow:hidden;border-radius:14px;background:#edf3fa}#after{font:16px/24px Inter;color:#075e42}'};
if(ellipsis)request={...request,html:'<div id=root><p id=a>ffi long title with repeated words and more words</p><p id=b>ffi long title</p><p id=after>Restored sibling</p></div>',css:'#root{width:100%;padding:12px;gap:8px}p{display:block;font:24px/40px Inter;white-space:nowrap;overflow:hidden}#a{width:100%;height:40px}#b{width:40px;height:40px;letter-spacing:2px}#after{font:16px/24px Inter}'};
if(strike)request.css+='p{text-decoration:line-through solid red 2px}';
if(transparentGlyphs)request.css+='p{color:transparent}';
fs.writeFileSync(path.join(dir,'source.json'),JSON.stringify(request));
const run=(bin,args)=>execFileSync(path.join(root,'target/debug',bin),args,{encoding:'utf8',maxBuffer:8*1024*1024,env:{...process.env,NUXIE_NATIVE_GLYPHS:'1',...(ellipsis?{NUXIE_EXPERIMENTAL_TEXT_ELLIPSIS:'1',NUXIE_EXPERIMENTAL_CSS_ELLIPSIS:'1'}:{})}});
run('html-to-riv',[path.join(dir,'source.json'),path.join(dir,'source.riv')]);
if(strike||clipping||ellipsis){
 const compiler=await createCompiler(fs.readFileSync(path.join(moduleDir,'dist/html-to-riv.wasm')));
 const result=compiler.compile({languageVersion:'nuxie-html-v1',...request});
 assert.equal(result.ok,true);
 assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(path.join(dir,'source.riv')));
 assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(path.join(dir,'source.map.json'))));
 assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(path.join(dir,'source.requirements.json'))));
}

const angle=12*Math.PI/180;
let controls=[
 {name:'identity',matrix:[1,0,0,1,0,0]},
 {name:'fractional-translation',matrix:[1,0,0,1,7.25,3.5]},
 {name:'clip-through-glyphs',matrix:[1,0,0,1,0,0],clip:[18,16,120,23]},
 {name:'opacity',matrix:[1,0,0,1,0,0],opacity:.5},
 {name:'uniform-scale',matrix:[1.25,0,0,1.25,.25,.5]},
 {name:'nonuniform-scale',matrix:[1.4,0,0,.8,2.25,3.5]},
 {name:'rotation',matrix:[Math.cos(angle),Math.sin(angle),-Math.sin(angle),Math.cos(angle),20,0]},
 {name:'shear',matrix:[1,.15,.25,1,5,2]},
 {name:'combined',matrix:[Math.cos(angle),Math.sin(angle),-Math.sin(angle),Math.cos(angle),20,0],clip:[18,16,120,42],opacity:.5},
];
if(clipPhase)controls=[2.24,2.245,2.249,2.25,2.251,2.255,2.26].map(tx=>({name:'clip-phase-'+tx,matrix:[1.4,0,0,.8,tx,3.5]}));
const browser=await chromium.launch();
const results=[];
try {
 for(const width of (clipPhase?[240]:[240,390,768])) {
  const page=await browser.newPage({viewport:{width,height:320},deviceScaleFactor:1});
  for(const control of controls) {
   const prefix=path.join(dir,control.name+'-'+width);
   const state={matrix:control.matrix,clip:control.clip??null,opacity:control.opacity??1,restore_copy:restoreCopy};
   fs.writeFileSync(prefix+'.state.json',JSON.stringify(state));
   run('examples/probe',[path.join(dir,'source.riv'),path.join(dir,'source.map.json'),String(width),'320',prefix,prefix+'.state.json']);
   if(ellipsis)assert.ok(Number(fs.readFileSync(prefix+'.glyph-cache.txt','utf8').match(/entries: (\d+)/)?.[1])>0,'Ellipsis profile must use native glyph rendering');
   run('renderer-replay',['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']);
   const clip=state.clip?`clip-path:inset(${state.clip[1]}px ${width-state.clip[0]-state.clip[2]}px ${320-state.clip[1]-state.clip[3]}px ${state.clip[0]}px);`:'';
   await page.setContent(`<!doctype html><style>@font-face{font-family:${decorated?'"Nuxie Japanese Fixture"':'Inter'};src:url(data:font/${decorated?'otf':'ttf'};base64,${font.toString('base64')})}${reset}${request.css}${ellipsis?'p{text-overflow:ellipsis}':''}#clip-host,#transformed,#restored{width:${width}px;height:320px}#clip-host{position:absolute;left:0;top:0;${clip}}#transformed{transform-origin:0 0;transform:matrix(${state.matrix.join(',')});opacity:${state.opacity}}#restored{position:absolute;left:0;top:160px}</style><div id="clip-host"><div id="transformed">${request.html}</div></div>${restoreCopy?`<div id="restored">${request.html}</div>`:''}`);
   await page.evaluate(()=>document.fonts.ready);
   if(ellipsis)assert.equal(await page.evaluate(()=>[...document.querySelectorAll('p')].every(p=>{
    const style=getComputedStyle(p);
    return style.display==='block'&&style.textOverflow==='ellipsis'&&style.overflowX==='hidden'&&style.whiteSpace==='nowrap';
   })),true,'Chrome ellipsis reference must establish a clipped block formatting context');
   await page.screenshot({path:prefix+'.browser.png'});
   const boxes=await page.evaluate(restoreCopy=>Object.fromEntries((restoreCopy?['transformed','restored']:['transformed']).flatMap(host=>[...document.getElementById(host).querySelectorAll('[id]')].map(node=>{
    const b=node.getBoundingClientRect();return [host+'-'+node.id,{x:b.x,y:b.y,width:b.width,height:b.height}];
   }))),restoreCopy);
   const native=JSON.parse(fs.readFileSync(prefix+'.bounds.json'));
   const geometry=[];
   for(const [id,b] of Object.entries(native)) for(const host of (restoreCopy?['transformed','restored']:['transformed'])) {
    const m=host==='transformed'?state.matrix:[1,0,0,1,0,160];
    const points=[[b.x,b.y],[b.x+b.width,b.y],[b.x,b.y+b.height],[b.x+b.width,b.y+b.height]].map(([x,y])=>[m[0]*x+m[2]*y+m[4],m[1]*x+m[3]*y+m[5]]);
    const xs=points.map(p=>p[0]),ys=points.map(p=>p[1]);
    const actual={x:Math.min(...xs),y:Math.min(...ys),width:Math.max(...xs)-Math.min(...xs),height:Math.max(...ys)-Math.min(...ys)};
    for(const key of ['x','y','width','height']) if(Math.abs(actual[key]-boxes[host+'-'+id][key])>.1) geometry.push(host+'-'+id+'.'+key);
   }
   const {metrics,failures,diff}=comparePixels(PNG.sync.read(fs.readFileSync(prefix+'.browser.png')),PNG.sync.read(fs.readFileSync(prefix+'.native.png')),boxes,true);
   // White-background ink coverage catches opacity errors that a sparse glyph
   // region's mean RGB can dilute. This strengthens, rather than replaces, the
   // shared pixel limits. Preserve up to 15% variation for independent glyph AA.
   const reference=PNG.sync.read(fs.readFileSync(prefix+'.browser.png'));
   const actual=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));
   if(decorated){
    metrics.redDecoration=compareRedDecoration(reference,actual,{x:0,y:0,width,height:320});
    if(!metrics.redDecoration.passed)failures.push('red decoration support');
   }
   if(strike&&control.name==='opacity'){
    // At identity and half group opacity, an opaque red stripe covers glyph
    // ink before compositing onto white. Compare its solid pink interior;
    // aggregate image/ink checks otherwise hide per-draw alpha darkening.
    let checked=0,mismatched=0;
    for(let i=0;i<reference.data.length;i+=4){
     const r=reference.data[i],g=reference.data[i+1],b=reference.data[i+2];
     if(r>250&&g>=125&&g<=130&&b>=125&&b<=130){
      checked++;
      const error=(Math.abs(r-actual.data[i])+Math.abs(g-actual.data[i+1])+Math.abs(b-actual.data[i+2]))/3;
      if(error>6)mismatched++;
     }
    }
    metrics.opacityStripeInterior={checked,mismatched};
    if(!checked||mismatched)failures.push('opacity stripe interior');
   }
   const inkRatios={};
   for(const [id,box] of Object.entries(boxes)) {
    let expectedInk=0,actualInk=0;
    for(let y=Math.max(0,Math.floor(box.y));y<Math.min(320,Math.ceil(box.y+box.height));y++)
     for(let x=Math.max(0,Math.floor(box.x));x<Math.min(width,Math.ceil(box.x+box.width));x++)
      for(let c=0;c<3;c++){const i=(y*width+x)*4+c;expectedInk+=255-reference.data[i];actualInk+=255-actual.data[i];}
    inkRatios[id]=expectedInk?actualInk/expectedInk:actualInk?null:1;
    if(inkRatios[id]===null || inkRatios[id]<.85 || inkRatios[id]>1.15)failures.push(id+': ink coverage');
   }
   metrics.inkCoverageRatios=inkRatios;
   fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
   results.push({name:control.name,width,metrics,failures:[...geometry.map(g=>'geometry: '+g),...failures],cache:fs.readFileSync(prefix+'.glyph-cache.txt','utf8')});
  }
  const review=`<!doctype html><style>body{font:14px system-ui;background:#ddd}section{background:white;padding:8px;margin:12px;width:max-content}article{display:flex;gap:8px}img{max-width:500px}</style><h2>${profile} host state controls — ${width}px</h2>`+controls.map(control=>`<section><h3>${control.name}</h3><article>`+['browser','native','diff'].map(kind=>`<div>${kind}<br><img src="data:image/png;base64,${fs.readFileSync(path.join(dir,`${control.name}-${width}.${kind}.png`)).toString('base64')}"></div>`).join('')+'</article></section>').join('');
  fs.writeFileSync(path.join(dir,`review-${width}.html`),review);await page.setViewportSize({width:1700,height:1000});await page.setContent(review);await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));await page.screenshot({path:path.join(dir,`review-${width}.png`),fullPage:true});
  for(let group=0;group<3;group++) {
   await page.evaluate(group=>[...document.querySelectorAll('section')].forEach((section,i)=>section.hidden=Math.floor(i/3)!==group),group);
   await page.screenshot({path:path.join(dir,`review-${width}-${group+1}.png`),fullPage:true});
  }
  await page.close();
 }
 fs.writeFileSync(path.join(dir,'results.json'),JSON.stringify(results,null,2));
 const failed=results.filter(r=>r.failures.length);console.log(JSON.stringify({passed:results.length-failed.length,total:results.length,failures:failed.map(r=>({name:r.name,width:r.width,failures:r.failures}))},null,2));
 if(failed.length)process.exitCode=1;
}finally{await browser.close();}
