import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {spawnSync} from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createCompiler} from '@nuxie/html-to-riv';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const nativeCompiler = process.env.NUXIE_NATIVE_COMPILER
  ? path.resolve(process.env.NUXIE_NATIVE_COMPILER)
  : path.join(process.env.CARGO_TARGET_DIR ? path.resolve(root, process.env.CARGO_TARGET_DIR) : path.join(root, 'target'), 'debug/html-to-riv');
const wasm = fs.readFileSync(new URL('../dist/html-to-riv.wasm', import.meta.url));
const oracle = JSON.parse(fs.readFileSync(new URL('assets/linear-gradient-initial-oracle.json', import.meta.url)));
const request = (css, html = '<div id=gradient></div>') => ({html, css, width:390, height:320});
const compile = (compiler, input) => compiler.compile({languageVersion:'nuxie-html-v1', ...input});

// Fresh output directories prevent a failed CLI call from leaving stale success artifacts.
function native(input) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'html-riv-gradient-parity-'));
  try {
    const file = path.join(directory, 'input.json');
    fs.writeFileSync(file, JSON.stringify(input));
    const process = spawnSync(nativeCompiler, [file, path.join(directory, 'scene.riv')], {encoding:'utf8'});
    assert.ifError(process.error);
    assert.equal(process.signal, null, process.stderr);
    if (process.status === 1) {
      assert.equal(fs.existsSync(path.join(directory, 'scene.riv')), false);
      return {ok:false, diagnostics:JSON.parse(process.stderr)};
    }
    assert.equal(process.status, 0, process.stderr);
    return {ok:true, riv:fs.readFileSync(path.join(directory, 'scene.riv')),
      sourceMap:JSON.parse(fs.readFileSync(path.join(directory, 'scene.map.json'))),
      runtimeRequirements:JSON.parse(fs.readFileSync(path.join(directory, 'scene.requirements.json')))};
  } finally { fs.rmSync(directory, {recursive:true, force:true}); }
}

function parity(compiler, input, label = input.css, accepted = true) {
  const actual = compile(compiler, input);
  assert.equal(actual.ok, accepted, `${label}: ${JSON.stringify(actual.diagnostics)}`);
  const published = native(input);
  assert.equal(published.ok, accepted, label);
  if (accepted) {
    assert.deepEqual(Buffer.from(actual.riv), published.riv, `${label}: RIV`);
    assert.deepEqual(actual.sourceMap, published.sourceMap, `${label}: source map`);
    assert.deepEqual(actual.runtimeRequirements, published.runtimeRequirements, `${label}: requirements`);
  } else {
    assert.equal('riv' in actual, false, label);
    assert.deepEqual(actual.diagnostics, published.diagnostics, label);
  }
  return actual;
}

function paint(result, id = 'gradient') {
  const node = result.sourceMap.find(node => node.id === id);
  assert.ok(node, `missing source map owner ${id}`);
  const requirements = result.runtimeRequirements;
  assert.equal(requirements.version, 26);
  assert.ok(requirements.capabilities.includes('layout-css-linear-gradient-v1'));
  assert.ok(requirements.layout_pixel_bounds.includes(node.object_id));
  const gradient = requirements.layout_linear_gradients.find(entry => entry.object_id === node.object_id);
  assert.ok(gradient, `missing gradient owner ${id}`);
  const {object_id, ...value} = gradient;
  return value;
}

// The authored expectations come from the independent diagnostic oracle, not compiler output.
function expectedPaint(spec) {
  return {direction:spec.direction.corner
    ? {corner:{right:spec.direction.corner[0], bottom:spec.direction.corner[1]}}
    : {degrees:((spec.direction.degrees % 360) + 360) % 360},
  stops:spec.colors.map((color, index) => ({color, position:spec.positions[index]}))};
}

const stop = (color, position = null) => ({color, position});
const red = 0xffff0000, blue = 0xff0000ff, lime = 0xff00ff00;

