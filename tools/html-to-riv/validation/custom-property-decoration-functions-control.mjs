import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
const browser = await chromium.launch();
try {
 const page = await browser.newPage();
 const rows = await page.evaluate(() => {
  const rows=[];
  for(const value of ["underline hsl(20 30 40)", "underline rgb(1 2)", "rgb(1, 2, 3, 4, 5) underline", "underline hsl(20 30)", "underline rgb(1 2 3) red", "hsl(20 30% 40%) underline rgb(1 2 3)", "underline underline rgba(1,2,3,.5)", "solid dashed hsl(20 30% 40%)", "underline rgb(1 2 3) 2px 3px", "underline rgb(1 2 3) 2px", "2px hsla(20,30%,40%,.5) underline", "underline color(display-p3 1 0 0)", "underline rgb(none 0 0)", "underline rgb(from red r g b)", "underline rgb(calc(1 + 2) 0 0)", "underline rgb(1 2 3) calc(2px + 1px)"]) {
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
