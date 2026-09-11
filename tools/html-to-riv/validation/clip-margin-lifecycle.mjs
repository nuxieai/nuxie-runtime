// Compare sequential original/clone draw frames with the equivalent live DOM.
// Usage: node clip-margin-lifecycle.mjs RECORDING_DIR TOOLCHAIN NEW_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';
import {lifecyclePixelProfile} from './lifecycle-pixel-profile.mjs';

const [recordingArg,toolchainArg,outArg]=process.argv.slice(2);
assert(outArg,'Expected RECORDING_DIR TOOLCHAIN NEW_OUTPUT');
const recording=path.resolve(recordingArg),toolchain=path.resolve(toolchainArg),out=path.resolve(outArg);
assert(!fs.existsSync(out),'Refusing to overwrite evidence');
fs.mkdirSync(out,{recursive:true});
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const manifest=JSON.parse(fs.readFileSync(path.join(toolchain,'manifest.json')));
const renderer=path.join(toolchain,'renderer-replay');
assert.equal(sha(fs.readFileSync(renderer)),manifest.files['renderer-replay'].sha256);
const lifecyclePath=path.join(recording,'lifecycle.json');
const lifecycle=JSON.parse(fs.readFileSync(lifecyclePath));
assert(lifecycle.kind === undefined || ['border-diagnostic','border-public','ellipse-diagnostic','opacity-diagnostic','opacity-public','gradient-diagnostic','gradient-public'].includes(lifecycle.kind), 'Unknown lifecycle kind');
const diagnosticBorder = lifecycle.kind === 'border-diagnostic';
const publicBorder = lifecycle.kind === 'border-public';
const diagnosticEllipse = lifecycle.kind === 'ellipse-diagnostic';
const diagnosticOpacity = lifecycle.kind === 'opacity-diagnostic';
const diagnosticGradient = lifecycle.kind === 'gradient-diagnostic';
const publicGradient = lifecycle.kind === 'gradient-public';
const publicOpacity = lifecycle.kind === 'opacity-public';
const reset=fs.readFileSync(new URL('../src/reset.css',import.meta.url),'utf8');
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const browser=await chromium.launch();
const cases=[];
try {
  assert.equal(browser.version(),'153.0.8010.12','Pinned Chrome version changed');
  const page=await browser.newPage({deviceScaleFactor:1});
  for(const fixture of lifecycle.cases) {
    const pixelProfile=lifecyclePixelProfile(fixture);
    if(lifecycle.corpus==='linear-gradient-realistic') {
      const authored=JSON.parse(fs.readFileSync(new URL('./linear-gradient-realistic-cases.json',import.meta.url))).find(c=>c.name===fixture.name);
      assert(authored && authored.html===fixture.html && authored.css===fixture.css);
      assert.equal(fixture.font,authored.font);assert.equal(fixture.image,authored.image);
      assert.equal(fixture.nativeGlyphs,lifecycle.nativeGlyphs);
      const request=JSON.parse(fs.readFileSync(path.join(recording,fixture.name+'.request.json')));
      assert.equal(request.html,authored.html);assert.equal(request.css,authored.css);
      assert.deepEqual(Buffer.from(request.assets.inter.bytes),fs.readFileSync(new URL('../tests/assets/Inter-Regular.ttf',import.meta.url)));
      assert.deepEqual(Buffer.from(request.assets.photo.bytes),fs.readFileSync(new URL('../tests/assets/quadrants.png',import.meta.url)));
      assert.equal(request.assets.inter.family,'Inter');assert.equal(request.assets.inter.weight,400);
    }
    const browserHtml = fixture.image
      ? fixture.html.replaceAll('asset:photo', `data:image/png;base64,${fs.readFileSync(new URL('../tests/assets/quadrants.png',import.meta.url)).toString('base64')}`)
      : fixture.html;
    const fontFace = fixture.font ? `@font-face{font-family:Inter;src:url(data:font/ttf;base64,${fs.readFileSync(new URL('../tests/assets/Inter-Regular.ttf',import.meta.url)).toString('base64')})}` : '';
    await page.setContent(`<!doctype html><style>${reset}\n${fontFace}\n${fixture.css}</style>${browserHtml}`);
    await page.evaluate(()=>document.fonts.ready);
    await page.evaluate(()=>Promise.all([...document.images].map(image=>image.decode())));
    for(const view of fixture.views) {
      await page.setViewportSize({width:view.width,height:view.height});
      if (!diagnosticBorder && !publicBorder && !diagnosticEllipse && !diagnosticOpacity && !publicOpacity && !diagnosticGradient && !publicGradient) await page.evaluate(({margin})=>{
        document.getElementById('a').style.overflowClipMargin=margin;
      },view);
      const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(el=>{
        const r=el.getBoundingClientRect();return [el.id,{x:r.x,y:r.y,width:r.width,height:r.height}];
      })));
      assert.deepEqual(Object.keys(boxes).sort(),Object.keys(view.bounds).sort());
      const geometryFailures=[];
      for(const [id,b] of Object.entries(boxes))for(const axis of ['x','y','width','height']) {
        const actual=view.bounds[id][axis];assert(Number.isFinite(actual));
        if(Math.abs(actual-b[axis])>.1)geometryFailures.push({id,axis,browser:b[axis],native:actual});
      }
      const name=`${fixture.name}-${view.instance}-step${view.frame}`;
      const prefix=path.join(out,name);
      const stream=path.resolve(root,view.stream);
      execFileSync(renderer,['--stream',stream,'--frame',String(view.frame),'--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic'],{stdio:'pipe'});
      await page.screenshot({path:prefix+'.browser.png'});
      const browserBytes=fs.readFileSync(prefix+'.browser.png'),nativeBytes=fs.readFileSync(prefix+'.native.png');
      const native=PNG.sync.read(nativeBytes);
      assert.deepEqual([native.width,native.height],[view.width,view.height]);
      const {metrics,failures,diff}=comparePixels(PNG.sync.read(browserBytes),native,boxes,pixelProfile.font);
      fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
      cases.push({name,width:view.width,prefix,html:fixture.html,css:fixture.css,pixelProfile,
        ...(diagnosticGradient ? {qualification:"runtime-experiment-only",compilerRequestCss:fixture.compilerRequestCss,
          injectedGradient:fixture.injectedGradient,injectedPixelBounds:fixture.injectedPixelBounds,runtimeRequirements:fixture.runtimeRequirements} : {}),
        ...(diagnosticOpacity ? {qualification:"runtime-experiment-only",compilerRequestCss:fixture.compilerRequestCss,
          injectedOpacity:fixture.injectedOpacity,runtimeRequirements:fixture.runtimeRequirements} : {}),
        ...((publicBorder || publicOpacity || publicGradient) ? {qualification:"public-compiler-lifecycle",runtimeRequirements:fixture.runtimeRequirements} : {}),
        ...(diagnosticEllipse ? {qualification:"runtime-experiment-only",compilerRequestCss:fixture.compilerRequestCss,
          injectedEllipseRadii:fixture.injectedEllipseRadii,runtimeRequirements:fixture.runtimeRequirements} : {}),
        ...(diagnosticBorder ? {qualification:"runtime-experiment-only",compilerRequestCss:fixture.compilerRequestCss,
          sideWidths:fixture.sideWidths,sideColors:fixture.sideColors,
          injectedBorderWidth:fixture.injectedBorderWidth,injectedBorderColor:fixture.injectedBorderColor} : {}),margin:view.margin,frame:view.frame,instance:view.instance,streamSha256:sha(fs.readFileSync(stream)),browserSha256:sha(browserBytes),nativeSha256:sha(nativeBytes),geometryFailures,metrics,failures});
      fs.writeFileSync(path.join(out,'replay.json'),JSON.stringify({browser:browser.version(),lifecycleSha256:sha(fs.readFileSync(lifecyclePath)),rendererSha256:manifest.files['renderer-replay'].sha256,cases},null,2)+'\n');
    }
    console.log(fixture.name,`${fixture.views.length} frames compared`);
  }
} finally { await browser.close(); }
// A repeated logical state must rasterize identically, including across clones.
// Compare native bytes independently of Chrome's possible edge repaint variation.
const repeatedStates = new Map();
for (const view of cases) {
  const key = JSON.stringify([view.html, view.css, view.width, view.margin]);
  const previous = repeatedStates.get(key);
  if (previous) assert.equal(view.nativeSha256, previous.nativeSha256,
    `Native lifecycle state changed: ${previous.name} versus ${view.name}`);
  else repeatedStates.set(key, view);
}
fs.writeFileSync(path.join(out, 'repeat-stability.json'), JSON.stringify({
  status: 'complete', frames: cases.length, states: repeatedStates.size,
  repeatedFrames: cases.length - repeatedStates.size,
  replaySha256: sha(fs.readFileSync(path.join(out, 'replay.json'))),
}, null, 2) + '\n');
const failed=cases.filter(c=>c.failures.length||c.geometryFailures.length);
console.log(JSON.stringify({total:cases.length,failed:failed.length}));
if(failed.length)process.exitCode=1;
