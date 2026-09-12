// Pinned Chrome grammar/computation characterization; native qualification is separate.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const fixturesPath=fileURLToPath(new URL('./public-empty-variable-forms.json',import.meta.url));
const resetPath=fileURLToPath(new URL('../src/reset.css',import.meta.url));
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const fixtures=JSON.parse(fs.readFileSync(fixturesPath));assert.equal(fixtures.length,231);
const reset=fs.readFileSync(resetPath,'utf8');
const properties=[...new Set(fixtures.map(f=>f.property))];assert.equal(properties.length,33);
fs.copyFileSync(fixturesPath,path.join(out,'forms.json'));fs.copyFileSync(resetPath,path.join(out,'reset.css'));
fs.copyFileSync(fileURLToPath(import.meta.url),path.join(out,'driver.mjs'));
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');
const rows=[],nonCssWhitespace=[];
try {
 const page=await browser.newPage({viewport:{width:240,height:160},deviceScaleFactor:1,colorScheme:'light',locale:'en-US'});
 for(const fixture of fixtures){
  const observe=async(css)=>{
   await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${fixture.html}`);
   return page.evaluate(properties=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{
    const s=getComputedStyle(e);const {x,y,width,height}=e.getBoundingClientRect();
    return [e.id,{style:Object.fromEntries(properties.map(p=>[p,s.getPropertyValue(p)])),box:{x,y,width,height}}];
   })),properties);
  };
  const actual=await observe(fixture.css),control=await observe(fixture.literalCss);
  assert.deepEqual(actual,control,fixture.name);
  rows.push({name:fixture.name,property:fixture.property,form:fixture.form,compilerExpectedAccepted:fixture.accepted,actual,control,exactComputedAndGeometry:true});
 }
 for(const codePoint of [0x00a0,0x000b]){
  await page.setContent(`<div id="a" style="--empty:${String.fromCodePoint(codePoint)};--observed:var(--empty,900px)"></div>`);
  const observation=await page.locator('#a').evaluate(e=>{const s=getComputedStyle(e);return {primary:[...s.getPropertyValue('--empty')].map(c=>c.codePointAt(0)),observer:[...s.getPropertyValue('--observed')].map(c=>c.codePointAt(0))};});
  assert.deepEqual(observation.primary,[codePoint]);assert.deepEqual(observation.observer,[codePoint]);
  nonCssWhitespace.push({codePoint,...observation});
 }
}finally{await browser.close();}
const bindings=[fixturesPath,resetPath,fileURLToPath(import.meta.url)].map(p=>({path:p,sha256:hash(p)}));
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({scope:'Pinned Chrome-only computed-style and DOM-geometry equality for all33 admitted names across seven empty token forms at240x160. This does not establish native rendering support for every combination.',browser:'153.0.8010.12',properties:33,forms:7,cases:231,bindings,nonCssWhitespace,rows},null,2)+'\n');
console.log(JSON.stringify({browser:'153.0.8010.12',cases:rows.length,exactComputedAndGeometry:rows.filter(r=>r.exactComputedAndGeometry).length}));
