import fs from 'node:fs';
import path from 'node:path';
import {chromium} from '@playwright/test';
const out=path.resolve(process.argv[2]);
if(fs.existsSync(out))throw new Error('Refusing overwrite');
const values=['calc(1in / 96px)','calc(1cm / 10mm)','calc(1Q / .25mm)',
 'calc(1pc / 12pt)','calc(1turn / 400grad)','calc(1rad / 1deg)',
 'calc(1s / 1000ms)','calc(1kHz / 1000Hz)','calc(96dpi / 1dppx)',
 'calc(1dpcm / 2.54dpi)','calc(1x / 1dppx)','calc(1p\\78 / 1px)',
 'calc(1e308in / 1e308in)','calc(1e400px / 1e400px)',
 'calc(1e308s / 1e308s)','calc(1e308% / 1e308%)',
 'calc(1e-320ms / 1e-320ms)','calc(1e308in - 1e308in)',
 'calc(1e308in / 1e308px)','calc(1e308px / 1e308in)'];
values.push('calc(1e40 / 1e39)', 'calc(1e40 - 1e39)', 'calc(1e-320 / 1e-320)', 'calc(1e-320s / 1e-320s)');
const browser=await chromium.launch();
try {
 const page=await browser.newPage();
 const rows=await page.evaluate(values=>values.map(value=>{
  const e=document.createElement('div');e.style.cssText=`width:120px;--ratio:${value};aspect-ratio:2;aspect-ratio:var(--ratio)`;document.body.append(e);
  const row={value,accepted:CSS.supports('aspect-ratio',value),computed:getComputedStyle(e).aspectRatio,height:e.getBoundingClientRect().height};e.remove();return row;
 }),values);
 fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,'probe.json'),JSON.stringify({browser:browser.version(),rows},null,2)+'\n');console.log(JSON.stringify(rows));
} finally {await browser.close();}
