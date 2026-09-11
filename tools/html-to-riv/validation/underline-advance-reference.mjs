// Diagnostic only: compare shaped Chrome prefix widths with runtime advances.
// Prefix and suffix subtraction expose contextual kerning differences; neither
// is a universal glyph-origin API. DOM ranges report rounded ink span edges.
import {chromium} from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const font=fs.readFileSync(path.join(moduleDir,'tests/assets/NuxieJapaneseFixture-Regular.otf'));
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 await page.setContent(`<style>@font-face{font-family:Fixture;src:url(data:font/otf;base64,${font.toString('base64')})}body{margin:0}span{font:24px/40px Fixture;white-space:pre}</style><span id="text">／＿ agypqj</span>`);
 if(process.env.NUXIE_DPR_TEXT!==undefined)await page.locator('#text').evaluate((node,text)=>node.textContent=text,process.env.NUXIE_DPR_TEXT);
 await page.evaluate(()=>document.fonts.ready);
 const result=await page.evaluate(()=>{
  const element=document.getElementById('text'),node=element.firstChild;
  const context=document.createElement('canvas').getContext('2d');
  context.font='24px Fixture';context.fontKerning='normal';
  const text=node.textContent,full=context.measureText(text);
  return {text,font:context.font,total:full.width,glyphs:[...text].map((scalar,i)=>{
   const range=document.createRange();range.setStart(node,i);range.setEnd(node,i+1);
   const rect=range.getBoundingClientRect();
   return {scalar,domX:rect.x,domWidth:rect.width,prefixWidth:context.measureText(text.slice(0,i)).width,
    suffixDerivedOrigin:full.width-context.measureText(text.slice(i)).width};
  })};
 });
 console.log(JSON.stringify({browser:browser.version(),...result},null,2));
} finally {await browser.close();}
