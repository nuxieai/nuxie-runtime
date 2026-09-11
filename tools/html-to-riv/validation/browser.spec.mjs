import { test, expect } from '@playwright/test';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import crypto from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { PNG } from 'pngjs';
import { comparePixels, compareInkPresence, compareRedDecoration } from './pixels.mjs';
import {compareGradientBorderPixels} from './gradient-border-pixels.mjs';

const moduleDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const root = path.resolve(moduleDir, '../..');
const target = path.resolve(root, process.env.CARGO_TARGET_DIR || 'target');
const toolchain=process.env.NUXIE_HTML_TOOLCHAIN && path.resolve(process.env.NUXIE_HTML_TOOLCHAIN);
if(toolchain) {
  const manifest=JSON.parse(fs.readFileSync(path.join(toolchain,'manifest.json'),'utf8'));
  for(const name of ['html-to-riv','probe','renderer-replay','html-to-riv.wasm']) {
    const hash=crypto.createHash('sha256').update(fs.readFileSync(path.join(toolchain,name))).digest('hex');
    if(hash!==manifest.files[name]?.sha256) throw new Error(`Frozen toolchain hash mismatch: ${name}`);
  }
  if(process.env.NUXIE_HTML_RENDERER) throw new Error('A frozen toolchain cannot override its renderer');
}
const compiler = toolchain ? path.join(toolchain,'html-to-riv') : path.join(target, 'debug/html-to-riv');
const probe = toolchain ? path.join(toolchain,'probe') : path.join(target, 'debug/examples/probe');
const replay = toolchain ? path.join(toolchain,'renderer-replay') : process.env.NUXIE_HTML_RENDERER || path.join(target, 'debug/renderer-replay');
const wasm = toolchain ? path.join(toolchain,'html-to-riv.wasm') : path.join(moduleDir,'dist/html-to-riv.wasm');
const backend = process.env.NUXIE_HTML_BACKEND || 'rust-metal';
if (backend === 'stub' || backend.startsWith('ffi-')) throw new Error('Browser validation requires a real Nuxie renderer');
const reset = fs.readFileSync(path.join(moduleDir, 'src/reset.css'), 'utf8');
const knownGaps = process.env.NUXIE_HTML_KNOWN_GAPS === '1';
const fixtures = JSON.parse(fs.readFileSync(path.join(moduleDir, knownGaps ? 'validation/known-gaps.json' : 'validation/cases.json'), 'utf8'));
// Keep qualified references in the normal regression gate as well as frozen replay.
if (!knownGaps) {
  // Public gradient candidates stay in the broad gate while qualification
  // continues. Rejection cases remain in the compiler/parity contract tests.
  for (const file of ['linear-gradient-initial-cases.json', 'linear-gradient-composition-cases.json', 'linear-gradient-realistic-cases.json']) {
    const gradients = JSON.parse(fs.readFileSync(path.join(moduleDir, 'validation', file), 'utf8'));
    fixtures.push(...gradients.filter(fixture => !fixture.expectedCompile || fixture.expectedCompile.status === 'accepted'));
  }
  for (const file of [
    'group-opacity-initial-cases.json', 'group-opacity-stacking-cases.json',
    'group-opacity-composition-cases.json', 'group-opacity-boundary-cases.json',
    'group-opacity-overflow-composition-cases.json', 'group-opacity-decoration-cases.json',
    'elliptical-radii-initial-cases.json', 'elliptical-radii-edge-cases.json',
    'elliptical-radii-composition-cases.json',
    'corner-radii-initial-cases.json', 'border-sides-initial-cases.json', 'border-sides-edge-cases.json',
    'stacking-context-cases.json', 'stacking-composition-cases.json', 'stacking-overlap-cases.json',
    'overflow-rounded-cases.json', 'overflow-nested-cases.json',
    'overflow-axis-composition-cases.json', 'overflow-axis-shortouter-cases.json',
    'overflow-axis-plain-cases.json', 'overflow-clip-margin-cases.json',
    'overflow-clip-margin-expanded-cases.json', 'overflow-clip-margin-signed-cases.json',
    'overflow-clip-margin-composition-cases.json',
  ]) {
    fixtures.push(...JSON.parse(fs.readFileSync(path.join(moduleDir, 'validation', file), 'utf8')));
  }
  // These four prospective axis cases compute to auto and intentionally reject.
  // The public compiler tests cover that rejection; only accepted scenes render.
  const axes = JSON.parse(fs.readFileSync(path.join(moduleDir, 'validation/overflow-axis-cases.json'), 'utf8'));
  fixtures.push(...axes.filter(fixture => !/-(visible-hidden|hidden-visible)$/.test(fixture.name)));
}
// Deterministic coverage of interacting properties, with Chromium computing
// every expected rectangle. These are not compiler-derived golden snapshots.
for(let i=0;!knownGaps && i<12;i++) fixtures.push({
  name:`matrix-${i}`,
  html:`<div id="root" class="frame"><div id="a" class="tile"></div><div id="b" class="tile"></div><div id="c" class="tile"></div></div>`,
  css:`.frame { width:100%; height:180px; flex-direction:${i%2?'row':'column'}; padding:${i+3}px ${i+7}px; gap:${i+2}px; align-items:${['flex-start','center','flex-end','stretch'][i%4]}; justify-content:${['flex-start','center','flex-end','space-between'][Math.floor(i/3)]}; background-color:#eaf0f6; border-radius:${i}px } .frame .tile { width:${24+i}px; height:${14+i}px; background-color:#369; border-radius:${i%5}px } #b { width:${15+i}%; background-color:#0a7c } #c { margin-left:${i}px; background-color:#d73 }`,
});
function run(binary, args) { return execFileSync(binary, args, { encoding: 'utf8', maxBuffer: 8 * 1024 * 1024 }); }

