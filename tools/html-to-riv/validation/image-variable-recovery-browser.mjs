// Bounded receiving-grammar observations. No browser measurements enter emission.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const corpus = path.join(root, 'validation/image-variable-recovery-cases.json');
const out = path.join(root, 'output/playwright/image-variable-recovery-r1');
assert(!fs.existsSync(out), 'Keep earlier observations; choose a new output directory');
fs.mkdirSync(out, { recursive: true });
const resetFile = path.join(root, 'src/reset.css');
const asset = path.join(root, 'fixtures/images/ordinary-r1/opaque.png');
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const cases = JSON.parse(fs.readFileSync(corpus));
const reset = fs.readFileSync(resetFile, 'utf8');
const uri = 'data:image/png;base64,' + fs.readFileSync(asset).toString('base64');
const bindings = [corpus, resetFile, asset, fileURLToPath(import.meta.url)]
  .map(p => ({ path: p, sha256: hash(p) }));
const rows = [], errors = [];
const browser = await chromium.launch();
const receipt = () => ({
  scope: 'Pinned Chrome receiving-value grammar and explicit unset/literal controls; no native geometry or pixel qualification.',
  browser: browser.version(), bindings, rows, errors,
});
try {
  assert.equal(browser.version(), '153.0.8010.12');
  const page = await browser.newPage({ viewport: { width: 240, height: 160 }, deviceScaleFactor: 1 });
  const base = '#parent{width:200px;height:160px;object-fit:cover;object-position:right bottom;image-rendering:pixelated}' +
    '#image{width:96px;height:64px;object-fit:contain;object-position:left top;image-rendering:auto}';
  const html = `<!doctype html><meta charset="utf-8"><style>${reset}${base}</style><div id="parent"><img id="image" src="${uri}"></div>`;
  fs.writeFileSync(path.join(out, 'reference.html'), html);
  await page.setContent(html);
  await page.locator('#image').evaluate(e => e.decode());
  for (const fixture of cases) {
    const observation = await page.locator('#image').evaluate((e, fixture) => {
      const p = fixture.property, v = fixture.value;
      const measure = () => ({ specified: e.style.getPropertyValue(p), computed: getComputedStyle(e).getPropertyValue(p),
        custom: getComputedStyle(e).getPropertyValue('--v'), priority: e.style.getPropertyPriority(p) });
      e.removeAttribute('style');
      const previous = measure();
      e.style.setProperty(p, v);
      const literal = measure();
      e.setAttribute('style', `--v:${v};${p}:var(--v)`);
      const variable = measure();
      e.setAttribute('style', `--v:${v};${p}:unset`);
      const unset = measure();
      return { supports: CSS.supports(p, v), previous, literal, variable, unset };
    }, fixture);
    const failures = [];
    if (fixture.browserSupports !== undefined && observation.supports !== fixture.browserSupports) failures.push('Unexpected CSS.supports result');
    if (fixture.validity === 'invalid') {
      if (observation.literal.computed !== observation.previous.computed) failures.push('Invalid literal replaced previous declaration');
      if (observation.variable.computed !== observation.unset.computed) failures.push('Invalid variable differs from explicit unset');
      if (observation.previous.computed === observation.unset.computed) failures.push('Reference does not expose cascade rollback');
    } else if (fixture.target === 'literal' || observation.supports) {
      if (observation.variable.computed !== observation.literal.computed) failures.push('Variable differs from supported literal');
    }
    rows.push({ ...fixture, observation, failures });
    if (failures.length) errors.push({ name: fixture.name, failures });
  }
  for (const b of bindings) assert.equal(hash(b.path), b.sha256);
} finally {
  fs.writeFileSync(path.join(out, 'receipt.json'), JSON.stringify(receipt(), null, 2) + '\n');
  await browser.close();
}
console.log(JSON.stringify({ rows: rows.length, errors }, null, 2));
if (errors.length) process.exitCode = 1;
