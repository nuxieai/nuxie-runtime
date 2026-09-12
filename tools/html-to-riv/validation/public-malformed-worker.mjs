// Isolated persistent public-WASM compiler. The parent bounds every message.
import {parentPort, workerData} from 'node:worker_threads';
import fs from 'node:fs';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';

const {createCompiler, LANGUAGE_VERSION} = await import(pathToFileURL(workerData.wrapper));
const compiler = await createCompiler(fs.readFileSync(workerData.wasm));
const compile = request => compiler.compile({languageVersion: LANGUAGE_VERSION, ...request});
function packed(result) {
  if (!result.ok) {
    assert.deepEqual(Object.keys(result).sort(), ['diagnostics', 'ok']);
    assert(Array.isArray(result.diagnostics) && result.diagnostics.length > 0);
    return {ok:false, diagnostics:result.diagnostics};
  }
  assert.deepEqual(Object.keys(result).sort(), ['languageVersion', 'ok', 'riv', 'sourceMap']);
  assert.equal(result.languageVersion, LANGUAGE_VERSION);
  assert(result.riv instanceof Uint8Array);
  return {ok:true, rivHex:Buffer.from(result.riv).toString('hex'), sourceMap:result.sourceMap};
}
const snapshot = result => JSON.parse(JSON.stringify(packed(result)));
const first = compile(workerData.control);
assert.equal(first.ok, true);
const expected = snapshot(first);
// A caller can modify its own array without changing subsequent compiler output.
first.riv[0] ^= 255;
assert.deepEqual(snapshot(compile(workerData.control)), expected);
const retained = compile(workerData.control);
parentPort.postMessage({kind:'ready', control:expected, callerMutationOwnership:true});
parentPort.on('message', ({kind, id, request}) => {
  // Test only the supervisor's failure/restart path. This deliberately does
  // not call the compiler and is not counted as a malformed-source finding.
  if(kind==='lifecycle-failure') throw new Error('intentional malformed-campaign worker lifecycle control');
  const started = performance.now();
  let stage = 'first', response;
  try {
    const one = compile(request);
    response = snapshot(one);
    stage = 'repeat';
    assert.deepEqual(snapshot(compile(request)), response, 'WASM deterministic repeat');
    assert.deepEqual(snapshot(one), response, 'first result changed during repeated call');
    stage = 'recovery';
    assert.deepEqual(snapshot(compile(workerData.control)), expected, 'successful control after request');
    assert.deepEqual(snapshot(one), response, 'first result changed during recovery');
    assert.deepEqual(snapshot(retained), expected, 'retained control changed across requests');
    parentPort.postMessage({kind:'result', id, response, elapsedMs:performance.now()-started,
      checks:{deterministicRepeat:true, firstOutputOwned:true, retainedControlOwned:true, successfulRecovery:true}});
  } catch (error) {
    parentPort.postMessage({kind:'failure', id, stage, response, elapsedMs:performance.now()-started,
      error:{name:error.name, message:error.message, stack:error.stack}});
  }
});
