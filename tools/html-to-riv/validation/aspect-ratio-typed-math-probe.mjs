import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);
if(fs.existsSync(out))throw new Error('Refusing overwrite');
const values=['calc(1px)','calc(1px + 1px)','calc(1px * 2)','calc(1px / 2)',
 'calc(1px * 1px)','calc(1px / 1px)','calc(1px + 1)','calc(1px - 1px)',
 'calc(0 * 1px)','calc(1px / 0px)','min(1px, 2px)','clamp(1px, 2px, 3px)',
 'clamp(1, 2px, 3)','calc(1% / 1%)','calc(1em / 1px)','calc(1s / 1s)',
 'calc(1px / 1s)','calc(1px * 1s / 1px / 1s)'];
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const rows=await page.evaluate(values=>values.map(value=>{
  const e=document.createElement('div');e.style.cssText=`width:120px;--ratio:${value};aspect-ratio:2;aspect-ratio:var(--ratio)`;document.body.append(e);
  const row={value,accepted:CSS.supports('aspect-ratio',value),computed:getComputedStyle(e).aspectRatio,height:e.getBoundingClientRect().height};e.remove();return row;
 }),values);
 fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,'probe.json'),JSON.stringify({browser:browser.version(),rows},null,2)+'\n');console.log(JSON.stringify(rows));
} finally {await browser.close();}
