// Supplemental host-background checks for the new authored-image composition.
// RECEIPT FRESH_OUTPUT CASE_NAME...; reuses exact recorded streams, no recompile.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {PNG} from 'pngjs';

const [receiptArg, outputArg, ...names] = process.argv.slice(2);
const receiptPath = path.resolve(receiptArg), output = path.resolve(outputArg);
const read = p => JSON.parse(fs.readFileSync(p));
const sha = p => createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const receipt = read(receiptPath), renderer = receipt.bindings.find(b => path.basename(b.path) === 'renderer-replay');
assert(renderer && sha(renderer.path) === renderer.sha256);
assert(names.length && new Set(names).size === names.length);
assert(!fs.existsSync(output)); fs.mkdirSync(output, {recursive:true});
const driver = fileURLToPath(import.meta.url), snapshot = path.join(output, 'driver.mjs');
fs.copyFileSync(driver, snapshot);
const bindings = [receiptPath, renderer.path, snapshot].map(p => ({path:p,sha256:sha(p)}));
const rows = [];
for (const name of names) {
  const frames = receipt.rows.filter(r => r.name === name); assert.equal(frames.length, 8);
  for (const frame of frames) {
    const stream = path.join(frame.probeDirectory, frame.stream), native = frame.prefix + '.native.png';
    assert.equal(sha(stream), frame.streamSha256); assert.equal(sha(native), frame.nativeSha256);
    const reference = PNG.sync.read(fs.readFileSync(native));
    for (const [clearName, value] of [['cyan','0xff00ffff'],['transparent','0x00000000']]) {
      const prefix = path.join(output, `${name}-${frame.frame}-${clearName}`);
      const command = [renderer.path,'--stream',stream,'--output',prefix+'.png','--backend','rust-metal',
        '--mode','clockwise-atomic','--frame',String(frame.frame),'--clear',value];
      const result = spawnSync(command[0],command.slice(1),{encoding:'utf8',timeout:30000});
      fs.writeFileSync(prefix+'.log',(result.stdout??'')+(result.stderr??''));
      assert.equal(result.status,0,`${result.error ?? result.stderr}`);
      const actual = PNG.sync.read(fs.readFileSync(prefix+'.png'));
      const samePixels = actual.width === reference.width && actual.height === reference.height && actual.data.equals(reference.data);
      rows.push({name,frame:frame.frame,instance:frame.instance,step:frame.step,clearName,value,command,
        stream,streamSha256:sha(stream),reference:native,referenceSha256:sha(native),output:prefix+'.png',outputSha256:sha(prefix+'.png'),samePixels});
    }
  }
  console.log(JSON.stringify({name,checks:rows.filter(r=>r.name===name).length,passed:rows.filter(r=>r.name===name&&r.samePixels).length}));
}
for (const b of bindings) assert.equal(sha(b.path),b.sha256);
assert.equal(sha(driver),sha(snapshot));
fs.writeFileSync(path.join(output,'receipt.json'),JSON.stringify({scope:'Selected alpha and overlapping-image original/clone streams remain independent of canvas clear; exact full RGBA equality',bindings,rows,total:rows.length,passed:rows.filter(r=>r.samePixels).length},null,2)+'\n');
assert(rows.every(r=>r.samePixels));
