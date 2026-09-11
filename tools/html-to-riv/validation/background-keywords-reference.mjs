// https://drafts.csswg.org/css-backgrounds/#propdef-background
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const invalid=['repeat repeat repeat','repeat-x no-repeat','repeat-y repeat-x','repeat red repeat','fixed scroll','local local','border-box padding-box content-box','none repeat repeat repeat','repeat, fixed scroll','red repeat, none'];
const valid=['repeat','repeat no-repeat','repeat-x','fixed','border-box padding-box','border-box red padding-box','repeat, fixed','none repeat no-repeat fixed border-box padding-box red'];
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const results=await page.evaluate(({invalid,valid})=>[...invalid,...valid].map(value=>{
  document.body.innerHTML='<div id="a"></div><div id="b"></div>';
  const a=document.getElementById('a'),b=document.getElementById('b');
  a.style.cssText=`--x:${value};background:green;background:var(--x,blue)`;
  b.style.background='unset';
  return {value,accepted:CSS.supports('background',value),expectedAccepted:valid.includes(value),actual:getComputedStyle(a).background,expected:getComputedStyle(b).background};
 }),{invalid,valid});
 const output='output/playwright/html-to-riv/background-keywords-reference';
 fs.mkdirSync(output,{recursive:true});
 fs.writeFileSync(`${output}/results.json`,JSON.stringify({browser:browser.version(),results},null,2));
 for(const row of results){assert.equal(row.accepted,row.expectedAccepted,JSON.stringify(row));if(!row.accepted)assert.equal(row.actual,row.expected,JSON.stringify(row));}
 console.log(`${results.length}/${results.length} background keyword references pass`);
} finally {await browser.close();}
