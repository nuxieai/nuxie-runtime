// Isolate CSSOM serialization from compiler behavior for pending shorthands.
import fs from 'node:fs';
import {chromium} from '@playwright/test';
const fixture=JSON.parse(fs.readFileSync(new URL('./cases.json',import.meta.url))).find(c=>c.name==='custom-flex-important');
const reset=fs.readFileSync(new URL('../src/reset.css',import.meta.url),'utf8');
const browser=await chromium.launch();
try {
  const page=await browser.newPage({viewport:{width:390,height:320}});
  const records=[];
  for(const mode of ['original','serialized','preserved']) {
    await page.setContent(`<style>${reset}</style>${fixture.html}`);
    records.push(await page.evaluate(({css,mode})=>{
      const sheet=new CSSStyleSheet();sheet.replaceSync(css);
      const serialized=[...sheet.cssRules].map(r=>r.cssText).join('\n');
      if(mode==='serialized') sheet.replaceSync(serialized);
      if(mode==='preserved') for(const rule of sheet.cssRules) rule.selectorText=`:where(body) :is(${rule.selectorText})`;
      document.adoptedStyleSheets=[sheet];
      const result={mode,serialized,elements:{}};
      for(const id of ['a','b']) {const e=document.getElementById(id),s=getComputedStyle(e);result.elements[id]={grow:s.flexGrow,shrink:s.flexShrink,basis:s.flexBasis,width:e.getBoundingClientRect().width};}
      return result;
    },{css:fixture.css,mode}));
  }
  if(JSON.stringify(records[0].elements)!==JSON.stringify(records[2].elements)) throw new Error('Selector mutation changed pending shorthand semantics');
  console.log(JSON.stringify({browserVersion:browser.version(),records},null,2));
} finally {await browser.close();}
