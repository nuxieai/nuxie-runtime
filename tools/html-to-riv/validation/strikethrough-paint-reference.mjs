// Chrome-only behavioral controls. Passing these does not qualify native output.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const dir = process.env.NUXIE_STRIKETHROUGH_PAINT_DIR || root + '/output/playwright/html-to-riv/strikethrough-paint';
fs.mkdirSync(dir, {recursive: true});
const browser = await chromium.launch();
const results = [];
try {
  const page = await browser.newPage({viewport: {width: 320, height: 80}, deviceScaleFactor: 1});
  for (const font of ['Inter', 'OpenSans']) {
    const bytes = fs.readFileSync(`${root}/tools/html-to-riv/tests/assets/${font}-Regular.ttf`);
    for (const size of [16, 24, 32]) for (const thickness of ['auto', 'from-font', '2px']) {
      const name = `${font}-${size}-${thickness}`;
      await page.setContent(`<style>@font-face{font-family:fixture;src:url(data:font/ttf;base64,${bytes.toString('base64')})}body{margin:0;background:white}p{margin:0;font:${size}px/60px fixture;color:black;text-decoration-color:red;text-decoration-thickness:${thickness};text-decoration-skip-ink:none}</style><p id="text">agypqj M</p>`);
      const loaded = await page.evaluate(() => document.fonts.load('24px fixture'));
      assert.ok(loaded.length, 'embedded font must load');
      async function capture(label, css, nested = false) {
        await page.evaluate(({css, nested}) => {
          const text = document.querySelector('#text');
          text.style.cssText = css;
          text.innerHTML = nested ? '<span style="text-decoration:none;text-decoration-color:blue">agypqj M</span>' : 'agypqj M';
        }, {css, nested});
        const data = await page.screenshot();
        fs.writeFileSync(`${dir}/${name}-${label}.png`, data);
        return PNG.sync.read(data).data;
      }
      const plain = await capture('plain', 'text-decoration-line:none');
      const strike = await capture('strike', 'text-decoration-line:line-through');
      const nested = await capture('nested-none', 'text-decoration-line:line-through', true);
      const skipAll = await capture('skip-all', 'text-decoration-line:line-through;text-decoration-skip-ink:all');
      const offset = await capture('offset', 'text-decoration-line:line-through;text-underline-offset:20px');
      const underline = await capture('underline', 'text-decoration-line:underline');
      const both = await capture('both', 'text-decoration-line:underline line-through');
      assert.deepEqual(nested, strike, `${name}: child none/color must preserve ancestor decoration`);
      assert.deepEqual(skipAll, strike, `${name}: skip-ink must not cut strikethrough`);
      assert.deepEqual(offset, strike, `${name}: underline offset must not move strikethrough`);
      let coveredGlyphPixels = 0;
      let strikePixels = 0;
      for (let i = 0; i < plain.length; i += 4) {
        const red = strike[i] > 200 && strike[i + 1] < 50 && strike[i + 2] < 50;
        if (red) {
          strikePixels++;
          if (plain[i] < 50 && plain[i + 1] < 50 && plain[i + 2] < 50) coveredGlyphPixels++;
        }
        // Independent lines must compose without changing either line's pixels.
        const strikeChanged = !strike.subarray(i, i + 4).equals(plain.subarray(i, i + 4));
        const expected = strikeChanged ? strike : underline;
        assert.ok(both.subarray(i, i + 4).equals(expected.subarray(i, i + 4)), `${name}: combined paint differs at pixel ${i / 4}`);
      }
      assert.ok(coveredGlyphPixels > 0, `${name}: red strikethrough must paint over opaque glyphs`);
      results.push({name, strikePixels, coveredGlyphPixels});
    }
  }
  fs.writeFileSync(`${dir}/report.json`, JSON.stringify({browser: browser.version(), cases: results, captures: results.length * 7}, null, 2));
  await page.setViewportSize({width: 1000, height: 650});
  for (const font of ['Inter', 'OpenSans']) {
    const labels = ['plain', 'strike', 'nested-none', 'skip-all', 'offset', 'underline', 'both'];
    await page.setContent('<style>body{font:14px system-ui;display:flex;flex-wrap:wrap;background:#ddd;gap:8px}section{background:white;padding:8px}img{display:block}</style>' + labels.map(label => `<section><b>${font} 24px 2px: ${label}</b><img src="data:image/png;base64,${fs.readFileSync(`${dir}/${font}-24-2px-${label}.png`).toString('base64')}"></section>`).join(''));
    await page.evaluate(() => Promise.all([...document.images].map(image => image.decode())));
    await page.screenshot({path: `${dir}/review-${font}.png`, fullPage: true});
  }
  console.log(`${results.length} Chrome paint/propagation controls passed (${results.length * 7} captures)`);
} finally {
  await browser.close();
}
