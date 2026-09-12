// Public image pipeline candidate: ROOT BASELINE_PROBE RENDERER.
// Uses frozen requests/Rive bytes. Source metadata identifies observations only.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { PNG } from 'pngjs';
import { observeImageContent, compareImageContent } from './image-content.mjs';

const changedOnly = process.argv.includes('--changed-only');
const [rootArg, probeArg, rendererArg, outputLabel = 'render', reuseLabel] = process.argv.slice(2).filter(a => a !== '--changed-only');
const [root, probe, renderer] = [rootArg, probeArg, rendererArg].map(p => path.resolve(p));
assert(/^[a-z0-9-]+$/.test(outputLabel) && (!reuseLabel || /^[a-z0-9-]+$/.test(reuseLabel)));
assert(root && probe && renderer);
const read = p => JSON.parse(fs.readFileSync(p));
const sha = p => createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const write = (p, value) => fs.writeFileSync(p, JSON.stringify(value, null, 2) + '\n');
const cases = read(path.join(root, 'cases.json'));
const compiled = read(path.join(root, 'compile-receipt.json'));
const selectedCases = cases.filter(c => {
  const row = compiled.find(r => r.name === c.name);
  return row?.compiled && (!changedOnly || row.previous?.sameRiv !== true || row.previous?.sameMap !== true || row.previous?.sameRequest !== true);
});
const resetPath = path.join(root, 'frozen/inputs/src/reset.css');
const bindings = [probe, renderer, resetPath, path.join(root, 'build-receipt.json'),
  path.join(root, 'compile-receipt.json'), path.join(root, 'cases.json'),
  fileURLToPath(import.meta.url), fileURLToPath(new URL('./pixels.mjs', import.meta.url)),
  fileURLToPath(new URL('./image-content.mjs', import.meta.url)),
].map(p => ({ path: p, sha256: sha(p) }));
let reuseReceipt = null;
if (reuseLabel) {
  const receiptPath = path.join(root, reuseLabel === 'render' ? 'native-receipt.json' : reuseLabel + '-receipt.json');
  reuseReceipt = read(receiptPath);
  assert.deepEqual(reuseReceipt.errors, [], 'Reused native receipt contains errors');
  assert.equal(reuseReceipt.browser, '153.0.8010.12', 'Reused receipt Chrome version differs');
  assert.equal(reuseReceipt.outputLabel, reuseLabel, 'Reused native receipt label differs');
  for (const tool of [probe, renderer, resetPath]) {
    const original = reuseReceipt.bindings.filter(binding => binding.path === tool);
    assert.equal(original.length, 1, `Missing or ambiguous reused tool binding: ${tool}`);
    assert.equal(original[0].sha256, sha(tool), `Reused tool/reset bytes differ: ${tool}`);
  }
  bindings.push({ path: receiptPath, sha256: sha(receiptPath) });
}
const sourceSnapshot = path.join(root, `validation-inputs-${outputLabel}`);
assert(!fs.existsSync(sourceSnapshot)); fs.mkdirSync(sourceSnapshot);
for (const b of bindings.filter(b => b.path.endsWith('.mjs'))) {
  b.snapshot = path.join(sourceSnapshot, path.basename(b.path)); fs.copyFileSync(b.path, b.snapshot);
  assert.equal(sha(b.snapshot), b.sha256);
}
const run = (command, log) => {
  const p = spawnSync(command[0], command.slice(1), { encoding: 'utf8', timeout: 30000, maxBuffer: 16 * 1024 * 1024 });
  fs.writeFileSync(log, (p.stdout ?? '') + (p.stderr ?? '') + (p.error ? '\n' + p.error : ''));
  assert.equal(p.status, 0, `Native process failed: ${p.error ?? p.stderr}`);
};
const browser = await chromium.launch();
assert.equal(browser.version(), '153.0.8010.12');
const rows = [], errors = [];
let resources = new Map(), resourceRequests = [];
const result = () => ({ scope: 'Public image layout candidate; full asset admission and public qualification remain separate.',
  browser: browser.version(), outputLabel, changedOnly, reusedNativeLabel: reuseLabel ?? null, bindings, rows, errors, counts: selectedCases.map(c => {
    const r = rows.filter(r => r.name === c.name);
    return { name: c.name, frames: r.length, expectedFrames: 8,
      geometryPass: r.filter(r => !r.metricFailures.length).length,
      pixelPass: r.filter(r => !r.pixelFailures.length).length,
      presencePass: r.filter(r => Object.values(r.imagePresence).every(v => v.passed)).length,
      firstMetricFailures: r[0]?.metricFailures, firstPixelFailures: r[0]?.pixelFailures };
  }) });
