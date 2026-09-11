// Chrome syntax/computed-value control for invalid substituted metrics.
import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  const rows = await page.evaluate(() => {
    const rows = [];
    for (const property of ['text-decoration-thickness', 'text-underline-offset']) {
      for (const value of ['-2px', '-10%', '2', '0', 'from-font', 'thin', 'banana', '2px 3px']) {
        document.body.innerHTML = '<div id="parent"><div id="child">agypqj</div></div>';
        const parent = document.querySelector('#parent'), child = document.querySelector('#child');
        parent.style.setProperty(property, '4px');
        child.style.setProperty('--x', value);
        child.style.setProperty(property, 'var(--x)');
        rows.push({ property, value, supports: CSS.supports(property, value), computed: getComputedStyle(child).getPropertyValue(property) });
      }
    }
    return rows;
  });
  for (const row of rows) {
    if (!row.supports) assert.equal(row.computed, row.property === 'text-decoration-thickness' ? 'auto' : '4px');
    if (row.value.startsWith('-')) assert.equal(row.supports, true);
  }
  console.log(JSON.stringify({ browser: browser.version(), rows }, null, 2));
} finally { await browser.close(); }
