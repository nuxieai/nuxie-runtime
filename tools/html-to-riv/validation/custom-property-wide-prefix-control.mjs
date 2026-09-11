import {chromium} from '@playwright/test';
const values=['inherit Inter','initial Inter','unset Inter','revert Inter','revert-layer Inter','default Inter','Inter inherit','inherit, Inter','INHERIT Inter','inherit/**/Inter'];
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const records=await page.evaluate(values=>{
  const records=[];
  for(const value of values) for(const mode of ['direct','variable','fallback']) {
   const parent=document.createElement('div');parent.style.fontFamily='ParentFont';
   const child=document.createElement('div');child.style.fontFamily='EarlierFont';
   if(mode==='variable')child.style.setProperty('--family',value);
   child.style.fontFamily=mode==='direct'?value:mode==='variable'?'var(--family)':`var(--missing,${value})`;
   parent.append(child);document.body.append(parent);
   records.push({value,mode,specified:child.style.fontFamily,custom:child.style.getPropertyValue('--family'),computed:getComputedStyle(child).fontFamily});parent.remove();
  }
  return records;
 },values);
 console.log(JSON.stringify({browserVersion:browser.version(),records},null,2));
} finally {await browser.close();}
