// Recheck the independent logical-text reference against installed Chromium.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';
const reference=JSON.parse(fs.readFileSync(new URL('./capitalize-reference.json',import.meta.url)));
const browser=await chromium.launch();
try {
  const page=await browser.newPage();
  await page.setContent('<style>p{white-space:pre-wrap;text-transform:capitalize}</style>');
  const actual=await page.evaluate(cases=>cases.map(({source})=>{
    const element=document.createElement('p');element.textContent=source;
    document.body.append(element);return {source,rendered:element.innerText};
  }),reference.cases);
  assert.deepEqual(actual,reference.cases);
  console.log(`${actual.length} capitalization references match Chromium ${browser.version()}`);
} finally { await browser.close(); }
