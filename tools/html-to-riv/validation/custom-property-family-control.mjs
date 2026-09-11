// Pinned-browser evidence for keyword positions inside font-family names.
import {chromium} from '@playwright/test';
const values=['serif Other','Other serif','system-ui Other','Other system-ui','serif serif','caption Other','caption','default','default,Inter','Inter,initial','Inter inherit','inherit Inter','default Inter','Inter default','serif,Inter','ui-serif','math','fangsong','emoji'];
const browser=await chromium.launch();
try {
  const page=await browser.newPage();
  const records=await page.evaluate(values=>values.map(value=>{
    const parent=document.createElement('div');parent.style.fontFamily='Inter';
    const child=document.createElement('div');child.style.setProperty('--family',value);child.style.fontFamily='var(--family)';parent.append(child);document.body.append(parent);
    const result={value,accepted:CSS.supports('font-family',value),customSpecified:child.style.getPropertyValue('--family'),customComputed:getComputedStyle(child).getPropertyValue('--family'),computed:getComputedStyle(child).fontFamily};parent.remove();return result;
  }),values);
  console.log(JSON.stringify({browserVersion:browser.version(),records},null,2));
} finally {await browser.close();}
