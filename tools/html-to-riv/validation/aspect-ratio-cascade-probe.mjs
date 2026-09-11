import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);
if(fs.existsSync(out))throw new Error('Refusing overwrite');
const values=['-1','1 / -2','auto auto','2 / 3 / 4','calc(1+2)','calc(1 + )','min()','clamp(1, 2)','calc(1px)','calc(1px / 1px)','sin(1)','garbage'];
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const rows=await page.evaluate(values=>values.map(value=>{
  const e=document.createElement('div');e.style.cssText=`width:120px;--ratio:${value};aspect-ratio:2;aspect-ratio:var(--ratio)`;document.body.append(e);
  const row={value,accepted:CSS.supports('aspect-ratio',value),computed:getComputedStyle(e).aspectRatio,height:e.getBoundingClientRect().height};e.remove();return row;
 }),values);
 fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,'probe.json'),JSON.stringify({browser:browser.version(),rows},null,2)+'\n');console.log(JSON.stringify(rows));
} finally {await browser.close();}
