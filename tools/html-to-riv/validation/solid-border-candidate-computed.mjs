// Read-only independent Chrome border-width observations; no authoring output.
import fs from 'node:fs';import assert from 'node:assert/strict';import{chromium}from'@playwright/test';
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');
try{const page=await browser.newPage({deviceScaleFactor:1});await page.setViewportSize({width:390,height:160});
 const widths=[0,.01,.25,.5,.99,1,1.01,1.5,1.99,2,2.75,3.125,9.999];const rows=[];
 for(const width of widths){await page.setContent(`<style>*{box-sizing:border-box}div{width:80px;height:40px;border:${width}px solid #123456;padding:3px}</style><div id="p"></div>`);const actual=await page.evaluate(()=>{const e=document.getElementById('p'),s=getComputedStyle(e);return{border:s.borderTopWidth,width:e.getBoundingClientRect().width,height:e.getBoundingClientRect().height}});rows.push({authoredWidth:width,...actual});}
 fs.writeFileSync(new URL('../output/solid-border-candidate-r1/chrome-computed-widths.json',import.meta.url),JSON.stringify({browser:browser.version(),dpr:1,rows},null,2)+'\n');
}finally{await browser.close()}
