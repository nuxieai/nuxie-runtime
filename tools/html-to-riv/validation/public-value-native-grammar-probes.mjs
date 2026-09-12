// Additional conservative-classifier pitfalls, observed independently in Chrome.
import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';import crypto from 'node:crypto';import {fileURLToPath} from 'node:url';import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);assert(!fs.existsSync(out));fs.mkdirSync(out,{recursive:true});
const cases=[];const add=(property,...values)=>values.forEach(value=>cases.push({property,value}));
add('width','1e-50','1e-500','-1e-50px','-1e-500px','-0px','-0.0px','0e0','0.0','1e999px','10vi','10cqw','10rex','10cap','10rcap','10rlh','10frob','stretch','fit-content','contain','min-content','max-content','none','auto','calc(-20px)','initial 1px','revert-layer 1px','revert-rule');
add('min-width','auto','none','stretch','contain','fit-content');add('max-width','auto','none','stretch','contain','fit-content');
add('font-size','120%','xxx-large','math','larger','medium','1e-50','-1e-50px','-1e-500px');
add('background','20px','-20px','0','0 0','red blue','border-box padding-box','none none','center/cover','red,none','text','border-area');
add('color','CanvasText','AccentColor','-webkit-link','light-dark(red,blue)','inherit red');
add('display','block flex','flex block','inline flow-root','flow-root inline list-item','list-item flow-root inline','table-cell','inline-block','run-in','math','block block','flex grid','list-item flex');
add('align-self','safe normal','unsafe normal','first baseline','baseline first','last baseline','safe baseline','safe stretch','unsafe auto','anchor-center',String.raw`safe\20 center`);
add('justify-content','safe left','unsafe right','safe normal','first baseline','space-between','stretch','safe space-evenly');
add('flex','0 0 auto','auto 0 0','0 auto 0','1px 0','0 1px','0 0 0','-0 0 auto','-1e-50 0 auto','none 0',String.raw`\30 \20 \30 \20 auto`);
add('order','1.0','1e0','-0','+0001','2147483648','999999999999999999999999999999999999');
add('margin','-1px','1e-50','-1e-50px','auto 0 auto 0','inherit auto');
add('gap','normal','normal 1px','1px normal','0 0','-1e-50px','auto','1px 2px 3px');
const browser=await chromium.launch();assert.equal(browser.version(),'153.0.8010.12');let rows;
try{const page=await browser.newPage({viewport:{width:390,height:200},deviceScaleFactor:1});await page.setContent('<!doctype html><div id=p style="width:200px;color:navy;font-size:24px;display:flex"><div id=a style="width:40px;height:20px;color:red"></div></div>');
 rows=await page.evaluate(cases=>cases.map(c=>{const e=document.getElementById('a');const base='width:40px;height:20px;color:red';e.style.cssText=base;e.style.setProperty(c.property,c.value);const direct={specified:e.style.getPropertyValue(c.property),computed:getComputedStyle(e).getPropertyValue(c.property)};e.style.cssText=base+';--v:'+c.value+';'+c.property+':var(--v)';const variable={specified:e.style.getPropertyValue(c.property),custom:getComputedStyle(e).getPropertyValue('--v'),computed:getComputedStyle(e).getPropertyValue(c.property)};return{...c,supports:CSS.supports(c.property,c.value),direct,variable};}),cases);
}finally{await browser.close();}
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');fs.copyFileSync(fileURLToPath(import.meta.url),path.join(out,'driver.mjs'));
fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({scope:'Independent Chrome153 grammar observations only; draft-spec validity, browser support and target admission are separate. No native qualification.',browser:'153.0.8010.12',driverSha256:hash(fileURLToPath(import.meta.url)),rows},null,2)+'\n');console.log(JSON.stringify(rows,null,2));
