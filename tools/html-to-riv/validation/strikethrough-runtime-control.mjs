// Runtime transport controls, not acceptance of strikethrough CSS by the compiler.
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
const dir=process.env.NUXIE_STRIKETHROUGH_RUNTIME_DIR || path.join(root,'output/playwright/html-to-riv/strikethrough-runtime-controls');
const compilerCss=process.argv.includes('--compiler-css');
const focus=process.argv.includes('--focus');
const transparent=process.argv.includes('--transparent-glyphs');
if(compilerCss || focus || transparent || process.env.NUXIE_STRIKETHROUGH_TEXT || process.argv.includes('--round-offset') || process.env.NUXIE_STRIKETHROUGH_PADDING) assert.ok(process.env.NUXIE_STRIKETHROUGH_RUNTIME_DIR, 'Use a separate output directory for diagnostic variants');
fs.mkdirSync(dir,{recursive:true});
const fontName=process.env.NUXIE_STRIKETHROUGH_FONT || 'Inter';
assert.ok(['Inter','OpenSans'].includes(fontName));
const fontSize=Number(process.env.NUXIE_STRIKETHROUGH_SIZE || 20);
const lineHeight=Number(process.env.NUXIE_STRIKETHROUGH_LINE_HEIGHT || 30);
assert.ok(fontSize>0 && Number.isFinite(fontSize) && lineHeight>0 && Number.isFinite(lineHeight));
const font=fs.readFileSync(path.join(moduleDir,`tests/assets/${fontName}-Regular.ttf`));
const reset=fs.readFileSync(path.join(moduleDir,'src/reset.css'),'utf8');
const source={html:'<p id="a">agypqj M agypqj M agypqj M agypqj M agypqj M agypqj M</p>',css:`p{width:100%;font:${fontSize}px/${lineHeight}px Inter;white-space:pre-wrap}`,width:390,height:240,assets:{inter:{kind:'font',family:'Inter',weight:400,bytes:[...font]}}};
if(transparent) source.css+='p{color:transparent}';
if(process.env.NUXIE_STRIKETHROUGH_TEXT) source.html='<p id="a">'+process.env.NUXIE_STRIKETHROUGH_TEXT.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;')+'</p>';
if(process.env.NUXIE_STRIKETHROUGH_PADDING) { const padding=Number(process.env.NUXIE_STRIKETHROUGH_PADDING);assert.ok(Number.isFinite(padding)&&padding>=0);source.html=`<div style="padding-top:${padding}px">${source.html}</div>`; }
const run=(bin,args,profile='0')=>execFileSync(path.join(root,'target/debug',bin),args,{encoding:'utf8',maxBuffer:8*1024*1024,env:{...process.env,NUXIE_NATIVE_GLYPHS:profile}});
fs.writeFileSync(path.join(dir,'source.json'),JSON.stringify(source));
run('html-to-riv',[path.join(dir,'source.json'),path.join(dir,'source.riv')]);
const manifest=JSON.parse(fs.readFileSync(path.join(dir,'source.requirements.json')));
assert.equal(manifest.text_policies.length,1);
// Resolve font metrics independently of browser layout, matching the compiler's
// existing explicit-line-height baseline policy. Family alias Inter embeds the
// selected fixture bytes identically in Chrome and the runtime.
const tables={};for(let i=0;i<font.readUInt16BE(4);i++){const at=12+i*16;tables[font.toString('ascii',at,at+4)]=font.readUInt32BE(at+8);}
const units=font.readUInt16BE(tables.head+18);
const ascent=Math.round(font.readInt16BE(tables.hhea+4)/units*fontSize);
const descent=Math.round(-font.readInt16BE(tables.hhea+6)/units*fontSize);
const lineBaseline=ascent+Math.floor((lineHeight-ascent-descent)/2);
const controls=[{name:'baseline',thickness:0,offset:0},... [1,2,3].map(thickness=>({name:'solid-'+thickness,thickness,offset:-ascent/3-thickness/2}))];
if(process.argv.includes('--round-offset')) for(const control of controls) control.offset=Math.floor(control.offset+0.5);
const browser=await chromium.launch();
const results=[];
try {
 assert.equal(browser.version(),'153.0.8010.12','Use the pinned Chromium reference');
 for(const control of controls.filter(c=>!focus||c.name==='solid-2')) {
  let compiledPath=path.join(dir,'source.riv');
  let compiledManifest=manifest;
  if(compilerCss) {
   const input={...source,css:source.css+`p{text-decoration:${control.name==='baseline'?'none':`line-through red ${control.thickness}px`}}`};
   const jsonPath=path.join(dir,control.name+'.json');compiledPath=path.join(dir,control.name+'.riv');
   fs.writeFileSync(jsonPath,JSON.stringify(input));run('html-to-riv',[jsonPath,compiledPath]);
   compiledManifest=JSON.parse(fs.readFileSync(compiledPath.replace(/\.riv$/,'.requirements.json')));
  }
  const resolved=compilerCss?compiledManifest:control.name==='baseline'?manifest:{...manifest,version:4,capabilities:[...manifest.capabilities,'text-solid-strikethroughs-v1'],text_strikethroughs:[{object_id:manifest.text_policies[0].object_id,lines:[{color:0xffff0000,thickness:control.thickness,offset:control.offset,line_baseline:lineBaseline}]}]};
  fs.writeFileSync(path.join(dir,'source.requirements.json'),JSON.stringify(resolved));
  fs.writeFileSync(path.join(dir,control.name+'.requirements.json'),JSON.stringify(resolved));
  for(const width of (focus?[390]:[240,390,768])) {
   const page=await browser.newPage({viewport:{width,height:240},deviceScaleFactor:1});
   const base=path.join(dir,control.name+'-'+width);
   await page.setContent(`<!doctype html><style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${font.toString('base64')})}${reset}${source.css}#a{text-decoration-line:${control.name==='baseline'?'none':'line-through'};text-decoration-color:red;text-decoration-thickness:${control.thickness}px;text-decoration-skip-ink:none}</style>${source.html}`);
   await page.evaluate(()=>document.fonts.ready);
   await page.screenshot({path:base+'.browser.png'});
   const boxes=await page.evaluate(()=>{const b=document.getElementById('a').getBoundingClientRect();return {a:{x:b.x,y:b.y,width:b.width,height:b.height}};});
   for(const profile of (focus?['1']:['0','1'])) {
    const name=control.name+'-'+(profile==='1'?'glyph':'vector');
    const prefix=path.join(dir,name+'-'+width);
    run('examples/probe',[compiledPath,compiledPath.replace(/\.riv$/,'.map.json'),String(width),'240',prefix],profile);
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
    const rows=image=>Array.from({length:image.height},(_,y)=>{
     let energy=0,count=0;const samples=[];
     for(let x=0;x<image.width;x++){const i=(y*image.width+x)*4;const red=image.data[i]-Math.max(image.data[i+1],image.data[i+2]);if(red>0){energy+=red;count++;if(x<12)samples.push([x,...image.data.subarray(i,i+4)]);}}
     return {y,energy,count,samples};
    }).filter(row=>row.energy>0);
    metrics.redRows={browser:rows(reference),native:rows(native)};
    // Transparent reduction: compare a stripe-interior column directly. The
    // aggregate text-box thresholds can hide a one-pixel error on one glyph.
    if(transparent && reference.width>5 && metrics.redRows.browser.some(row=>row.samples.some(sample=>sample[0]===5))) {
     for(let y=0;y<reference.height;y++) {
      const i=(y*reference.width+5)*4;
      if(Math.abs(reference.data[i+1]-native.data[i+1])>6) {
       failures.push('stripe interior column differs by more than 6');break;
      }
     }
    }
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
  if(!cases.length)continue;
  const html='<!doctype html><style>body{font:14px system-ui;background:#ddd}section{background:white;margin:12px;padding:8px;width:max-content}article{display:flex;gap:8px}img{max-width:500px}</style>'+cases.map(r=>`<section><h3>${r.name} ${width}: ${r.failures.join(', ')||'pass'}</h3><article>`+Object.entries(r.images).map(([kind,file])=>`<div>${kind}<br><img src="data:image/png;base64,${fs.readFileSync(file).toString('base64')}"></div>`).join('')+'</article></section>').join('');
  const name=`review-${profile}-${width}`;fs.writeFileSync(path.join(dir,name+'.html'),html);
  await review.setContent(html);await review.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  await review.screenshot({path:path.join(dir,name+'.png'),fullPage:true});
 }
 const failed=results.filter(r=>r.failures.length);
 console.log(JSON.stringify({passed:results.length-failed.length,total:results.length,failures:failed.map(({name,width,failures})=>({name,width,failures}))},null,2));
 if(failed.length)process.exitCode=1;
}finally{await browser.close();}
