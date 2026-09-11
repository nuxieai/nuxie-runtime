import {chromium} from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
const output=path.resolve(process.argv[2]);
fs.mkdirSync(output,{recursive:true});
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const results=[];
 for(const [name,custom] of [
  ['unused-fallback','--safe:20%;--x:var(--safe,var(--y));--y:var(--x)'],
  ['used-fallback','--x:var(--safe,var(--y));--y:var(--x)'],
  ['self-fallback','--safe:20%;--x:var(--safe,var(--x))'],
  ['direct-cycle','--x:var(--y);--y:var(--x)'],
 ]) {
  await page.setContent(`<style>#root{width:200px;${custom}}#child{width:var(--x,30%);height:20px}</style><div id=root><div id=child></div></div>`);
  results.push(await page.evaluate(name=>({name,rootX:getComputedStyle(document.querySelector('#root')).getPropertyValue('--x'),rootY:getComputedStyle(document.querySelector('#root')).getPropertyValue('--y'),childWidth:document.querySelector('#child').getBoundingClientRect().width,css:document.styleSheets[0].cssRules[0].cssText}),name));
 }
 fs.writeFileSync(path.join(output,'direct-chromium-cycles.json'),JSON.stringify({browser:await browser.version(),results},null,2)+'\n');
} finally {await browser.close();}
