// Compare sequential original/clone draw frames with the equivalent live DOM.
// Usage: node stacking-clip-lifecycle.mjs RECORDING_DIR TOOLCHAIN NEW_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';

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
const reset=fs.readFileSync(new URL('../src/reset.css',import.meta.url),'utf8');
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const browser=await chromium.launch();
const cases=[];
try {
  assert.equal(browser.version(),'153.0.8010.12','Pinned Chrome version changed');
  const page=await browser.newPage({deviceScaleFactor:1});
  for(const fixture of lifecycle.cases) {
    await page.setContent(`<!doctype html><style>${reset}\n${fixture.css}</style>${fixture.html}`);
    for(const view of fixture.views) {
      await page.setViewportSize({width:view.width,height:view.height});
      await page.evaluate(({hostClip,parentClip})=>{
        document.getElementById('host').style.overflow=hostClip?'clip':'visible';
        document.getElementById('a').style.overflow=parentClip?'clip':'visible';
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
      const {metrics,failures,diff}=comparePixels(PNG.sync.read(browserBytes),native,boxes,false);
      fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
      cases.push({name,width:view.width,prefix,html:fixture.html,css:fixture.css,hostClip:view.hostClip,parentClip:view.parentClip,frame:view.frame,instance:view.instance,streamSha256:sha(fs.readFileSync(stream)),browserSha256:sha(browserBytes),nativeSha256:sha(nativeBytes),geometryFailures,metrics,failures});
      fs.writeFileSync(path.join(out,'replay.json'),JSON.stringify({browser:browser.version(),lifecycleSha256:sha(fs.readFileSync(lifecyclePath)),rendererSha256:manifest.files['renderer-replay'].sha256,cases},null,2)+'\n');
    }
    console.log(fixture.name,`${fixture.views.length} frames compared`);
  }
} finally { await browser.close(); }
const failed=cases.filter(c=>c.failures.length||c.geometryFailures.length);
console.log(JSON.stringify({total:cases.length,failed:failed.length}));
if(failed.length)process.exitCode=1;
