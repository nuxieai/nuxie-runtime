// Capture Chrome admission/computed values independently of native compilation.
import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);
if(fs.existsSync(out)) throw new Error('Refusing to overwrite evidence');
const cases=JSON.parse(fs.readFileSync(new URL('./aspect-ratio-math-cases.json',import.meta.url)));
const invalid=['calc(1px)','calc(50%)','calc(1+2)','calc(1 + )','min()','clamp(1, 2)','calc(2) / -1'];
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const rows=await page.evaluate(values=>values.map(value=>{
  const e=document.createElement('div');e.style.aspectRatio=value;document.body.append(e);
  const result={value,accepted:CSS.supports('aspect-ratio',value),authored:e.style.aspectRatio,computed:getComputedStyle(e).aspectRatio};e.remove();return result;
 }),[...cases.map(c=>c.ratioValue),...invalid]);
 fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,'probe.json'),JSON.stringify({browser:browser.version(),rows},null,2)+'\n');console.log(JSON.stringify(rows,null,2));
} finally {await browser.close();}
