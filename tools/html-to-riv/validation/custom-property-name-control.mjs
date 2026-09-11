import {chromium} from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
const output=path.resolve(process.argv[2]);fs.mkdirSync(output,{recursive:true});
const browser=await chromium.launch();
try {
 const page=await browser.newPage(),results=[];
 for (const [name,value] of [['literal','var(--w)'],['dynamic','var(var(--name))'],['dynamic-fallback','var(var(--missing,--w),60px)'],['bad-name','var(var(--bad),60px)']]) {
  await page.setContent(`<style>#root{--name:--w;--w:40px;--bad:20px}#child{width:10px;width:${value};height:10px}</style><div id=root><div id=child></div></div>`);
  results.push(await page.evaluate(({name,value})=>({name,value,supported:CSS.supports('width',value),width:document.querySelector('#child').getBoundingClientRect().width,rule:document.styleSheets[0].cssRules[1].cssText}),{name,value}));
 }
 fs.writeFileSync(path.join(output,'direct-chromium-names.json'),JSON.stringify({browser:await browser.version(),results},null,2)+'\n');
} finally {await browser.close();}
