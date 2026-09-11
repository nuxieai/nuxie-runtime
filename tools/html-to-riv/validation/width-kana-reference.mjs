// Firefox innerText omits width/kana conversions. Compare actual painted text
// with independent, explicit Unicode references in the same pinned font.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {firefox} from '@playwright/test';
import {PNG} from 'pngjs';
const moduleDir=path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const output=process.env.NUXIE_TRANSFORM_REFERENCE_DIR || path.resolve(moduleDir,'../../output/playwright/html-to-riv/width-kana-reference');
fs.mkdirSync(output,{recursive:true});
const reference=JSON.parse(fs.readFileSync(new URL('./width-kana-reference.json',import.meta.url)));
const font=fs.readFileSync(path.join(moduleDir,'tests/assets/NuxieJapaneseFixture-Regular.otf'));
const browser=await firefox.launch();
const results=[];
try {
  const page=await browser.newPage({viewport:{width:800,height:200},deviceScaleFactor:1});
  await page.setContent(`<style>@font-face{font-family:Fixture;src:url(data:font/otf;base64,${font.toString('base64')})}body{margin:0;background:white}p{margin:0;width:800px;height:80px;font:24px/40px Fixture;color:black;white-space:pre}</style><p id=a></p><p id=b></p>`);
  await page.evaluate(()=>document.fonts.load('24px Fixture'));
  for(const item of reference.cases) {
    const supported=await page.evaluate(({mode,white,source,expected})=>{
      const a=document.querySelector('#a'),b=document.querySelector('#b');
      a.style.textTransform=mode;a.style.whiteSpace=white;a.textContent=source;b.textContent=expected;
      return CSS.supports('text-transform',mode) && getComputedStyle(a).textTransform!=='none';
    },item);
    assert.ok(supported,`${item.name}: reference browser must apply the transform`);
    const actual=await page.locator('#a').screenshot(),expected=await page.locator('#b').screenshot();
    const a=PNG.sync.read(actual),b=PNG.sync.read(expected);
    let changedPixels=0;
    for(let i=0;i<a.data.length;i+=4)if(!a.data.subarray(i,i+4).equals(b.data.subarray(i,i+4)))changedPixels++;
    fs.writeFileSync(path.join(output,`${item.name}-actual.png`),actual);
    fs.writeFileSync(path.join(output,`${item.name}-expected.png`),expected);
    results.push({...item,changedPixels,passed:item.expectDifference?changedPixels>0:changedPixels===0});
  }
  fs.writeFileSync(path.join(output,'report.json'),JSON.stringify({browser:browser.version(),cases:results},null,2));
  fs.writeFileSync(path.join(output,'gallery.html'), '<!doctype html><meta charset="utf-8"><title>Width/kana browser reference</title><style>body{font:15px system-ui;margin:24px}section{margin:24px 0}.pair{display:flex;gap:16px;overflow:auto}img{width:800px;height:80px;border:1px solid #ddd}</style><h1>Firefox transform reference</h1><p>Transformed source (left) and explicit Unicode reference (right). This is browser-reference validation, not native renderer validation.</p>'+results.map(c=>`<section><strong>${c.name}: ${c.passed?'PASS':'FAIL'}${c.expectDifference?' (negative control)':''}</strong><div class="pair">${['actual','expected'].map(k=>`<img src="data:image/png;base64,${fs.readFileSync(path.join(output,c.name+'-'+k+'.png')).toString('base64')}">`).join('')}</div></section>`).join(''));
  assert.deepEqual(results.filter(result=>!result.passed),[], 'Transform rendering references failed');
  console.log(`${results.length} width/kana references and negative controls pass Firefox ${browser.version()}`);
} finally {await browser.close();}