for (const fixture of fixtures) {
  for (const width of [240, 390, 768]) {
    test(`${fixture.name} at ${width}px`, async ({ page }, info) => {
      const pixels=info.project.metadata.pixels !== false;
      const height = 320;
      const prefix = info.outputPath('scene');
      fs.writeFileSync(`${prefix}.review.json`,JSON.stringify({name:fixture.name,width,height,pixels,html:fixture.html,css:fixture.css,artifactPrefix:prefix}));
      await info.attach('visual-review',{path:`${prefix}.review.json`,contentType:'application/json'});
      for (const binary of pixels ? [compiler, probe, replay] : [compiler, probe]) expect(fs.existsSync(binary), `Build required executable: ${binary}`).toBeTruthy();
      const request = { html: fixture.html, css: fixture.css, width: 390, height };
      let fontCss = '';
      let browserHtml = fixture.html;
      if (fixture.font) {
        const bytes = fs.readFileSync(path.join(moduleDir, 'tests/assets',['OpenSans-Regular.ttf','NotoSansOgham-Regular.ttf','NuxieJapaneseFixture-Regular.otf'].includes(fixture.fontAsset) ? fixture.fontAsset : 'Inter-Regular.ttf'));
        const family=fixture.fontFamily || 'Inter';
        request.assets = { inter: { kind:'font', family, weight:400, bytes:[...bytes] } };
        fontCss = `@font-face { font-family:${JSON.stringify(family)}; font-weight:400; font-style:normal; src:url(data:font/ttf;base64,${bytes.toString('base64')}); }`;
      }
      if (fixture.image) {
        const bytes = fs.readFileSync(path.join(moduleDir, 'tests/assets/quadrants.png'));
        request.assets = { ...request.assets, photo:{kind:'image', bytes:[...bytes]} };
        browserHtml = browserHtml.replaceAll('asset:photo', `data:image/png;base64,${bytes.toString('base64')}`);
      }
      fs.writeFileSync(`${prefix}.json`, JSON.stringify(request));
      run(compiler, [`${prefix}.json`, `${prefix}.riv`]);
      if (fixture.name === 'card') {
        // This scene is compiled in Chromium through the shipping JS API,
        // then imported/drawn by Nuxie below. No native compiler substitute.
        const compiled = await page.evaluate(async ({source,bytes,request}) => {
          const {createCompiler} = await import(`data:text/javascript;base64,${source}`);
          const compiler = await createCompiler(new Uint8Array(bytes));
          const result = compiler.compile({languageVersion:'nuxie-html-v1',...request});
          if (!result.ok) throw new Error(JSON.stringify(result.diagnostics));
          return {riv:Array.from(result.riv),sourceMap:result.sourceMap,runtimeRequirements:result.runtimeRequirements};
        }, {
          source:fs.readFileSync(path.join(moduleDir,'js/index.mjs')).toString('base64'),
          bytes:[...fs.readFileSync(wasm)],
          request,
        });
        expect(Buffer.from(compiled.riv).equals(fs.readFileSync(`${prefix}.riv`))).toBeTruthy();
        expect(compiled.sourceMap).toEqual(JSON.parse(fs.readFileSync(`${prefix}.map.json`)));
        fs.writeFileSync(`${prefix}.riv`,Buffer.from(compiled.riv));
        fs.writeFileSync(`${prefix}.map.json`,JSON.stringify(compiled.sourceMap));
        expect(compiled.runtimeRequirements).toEqual(JSON.parse(fs.readFileSync(`${prefix}.requirements.json`)));
        fs.writeFileSync(`${prefix}.requirements.json`,JSON.stringify(compiled.runtimeRequirements));
      }
      // Width differs from the compile viewport: proves runtime reflow rather
      // than separately compiling fixed browser rectangles at each size.
      run(probe, [`${prefix}.riv`, `${prefix}.map.json`, String(width), String(height), prefix]);
      if(pixels) run(replay, ['--stream', `${prefix}.stream`, '--output', `${prefix}.native.png`, '--backend', backend, '--mode', 'clockwise-atomic']);
      await page.setViewportSize({ width, height });
      // The host canvas is outside the authored fragment. Scope each selector
      // independently so comma groups keep their original specificity.
      await page.setContent(`<!doctype html><html><head><style>${fontCss}\n${reset}</style></head><body>${browserHtml}</body></html>`);
      await page.evaluate(css => {
        const sheet = new CSSStyleSheet(); sheet.replaceSync(css);
        const splitSelectors = text => {
          const result = []; let start = 0, depth = 0, quote = null;
          for (let i = 0; i < text.length; i++) {
            const c = text[i];
            if (c === '\\') { i++; continue; }
            if (quote) { if (c === quote) quote = null; continue; }
            if (c === '"' || c === "'") { quote = c; continue; }
            if (c === '[' || c === '(') depth++;
            else if (c === ']' || c === ')') depth--;
            else if (c === ',' && depth === 0) { result.push(text.slice(start, i)); start = i + 1; }
          }
          result.push(text.slice(start)); return result;
        };
        // Keep pending shorthand substitutions intact: cssText serialization
        // emits empty longhands when var() shorthands have later overrides.
        for (const rule of sheet.cssRules) {
          rule.selectorText = splitSelectors(rule.selectorText).map(s => `:where(body) :is(${s})`).join(',');
        }
        document.adoptedStyleSheets = [sheet];
      }, fixture.css);
      await page.evaluate(() => document.fonts.ready);
      await page.evaluate(() => Promise.all([...document.images].map(image => image.decode())));
      const sourceMap = JSON.parse(fs.readFileSync(`${prefix}.map.json`, 'utf8'));
      const browser = await page.evaluate(map => Object.fromEntries(map.map(node => {
        let element = document.querySelector(`[data-nuxie-id="${CSS.escape(node.id)}"]`) || document.getElementById(node.id);
        if (!element) {
          element = document.body;
          for (const index of node.path.split('/').filter(Boolean)) element = element.children[Number(index)];
        }
        const r = element.getBoundingClientRect();
        return [node.id, { x:r.x, y:r.y, width:r.width, height:r.height, hidden:getComputedStyle(element).display === 'none' }];
      })), sourceMap);
      // Fixture coverage checks use Chromium alone; they cannot make native
      // geometry pass. They guard cases intended to cross a wrapping threshold.
      for (const check of (fixture.separationChecks || []).filter(check => check.width === width)) {
        const separation = Math.abs(browser[check.first][check.axis] - browser[check.second][check.axis]);
        if (check.min !== undefined) expect(separation, 'browser fixture minimum separation').toBeGreaterThanOrEqual(check.min);
        if (check.max !== undefined) expect(separation, 'browser fixture maximum separation').toBeLessThanOrEqual(check.max);
      }
      fs.writeFileSync(`${prefix}.browser.json`, JSON.stringify(browser, null, 2));
      const native = JSON.parse(fs.readFileSync(`${prefix}.bounds.json`, 'utf8'));
      await page.screenshot({ path: `${prefix}.browser.png`, animations: 'disabled' });
      expect.soft(Object.keys(native)).toEqual(Object.keys(browser).sort());
      for (const [id, bounds] of Object.entries(browser)) {
        for (const key of ['x', 'y', 'width', 'height']) {
          expect.soft(Math.abs((native[id]?.[key] ?? Infinity) - bounds[key]), `${id}.${key}: native=${native[id]?.[key]} browser=${bounds[key]}`).toBeLessThanOrEqual(0.1);
        }
      }
      if(!pixels) return; // Explicit CPU geometry lane; never a visual pass.
      const reference = PNG.sync.read(fs.readFileSync(`${prefix}.browser.png`));
      const actual = PNG.sync.read(fs.readFileSync(`${prefix}.native.png`));
      expect([actual.width, actual.height]).toEqual([reference.width, reference.height]);
      if (fixture.exactPixels) expect(actual.data.equals(reference.data), 'hidden-content fixture must have exactly the reference pixels').toBe(true);
      // Optional tight text regions catch missing words that a whole-scene or
      // element average can hide. Bounds come only from the browser reference.
      const textRegions = fixture.textRegions ? await page.evaluate(regions => Object.fromEntries(regions.map(({id,start,end}) => {
        const node=document.getElementById(id)?.firstChild;
        if(!node || node.nodeType!==Node.TEXT_NODE || !Number.isInteger(start) || !Number.isInteger(end) || start<0 || end<=start || end>node.length) throw new Error(`Invalid direct-text region for ${id}`);
        const range=document.createRange();range.setStart(node,start);range.setEnd(node,end);
        const r=range.getBoundingClientRect();
        if(r.width<=0 || r.height<=0) throw new Error(`Empty text region for ${id}`);
        return [`${id}:text[${start}:${end}]`,{x:r.x,y:r.y,width:r.width,height:r.height}];
      })),fixture.textRegions) : {};
      const comparison = comparePixels(reference, actual, {...browser,...textRegions}, !!fixture.font);
      const {diff, failures} = comparison;
      if (fixture.name === 'linear-gradient-composition-fractional-border') {
        const usedBorder = await page.evaluate(() => {
          const element = document.getElementById('gradient');
          if (!element) throw new Error('Missing gradient border sentinel');
          const style = getComputedStyle(element);
          return {
            borderLeftWidth: parseFloat(style.borderLeftWidth),
            borderRightWidth: parseFloat(style.borderRightWidth),
          };
        });
        const borderRepeat = compareGradientBorderPixels(reference, actual, {
          viewport: {width, height}, box: browser.gradient, ...usedBorder,
        });
        comparison.metrics.gradientBorderRepeat = borderRepeat;
        if (!borderRepeat.passed) failures.push('gradient border repeat mismatch');
      }
      const inkPresence = {};
      for (const region of fixture.textRegions || []) if (region.inkBackground) {
        const key = `${region.id}:text[${region.start}:${region.end}]`;
        inkPresence[key] = compareInkPresence(reference, actual, textRegions[key], region.inkBackground);
        if (!inkPresence[key].passed) failures.push(`${key}: required ink missing`);
      }
      if (Object.keys(inkPresence).length) comparison.metrics.inkPresence = inkPresence;
      if(fixture.redDecoration) {
        const redDecoration=compareRedDecoration(reference,actual,{x:0,y:0,width:reference.width,height:reference.height});
        comparison.metrics.redDecoration=redDecoration;
        if(!redDecoration.passed) failures.push('red decoration mismatch');
      }
      const metrics = { ...comparison.metrics, ...(fixture.textRegions ? {textRegions} : {}), browserVersion:page.context().browser().version(), backend };
      fs.writeFileSync(`${prefix}.metrics.json`, JSON.stringify(metrics, null, 2));
      fs.writeFileSync(`${prefix}.diff.png`, PNG.sync.write(diff));
      for (const suffix of ['browser.png', 'native.png', 'diff.png', 'metrics.json']) {
        await info.attach(suffix, { path: `${prefix}.${suffix}` });
      }
      expect(failures, JSON.stringify(metrics)).toEqual([]);
    });
  }
}
