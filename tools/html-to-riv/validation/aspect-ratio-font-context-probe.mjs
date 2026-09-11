import fs from 'node:fs';
import {chromium} from '@playwright/test';
const output=process.argv[2];
if(fs.existsSync(output))throw new Error('Refusing to overwrite probe evidence');
const browser=await chromium.launch();
try {
 const page=await browser.newPage();const rows=[];
 for(const size of [0.000001,0.001,17.808,9999,10000,10001,16384,100000,1000000]) {
  await page.setContent(`<div id=a style="font-size:${size}px;width:120px;aspect-ratio:calc(16px / 1em)"></div>`);
  rows.push(await page.$eval('#a',(el)=>{const s=getComputedStyle(el),b=el.getBoundingClientRect();return {authored:el.style.fontSize,fontSize:s.fontSize,aspectRatio:s.aspectRatio,width:b.width,height:b.height}}));
 }
 fs.writeFileSync(output,JSON.stringify({browser:browser.version(),rows},null,2)+'\n');
}finally{await browser.close()}
