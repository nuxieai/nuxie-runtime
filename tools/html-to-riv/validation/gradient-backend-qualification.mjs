// Reuse the pinned Chrome captures and existing pixel thresholds for alternate native execution.
// Default atomics preserves CSS path fill rules; clockwise-atomic is an explicit forced-winding control.
// Usage: node SCRIPT RECORDING REPLAY TOOLCHAIN NEW_OUTPUT [atomics|clockwise-atomic]
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';
const [recording,replay,toolchain,out] = process.argv.slice(2,6).map(p=>path.resolve(p));
const selectedMode=process.argv[6]??'atomics';
assert(['atomics','clockwise-atomic'].includes(selectedMode));
assert(out && !fs.existsSync(out));fs.mkdirSync(out);
const read=p=>JSON.parse(fs.readFileSync(p));
const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const renderer=path.join(toolchain,'renderer-replay');
const manifest=read(path.join(toolchain,'manifest.json'));
assert.equal(sha(renderer),manifest.files['renderer-replay'].sha256);
const source=read(path.join(recording,'lifecycle.json')), reference=read(path.join(replay,'replay.json'));
assert.equal(source.corpus,'linear-gradient-composition');assert.equal(reference.browser,'153.0.8010.12');
assert.equal(reference.lifecycleSha256,sha(path.join(recording,'lifecycle.json')));
const refs=new Map(reference.cases.map(c=>[c.name,c]));
const cases=[], probes=[], repeats=[];
const report=()=>fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({status:cases.length===54&&cases.every(c=>!c.failures.length)&&repeats.every(c=>c.identical)?'passed':'incomplete-or-failed',scope:selectedMode==='atomics'?'Ordinary Atomics Metal replay preserving path fill rules against pinned Chrome pixels; MSAA availability probed separately':'Explicit forced-clockwise control; not CSS equivalence qualification',cssEquivalenceEligible:selectedMode==='atomics',rendererSha256:sha(renderer),mode:selectedMode,referenceRendererSha256:reference.rendererSha256,lifecycleSha256:sha(path.join(recording,'lifecycle.json')),referenceSha256:sha(path.join(replay,'replay.json')),cases,repeats,probes},null,2)+'\n');
const run=(view,backend,mode,output)=>spawnSync(renderer,['--stream',view.stream,'--frame',String(view.frame),'--backend',backend,'--mode',mode,'--output',output],{encoding:'utf8'});
for(const fixture of source.cases) for(const view of fixture.views.filter(v=>v.instance===0&&v.frame<3)) {
 const name=`${fixture.name}-${view.instance}-step${view.frame}`,ref=refs.get(name);assert(ref);
 assert.equal(ref.streamSha256,sha(view.stream));assert.equal(ref.browserSha256,sha(ref.prefix+'.browser.png'));assert.equal(ref.nativeSha256,sha(ref.prefix+'.native.png'));
 const output=path.join(out,name+'.atomic.png'), result=run(view,'rust-metal-atomic',selectedMode,output);
 assert.equal(result.status,0,result.stderr);
 const browser=PNG.sync.read(fs.readFileSync(ref.prefix+'.browser.png')),actual=PNG.sync.read(fs.readFileSync(output));
 const {metrics,failures,diff}=comparePixels(browser,actual,view.bounds,false);
 fs.writeFileSync(path.join(out,name+'.diff.png'),PNG.sync.write(diff));
 cases.push({name,width:view.width,frame:view.frame,backend:'rust-metal-atomic',mode:selectedMode,streamSha256:sha(view.stream),browserSha256:ref.browserSha256,nativeSha256:sha(output),referenceMetalSha256:ref.nativeSha256,identicalToMetal:sha(output)===ref.nativeSha256,metrics,failures});
 if(view.frame===1) {
  const again=path.join(out,name+'.repeat.png');const repeat=run(view,'rust-metal-atomic',selectedMode,again);assert.equal(repeat.status,0,repeat.stderr);
  repeats.push({name,firstSha256:sha(output),repeatSha256:sha(again),identical:sha(output)===sha(again)});
 }
 report();
}
const example=source.cases.find(c=>c.name.includes('corner-transparent')).views[1];
for(const [backend,mode] of [['rust-metal','msaa'],['rust-metal-atomic','msaa'],['rust-webgpu-exact','msaa'],['rust-vulkan-exact','msaa'],['ffi-dawn','msaa'],['ffi-metal','clockwise-atomic']]) {
 const output=path.join(out,backend+'-'+mode+'.png'),result=run(example,backend,mode,output);
 probes.push({backend,mode,exitCode:result.status,signal:result.signal,stdout:result.stdout,stderr:result.stderr,rendered:fs.existsSync(output),...(fs.existsSync(output)?{imageSha256:sha(output)}:{})});
}
report();
console.log(JSON.stringify({cases:cases.length,failed:cases.filter(c=>c.failures.length).length,identicalToMetal:cases.filter(c=>c.identicalToMetal).length,repeats:repeats.length,probes:probes.map(p=>({backend:p.backend,mode:p.mode,exitCode:p.exitCode,rendered:p.rendered}))}));
assert(cases.every(c=>c.failures.length===0));assert(repeats.every(c=>c.identical));
