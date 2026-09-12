// Replay one bound native stream against its unchanged Chrome reference.
// This is a fast pixel diagnostic, not a compiler or resize qualification.
// Usage: node SCRIPT RECEIPT CASE_NAME FRAME FRESH_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';

const [receiptFile, name, frameText, output] = process.argv.slice(2);
assert(receiptFile && name && frameText && output && process.argv.length === 6,
  'RECEIPT CASE_NAME FRAME FRESH_OUTPUT');
const sha = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const receipt = JSON.parse(fs.readFileSync(receiptFile, 'utf8'));
const frame = Number(frameText);
assert(Number.isInteger(frame) && frame >= 0);
const rows = receipt.rows.filter(row => row.name === name && row.frame === frame);
assert.equal(rows.length, 1, 'exactly one source frame required');
const row = rows[0], renderer = receipt.tools.renderer;
const stream = row.command[row.command.indexOf('--stream') + 1];
const reference = row.prefix + '.chrome.png';
const gate = fileURLToPath(new URL('./pixels.mjs', import.meta.url));
assert.equal(sha(renderer), receipt.toolHashes.renderer);
assert.equal(sha(stream), row.streamSha256);
assert.equal(sha(reference), row.chromeSha256);
assert.equal(sha(gate), receipt.pixelGateSha256);
assert.equal(receipt.browser, '153.0.8010.12');
assert.equal(receipt.effectiveMode, 'RasterOrdering');
const out = path.resolve(output);
fs.mkdirSync(out, {recursive: false});
const command = [renderer, '--stream', stream, '--output', path.join(out, 'native.png'),
  '--backend', 'rust-metal', '--mode', 'clockwise-atomic', '--frame', String(frame)];
const result = spawnSync(command[0], command.slice(1), {encoding:'utf8'});
fs.writeFileSync(path.join(out, 'render.log'), (result.stdout ?? '') + (result.stderr ?? ''));
assert.ifError(result.error); assert.equal(result.signal, null); assert.equal(result.status, 0);
fs.copyFileSync(reference, path.join(out, 'chrome.png'));
const comparison = comparePixels(PNG.sync.read(fs.readFileSync(reference)),
  PNG.sync.read(fs.readFileSync(path.join(out, 'native.png'))), row.boxes, false);
fs.writeFileSync(path.join(out, 'diff.png'), PNG.sync.write(comparison.diff));
const report = {scope:'One frozen stream replay; no new compiler/resize qualification',
  name, frame, width:row.width, height:row.height, command,
  sourceReceipt:path.resolve(receiptFile), sourceReceiptSha256:sha(receiptFile),
  streamSha256:row.streamSha256, chromeSha256:row.chromeSha256,
  nativeSha256:sha(path.join(out, 'native.png')),
  exactNativeRepeat:sha(path.join(out, 'native.png')) === row.nativeSha256,
  pixelGateSha256:sha(gate), driverSha256:sha(fileURLToPath(import.meta.url)),
  failures:comparison.failures, metrics:comparison.metrics};
fs.writeFileSync(path.join(out, 'receipt.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({name,frame,exactNativeRepeat:report.exactNativeRepeat,
  failures:report.failures, mismatchRatio:report.metrics.mismatchRatio}));
if (comparison.failures.length || !report.exactNativeRepeat) process.exitCode = 1;
