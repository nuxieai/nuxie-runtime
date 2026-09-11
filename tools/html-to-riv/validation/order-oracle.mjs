import {chromium} from '@playwright/test';
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 console.log(JSON.stringify(await page.evaluate(()=>{
  const e=document.createElement('div');document.body.append(e);
  return ['-2147483649','-2147483648','2147483647','2147483648','999999999999999999999999','1.0','1e0','+0002'].map(value=>{
   e.style.order='7';e.style.setProperty('order',value);
   return {value,specified:e.style.order,computed:getComputedStyle(e).order};
  });
 }),null,2));
} finally {await browser.close();}
