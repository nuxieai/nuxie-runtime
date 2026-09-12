// Focused diagnosis only: immutable ordinary file geometry and extra Chrome measurements.
// Usage: node SCRIPT CASES_JSON NEW_OUTPUT; exit 1 preserves any >0.1px geometry difference.
import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';
import{fileURLToPath}from'node:url';import{spawnSync}from'node:child_process';import{chromium}from'@playwright/test';
const module=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const[fixtureFile,out]=process.argv.slice(2).map(p=>path.resolve(p));assert(fixtureFile&&out&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const read=p=>JSON.parse(fs.readFileSync(p));const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const freeze=read(path.join(module,'output/public-content-box-r2/source-bindings.json'));
const compiler=freeze.compiler.path,probe=path.join(module,'output/immutable-baseline-toolchain-r2/baseline-probe');
assert.equal(hash(compiler),freeze.compiler.sha256);assert.equal(hash(probe),'7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a');
const reset=fs.readFileSync(path.join(module,'src/reset.css'),'utf8');fs.writeFileSync(path.join(out,'browser-reset.css'),reset);fs.copyFileSync(fixtureFile,path.join(out,'source-cases.json'));
const run=(cmd,log)=>{const r=spawnSync(cmd[0],cmd.slice(1),{encoding:'utf8'});fs.writeFileSync(log,JSON.stringify({cmd,status:r.status,signal:r.signal,error:r.error?.message,stdout:r.stdout,stderr:r.stderr},null,2));assert.ifError(r.error);assert.equal(r.status,0,r.stderr);};
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');const rows=[];
try{const page=await browser.newPage({deviceScaleFactor:1,colorScheme:'light',locale:'en-US',reducedMotion:'reduce'});await page.setViewportSize({width:390,height:200});
for(const c of read(fixtureFile)){
 const dir=path.join(out,c.name);fs.mkdirSync(dir);const request=path.join(dir,'request.json'),riv=path.join(dir,'scene.riv'),probeOut=path.join(dir,'probe');
 fs.writeFileSync(request,JSON.stringify({html:c.html,css:c.css,width:390,height:160},null,2));
 run([compiler,request,riv],path.join(dir,'compile.log'));run([probe,riv,probeOut,'390x200'],path.join(dir,'probe.log'));
 assert.equal(hash(riv),hash(path.join(probeOut,'scene.riv')));
 const sourceMap=read(path.join(dir,'scene.map.json')),frames=read(path.join(probeOut,'frames.json')).frames;
 const html=`<!doctype html><style>${reset}\n${c.css}</style>${c.html}`;fs.writeFileSync(path.join(dir,'reference.html'),html);await page.setContent(html);
 const measured=await page.evaluate(async()=>{
  const nodes=[...document.querySelectorAll('[id]')];const observed=await new Promise(resolve=>{
   const ro=new ResizeObserver(entries=>{ro.disconnect();resolve(Object.fromEntries(entries.map(e=>[e.target.id,{contentRect:e.contentRect.toJSON(),contentBoxSize:Array.from(e.contentBoxSize,x=>({inlineSize:x.inlineSize,blockSize:x.blockSize})),borderBoxSize:Array.from(e.borderBoxSize,x=>({inlineSize:x.inlineSize,blockSize:x.blockSize}))}])));});nodes.forEach(n=>ro.observe(n));
  });
  return Object.fromEntries(nodes.map(e=>{
   const s=getComputedStyle(e),typed=e.computedStyleMap?.();const typedValue=k=>{const v=typed?.get(k);return v?{type:v.constructor.name,text:String(v),value:v.value??null,unit:v.unit??null}:null;};
   const rect=e.getBoundingClientRect();return[e.id,{rect:rect.toJSON(),clientWidth:e.clientWidth,offsetWidth:e.offsetWidth,offsetLeft:e.offsetLeft,computed:{width:s.width,minWidth:s.minWidth,maxWidth:s.maxWidth,paddingLeft:s.paddingLeft,paddingRight:s.paddingRight},typed:{width:typedValue('width'),minWidth:typedValue('min-width'),paddingLeft:typedValue('padding-left'),paddingRight:typedValue('padding-right')},resizeObserver:observed[e.id]}];
  }));
 });
 const native=frames.map(f=>({...f,geometry:read(path.join(probeOut,f.geometry))}));const differences=[];
 for(const f of native)for(const item of sourceMap){const g=f.geometry.find(g=>g.objectId===item.object_id);assert(g);const actual={x:g.worldMatrix[4],y:g.worldMatrix[5],width:g.width,height:g.height};
  for(const[k,v]of Object.entries(actual))if(Math.abs(v-measured[item.id].rect[k])>.1)differences.push({frame:f.frame,id:item.id,axis:k,native:v,chrome:measured[item.id].rect[k]});}
 rows.push({name:c.name,source:c,requestSha256:hash(request),rivSha256:hash(riv),mapSha256:hash(path.join(dir,'scene.map.json')),measured,native,differences});
}}
finally{await browser.close();}
const receipt={scope:'Read-only bounded geometry diagnosis; no pixel or public qualification.',compiler:freeze.compiler,probe:{path:probe,sha256:hash(probe)},browser:'153.0.8010.12',scriptSha256:hash(fileURLToPath(import.meta.url)),fixtureSha256:hash(fixtureFile),resetSha256:hash(path.join(out,'browser-reset.css')),rows};
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
console.log(JSON.stringify(rows.map(r=>({name:r.name,differences:r.differences,local:Object.fromEntries(Object.entries(r.measured).map(([id,m])=>[id,{rect:m.rect.width,resize:m.resizeObserver.contentRect.width,border:m.resizeObserver.borderBoxSize[0].inlineSize,typed:m.typed.width}]))})),null,2));
if(rows.some(r=>r.differences.length))process.exitCode=1;
