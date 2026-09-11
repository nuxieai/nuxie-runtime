// Pin browser behavior for the isolated opacity component parser's candidate subset.
import {chromium} from '@playwright/test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const output=process.argv[2];
assert(output && !fs.existsSync(output),'Provide a new evidence file');
const accepted=[['.5',.5],['50%',.5],['+2.5e1%',.25],[' /* lead */ .125 /* tail */ ',.125],['1e-2',.01],['-1',0],['-200%',0],['-1e100',0],['-0',0],['-0%',0],['2',1],['200%',1],['1e100',1],['1e100%',1]];
const rejected=['0 1','50 %','1px','NaN','infinity','.5,','--1'];
const browser=await chromium.launch();
try {
 assert.equal(browser.version(),'153.0.8010.12');
 const page=await browser.newPage();
 const rows=await page.evaluate(({accepted,rejected})=>{
  const el=document.createElement('div');document.body.append(el);
  return [...accepted.map(([value,expected])=>({value,expected})),...rejected.map(value=>({value,expected:null}))].map(row=>{
   el.style.opacity='';el.style.opacity=row.value;
   return {...row,accepted:el.style.opacity!=='',computed:getComputedStyle(el).opacity};
  });
 },{accepted,rejected});
 for(const row of rows){
  assert.equal(row.accepted,row.expected!==null,row.value);
  if(row.accepted) assert.equal(Number(row.computed),row.expected,row.value);
 }
 fs.writeFileSync(output,JSON.stringify({browser:browser.version(),status:'reference-pass',rows},null,2)+'\n');
 console.log(`${rows.length} Chrome opacity value controls pass`);
} finally {await browser.close();}