try {
  const page = await browser.newPage({ deviceScaleFactor: 1, colorScheme: 'light', locale: 'en-US' });
  await page.route('**/*', async route => {
    const asset = resources.get(route.request().url());
    assert(asset, `Unbound browser request ${route.request().url()}`);
    resourceRequests.push({ url: route.request().url(), sha256: asset.sha256 });
    await route.fulfill({ status: 200, contentType: asset.mime, body: asset.bytes });
  });
  for (const fixture of selectedCases) {
    const dir = path.join(root, fixture.name), out = path.join(dir, outputLabel);
    // Distinct URLs prevent the decoded-image cache from reusing another
    // fixture's bytes, even when both HTML sources spell src="logo".
    const base = `http://html-to-riv.invalid/${outputLabel}/${fixture.name}/`;
    assert(!fs.existsSync(out), 'Use fresh compiled output'); fs.mkdirSync(out);
    try {
      const input = read(path.join(dir, 'request.json')), map = read(path.join(dir, 'scene.map.json'));
      const riv = path.join(dir, 'scene.riv'), rivSha256 = sha(riv);
      const observed = path.join(reuseLabel ? path.join(dir, reuseLabel) : out, 'probe');
      if (!reuseLabel) run([probe, riv, observed, '240x240', '390x320', '768x560', '240x240'], path.join(out, 'probe.log'));
      assert.equal(sha(path.join(observed, 'scene.riv')), rivSha256);
      const frames = read(path.join(observed, 'frames.json')).frames;
      assert.equal(frames.length, 8);
      const html = `<!doctype html><meta charset="utf-8"><base href="${base}"><style>${fs.readFileSync(resetPath, 'utf8')}\n${input.css}</style>${input.html}`;
      fs.writeFileSync(path.join(out, 'reference.html'), html);
      resources = new Map(); resourceRequests = [];
      for (const [key, asset] of Object.entries(input.assets)) {
        const bytes = Buffer.from(asset.bytes), url = new URL(key, base).href;
        const sha256 = createHash('sha256').update(bytes).digest('hex');
        const mime = bytes[0] === 137 ? 'image/png' : bytes[0] === 255 ? 'image/jpeg' : 'image/webp';
        assert(!resources.has(url) || resources.get(url).sha256 === sha256, 'Distinct source bytes collide in browser URL resolution');
        resources.set(url, { bytes, sha256, mime });
      }
      write(path.join(out, 'browser-assets.json'), [...resources].map(([url, a]) => ({ url, sha256: a.sha256, mime: a.mime })));
      for (const f of frames) {
        const prefix = path.join(out, `frame-${f.frame}`);
        await page.setViewportSize({ width: f.width, height: f.height }); await page.setContent(html);
        const metrics = await page.evaluate(observeImageContent, fixture.observeIds);
        for (const image of Object.values(metrics.images)) assert(image.complete && image.naturalWidth > 0 && image.naturalHeight > 0);
        await page.screenshot({ path: prefix + '.chrome.png' });
        const geometry = read(path.join(observed, f.geometry)), nativeBoxes = {}, metricFailures = [];
        for (const [id, rect] of Object.entries(metrics.rectangles)) {
          const node = map.find(n => n.id === id); assert(node);
          const owner = geometry.find(g => g.objectId === node.object_id); assert(owner);
          nativeBoxes[id] = owner;
          const actual = { x: owner.worldMatrix[4], y: owner.worldMatrix[5], width: owner.width, height: owner.height };
          for (const [key, value] of Object.entries(actual)) if (Math.abs(value - rect[key]) > .1) metricFailures.push(`${id}.${key}: ${value} vs ${rect[key]}`);
        }
        let transferredNative = null;
        if (reuseLabel) {
          const prior = path.join(dir, reuseLabel, `frame-${f.frame}`), old = read(prior + '.result.json');
          const receiptRows = reuseReceipt.rows.filter(row => row.name === fixture.name && row.frame === f.frame);
          assert.equal(receiptRows.length, 1, 'Reused frame must have exactly one raw receipt row');
          assert.deepEqual(old, receiptRows[0], 'Reused result differs from its raw receipt row');
          assert.equal(old.rivSha256, rivSha256);
          assert.equal(old.streamSha256, sha(path.join(observed, f.stream)));
          assert.equal(old.geometrySha256, sha(path.join(observed, f.geometry)));
          assert.equal(old.nativeSha256, sha(prior + '.native.png'));
          assert.deepEqual([old.instance, old.step, old.width, old.height], [f.instance, f.step, f.width, f.height]);
          fs.copyFileSync(prior + '.native.png', prefix + '.native.png');
          transferredNative = { result: prior + '.result.json', sha256: sha(prior + '.result.json') };
        } else {
          run([renderer, '--stream', path.join(observed, f.stream), '--output', prefix + '.native.png',
            '--backend', 'rust-metal', '--mode', 'clockwise-atomic', '--frame', String(f.frame)], prefix + '.native.log');
        }
        const expected = PNG.sync.read(fs.readFileSync(prefix + '.chrome.png')), actual = PNG.sync.read(fs.readFileSync(prefix + '.native.png'));
        const compared = compareImageContent(expected, actual, metrics);
        const { imagePresence, borderBoxImagePresence, pixelRegions } = compared;
        fs.writeFileSync(prefix + '.diff.png', PNG.sync.write(compared.diff));
        const row = { name: fixture.name, ...f, prefix, probeDirectory: observed, transferredNative, browserMetrics: metrics, nativeBoxes, metricFailures,
          pixelFailures: compared.failures, pixelMetrics: compared.metrics, imagePresence, borderBoxImagePresence, pixelRegions,
          rivSha256, requestSha256: sha(path.join(dir, 'request.json')), mapSha256: sha(path.join(dir, 'scene.map.json')),
          htmlSha256: sha(path.join(out, 'reference.html')), streamSha256: sha(path.join(observed, f.stream)),
          geometrySha256: sha(path.join(observed, f.geometry)), chromeSha256: sha(prefix + '.chrome.png'), nativeSha256: sha(prefix + '.native.png') };
        rows.push(row); write(prefix + '.result.json', row);
      }
      write(path.join(out, 'browser-requests.json'), resourceRequests);
      assert.equal(sha(riv), rivSha256);
    } catch (error) {
      const failure = { name: fixture.name, error: String(error), stack: error.stack };
      errors.push(failure); write(path.join(out, 'failure.json'), failure);
    }
    write(path.join(root, outputLabel === 'render' ? 'native-receipt.json' : outputLabel + '-receipt.json'), result());
    console.log(JSON.stringify(result().counts.find(c => c.name === fixture.name)));
  }
  for (const b of bindings) assert.equal(sha(b.path), b.sha256);
} finally {
  write(path.join(root, outputLabel === 'render' ? 'native-receipt.json' : outputLabel + '-receipt.json'), result()); await browser.close();
}
if (errors.length) process.exitCode = 1;
