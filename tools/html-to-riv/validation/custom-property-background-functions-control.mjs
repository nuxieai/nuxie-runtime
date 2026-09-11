import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const rows=await page.evaluate(()=>{
  const rows=[];
  for(const value of ["rgb(1 2 3) red", "none rgb(1 2)", "none hsl(20 30)", "rgb(1 2 3), none", "none, rgb(1 2 3), none", "rgb(1 2 3) hsl(20 30 40)", "none none rgb(1 2 3)", "rgb(1 2 3)", "hsl(20 30 40)", "rgba(20,30,40,.5)", "none rgb(1 2 3)", "none, hsl(20 30 40)", "left rgb(1 2 3)", "linear-gradient(red,blue)", "rgb(from red r g b)", "color(display-p3 1 0 0)"]) {
   document.body.innerHTML='<div id="a"></div>';
   const a=document.querySelector('#a');a.style.cssText='--x:'+value+';background:green;background:var(--x,blue)';
   rows.push({value,supports:CSS.supports('background',value),color:getComputedStyle(a).backgroundColor});
  }
  return rows;
 });
 for(const row of rows) if(!row.supports) assert.equal(row.color,'rgba(0, 0, 0, 0)');
 console.log(JSON.stringify({browser:browser.version(),rows},null,2));
} finally {await browser.close();}