test('public gradients match independent paints and exact native/WASM artifacts at all reference widths', async () => {
  const compiler = await createCompiler(wasm);
  assert.equal(oracle.cases.length, 16);
  for (const fixture of oracle.cases) {
    const expected = expectedPaint(fixture.diagnosticGradient);
    for (const width of [240, 390, 768]) {
      const result = parity(compiler, {...request(fixture.css, fixture.html), width}, `${fixture.name}@${width}`);
      assert.equal(result.runtimeRequirements.layout_linear_gradients.length, 1);
      // Corner directions, percentages, decreasing stops and omitted positions must survive
      // compilation unchanged. These transport assertions do not replace runtime resize tests.
      assert.deepEqual(paint(result), expected, `${fixture.name}@${width}`);
    }
  }
});

test('public gradient currentColor, relative units and inheritance retain computed-value semantics', async () => {
  const compiler = await createCompiler(wasm);
  const result = parity(compiler, request(
    '#parent{background-image:linear-gradient(currentColor 2em,blue 2rem);font-size:20px;color:lime;opacity:.5}' +
    '#child{font-size:10px;color:red;background-image:inherit}',
    '<div id=parent><div id=child></div><div id=plain></div></div>'));
  assert.equal(result.runtimeRequirements.layout_linear_gradients.length, 2);
  assert.equal(result.runtimeRequirements.layout_group_opacity.length, 1);
  for (const [id, color] of [['parent', lime], ['child', red]]) {
    assert.deepEqual(paint(result, id), {direction:{degrees:180}, stops:[stop(color, {pixels:40}), stop(blue, {pixels:32})]});
  }
  const expanded = parity(compiler, request('div{--paint:linear-gradient(to top right,currentColor -1em 25%,rgba(0,0,255,.5) 2rem);background:var(--paint);font-size:20px;color:red}'));
  assert.deepEqual(paint(expanded), {direction:{corner:{right:true,bottom:false}},
    stops:[stop(red,{pixels:-20}), stop(red,{percent:25}), stop(0x800000ff,{pixels:32})]});
});

test('gradient aliases and component colors produce equivalent public artifacts', async () => {
  const compiler = await createCompiler(wasm);
  let baseline;
  for (const direction of ['to right', '90deg', '100grad', '.25turn', '-270deg', '450deg']) {
    const result = parity(compiler, request(`div{background:linear-gradient(${direction},red,blue)}`));
    assert.deepEqual(paint(result), {direction:{degrees:90},stops:[stop(red),stop(blue)]});
    if (baseline) assert.deepEqual(result, baseline);
    baseline = result;
  }
  const colors = parity(compiler, request('div{background:LINEAR-GRADIENT(/*direction*/ to right,rgb(255 0 0),hsl(240 100% 50%))}'));
  assert.deepEqual(colors, baseline);
});

test('cascade resets remove gradient requirements and preserve native/WASM parity', async () => {
  const compiler = await createCompiler(wasm);
  for (const reset of ['background-image:none', 'background-image:initial', 'background-image:unset',
    'background:lime', 'background:none', 'background:initial', 'background:unset',
    'background-image:var(--missing)', 'background-image:var(--bad);--bad:12px']) {
    const result = parity(compiler, request(`div{background:linear-gradient(red,blue);${reset}}`));
    assert.equal(result.runtimeRequirements.layout_linear_gradients, undefined, reset);
    assert.equal(result.runtimeRequirements.capabilities.includes('layout-css-linear-gradient-v1'), false, reset);
  }
  for (const suffix of ['background-color:lime', 'background-image:var(--missing,linear-gradient(red,blue))']) {
    const result = parity(compiler, request(`div{background-image:linear-gradient(red,blue);${suffix}}`));
    assert.deepEqual(paint(result), {direction:{degrees:180},stops:[stop(red),stop(blue)]});
  }
});

test('expanded stop limit counts dual-position stops and rejects over-limit public input identically', async () => {
  const compiler = await createCompiler(wasm);
  for (const [authored, count] of [[['red 0% 100%'],2], [Array(128).fill('red 0% 100%'),256], [Array(256).fill('red'),256]]) {
    const result = parity(compiler, request(`div{background-image:linear-gradient(${authored.join(',')})}`));
    assert.equal(paint(result).stops.length, count);
  }
  for (const authored of [Array(257).fill('red'), [...Array(128).fill('red 0% 100%'), 'blue']]) {
    const result = parity(compiler, request(`div{background-image:linear-gradient(${authored.join(',')})}`), 'expanded stop limit', false);
    assert.ok(result.diagnostics.some(diagnostic => diagnostic.code === 'gradient-stop-limit'));
  }
});

