// https://drafts.csswg.org/css-variables/#invalid-variables
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const results=await page.evaluate(()=>{
  const rows=[];
  for(const property of ['width','padding','margin','gap','border-radius','background','color','font-size','line-height','font','text-decoration','display','box-sizing']) {
   for(const value of ['(20px)','[20px]','{20px}','calc(20px) [20px]','[20px] calc(20px)']) {
    document.body.innerHTML='<div style="width:300px;font:20px/1.5 Inter;color:rgb(1,2,3)"><div id="a">Text</div><div id="b">Text</div></div>';
    const a=document.getElementById('a'),b=document.getElementById('b');
    a.style.setProperty('--x',value);a.style.setProperty(property,'var(--x,12px)');b.style.setProperty(property,'unset');
    rows.push({property,value,accepted:CSS.supports(property,value),actual:getComputedStyle(a).getPropertyValue(property),expected:getComputedStyle(b).getPropertyValue(property)});
   }
  }
  return rows;
 });
 const output='output/playwright/html-to-riv/substitution-block-reference';fs.mkdirSync(output,{recursive:true});
 fs.writeFileSync(`${output}/results.json`,JSON.stringify({browser:browser.version(),results},null,2));
 for(const row of results){assert.equal(row.accepted,false,JSON.stringify(row));assert.equal(row.actual,row.expected,JSON.stringify(row));}
 console.log(`${results.length}/${results.length} bare block reset references pass`);
} finally {await browser.close();}
