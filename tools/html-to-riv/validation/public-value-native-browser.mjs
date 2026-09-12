// Independent pinned Chrome characterization. No compiler or native execution.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {isDeepStrictEqual} from 'node:util';
import {chromium} from '@playwright/test';

const root=fileURLToPath(new URL('../',import.meta.url));
const out=path.resolve(process.argv[2]);assert(out&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const formFile=path.join(root,'validation/public-value-native-forms.json');
const sceneFile=path.join(root,'validation/public-value-native-cases.json');
const resetFile=path.join(root,'src/reset.css');
const forms=JSON.parse(fs.readFileSync(formFile));const scenes=JSON.parse(fs.readFileSync(sceneFile));
const reset=fs.readFileSync(resetFile,'utf8');
const properties=[...new Set(forms.map(f=>f.property))];assert.equal(properties.length,33);
const observedProperties=[...properties,'background-image','background-position','background-repeat','background-size','background-origin','background-clip','background-attachment'];
for(const [file,snapshot] of [[formFile,'forms.json'],[sceneFile,'cases.json'],[resetFile,'reset.css'],[fileURLToPath(import.meta.url),'driver.mjs']])fs.copyFileSync(file,path.join(out,snapshot));
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');
const rows=[],sceneRows=[];
try{
 const page=await browser.newPage({viewport:{width:390,height:200},deviceScaleFactor:1,colorScheme:'light',locale:'en-US',reducedMotion:'reduce'});
 const observe=async(html,css)=>{
  await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
  return page.evaluate(properties=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{
   const s=getComputedStyle(e);const{x,y,width,height}=e.getBoundingClientRect();
   return[e.id,{style:Object.fromEntries(properties.map(p=>[p,s.getPropertyValue(p)])),box:{x,y,width,height}}];
  })),observedProperties);
 };
 for(const f of forms){
  const literalSupport=await page.evaluate(({property,tokens})=>CSS.supports(property,tokens),f);
  const actual=await observe(f.html,f.css),control=await observe(f.html,f.controlCss);
  const direct=await observe(f.html,f.directCss),prior=await observe(f.html,f.prefixCss);
  const failures=[];
  if(literalSupport!==f.expectedLiteralSupport)failures.push('literal-support-expectation');
  if(!isDeepStrictEqual(actual,control))failures.push('variable-vs-explicit-control');
  if(!literalSupport&&!isDeepStrictEqual(direct,prior))failures.push('literal-invalid-declaration-does-not-preserve-prior');
  rows.push({name:f.name,property:f.property,form:f.form,classification:f.classification,literalSupport,expectedLiteralSupport:f.expectedLiteralSupport,exactComputedAndGeometry:isDeepStrictEqual(actual,control),directInvalidIgnored:literalSupport?null:isDeepStrictEqual(direct,prior),variableDiffersFromPrior:!isDeepStrictEqual(actual,prior),actual,control,direct,prior,failures});
 }
 for(const f of scenes){
  const dir=path.join(out,f.name);fs.mkdirSync(dir);
  for(const [width,height] of [[240,160],[390,200],[768,120]]){
   await page.setViewportSize({width,height});
   const actual=await observe(f.html,f.css);const actualPath=path.join(dir,`${width}x${height}.actual.png`);await page.screenshot({path:actualPath,animations:'disabled'});
   const control=await observe(f.html,f.literalCss);const controlPath=path.join(dir,`${width}x${height}.control.png`);await page.screenshot({path:controlPath,animations:'disabled'});
   const paintFailures=Object.entries(f.expectedPaint).filter(([id,color])=>actual[id]?.style['background-color']!==color).map(([id,color])=>({id,expected:color,actual:actual[id]?.style['background-color']}));
   const failures=[];if(!isDeepStrictEqual(actual,control))failures.push('painted-source-vs-control');if(hash(actualPath)!==hash(controlPath))failures.push('painted-source-vs-control-png');if(paintFailures.length)failures.push('paint-expectation');
   sceneRows.push({name:f.name,width,height,actual,control,actualPath,actualSha256:hash(actualPath),controlPath,controlSha256:hash(controlPath),paintFailures,failures});
  }
 }
}finally{await browser.close();}
const bindings=[formFile,sceneFile,resetFile,fileURLToPath(import.meta.url)].map(p=>({path:p,sha256:hash(p)}));
const receipt={scope:'Chrome-only grammar and computation observations for 33 currently admitted property names. Browser invalidity does not itself classify all compiler diagnostics as recoverable; valid unsupported tokens must remain separate. Native qualification is pending.',browser:'153.0.8010.12',properties:33,forms:rows.length,paintedScenes:scenes.length,paintedObservations:sceneRows.length,bindings,rows,sceneRows};
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
const failures=[...rows,...sceneRows].filter(r=>r.failures.length).map(r=>({name:r.name,width:r.width,failures:r.failures}));
console.log(JSON.stringify({forms:rows.length,paintedObservations:sceneRows.length,failures},null,2));
if(failures.length)process.exitCode=1;