test('excluded and malformed gradient syntax has exact native/WASM diagnostics and does not poison later calls', async () => {
  const compiler = await createCompiler(wasm);
  const good = request('div{background:linear-gradient(red,blue)}');
  const saved = compile(compiler, good);
  assert.equal(saved.ok, true);
  for (const value of ['linear-gradient(red)', 'linear-gradient(red,)', 'linear-gradient(to right left,red,blue)',
    'linear-gradient(red 2,blue)', 'linear-gradient(red calc(2px),blue)', 'linear-gradient(red 1vw,blue)',
    'linear-gradient(red,40%,blue)', 'linear-gradient(in oklab,red,blue)',
    'repeating-linear-gradient(red,blue)', 'radial-gradient(red,blue)',
    'linear-gradient(red,blue),linear-gradient(red,blue)', 'linear-gradient(1e999deg,red,blue)']) {
    const result = parity(compiler, request(`div{background-image:${value}}`), value, false);
    assert.ok(result.diagnostics.length > 0);
    assert.deepEqual(compile(compiler, good), saved);
  }
  const second = await createCompiler(wasm);
  assert.deepEqual(compile(second, good), saved);
});

test('realistic gradients preserve native/WASM artifacts with embedded Inter and image assets',async()=>{
 const compiler=await createCompiler(wasm);
 const fixtures=JSON.parse(fs.readFileSync(new URL('../validation/linear-gradient-realistic-cases.json',import.meta.url)));
 assert.equal(fixtures.length,6);
 const assets={inter:{kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/Inter-Regular.ttf',import.meta.url))]},photo:{kind:'image',bytes:[...fs.readFileSync(new URL('assets/quadrants.png',import.meta.url))]}};
 for(const fixture of fixtures){
  const result=parity(compiler,{...request(fixture.css,fixture.html),assets},fixture.name);
  assert.equal(result.runtimeRequirements.version,26);
  assert.ok(result.runtimeRequirements.layout_linear_gradients.length>0);
 }
});

test('extreme finite percentage stops retain exact public transport across native and WASM', async () => {
  const compiler = await createCompiler(wasm);
  // Independently authored inputs shared in meaning with the runtime numeric
  // regression. These transport checks do not assert browser/render pixels.
  const cases = [
    ['positive', 'red 0%, blue 3e38%', [red, blue], [0, 3e38]],
    ['negative', 'red -3e38%, blue 100%', [red, blue], [-3e38, 100]],
    ['symmetric-alpha', 'rgba(255,0,0,.25) -3e38%, rgba(0,0,255,.75) 3e38%', [0x40ff0000, 0xbf0000ff], [-3e38, 3e38]],
    ['hard-interior', 'red -3e38%, red 50%, blue 50%, blue 3e38%', [red, red, blue, blue], [-3e38, 50, 50, 3e38]],
  ];
  for (const [name, stops, colors, positions] of cases) {
    let baseline;
    for (const width of [240, 390, 768]) {
      const result = parity(compiler, {...request(`#gradient{width:100%;height:140px;background:linear-gradient(to right,${stops});}`), width}, `${name}@${width}`);
      const gradient = paint(result);
      assert.deepEqual(gradient.direction, {degrees:90});
      assert.equal(gradient.stops.length, positions.length);
      gradient.stops.forEach((entry, index) => {
        assert.equal(entry.color, colors[index]);
        assert.deepEqual(Object.keys(entry.position), ['percent']);
        assert.ok(Number.isFinite(entry.position.percent));
        assert.equal(Math.fround(entry.position.percent), Math.fround(positions[index]));
      });
      if (baseline) assert.deepEqual(gradient, baseline, 'stop transport is independent of the compile viewport');
      baseline = gradient;
    }
  }
});
