// L08 realistic composition references.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
const here = path.dirname(fileURLToPath(import.meta.url));
const reset = fs.readFileSync(path.join(here, '../src/reset.css'), 'utf8');
const font = fs.readFileSync(path.join(here, '../tests/assets/Inter-Regular.ttf')).toString('base64');
const fixtures = JSON.parse(fs.readFileSync(path.join(here, 'independent-flex-compositions.json'), 'utf8'));
const out = path.resolve('output/playwright/html-to-riv/independent-flex-composition-oracle');
fs.mkdirSync(out, {recursive:true});
const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  const cases = [];
  for (const fixture of fixtures) {
    const viewports = [];
    for (const width of [240, 390, 768]) {
      await page.setViewportSize({width, height:320});
      await page.setContent(`<!doctype html><style>${reset}\n${fixture.font ? `@font-face{font-family:Inter;src:url(data:font/ttf;base64,${font})}` : ''}\n${fixture.css}</style>${fixture.html}`);
      await page.evaluate(() => document.fonts.ready);
      const boxes = await page.evaluate(() => Object.fromEntries([...document.querySelectorAll('[id]')].map(el => {
        const r = el.getBoundingClientRect();
        return [el.id, {x:r.x, y:r.y, width:r.width, height:r.height}];
      })));
      await page.screenshot({path:path.join(out, `${fixture.name}-${width}.png`)});
      viewports.push({width, boxes});
    }
    cases.push({...fixture, viewports});
  }
  fs.writeFileSync(path.join(out, 'oracle.json'), JSON.stringify({browser:browser.version(), cases}, null, 2) + '\n');
  console.log(JSON.stringify({scenes:cases.length, viewports:cases.length*3, browser:browser.version()}));
} finally {
  await browser.close();
}
