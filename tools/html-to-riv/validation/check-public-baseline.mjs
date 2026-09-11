// Public HTML/CSS -> ordinary RIV -> immutable probe/renderer -> independent Chrome.
// No runtime policy installers, custom font modes or CSS-aware rendering hooks.
// Usage: node SCRIPT CASES_JSON COMPILER BASELINE_PROBE BASELINE_RENDERER NEW_OUTPUT
import fs from 'node:fs';import path from 'node:path';import crypto from 'node:crypto';import assert from 'node:assert/strict';import{spawnSync}from'node:child_process';import{fileURLToPath}from'node:url';
import{chromium}from'@playwright/test';import{PNG}from'pngjs';import{comparePixels}from'./pixels.mjs';
const argv=process.argv.slice(2);assert.equal(argv.length,5,'CASES_JSON COMPILER BASELINE_PROBE BASELINE_RENDERER NEW_OUTPUT');
const[fixturesFile,compiler,probe,renderer,out]=argv.map(p=>path.resolve(p));assert(!fs.existsSync(out),'fresh output required');fs.mkdirSync(out,{recursive:true});
const hash=f=>crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex');const read=f=>JSON.parse(fs.readFileSync(f));
const tools={compiler,probe,renderer};const toolHashes=Object.fromEntries(Object.entries(tools).map(([name,file])=>[name,hash(file)]));
const fixtures=read(fixturesFile);assert(fixtures.length>0&&new Set(fixtures.map(f=>f.name)).size===fixtures.length);for(const f of fixtures)assert(/^[a-z0-9-]+$/.test(f.name));
fs.copyFileSync(fixturesFile,path.join(out,'source-cases.json'));
// Preserve the exact comparison implementation even when a later run changes it.
fs.copyFileSync(fileURLToPath(import.meta.url),path.join(out,'check-public-baseline.mjs'));
fs.copyFileSync(fileURLToPath(new URL('./pixels.mjs',import.meta.url)),path.join(out,'pixels.mjs'));
const defaultSizes=[[240,160],[390,200],[768,120],[240,160]];
const reset=fs.readFileSync(new URL('../src/reset.css',import.meta.url),'utf8');fs.writeFileSync(path.join(out,'browser-reset.css'),reset);
const sourceBindings=[[fixturesFile,'source-cases.json'],[fileURLToPath(import.meta.url),'check-public-baseline.mjs'],[fileURLToPath(new URL('./pixels.mjs',import.meta.url)),'pixels.mjs'],[fileURLToPath(new URL('../src/reset.css',import.meta.url)),'browser-reset.css']].map(([source,snapshot])=>({source,snapshot,sha256:hash(path.join(out,snapshot))}));
for(const binding of sourceBindings)assert.equal(hash(binding.source),binding.sha256,'comparison source changed while snapshotting');
const rows=[],artifacts=[];const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');
const run=(command,log)=>{const r=spawnSync(command[0],command.slice(1),{encoding:'utf8'});fs.writeFileSync(log,(r.stdout??'')+(r.stderr??''));assert.ifError(r.error);assert.equal(r.signal,null);assert.equal(r.status,0,r.stderr);};
try{const page=await browser.newPage({deviceScaleFactor:1,colorScheme:'light',locale:'en-US',reducedMotion:'reduce'});
 for(const fixture of fixtures){
  const sizes=fixture.viewports??defaultSizes;assert(Array.isArray(sizes)&&sizes.length===4,'fixture.viewports must have exactly four entries');
  for(const size of sizes)assert(Array.isArray(size)&&size.length===2&&size.every(v=>Number.isInteger(v)&&v>0&&v<=16384),'viewport must be a positive bounded integer pair');
  if(fixture.swatches){assert(Number.isInteger(fixture.swatchHeight)&&fixture.swatchHeight>0);assert(fixture.swatches.length>0);assert.equal(new Set(fixture.swatches.map(s=>s.id)).size,fixture.swatches.length);for(const[,height]of sizes)assert(height>=fixture.swatches.length*fixture.swatchHeight,'entire palette must fit vertically');}
  const dir=path.join(out,fixture.name);fs.mkdirSync(dir);const request=path.join(dir,'request.json'),riv=path.join(dir,'scene.riv'),map=path.join(dir,'scene.map.json'),observed=path.join(dir,'probe');
  fs.writeFileSync(request,JSON.stringify({html:fixture.html,css:fixture.css,width:390,height:160},null,2));
  run([compiler,request,riv],path.join(dir,'compile.log'));assert(!fs.existsSync(path.join(dir,'scene.requirements.json')),'ordinary file must not depend on a requirements sidecar');
  const sourceMap=read(map);assert(Array.isArray(sourceMap));assert.equal(new Set(sourceMap.map(n=>n.object_id)).size,sourceMap.length);
  // The observer receives only RIV and resize sizes. Source IDs are read later,
  // solely to join measured runtime objects to independently measured DOM IDs.
  run([probe,riv,observed,...sizes.map(([w,h])=>`${w}x${h}`)],path.join(dir,'probe.log'));assert.equal(hash(riv),hash(path.join(observed,'scene.riv')));
  const frames=read(path.join(observed,'frames.json')).frames;assert.equal(frames.length,8);
  artifacts.push({name:fixture.name,features:fixture.features,requestSha256:hash(request),rivSha256:hash(riv),mapSha256:hash(map),probeManifestSha256:hash(path.join(observed,'frames.json'))});
  const html=`<!doctype html><style>${reset}\n${fixture.css}</style>${fixture.html}`;fs.writeFileSync(path.join(dir,'reference.html'),html);
  for(const f of frames){assert.equal(f.instance,Math.floor(f.frame/4));assert.equal(f.step,f.frame%4);assert.deepEqual([f.width,f.height],sizes[f.step]);
   const prefix=path.join(dir,`frame-${f.frame}`);await page.setViewportSize({width:f.width,height:f.height});await page.setContent(html);
   const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{const{x,y,width,height}=e.getBoundingClientRect();return[e.id,{x,y,width,height}]})));
   const computedStyles=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{const s=getComputedStyle(e);return[e.id,{width:s.width,height:s.height,color:s.color,backgroundColor:s.backgroundColor,display:s.display,flexDirection:s.flexDirection,alignSelf:s.alignSelf,order:s.order}]})));
   if(fixture.expectedPaint){
    for(const [id,color] of Object.entries(fixture.expectedPaint)){
     assert(computedStyles[id],`missing expected paint node ${id}`);
     assert.equal(computedStyles[id].backgroundColor,color,`${fixture.name}: independent expected paint for ${id}`);
    }
   }
   if(fixture.expectedAlignment){
    for(const [id,value] of Object.entries(fixture.expectedAlignment)){
     assert(computedStyles[id],`missing expected alignment node ${id}`);
     assert.equal(computedStyles[id].alignSelf,value,`${fixture.name}: independent expected alignment for ${id}`);
    }
   }
   const swatchObservations=[];
   if(fixture.swatches){
    for(const[index,swatch]of fixture.swatches.entries()){
     assert(await page.evaluate(color=>CSS.supports('background-color',color),swatch.color),`invalid browser color ${swatch.color}`);
     const box=boxes[swatch.id];assert(box,`missing swatch ${swatch.id}`);
     assert.equal(box.x,0);assert.equal(box.y,index*fixture.swatchHeight);assert.equal(box.width,f.width);assert.equal(box.height,fixture.swatchHeight);
     assert(box.y+box.height<=f.height,'swatch clipped from screenshot');
     const computedColor=await page.evaluate(id=>getComputedStyle(document.getElementById(id)).backgroundColor,swatch.id);
     swatchObservations.push({...swatch,box,computedColor});
    }
   }
   assert.deepEqual(sourceMap.map(n=>n.id).sort(),Object.keys(boxes).sort(),'every authored DOM ID must have exactly one read-only mapping');
   await page.screenshot({path:prefix+'.chrome.png',animations:'disabled'});const geometryFile=path.join(observed,f.geometry),streamFile=path.join(observed,f.stream),geometry=read(geometryFile),geometryFailures=[];
   for(const node of sourceMap){const actual=geometry.find(g=>g.objectId===node.object_id);assert(actual,`missing layout object ${node.object_id}`);assert.deepEqual(actual.worldMatrix.slice(0,4),[1,0,0,1]);const measured={x:actual.worldMatrix[4],y:actual.worldMatrix[5],width:actual.width,height:actual.height};for(const axis of Object.keys(measured))if(Math.abs(measured[axis]-boxes[node.id][axis])>0.1)geometryFailures.push(`${node.id}.${axis}: ${measured[axis]} vs ${boxes[node.id][axis]}`);}
   const command=[renderer,'--stream',streamFile,'--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic','--frame',String(f.frame)];
   // Baseline rust-metal selects raster ordering; clockwise-atomic is its CLI's
   // accepted non-MSAA token, not a claim that this path uses forced winding.
   run(command,prefix+'.native.log');
   // The fixed white host must come from ordinary Rive paint, not canvas clear.
   const clearChecks=[];
   for(const [name,value] of [['cyan','0xff00ffff'],['transparent','0x00000000']]){
    const clearFile=prefix+'.clear-'+name+'.png',clearCommand=[...command];clearCommand[clearCommand.indexOf('--output')+1]=clearFile;clearCommand.push('--clear',value);
    run(clearCommand,prefix+'.clear-'+name+'.log');
    const samePixels=PNG.sync.read(fs.readFileSync(clearFile)).data.equals(PNG.sync.read(fs.readFileSync(prefix+'.native.png')).data);
    clearChecks.push({name,value,path:clearFile,sha256:hash(clearFile),samePixels,command:clearCommand});
   }
   const comparison=comparePixels(PNG.sync.read(fs.readFileSync(prefix+'.chrome.png')),PNG.sync.read(fs.readFileSync(prefix+'.native.png')),boxes,false);fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(comparison.diff));
   rows.push({name:fixture.name,...f,prefix,clearChecks,boxes,computedStyles,swatchObservations,geometry,geometryFailures,pixelFailures:comparison.failures,metrics:comparison.metrics,command,requestSha256:hash(request),rivSha256:hash(riv),sourceMapSha256:hash(map),streamSha256:hash(streamFile),geometrySha256:hash(geometryFile),chromeSha256:hash(prefix+'.chrome.png'),nativeSha256:hash(prefix+'.native.png')});
  }
 }
}finally{await browser.close();}
for(const[name,file]of Object.entries(tools))assert.equal(hash(file),toolHashes[name]);
for(const binding of sourceBindings)assert.equal(hash(binding.source),binding.sha256,'comparison source changed during run');
const repeated=rows.map(r=>{const first=rows.find(p=>p.name===r.name&&p.width===r.width&&p.height===r.height);return{name:r.name,frame:r.frame,firstFrame:first.frame,nativeIdentical:r.nativeSha256===first.nativeSha256,chromeIdentical:r.chromeSha256===first.chromeSha256};}).filter(r=>r.frame!==r.firstFrame);
const passed=rows.every(r=>!r.geometryFailures.length&&!r.pixelFailures.length&&r.clearChecks.every(c=>c.samePixels))&&repeated.every(r=>r.nativeIdentical&&r.chromeIdentical);
const receipt={status:passed?'passed-public-baseline':'failed-public-baseline',scope:'Bounded public compiler ordinary files on immutable runtime; independently captured Chrome geometry/pixels, original/clone resizing. No complete language coverage or direct visual-review claim.',browser:'153.0.8010.12',backend:'rust-metal',effectiveMode:'RasterOrdering',cliModeToken:'clockwise-atomic',tools,toolHashes,sourceBindings,driverSha256:hash(fileURLToPath(import.meta.url)),fixturesSha256:hash(fixturesFile),resetSha256:hash(path.join(out,'browser-reset.css')),pixelGateSha256:hash(fileURLToPath(new URL('./pixels.mjs',import.meta.url))),artifacts,rows,repeated};
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
const cells=rows.map(r=>`<section><h2>${r.name} · ${r.width}×${r.height} · instance ${r.instance} step ${r.step}</h2><p>${[...r.geometryFailures,...r.pixelFailures].join('; ')||'metric gates pass'}</p><div><img src="${r.name}/frame-${r.frame}.chrome.png" alt="Chrome"><img src="${r.name}/frame-${r.frame}.native.png" alt="Immutable native"></div></section>`).join('');
fs.writeFileSync(path.join(out,'gallery.html'),`<!doctype html><meta charset="utf-8"><title>Public compiler / immutable Rive</title><style>body{font:15px system-ui;margin:24px}section{margin:32px 0}section div{display:flex;gap:16px}img{max-width:48%;object-fit:contain;object-position:left top;border:1px solid #aaa}</style><h1>Chrome / immutable native Rive</h1><p>Bounded public compiler validation. Direct visual review is separate.</p>${cells}`);
console.log(JSON.stringify({status:receipt.status,cases:fixtures.length,frames:rows.length,geometryPass:rows.filter(r=>!r.geometryFailures.length).length,pixelPass:rows.filter(r=>!r.pixelFailures.length).length,clearPass:rows.flatMap(r=>r.clearChecks).filter(c=>c.samePixels).length}));
if(!passed)process.exitCode=1;
