// Diagnostic controls distinguish flex targets, CSS used-size rounding and
// fractional paint coverage. Fixed snapshots are never a shipping output mode.
// Usage: node SCRIPT FRESH_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {PNG} from 'pngjs';

const base = fileURLToPath(new URL('../', import.meta.url));
const sha = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const read = p => JSON.parse(fs.readFileSync(p));
const write = (p, v) => fs.writeFileSync(p, JSON.stringify(v, null, 2) + '\n');
const [output] = process.argv.slice(2);
assert(output && process.argv.length === 3, 'FRESH_OUTPUT');
const out = path.resolve(output); fs.mkdirSync(out);
const caseFile = path.join(base, 'validation/flex-fixed-controls-cases.json');
const cases = read(caseFile);
const referenceFile = path.join(base, cases.bindings[0].source);
assert.equal(sha(referenceFile), cases.originalReceiptSha256);
const reference = read(referenceFile);
const driver = path.join(base, 'validation/check-public-baseline.mjs');
const publicCompiler = path.join(base, 'output/flex-proof-checkpoint-r2/html-to-riv');
const candidate = path.join(base, 'output/flex-proof-bridge-r2/aggregate-driver-check/harness');
const baseline = path.join(base, 'output/immutable-baseline-toolchain-r2');
const probe = path.join(baseline, 'baseline-probe');
const renderer = path.join(baseline, 'renderer-replay');
const binaryHashes = Object.fromEntries([publicCompiler, candidate, probe, renderer].map(p => [p, sha(p)]));
const checkpoint = read(path.join(base, 'validation/flex-scene-receipt.json'));
assert.equal(sha(publicCompiler), checkpoint.artifacts['tools/html-to-riv/output/flex-proof-checkpoint-r2/html-to-riv']);
const candidateReceipt = read(path.join(base, 'output/flex-proof-bridge-r2/aggregate-driver-check/build-receipt.json'));
assert.equal(sha(candidate), candidateReceipt.harnessBinarySha256);
assert.equal(sha(probe), reference.toolHashes.probe);
assert.equal(sha(renderer), reference.toolHashes.renderer);
fs.copyFileSync(caseFile, path.join(out, 'cases.json'));
fs.copyFileSync(fileURLToPath(import.meta.url), path.join(out, 'invoked-driver.mjs'));
const runs = {};
for (const group of ['reductions', 'edges', 'fixed']) {
  const fixture = path.join(out, group + '.json'); write(fixture, cases[group]);
  const command = ['node', driver, fixture, group === 'reductions' ? candidate : publicCompiler,
    probe, renderer, path.join(out, group + '-render')];
  const result = spawnSync(command[0], command.slice(1), {encoding:'utf8'});
  fs.writeFileSync(path.join(out, group + '.log'), (result.stdout ?? '') + (result.stderr ?? ''));
  assert.ifError(result.error); assert.equal(result.signal, null);
  assert([0, 1].includes(result.status), 'driver failed unexpectedly');
  const file = path.join(out, group + '-render/receipt.json');
  const receipt = read(file);
  assert.equal(receipt.rows.length, cases[group].length * 8);
  runs[group] = {command, exitCode:result.status, receipt, receiptSha256:sha(file)};
}
const image = p => PNG.sync.read(fs.readFileSync(p));
const fixedComparisons = [];
for (const binding of cases.bindings) {
  const original = reference.rows.find(r => r.name === binding.sourceName && r.frame === binding.sourceFrame);
  const control = runs.fixed.receipt.rows.find(r => r.name === binding.control && r.frame === 0);
  assert(original && control);
  for (const [key, expected] of [['requestSha256','originalRequestSha256'], ['rivSha256','originalRivSha256'],
    ['chromeSha256','originalChromeSha256'], ['nativeSha256','originalNativeSha256']]) {
    assert.equal(original[key], binding[expected]);
  }
  const same = {};
  for (const kind of ['chrome', 'native']) {
    const oldFile = original.prefix + '.' + kind + '.png';
    const newFile = control.prefix + '.' + kind + '.png';
    assert.equal(sha(oldFile), original[kind+'Sha256']);
    const a = image(oldFile), b = image(newFile);
    assert.equal(a.width, b.width); assert.equal(a.height, b.height);
    let different = 0; for (let i = 0; i < a.data.length; ++i) if (a.data[i] !== b.data[i]) different++;
    same[kind] = {rgbaIdentical:different === 0, differentChannels:different, originalSha256:sha(oldFile), controlSha256:sha(newFile)};
  }
  fixedComparisons.push({...binding, images:same, pixelFailures:control.pixelFailures});
}
const edgeSamples = runs.edges.receipt.rows.filter(r => r.frame === 0).map(row => {
  const sampleRow = Math.max(0, Math.ceil(row.boxes.p.height) - 1);
  const sample = file => {const p = image(file), i = (sampleRow * p.width + 20) * 4; return [...p.data.subarray(i, i + 4)];};
  const sourceMap = read(path.join(path.dirname(row.prefix), 'scene.map.json'));
  const id = sourceMap.find(n => n.id === 'p').object_id;
  const nativeHeight = row.geometry.find(n => n.objectId === id).height;
  return {name:row.name, chromeHeight:row.boxes.p.height, nativeHeight, sampleX:20, sampleY:sampleRow,
    chrome:sample(row.prefix+'.chrome.png'), native:sample(row.prefix+'.native.png'), pixelFailures:row.pixelFailures};
});
for (const [file, hash] of Object.entries(binaryHashes)) assert.equal(sha(file), hash);
assert.equal(sha(caseFile), sha(path.join(out, 'cases.json')));
const totals = Object.fromEntries(Object.entries(runs).map(([name,run]) => [name, {
  cases:cases[name].length, frames:run.receipt.rows.length,
  geometryPass:run.receipt.rows.filter(r => !r.geometryFailures.length).length,
  pixelPass:run.receipt.rows.filter(r => !r.pixelFailures.length).length,
  clearPass:run.receipt.rows.flatMap(r => r.clearChecks).filter(c => c.samePixels).length,
}]));
const report = {scope:cases.scope, totals, binaryHashes, caseSha256:sha(caseFile),
  driverSha256:sha(fileURLToPath(import.meta.url)), sourceReceiptSha256:sha(referenceFile),
  runs:Object.fromEntries(Object.entries(runs).map(([name,r]) => [name, {command:r.command, exitCode:r.exitCode, receiptSha256:r.receiptSha256}])),
  fixedComparisons, edgeSamples};
write(path.join(out, 'receipt.json'), report);
console.log(JSON.stringify(totals));
// A recorded expected limitation still fails the pixel qualification command.
if (Object.values(runs).some(r => r.exitCode !== 0)) process.exitCode = 1;
