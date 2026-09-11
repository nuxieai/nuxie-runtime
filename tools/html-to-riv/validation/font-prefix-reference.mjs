// https://drafts.csswg.org/css-fonts/#font-prop
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const invalid=['italic italic 20px Inter','italic oblique 20px Inter','small-caps small-caps 20px Inter','condensed expanded 20px Inter','italic bold 700 20px Inter','normal normal normal italic bold 20px Inter','italic 20px','italic 20px/ Inter','italic medium/2','small-caps -2px Inter','oblique 91deg 20px Inter','oblique 20deg italic 20px Inter','oblique 20deg 30deg 20px Inter'];
const valid=['italic 20px Inter','oblique 20px Inter','oblique 20deg 20px Inter','oblique -0.25turn 20px Inter','oblique 50grad 20px Inter','small-caps 20px Inter','condensed 20px Inter','normal normal normal italic 20px Inter','italic small-caps bold condensed 20px/2 Inter','italic medium Inter','normal italic normal normal 20px Inter'];
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const results=await page.evaluate(({invalid,valid})=>[...invalid,...valid].map(value=>{
  document.body.innerHTML='<div style="font:20px/1.5 Inter"><div id="a">Text</div><div id="b">Text</div></div>';
  const a=document.getElementById('a'),b=document.getElementById('b');
  a.style.cssText=`--x:${value};font:32px/2 Missing;font:var(--x,24px Missing)`;
  b.style.font='unset';
  const props=['fontStyle','fontVariant','fontWeight','fontStretch','fontSize','lineHeight','fontFamily'];
  return {value,accepted:CSS.supports('font',value),expectedAccepted:valid.includes(value),actual:props.map(p=>getComputedStyle(a)[p]),expected:props.map(p=>getComputedStyle(b)[p])};
 }),{invalid,valid});
 const output='output/playwright/html-to-riv/font-prefix-reference';fs.mkdirSync(output,{recursive:true});
 fs.writeFileSync(`${output}/results.json`,JSON.stringify({browser:browser.version(),results},null,2));
 for(const row of results){assert.equal(row.accepted,row.expectedAccepted,JSON.stringify(row));if(!row.accepted)assert.deepEqual(row.actual,row.expected,JSON.stringify(row));}
 console.log(`${results.length}/${results.length} font prefix references pass`);
} finally {await browser.close();}
