// Independent computed-style evidence; no compiler or renderer qualification.
// Usage: node border-width-reference.mjs NEW_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';

const out = path.resolve(process.argv[2]);
assert(!fs.existsSync(out), 'Refusing to overwrite reference evidence');
const reset = fs.readFileSync(new URL('../src/reset.css', import.meta.url), 'utf8');
const widths = ['0', '0.00000001px', '0.001px', '0.1px', '0.5px', '0.99px', '1px',
  '1.5px', '1.99px', '2.25px', 'thin', 'medium', 'thick', '0.075em', '0.125rem'];
const cases = widths.map(width => ({name:width, css:`border:${width} solid #123456;font-size:20px`}));
cases.push(...[
  ['style-only', 'border-style:solid'], ['shorthand-style-only', 'border:solid'],
  ['shorthand-width-only', 'border:8px'], ['hidden', 'border:8px hidden'],
  ['current-color', 'border:8px solid currentColor;color:#123456'],
  ['width-before-font', 'border:0.075em solid;font-size:20px'],
].map(([name,css]) => ({name,css})));
const browser = await chromium.launch();
try {
  assert.equal(browser.version(), '153.0.8010.12', 'Unqualified Chrome version');
  const rows = [];
  for (const dpr of [1,2]) {
    const context = await browser.newContext({viewport:{width:390,height:320},deviceScaleFactor:dpr});
    const page = await context.newPage();
    for (const fixture of cases) {
      await page.setContent(`<!doctype html><style>${reset}\n#a{width:100px;height:80px;${fixture.css}}</style><div id=a></div>`);
      const result = await page.locator('#a').evaluate(el => {
        const s = getComputedStyle(el), r = el.getBoundingClientRect();
        return {width:s.borderTopWidth,style:s.borderTopStyle,color:s.borderTopColor,
          fontSize:s.fontSize,clientWidth:el.clientWidth,outerWidth:r.width,dpr:devicePixelRatio};
      });
      rows.push({...fixture,dpr,result});
    }
    await context.close();
  }
  fs.mkdirSync(out,{recursive:true});
  fs.writeFileSync(path.join(out,'reference.json'),JSON.stringify({browser:browser.version(),reset,cases:rows},null,2)+'\n');
  console.log(JSON.stringify({browser:browser.version(),cases:rows.length}));
} finally { await browser.close(); }
