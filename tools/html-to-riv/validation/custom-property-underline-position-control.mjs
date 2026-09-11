import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  const rows = await page.evaluate(() => {
    const rows = [];
    for (const value of ['auto', 'under', 'left', 'right', 'from-font', 'under left', 'right under', 'from-font left', 'auto left', 'auto under', 'left right', 'under under', 'from-font under', 'banana', '2px', '0', "'auto'"]) {
      document.body.innerHTML = '<div id="parent"><div id="child">agypqj</div></div>';
      const parent = document.querySelector('#parent'), child = document.querySelector('#child');
      parent.style.textUnderlinePosition = 'auto';
      child.style.setProperty('--x', value);
      child.style.textUnderlinePosition = 'var(--x)';
      rows.push({value, supports: CSS.supports('text-underline-position', value), computed: getComputedStyle(child).textUnderlinePosition});
    }
    return rows;
  });
  for (const row of rows) if (!row.supports) assert.equal(row.computed, 'auto');
  console.log(JSON.stringify({browser: browser.version(), rows}, null, 2));
} finally { await browser.close(); }
