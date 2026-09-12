// Characterize typed-invalid substitutions independently of compiler admission.
// This is a pinned-browser audit, not a shipping parser or native qualification.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {chromium} from '@playwright/test';

const module=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const out=path.resolve(process.argv[2]??'');
assert(process.argv[2], 'Usage: node SCRIPT FRESH_OUTPUT');
fs.mkdirSync(out,{recursive:false});
fs.copyFileSync(fileURLToPath(import.meta.url),path.join(out,'driver.mjs'));
const sha=f=>crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex');
const compilerSource=path.join(module,'output/public-variable-recovery-r1/html-to-riv');
const compiler=path.join(out,'html-to-riv');fs.copyFileSync(compilerSource,compiler);
const resetPath=path.join(module,'src/reset.css'),reset=fs.readFileSync(resetPath,'utf8');
fs.writeFileSync(path.join(out,'reset.css'),reset);
const html='<div id=p><div id=a><div id=c></div><div id=d></div></div><div id=b></div></div>';
const base='#p{width:200px;height:120px;background:teal;color:navy;font-size:24px}#a{width:80px;height:50px;background:coral;color:red;font-size:16px;order:-2}#b{width:24px;height:16px;background:gold;order:-1}#c,#d{width:12px;height:8px;background:currentColor}';
const ordinary=['width','height','min-width','min-height','max-width','max-height','font-size','color','background','background-color','display','flex-direction','flex','flex-grow','flex-shrink','flex-basis','order','align-self','justify-content','margin','margin-left','margin-top','margin-right','margin-bottom','padding','padding-left','padding-top','padding-right','padding-bottom','gap','row-gap','column-gap'];
const cases=[];
for(const property of ordinary){
 for(const kind of ['empty-fallback','empty-primary','whitespace-primary']){
  const decl=kind==='empty-fallback'?`${property}:var(--missing,)`:kind==='empty-primary'?`--x:;${property}:var(--x,40px)`:`--x:/**/ ;${property}:var(--x,40px)`;
  cases.push({name:`${property}-${kind}`,property,value:'',declarations:decl,invalid:true,group:'empty'});
 }
}
for(const [index,[property,value]] of [
 ['width','red'],['height','2deg'],['min-width','"40px"'],['max-height','10/**/px'],
 ['font-size','auto'],['color','20px'],['background-color','20px'],
 ['order','1px'],['order','1.5'],['flex-grow','10%'],['flex-shrink','auto'],
 ['flex-basis','red'],['padding','auto'],['gap','auto'],['align-self','12px'],['justify-content','12px'],
].entries()) cases.push({name:`wrong-type-${property}-${index}`,property,value,declarations:`--x:${value};${property}:var(--x)`,invalid:true,group:'typed-invalid'});
for(const [name,property,value] of [
 ['zero-width','width','0'],['large-width','width','1000001px'],['calc-width','width','calc(20px + 10px)'],
 ['negative-margin','margin-left','-2px'],['intrinsic-width','width','min-content'],['font-keyword','font-size','large'],
 ['integer-order','order','3'],['color-function','color','color(display-p3 1 0 0)'],['display-block','display','block'],
 ['background-position','background','20px'],
 ['shrink-one','flex-shrink','1'],['empty-unused-fallback','width','40px'],
]) cases.push({name:`valid-${name}`,property,value,declarations:name==='empty-unused-fallback'?`--x:40px;${property}:var(--x,)`:`--x:${value};${property}:var(--x)`,invalid:false,group:'valid-control'});
assert.equal(new Set(cases.map(c=>c.name)).size,cases.length,'Unique case identities');
fs.writeFileSync(path.join(out,'cases.json'),JSON.stringify(cases,null,2)+'\n');
const properties=[...new Set([...ordinary,'box-sizing','background-position'])];
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');
const rows=[];
try{
 const page=await browser.newPage({viewport:{width:240,height:160},deviceScaleFactor:1,colorScheme:'light'});
 const observe=async(css)=>{
  await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
  return page.evaluate(properties=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{
   const s=getComputedStyle(e),{x,y,width,height}=e.getBoundingClientRect();
   return [e.id,{style:Object.fromEntries(properties.map(p=>[p,s.getPropertyValue(p)])),box:{x,y,width,height}}];
  })),properties);
 };
 for(const sample of cases){
  const dir=path.join(out,sample.name);fs.mkdirSync(dir);
  const css=base+`#a{${sample.declarations}}`,request={html,css,width:240,height:160};
  fs.writeFileSync(path.join(dir,'request.json'),JSON.stringify(request,null,2)+'\n');
  const actual=await observe(css);
  // Literal validity is a characterization oracle only, never compiler logic.
  const literalSupported=await page.evaluate(({property,value})=>CSS.supports(property,value),sample);
  assert.equal(literalSupported,!sample.invalid,`${sample.name}: literal grammar`);
  let unset=null,unsetCompile=null;
  if(sample.invalid){
   const controlCss=base+`#a{${sample.property}:unset}`;
   unset=await observe(controlCss);assert.deepEqual(actual,unset,`${sample.name}: invalid substitution computes as unset`);
   const control={...request,css:controlCss},controlPath=path.join(dir,'control.json');
   fs.writeFileSync(controlPath,JSON.stringify(control,null,2)+'\n');
   const result=spawnSync(compiler,[controlPath,path.join(dir,'control.riv')],{encoding:'utf8'});assert.ifError(result.error);assert.equal(result.signal,null);
   unsetCompile={status:result.status,diagnostics:result.status===0?[]:JSON.parse(result.stderr)};
  }
  const result=spawnSync(compiler,[path.join(dir,'request.json'),path.join(dir,'scene.riv')],{encoding:'utf8'});assert.ifError(result.error);assert.equal(result.signal,null);
  if(result.status!==0){assert(!fs.existsSync(path.join(dir,'scene.riv')));assert(!fs.existsSync(path.join(dir,'scene.map.json')));}
  const row={...sample,literalSupported,actual,unset,unsetCompile,compilerStatus:result.status,diagnostics:result.status===0?[]:JSON.parse(result.stderr)};
  fs.writeFileSync(path.join(dir,'observation.json'),JSON.stringify(row,null,2)+'\n');
  rows.push(row);
 }
}finally{await browser.close();}
const counts={cases:rows.length,invalid:rows.filter(r=>r.invalid).length,empty:rows.filter(r=>r.group==='empty').length,typedInvalid:rows.filter(r=>r.group==='typed-invalid').length,invalidWithAdmittedUnset:rows.filter(r=>r.invalid&&r.unsetCompile.status===0).length,validControls:rows.filter(r=>!r.invalid).length};
const receipt={scope:'Pinned Chrome property-grammar and computed-unset characterization; frozen compiler admission. No native pixels or public feature admission.',browser:'153.0.8010.12',counts,compilerSource,compilerSha256:sha(compiler),driverSha256:sha(fileURLToPath(import.meta.url)),resetSha256:sha(resetPath),casesSha256:sha(path.join(out,'cases.json')),rows};
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
console.log(JSON.stringify(counts));
