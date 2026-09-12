// Browser-only numeric consumer characterization; no native qualification.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import crypto from 'node:crypto';import{fileURLToPath}from'node:url';import{chromium}from'@playwright/test';
const[fixturesPath,out]=process.argv.slice(2).map(p=>path.resolve(p));assert(fixturesPath&&out&&!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const resetPath=fileURLToPath(new URL('../src/reset.css',import.meta.url)),reset=fs.readFileSync(resetPath,'utf8');
const fixtures=JSON.parse(fs.readFileSync(fixturesPath));fs.copyFileSync(fixturesPath,path.join(out,'cases.json'));fs.copyFileSync(resetPath,path.join(out,'reset.css'));
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');const rows=[];
try{const page=await browser.newPage({deviceScaleFactor:1,colorScheme:'light',locale:'en-US'});
 for(const fixture of fixtures)for(const [width,height]of[[240,160],[390,200],[768,120]]){
  await page.setViewportSize({width,height});await page.setContent(`<!doctype html><style>${reset}\n${fixture.css}</style>${fixture.html}`);
  const boxes=await page.evaluate(()=>Object.fromEntries([...document.querySelectorAll('[id]')].map(e=>{const s=getComputedStyle(e),r=e.getBoundingClientRect(),m=e.computedStyleMap();return[e.id,{x:r.x,y:r.y,width:r.width,height:r.height,color:s.backgroundColor,typedWidth:{value:m.get('width').value??null,unit:m.get('width').unit??null},typedFontSize:{value:m.get('font-size').value??null,unit:m.get('font-size').unit??null}}];})));
  const paintFailures=Object.entries(fixture.expectedPaint??{}).filter(([id,color])=>boxes[id]?.color!==color).map(([id,color])=>({id,expected:color,actual:boxes[id]?.color}));
  rows.push({name:fixture.name,width,height,boxes,paintFailures});
 }
}finally{await browser.close();}
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({scope:'Pinned Chrome-only CSSOM/TypedOM and rectangle observations before native compiler qualification.',browser:'153.0.8010.12',fixturesSha256:hash(fixturesPath),resetSha256:hash(resetPath),driverSha256:hash(fileURLToPath(import.meta.url)),rows},null,2)+'\n');
console.log(JSON.stringify({cases:fixtures.length,observations:rows.length,paintFailures:rows.filter(r=>r.paintFailures.length).map(r=>({name:r.name,failures:r.paintFailures}))}));
if(rows.some(r=>r.paintFailures.length))process.exitCode=1;
