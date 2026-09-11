// Recheck language-sensitive logical text independently of compiler output.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
const reference=JSON.parse(fs.readFileSync(new URL('./locale-reference.json',import.meta.url)));
const browser=await chromium.launch();
try {
  const page=await browser.newPage();
  await page.setContent('<style>p{white-space:pre-wrap}</style>');
  const actual=await page.evaluate(cases=>cases.map(({lang,mode,source})=>{
    const element=document.createElement('p');element.lang=lang;
    element.style.textTransform=mode;element.textContent=source;
    document.body.append(element);return {lang,mode,source,rendered:element.innerText};
  }),reference.cases);
  assert.deepEqual(actual,reference.cases);
  console.log(`${actual.length} language casing references match Chromium ${browser.version()}`);
} finally { await browser.close(); }
