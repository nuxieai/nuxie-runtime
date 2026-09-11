// CSS Color/Image values and CSS Values numeric math are distinct grammar types.
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const cases=JSON.parse(fs.readFileSync(new URL('./function-type-cases.json',import.meta.url)));
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const results=await page.evaluate(({groups})=>{
  const rows=[];
  for(const {properties,values} of groups)for(const property of properties)for(const value of values){
   document.body.innerHTML='<div style="width:300px;font:20px/1.5 Inter;color:rgb(1,2,3)"><div id="a">Text</div><div id="b">Text</div></div>';
   const a=document.getElementById('a'),b=document.getElementById('b');
   a.style.setProperty('--x',value);a.style.setProperty(property,'var(--x,12px)');b.style.setProperty(property,'unset');
   rows.push({property,value,accepted:CSS.supports(property,value),actual:getComputedStyle(a).getPropertyValue(property),expected:getComputedStyle(b).getPropertyValue(property)});
  }return rows;
 },cases);
 const output='output/playwright/html-to-riv/function-type-reference';fs.mkdirSync(output,{recursive:true});fs.writeFileSync(`${output}/results.json`,JSON.stringify({browser:browser.version(),results},null,2));
 for(const row of results){assert.equal(row.accepted,false,JSON.stringify(row));assert.equal(row.actual,row.expected,JSON.stringify(row));}
 console.log(`${results.length}/${results.length} function type reset references pass`);
} finally {await browser.close();}
