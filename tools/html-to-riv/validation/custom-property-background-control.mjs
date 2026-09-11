import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const rows=await page.evaluate(()=>{
  const rows=[];
  for(const value of ['banana',"'red'",'red blue','none none','red none blue','#ggg','#123 #456','red, blue',',red','red,','none,,none','red none','none red','none, red','left red','red center / cover','repeat red','border-box red','url(example.png)']) {
   document.body.innerHTML='<div id="a"></div>';
   const a=document.querySelector('#a');a.style.cssText='--x:'+value+';background:green;background:var(--x,blue)';
   rows.push({value,supports:CSS.supports('background',value),color:getComputedStyle(a).backgroundColor});
  }
  return rows;
 });
 for(const row of rows) if(!row.supports) assert.equal(row.color,'rgba(0, 0, 0, 0)');
 console.log(JSON.stringify({browser:browser.version(),rows},null,2));
} finally {await browser.close();}
