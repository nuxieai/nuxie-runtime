import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
const dir=path.resolve(process.argv[2]);
const summary=JSON.parse(fs.readFileSync(dir+'/summary.json'));
const groups=Map.groupBy(summary,c=>c.name.split('-').slice(0,3).join('-'));
const browser=await chromium.launch();
try {
 const page=await browser.newPage({viewport:{width:800,height:1000}});
 for(const [name,cases] of groups) {
  const html='<style>body{font:14px system-ui;background:#ddd}section{background:white;padding:8px;margin:8px}article{display:flex;gap:12px}.crop{width:180px;height:170px;overflow:hidden}img{width:780px;height:480px;max-width:none;image-rendering:pixelated}</style>'+cases.map(c=>{
   const report=JSON.parse(fs.readFileSync(`${dir}/${c.name}/report.json`));const r=report.cases[0];
   return `<section><b>${c.name}: ${r.failures.join(', ')||'pass'}</b><article>`+Object.entries(r.images).map(([kind,file])=>`<div>${kind}<div class=crop><img src="data:image/png;base64,${fs.readFileSync(file).toString('base64')}"></div></div>`).join('')+'</article></section>';
  }).join('');
  await page.setContent(html);await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  await page.screenshot({path:dir+'/review-'+name+'.png',fullPage:true});
 }
}finally{await browser.close();}
