// Preserved residual nonempty-token disagreement; outside empty-value qualification.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import crypto from 'node:crypto';import{spawnSync}from'node:child_process';import{chromium}from'@playwright/test';
const out=path.resolve(process.argv[2]),compiler=path.resolve(process.argv[3]),priorCompiler=path.resolve(process.argv[4]);assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const html='<div id=p><div id=a></div></div>';
const base='#p{width:120px;height:80px;color:navy}#a{width:40px;height:20px;background:currentColor}';
const cases=[{name:'nbsp-around-color',css:base+'#a{--e:\u00a0red\u00a0;color:var(--e)}'},{name:'literal-red',css:base+'#a{color:red}'},{name:'literal-unset',css:base+'#a{color:unset}'}];
const rows=[];const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');
try{const page=await browser.newPage();for(const sample of cases){
 const request=path.join(out,sample.name+'.json');fs.writeFileSync(request,JSON.stringify({html,css:sample.css,width:240,height:160},null,2));
 const compiled={};for(const[label,binary]of[['current',compiler],['prior',priorCompiler]]){
  const riv=path.join(out,sample.name+'.'+label+'.riv');const result=spawnSync(binary,[request,riv],{encoding:'utf8'});assert.ifError(result.error);assert.equal(result.status,0,result.stderr);
  compiled[label]={compilerSha256:hash(binary),rivSha256:hash(riv),mapSha256:hash(riv.replace(/\.riv$/,'.map.json'))};
 }
 await page.setContent(`<style>${base}</style>${html}`);await page.addStyleTag({content:sample.css});
 const observed=await page.locator('#a').evaluate(e=>{const s=getComputedStyle(e);return {color:s.color,background:s.backgroundColor,customCodePoints:[...s.getPropertyValue('--e')].map(c=>c.codePointAt(0))};});
 rows.push({...sample,requestSha256:hash(request),compiled,observed});
}}finally{await browser.close();}
for(const label of ['current','prior'])assert.equal(rows[0].compiled[label].rivSha256,rows[1].compiled[label].rivSha256);
assert.equal(rows[0].observed.color,'rgb(0, 0, 128)');assert.equal(rows[1].observed.color,'rgb(255, 0, 0)');assert.equal(rows[0].observed.color,rows[2].observed.color);
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({scope:'Preserved pre-existing compiler/browser mismatch for nonempty NBSP-wrapped color tokens. Current and pre-empty-recovery compiler emit exact literal-red bytes; Chrome computes inherited navy. No native-render qualification or runtime limitation claim.',browser:'153.0.8010.12',rows},null,2)+'\n');
console.log('preserved nonempty Unicode-boundary mismatch in both old and new compiler');
