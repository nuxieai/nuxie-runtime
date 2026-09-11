// Read-only Chrome characterization, not compiler qualification.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import crypto from 'node:crypto';
import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const samples=[
 ['unused-self',[['--good','navy'],['--a','var(--good,var(--a))']]],
 ['unused-mutual',[['--good','navy'],['--a','var(--good,var(--b))'],['--b','var(--a)']]],
 ['traversed-self',[['--a','var(--missing,var(--a))']]],
 ['traversed-mutual',[['--a','var(--missing,var(--b))'],['--b','var(--a)']]],
 ['direct-fallback-cycle',[['--a','var(--b,red)'],['--b','var(--a,blue)']]],
 ['dependent-recovers',[['--a','var(--b,red)'],['--b','var(--a,blue)'],['--c','var(--a,teal)']]],
 ['outside-self-recovers',[['--a','var(--b,teal)'],['--b','var(--b)']]],
 ['cycle-through-required',[['--a','var(--b,red)'],['--b','var(--c)'],['--c','var(--a)']]],
 ['valid-primary-breaks-cycle',[['--a','var(--b,red)'],['--b','var(--c,var(--a))'],['--c','navy']]],
 ['missing-primary-enters-cycle',[['--a','var(--b,red)'],['--b','var(--c,var(--a))']]],
 ['invalid-primary-enters-cycle',[['--a','var(--bad,var(--a))'],['--bad','var(--bad)']]],
 ['empty-primary-skips-cycle',[['--empty',''],['--a','var(--empty,var(--a))']]],
 ['unused-invalid-fallback',[['--good','navy'],['--a','var(--good,var(--missing))']]],
 ['two-cycles-dependent',[['--a','var(--b)'],['--b','var(--a)'],['--c','var(--d,var(--a))'],['--d','var(--c)'],['--e','var(--a,var(--c,teal))']]],
];
fs.writeFileSync(path.join(out,'samples.json'),JSON.stringify(samples,null,2));fs.copyFileSync(new URL(import.meta.url),path.join(out,'probe-variable-cycles.mjs'));
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');const results=[];
try{const page=await browser.newPage();for(const [name,decls] of samples)for(const reverseDeclarations of [false,true])for(const reverseReads of [false,true]){
 const entries=reverseDeclarations?[...decls].reverse():decls;const names=decls.map(([n])=>n);if(reverseReads)names.reverse();
 const css=entries.map(([n,v])=>`${n}:${v}`).join(';')+';'+decls.map(([n],i)=>`--observed-${i}:var(${n},__invalid__)`).join(';');
 await page.setContent('<div id="box"></div>');await page.locator('#box').evaluate((e,css)=>e.setAttribute('style',css),css);
 const values=await page.locator('#box').evaluate((e,names)=>{const s=getComputedStyle(e);return Object.fromEntries(names.map(n=>[n,s.getPropertyValue(n).trim()]));},[...names,...decls.map((_,i)=>`--observed-${i}`)]);
 results.push({name,reverseDeclarations,reverseReads,css,values});
}}finally{await browser.close();}
for(const [name]of samples){const rows=results.filter(r=>r.name===name);const canonical=r=>JSON.stringify(Object.entries(r.values).sort());assert(rows.every(r=>canonical(r)===canonical(rows[0])),`order-dependent ${name}`);}
const files=Object.fromEntries(fs.readdirSync(out).map(n=>[n,crypto.createHash('sha256').update(fs.readFileSync(path.join(out,n))).digest('hex')]));
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({scope:'Browser-only characterization; observed sentinel variables distinguish invalid from valid empty',browser:'153.0.8010.12',files,results},null,2));
console.log(JSON.stringify(results.filter(r=>!r.reverseDeclarations&&!r.reverseReads).map(({name,values})=>({name,values})),null,2));
