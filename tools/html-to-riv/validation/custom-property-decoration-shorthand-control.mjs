import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
const browser = await chromium.launch();
try {
 const page = await browser.newPage();
 const rows = await page.evaluate(() => {
  const rows=[];
  for(const value of ['banana','underline underline','none underline','underline red blue','solid dashed','underline 2px 3px','underline 2','grammar-error underline',"'underline'",'underline overline','underline dashed','underline -2px','underline 1vh','spelling-error red']) {
   document.body.innerHTML='<div id="a">agypqj</div>';
   const a=document.querySelector('#a');
   a.style.cssText='text-decoration:line-through red 3px;text-underline-offset:4px;--x:'+value+';text-decoration:var(--x)';
   const s=getComputedStyle(a);
   rows.push({value,supports:CSS.supports('text-decoration',value),line:s.textDecorationLine,color:s.textDecorationColor,thickness:s.textDecorationThickness,offset:s.textUnderlineOffset});
  }
  return rows;
 });
 for(const row of rows) if(!row.supports) {
  assert.equal(row.line,'none');assert.equal(row.thickness,'auto');assert.equal(row.offset,'4px');
 }
 console.log(JSON.stringify({browser:browser.version(),rows},null,2));
} finally {await browser.close();}
