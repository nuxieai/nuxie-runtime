// Replay independently captured Chromium fixtures with one immutable toolchain.
// No shared Playwright reporter files are read or written.
// Usage: node replay-oracle.mjs ORACLE_JSON TOOLCHAIN NEW_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';
const [oracleArg, toolchainArg, outputArg, experimentArg] = process.argv.slice(2);
assert(!experimentArg || experimentArg === '--clip-margin-experiment', 'Unknown experiment');
const clipMarginExperiment = experimentArg === '--clip-margin-experiment';
if (!outputArg) throw new Error('Expected ORACLE_JSON TOOLCHAIN NEW_OUTPUT');
const oraclePath=path.resolve(oracleArg), toolchain=path.resolve(toolchainArg), output=path.resolve(outputArg);
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const sha=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const manifest=JSON.parse(fs.readFileSync(path.join(toolchain,'manifest.json')));
const artifacts = ['html-to-riv','probe','renderer-replay'];
if (manifest.files['html-to-riv.wasm']) artifacts.push('html-to-riv.wasm');
else assert.equal(manifest.profile, 'native-pixel-experiment', 'Missing WASM requires an explicit native-only experiment');
for(const name of artifacts)
  assert.equal(sha(fs.readFileSync(path.join(toolchain,name))),manifest.files[name].sha256,name);
assert(!fs.existsSync(output),'Refusing to overwrite evidence');
fs.mkdirSync(output,{recursive:true});
const oracle=JSON.parse(fs.readFileSync(oraclePath)),cases=[];
const env={...process.env,NUXIE_NATIVE_GLYPHS:process.env.NUXIE_NATIVE_GLYPHS||'1'};
const run=(name,args)=>execFileSync(path.join(toolchain,name),args,{env,stdio:'pipe'});
for(const fixture of oracle.cases){
  // This runner handles the standard Inter font and quadrant image fixtures.
  assert(!fixture.fontAsset && !fixture.textRegions,'Fixture needs a richer oracle runner');
  const request={html:fixture.html,css:fixture.css,width:390,height:320};
  let runtimeClipMarginExperiment = null;
  if (clipMarginExperiment) {
    const declarations = [...fixture.css.matchAll(/overflow-clip-margin:(content-box|padding-box|border-box) ([0-9.]+)px/g)];
    assert.equal(declarations.length, 1, 'Experiment requires one explicit fixture margin');
    const [, origin, offset] = declarations[0];
    request.css = fixture.css.replace(declarations[0][0], '');
    // This diagnostic matrix has exactly one host declaration. Hidden and
    // single-axis controls keep their normal clip policy, as pinned Chrome does.
    assert.equal([...fixture.css.matchAll(/overflow:/g)].length, 1);
    if (/overflow:clip;/.test(fixture.css)) {
      runtimeClipMarginExperiment = ['host', origin, Number(offset)];
      env.NUXIE_CSS_CLIP_MARGIN_EXPERIMENT = JSON.stringify(runtimeClipMarginExperiment);
    } else {
      delete env.NUXIE_CSS_CLIP_MARGIN_EXPERIMENT;
    }
  }
  if(fixture.font)request.assets={inter:{kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(path.join(root,'tools/html-to-riv/tests/assets/Inter-Regular.ttf'))]}};
  if(fixture.image){
    const imageAsset=fixture.imageAsset||'quadrants.png';
    assert.equal(path.basename(imageAsset),imageAsset);
    request.assets={...request.assets,photo:{kind:'image',bytes:[...fs.readFileSync(path.join(root,'tools/html-to-riv/tests/assets',imageAsset))]}};
  }
  const compiled=path.join(output,fixture.name);
  fs.writeFileSync(compiled+'.json',JSON.stringify(request));
  run('html-to-riv',[compiled+'.json',compiled+'.riv']);
  const rivHash=sha(fs.readFileSync(compiled+'.riv'));
  for(const viewport of fixture.viewports){
    const prefix=compiled+'-'+viewport.width;
    run('probe',[compiled+'.riv',compiled+'.map.json',String(viewport.width),'320',prefix]);
    run('renderer-replay',['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic']);
    assert.equal(sha(fs.readFileSync(compiled+'.riv')),rivHash);
    const bounds=JSON.parse(fs.readFileSync(prefix+'.bounds.json'));
    assert.deepEqual(Object.keys(bounds).sort(),Object.keys(viewport.boxes).sort());
    const geometryFailures=[];let maxGeometryError=0;
    for(const [id,box] of Object.entries(viewport.boxes))for(const axis of ['x','y','width','height']){
      assert(Number.isFinite(box[axis]), `Non-finite browser geometry: ${fixture.name}/${id}/${axis}`);
      assert(Number.isFinite(bounds[id][axis]), `Non-finite native geometry: ${fixture.name}/${id}/${axis}`);
      const error=Math.abs(bounds[id][axis]-box[axis]);maxGeometryError=Math.max(maxGeometryError,error);
      if(error>.1)geometryFailures.push({id,axis,error,browser:box[axis],native:bounds[id][axis]});
    }
    const browserFile=path.join(path.dirname(oraclePath),`${fixture.name}-${viewport.width}.png`);
    fs.copyFileSync(browserFile,prefix+'.browser.png');
    const browser=PNG.sync.read(fs.readFileSync(browserFile)),native=PNG.sync.read(fs.readFileSync(prefix+'.native.png'));
    assert.deepEqual([native.width,native.height],[viewport.width,320]);
    const {metrics,failures,diff}=comparePixels(browser,native,viewport.boxes,!!fixture.font);
    fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
    cases.push({name:fixture.name,width:viewport.width,html:fixture.html,css:fixture.css,...(clipMarginExperiment ? {compilerRequestCss:request.css,runtimeClipMarginExperiment,qualification:'runtime-experiment-only'} : {}),prefix,rivSha256:rivHash,browserSha256:sha(fs.readFileSync(browserFile)),nativeSha256:sha(fs.readFileSync(prefix+'.native.png')),maxGeometryError,geometryFailures,metrics,failures});
    fs.writeFileSync(path.join(output,'replay.json'),JSON.stringify({oracle:oraclePath,oracleSha256:sha(fs.readFileSync(oraclePath)),toolchain,manifest,nativeGlyphs:env.NUXIE_NATIVE_GLYPHS,cases},null,2)+'\n');
  }
}
const failed=cases.filter(c=>c.geometryFailures.length||c.failures.length).length;
console.log(JSON.stringify({comparisons:cases.length,passed:cases.length-failed,failed}));
if(failed)process.exitCode=1;
