// CSS Values distinguishes angles, times, frequencies, resolutions and flex
// fractions from lengths: https://drafts.csswg.org/css-values-4/
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
const output=path.resolve(process.env.NUXIE_HTML_REVIEW_DIR??'output/playwright/html-to-riv/non-length-dimensions-reference');
fs.mkdirSync(output,{recursive:true});
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const results=await page.evaluate(()=>{
  const rows=[];
  for(const unit of ['deg','grad','rad','turn','s','ms','Hz','kHz','dpi','dpcm','dppx','x','fr']) {
   for(const property of ['width','min-width','max-width','padding','margin','gap','border-radius','flex-basis','font-size','line-height','letter-spacing','word-spacing','text-decoration-thickness','text-underline-offset']) {
    document.body.innerHTML='<div style="width:300px;font-size:20px;line-height:1.5;letter-spacing:3px;word-spacing:4px;text-underline-offset:5px"><div id="actual">Text</div><div id="expected">Text</div></div>';
    const actual=document.getElementById('actual'),expected=document.getElementById('expected');
    actual.style.cssText=`--bad:2${unit};${property}:12px;${property}:var(--bad,8px)`;
    expected.style.setProperty(property,'unset');
    rows.push({property,unit,directAccepted:CSS.supports(property,`2${unit}`),actual:getComputedStyle(actual).getPropertyValue(property),expected:getComputedStyle(expected).getPropertyValue(property)});
   }
  }
  return rows;
 });
 fs.writeFileSync(path.join(output,'results.json'),JSON.stringify({browser:browser.version(),results},null,2));
 for(const row of results){assert.equal(row.directAccepted,false,JSON.stringify(row));assert.equal(row.actual,row.expected,JSON.stringify(row));}
 console.log(`${results.length}/${results.length} non-length dimension reset references pass`);
} finally {await browser.close();}
