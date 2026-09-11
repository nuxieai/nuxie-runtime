// https://drafts.csswg.org/css-backgrounds/#propdef-background-position
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const cases=JSON.parse(fs.readFileSync(new URL('./background-position-cases.json',import.meta.url)));
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const results=await page.evaluate(({invalid,valid})=>[...invalid,...valid].map(value=>{
  document.body.innerHTML='<div id="a"></div><div id="b"></div>';
  const a=document.getElementById('a'),b=document.getElementById('b');
  a.style.cssText=`--x:${value};background:green;background:var(--x,blue)`;b.style.background='unset';
  return {value,accepted:CSS.supports('background',value),expectedAccepted:valid.includes(value),actual:getComputedStyle(a).background,expected:getComputedStyle(b).background};
 }),cases);
 const output='output/playwright/html-to-riv/background-position-reference';fs.mkdirSync(output,{recursive:true});fs.writeFileSync(`${output}/results.json`,JSON.stringify({browser:browser.version(),results},null,2));
 for(const row of results){assert.equal(row.accepted,row.expectedAccepted,JSON.stringify(row));if(!row.accepted)assert.equal(row.actual,row.expected,JSON.stringify(row));}
 console.log(`${results.length}/${results.length} background position/size references pass`);
} finally {await browser.close();}
