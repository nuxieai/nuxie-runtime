import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {PNG} from 'pngjs';
import {compareGradientBorderPixels} from './gradient-border-pixels.mjs';
const root=path.resolve(new URL('../../../',import.meta.url).pathname);
const base=path.join(root,'output/playwright/html-to-riv');
const sha=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const read=p=>JSON.parse(fs.readFileSync(p));
const usedPath=path.join(base,'linear-gradient-composition-reference/fractional-border-used-values.json');
const used=read(usedPath);assert.equal(used.browser,'153.0.8010.12');
const rows=[];let rejectionControls=0;
for(const revision of ['r1','r3']) {
 const replayPath=path.join(base,`linear-gradient-composition-native-${revision}/replay.json`);
 for(const row of read(replayPath).cases.filter(row=>row.name.endsWith('fractional-border'))) {
  const measure=used.rows.find(v=>v.viewportWidth===row.width);assert(measure);
  assert.equal(row.css,used.css);assert.equal(row.html,used.html);
  const files=['browser','native'].map(kind=>row.prefix+'.'+kind+'.png');
  files.forEach((file,i)=>assert.equal(sha(file),row[['browserSha256','nativeSha256'][i]]));
  const [browser,native]=files.map(file=>PNG.sync.read(fs.readFileSync(file)));
  const options={viewport:{width:row.width,height:320},box:measure,borderLeftWidth:parseFloat(measure.borderLeftWidth),borderRightWidth:parseFloat(measure.borderRightWidth)};
  const result=compareGradientBorderPixels(browser,native,options);
  assert.equal(result.passed,revision==='r3');
  assert.equal(compareGradientBorderPixels(browser,browser,options).passed,true);
  for(const change of [o=>delete o.box.x,o=>o.box.x=-100,o=>o.box.y=400,o=>o.viewport.width++,o=>o.borderLeftWidth=2]) {
   const bad=structuredClone(options);change(bad);assert.throws(()=>compareGradientBorderPixels(browser,native,bad));rejectionControls++;
  }
  rows.push({revision,width:row.width,passed:result.passed,samples:result.samples,browserSha256:sha(files[0]),nativeSha256:sha(files[1]),replaySha256:sha(replayPath)});
 }
}
assert.equal(rows.length,6);
const receipt={status:'controls-passed',redFrames:3,greenFrames:3,selfControlFrames:6,rejectionControls,usedEvidenceSha256:sha(usedPath),moduleSha256:sha(new URL('./gradient-border-pixels.mjs',import.meta.url)),rows};
const output=path.join(base,'linear-gradient-border-js-controls.json');assert(!fs.existsSync(output),'Refusing to overwrite evidence');fs.writeFileSync(output,JSON.stringify(receipt,null,2)+'\n');
console.log(JSON.stringify({status:receipt.status,redFrames:3,greenFrames:3,selfControlFrames:6,rejectionControls}));
