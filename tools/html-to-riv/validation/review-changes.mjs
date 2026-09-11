// Render full browser/native/diff images for changed and newly added cases.
import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
const dir=path.resolve(process.argv[2]);
const report=JSON.parse(fs.readFileSync(path.join(dir,'review.json')));
const comparison=JSON.parse(fs.readFileSync(path.join(dir,'baseline-comparison.json')));
const keys=new Set([...comparison.metricChanges,...comparison.added,...comparison.regressions]);
const cases=report.cases.filter(c=>keys.has(`${c.project}:${c.name}:${c.width}`));
const escape=s=>String(s).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('"','&quot;');
const browser=await chromium.launch();
try {
 const page=await browser.newPage({viewport:{width:2400,height:1000}});
 for(let start=0;start<cases.length;start+=6){
  const subset=cases.slice(start,start+6);
  const html='<style>body{font:14px system-ui;background:#ddd}section{background:white;padding:8px;margin:8px;width:max-content}article{display:flex;gap:12px}img{display:block}</style>'+subset.map(c=>`<section><b>${escape(c.name)} ${c.width} ${escape(c.status)}</b><article>`+['browser','native','diff'].map(k=>`<div>${k}<img src="data:image/png;base64,${fs.readFileSync(c.artifactPrefix+'.'+k+'.png').toString('base64')}"></div>`).join('')+'</article></section>').join('');
  await page.setContent(html);await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  await page.screenshot({path:path.join(dir,`changed-${start/6}.png`),fullPage:true});
 }
 fs.writeFileSync(path.join(dir,'changed-contact-index.json'),JSON.stringify(cases.map((c,i)=>({sheet:Math.floor(i/6),name:c.name,width:c.width})),null,2));
 console.log(JSON.stringify({cases:cases.length,sheets:Math.ceil(cases.length/6),visuallyInspected:false}));
}finally{await browser.close();}
