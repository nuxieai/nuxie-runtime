// L07: independent Chromium references, before enabling compiler auto margins.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';

const here = path.dirname(fileURLToPath(import.meta.url));
const reset = fs.readFileSync(path.join(here, '../src/reset.css'), 'utf8');
const out = path.resolve(process.argv[2] || 'output/playwright/html-to-riv/auto-margin-oracle.json');
const browser = await chromium.launch();
const cases = [];
try {
  const page = await browser.newPage();
  for (const direction of ['row', 'row-reverse', 'column', 'column-reverse']) {
    const row = direction.startsWith('row');
    const mainStart = row ? 'left' : 'top';
    const mainEnd = row ? 'right' : 'bottom';
    const crossStart = row ? 'top' : 'left';
    const crossEnd = row ? 'bottom' : 'right';
    const patterns = {
      'main-start': `#b{margin-${mainStart}:auto}`,
      'main-end': `#b{margin-${mainEnd}:auto}`,
      'main-both': `#b{margin-${mainStart}:auto;margin-${mainEnd}:auto}`,
      'main-shared': `#a{margin-${mainStart}:auto}#b{margin-${mainEnd}:auto}`,
      'cross-start': `#b{margin-${crossStart}:auto;align-self:center}`,
      'cross-end': `#b{margin-${crossEnd}:auto;align-self:center}`,
      'cross-both': `#b{margin-${crossStart}:auto;margin-${crossEnd}:auto;align-self:flex-end}`,
      'all': '#b{margin:auto}',
    };
    for (const [pattern, margins] of Object.entries(patterns)) {
      for (const mode of ['free', 'overflow', 'wrap', 'wrap-reverse']) {
        const html = '<div id="root"><div id="a"></div><div id="b"></div><div id="c"></div></div>';
        const rootSize = row ? `height:${mode === 'overflow' ? 40 : 180}px` : `height:${mode === 'overflow' ? 120 : (mode === 'free' ? 280 : 180)}px`;
        const itemSize = row ? `width:${mode === 'overflow' ? 280 : 85}px;height:50px` : `height:65px;width:${mode === 'overflow' ? 300 : 50}px`;
        const wrap = mode.startsWith('wrap') ? mode : 'nowrap';
        const css = `#root{width:100%;${rootSize};padding:10px;gap:10px;flex-direction:${direction};flex-wrap:${wrap};align-items:stretch;align-content:space-between;justify-content:space-evenly}#root>div{${itemSize}}${margins}`;
        const viewports = [];
        for (const width of [240, 390, 768]) {
          await page.setViewportSize({width, height:320});
          await page.setContent(`<!doctype html><style>${reset}\n${css}</style>${html}`);
          const boxes = await page.evaluate(() => Object.fromEntries([...document.querySelectorAll('[id]')].map(el => {
            const r = el.getBoundingClientRect();
            return [el.id, {x:r.x, y:r.y, width:r.width, height:r.height}];
          })));
          for (const id of ['a', 'b', 'c']) {
            if (boxes[id][row ? 'width' : 'height'] !== (row ? (mode === 'overflow' ? 280 : 85) : 65)) {
              throw new Error(`Fixture shrank unexpectedly: ${direction}/${pattern}/${mode}/${width}/${id}`);
            }
          }
          if (!row && mode === 'free' && boxes.root.height - 20 <= 3 * 65 + 20) {
            throw new Error('Free-space column must have positive remaining main-axis space');
          }
          viewports.push({width, boxes});
        }
        cases.push({direction, pattern, mode, html, css, viewports});
      }
    }
  }
  fs.mkdirSync(path.dirname(out), {recursive:true});
  fs.writeFileSync(out, JSON.stringify({browser:browser.version(), generator:'validation/auto-margin-oracle.mjs', purpose:'Independent browser reference; compiler/native qualification pending', cases}, null, 2) + '\n');
  console.log(JSON.stringify({out, scenes:cases.length, viewports:cases.length*3, browser:browser.version()}));
} finally {
  await browser.close();
}
