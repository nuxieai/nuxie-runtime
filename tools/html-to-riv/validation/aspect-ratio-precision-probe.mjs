import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);
if(fs.existsSync(out))throw new Error('Refusing overwrite');
const values=['calc(16777217 - 16777216)','calc(100000001 - 100000000)','calc(0.3 - 0.2)','calc(1e40 / 1e40)','calc(1e-50 / 1e-50)','calc(1e-20 * 1e-20)','calc(1e-40)','calc(1e-38)','calc(1e-37)','calc(1 / 3 * 3)','calc(pi)','calc(e)'];
values.push(...Array.from({length:36},(_,i)=>`calc(1e-${i+1})`));
// SizeF clamps each float component at eight float epsilons. Probe both
// sides, float-rounding neighbors, and equal components before division.
for (const value of ['0.0000009536742595628311','0.00000095367431640625',
 '0.0000009536743732496689','0.0000009536744300930877','1e-7','1e-6']) {
 values.push(value, `calc(${value})`, `${value} / ${value}`,
  `calc(${value}) / calc(${value})`, `1 / calc(${value})`);
}
values.push('1 / 1048576', 'calc(1 / 1048576)', '1 / 1000000', 'calc(1 / 1000000)', '2 / 2097152', '0.015625 / 16384');
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const rows=await page.evaluate(values=>values.map(value=>{
  const e=document.createElement('div');e.style.cssText=`width:120px;aspect-ratio:${value}`;document.body.append(e);
  const row={value,accepted:CSS.supports('aspect-ratio',value),computed:getComputedStyle(e).aspectRatio,height:e.getBoundingClientRect().height};e.remove();return row;
 }),values);
 fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,'probe.json'),JSON.stringify({browser:browser.version(),rows},null,2)+'\n');console.log(JSON.stringify(rows));
} finally {await browser.close();}
