import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
// Usage: node composition-review.mjs TEST_RESULTS OUTPUT [CASE_NAME_PREFIX]
const input=path.resolve(process.argv[2]),output=path.resolve(process.argv[3]);
const casePrefix=process.argv[4]||'';
fs.mkdirSync(output,{recursive:true});const entries=[];
for(const folder of fs.readdirSync(input)) {
 const attachments=path.join(input,folder,'attachments');if(!fs.existsSync(attachments))continue;
 const files=fs.readdirSync(attachments),record=files.find(n=>n.startsWith('visual-review-'));
 if(!record)continue;const data=JSON.parse(fs.readFileSync(path.join(attachments,record)));
 if(!data.name.startsWith(casePrefix))continue;
 const images=Object.fromEntries(['browser','native','diff'].map(kind=>[kind,files.find(n=>n.startsWith(kind+'-png-'))]));
 if(Object.values(images).some(v=>!v))continue;
 entries.push({...data,images:Object.fromEntries(Object.entries(images).map(([k,v])=>[k,path.join(attachments,v)]))});
}
const browser=await chromium.launch();try {
 const page=await browser.newPage({viewport:{width:1250,height:1100}});
 for(const [name,cases] of Map.groupBy(entries,e=>e.name)) {
  cases.sort((a,b)=>a.width-b.width);
  await page.setContent('<style>body{font:14px system-ui;background:#ddd}section{background:white;padding:8px;margin:8px}article{display:flex;gap:12px}img{max-width:380px}</style>'+cases.map(c=>`<section><b>${name}, ${c.width}px</b><article>`+Object.entries(c.images).map(([k,v])=>`<div>${k}<br><img src="data:image/png;base64,${fs.readFileSync(v).toString('base64')}"></div>`).join('')+'</article></section>').join(''));
  await page.evaluate(()=>Promise.all([...document.images].map(i=>i.decode())));
  await page.screenshot({path:output+'/'+name+'.png',fullPage:true});
 }
}finally{await browser.close();}
