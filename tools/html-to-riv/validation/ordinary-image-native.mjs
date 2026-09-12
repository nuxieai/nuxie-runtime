// Private authored-image experiment. Usage: CASES_JSON GENERATED_DIR BASELINE_PROBE RENDERER
// Browser rectangles are observations only; generation has already finished.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { PNG } from 'pngjs';
import { comparePixels, compareInkPresence } from './pixels.mjs';

assert.equal(process.argv.length, 6, 'CASES_JSON GENERATED_DIR BASELINE_PROBE RENDERER');
const [casesFile, root, probe, renderer] = process.argv.slice(2).map(p => path.resolve(p));
const cases = JSON.parse(fs.readFileSync(casesFile));
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const write = (p, value) => fs.writeFileSync(p, JSON.stringify(value, null, 2) + '\n');
const bindings = [casesFile, probe, renderer, path.join(root, 'generator'),
  path.join(root, 'image-probe'), path.join(root, 'build-receipt.json'),
  path.join(root, 'generation-receipt.json'), fileURLToPath(import.meta.url),
  fileURLToPath(new URL('./pixels.mjs', import.meta.url)),
].map(p => ({ path: p, sha256: hash(p) }));
const run = (command, log) => {
  const r = spawnSync(command[0], command.slice(1), {
    encoding: 'utf8', timeout: 30000, maxBuffer: 16 * 1024 * 1024,
  });
  fs.writeFileSync(log, (r.stdout ?? '') + (r.stderr ?? '') + (r.error ? '\n' + r.error : ''));
  assert.equal(r.status, 0, `Command failed: ${command[0]}; ${r.error ?? r.stderr}`);
};
const browser = await chromium.launch();
const rows = [], errors = [];
const receipt = () => ({
  scope: 'Private ordinary embedded-image composition. Same authored bytes in Chrome and Rive. No public asset admission.',
  browser: browser.version(), rendererMode: 'RustMetal RasterOrdering',
  recordingDecodeScope: 'Header observations only; successful actual native replay and pixels are separate evidence.',
  bindings, rows, errors,
  counts: cases.map(c => {
    const r = rows.filter(r => r.name === c.name);
    return { name: c.name, expectedFrames: 8, frames: r.length,
      metricPass: r.filter(r => r.metricFailures.length === 0).length,
      pixelPass: r.filter(r => r.pixelFailures.length === 0 && r.imagePresence.passed).length,
      firstMetricFailures: r[0]?.metricFailures, firstPixelFailures: r[0]?.pixelFailures,
      firstBrowser: r[0]?.browserMetrics, firstNativeOwner: r[0]?.nativeOwner };
  }),
});
try {
  assert.equal(browser.version(), '153.0.8010.12');
  const page = await browser.newPage({ deviceScaleFactor: 1, colorScheme: 'light', locale: 'en-US' });
  for (const fixture of cases) {
    const dir = path.join(root, fixture.name), out = path.join(dir, 'render');
    assert(!fs.existsSync(out), 'Use a fresh generated directory');
    fs.mkdirSync(out);
    try {
      const riv = path.join(dir, 'scene.riv'), asset = path.join(dir, fixture.asset);
      const mime = { '.png': 'image/png', '.jpg': 'image/jpeg', '.webp': 'image/webp' }[path.extname(asset)];
      assert(mime);
      const rivSha256 = hash(riv), assetSha256 = hash(asset);
      const observed = path.join(out, 'probe');
      run([probe, riv, observed, '240x240', '390x320', '768x560', '240x240'], path.join(out, 'probe.log'));
      assert.equal(hash(path.join(observed, 'scene.riv')), rivSha256);
      const frames = JSON.parse(fs.readFileSync(path.join(observed, 'frames.json'))).frames;
      const observation = path.join(dir, 'image-observation/image-metrics.json');
      const imageFrames = JSON.parse(fs.readFileSync(observation)).frames;
      assert.equal(frames.length, 8); assert.equal(imageFrames.length, 8);
      const html = `<!doctype html><meta charset="utf-8"><style>html,body{margin:0;background:white}body{padding:20px;box-sizing:border-box}${fixture.css}</style><img id="image" src="data:${mime};base64,${fs.readFileSync(asset).toString('base64')}">`;
      fs.writeFileSync(path.join(out, 'reference.html'), html);
      write(path.join(out, 'reference-source.json'), fixture);
      for (const f of frames) {
        const prefix = path.join(out, `frame-${f.frame}`);
        await page.setViewportSize({ width: f.width, height: f.height });
        await page.setContent(html);
        const metrics = await page.locator('#image').evaluate(async e => {
          await e.decode();
          const r = e.getBoundingClientRect(), s = getComputedStyle(e);
          return { box: { x: r.x, y: r.y, width: r.width, height: r.height },
            complete: e.complete, naturalWidth: e.naturalWidth, naturalHeight: e.naturalHeight,
            objectFit: s.objectFit, objectPosition: s.objectPosition, imageRendering: s.imageRendering };
        });
        assert(metrics.complete);
        assert.equal(metrics.naturalWidth, fixture.request.assetWidth);
        assert.equal(metrics.naturalHeight, fixture.request.assetHeight);
        await page.screenshot({ path: prefix + '.chrome.png' });
        const geometry = JSON.parse(fs.readFileSync(path.join(observed, f.geometry)));
        const owner = geometry.find(g => g.objectId === 4), imageFrame = imageFrames[f.frame];
        assert(owner); assert.equal(imageFrame.instance, f.instance === 0 ? 'original' : 'clone');
        assert.equal(imageFrame.step, f.step); assert.deepEqual(imageFrame.viewport, [f.width, f.height]);
        assert.equal(imageFrame.images.length, 1);
        const image = imageFrame.images[0];
        assert.equal(image.objectId, 8); assert.equal(image.assetIndex, 0);
        assert(image.recordedImage && image.asset && image.participant);
        const measured = { x: owner.worldMatrix[4], y: owner.worldMatrix[5], width: owner.width, height: owner.height };
        const metricFailures = [];
        for (const [k, v] of Object.entries(measured)) {
          if (Math.abs(v - metrics.box[k]) > .1) metricFailures.push(`box.${k}: ${v} vs ${metrics.box[k]}`);
        }
        const command = [renderer, '--stream', path.join(observed, f.stream), '--output', prefix + '.native.png',
          '--backend', 'rust-metal', '--mode', 'clockwise-atomic', '--frame', String(f.frame)];
        run(command, prefix + '.native.log');
        const expected = PNG.sync.read(fs.readFileSync(prefix + '.chrome.png'));
        const actual = PNG.sync.read(fs.readFileSync(prefix + '.native.png'));
        const comparison = comparePixels(expected, actual, { image: metrics.box }, false);
        // The known image-box background cannot stand in for the actual colored image.
        const imagePresence = compareInkPresence(expected, actual, metrics.box, [233, 240, 246]);
        fs.writeFileSync(prefix + '.diff.png', PNG.sync.write(comparison.diff));
        const row = { name: fixture.name, ...f, prefix, browserMetrics: metrics, nativeOwner: owner, nativeImage: image,
          metricFailures, pixelMetrics: comparison.metrics, pixelFailures: comparison.failures, imagePresence,
          rivSha256, assetSha256, sourceHtmlSha256: hash(path.join(out, 'reference.html')),
          streamSha256: hash(path.join(observed, f.stream)), geometrySha256: hash(path.join(observed, f.geometry)),
          imageObservationSha256: hash(observation), chromeSha256: hash(prefix + '.chrome.png'),
          nativeSha256: hash(prefix + '.native.png'), diffSha256: hash(prefix + '.diff.png') };
        rows.push(row); write(prefix + '.result.json', row);
      }
      assert.equal(hash(riv), rivSha256); assert.equal(hash(asset), assetSha256);
    } catch (error) {
      const failure = { name: fixture.name, error: String(error), stack: error.stack };
      errors.push(failure); write(path.join(out, 'failure.json'), failure);
    }
    write(path.join(root, 'native-receipt.json'), receipt());
    console.log(JSON.stringify(receipt().counts.find(c => c.name === fixture.name)));
  }
  for (const b of bindings) assert.equal(hash(b.path), b.sha256);
} finally {
  write(path.join(root, 'native-receipt.json'), receipt());
  await browser.close();
}
if (errors.length) process.exitCode = 1;
