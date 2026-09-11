import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);
if(fs.existsSync(out))throw new Error('Refusing overwrite');
const values=['calc(infinity)','calc(-infinity)','calc(NaN)','calc(1 / 0)',
 '1 / calc(infinity)', 'calc(infinity) / calc(infinity)', 'calc(infinity / infinity)',
 '1e40','calc(1e40)','1 / 1e40','1e40 / 1e40','1e400','calc(1e400)',
 'calc(infinity - infinity)','calc(0 * infinity)','calc(1e308 * 1e308)',
 '3.4028234663852886e38','3.4028236e38','calc(3.4028236e38)',
 'calc(max(infinity, 1))','calc(min(infinity, 2))','clamp(1, infinity, 2)'];
values.push('calc(1e400 / 1e400)', 'calc(1e400 - 1e400)', 'calc(1e400 / infinity)');
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const rows=await page.evaluate(values=>values.map(value=>{
  const e=document.createElement('div');e.style.cssText=`width:120px;aspect-ratio:${value}`;document.body.append(e);
  const row={value,accepted:CSS.supports('aspect-ratio',value),computed:getComputedStyle(e).aspectRatio,height:e.getBoundingClientRect().height};e.remove();return row;
 }),values);
 fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,'probe.json'),JSON.stringify({browser:browser.version(),rows},null,2)+'\n');console.log(JSON.stringify(rows));
} finally {await browser.close();}
