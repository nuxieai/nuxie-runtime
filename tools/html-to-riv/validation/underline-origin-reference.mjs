// Independent Chrome reference: computed lengths versus used font metrics.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
const root=fileURLToPath(new URL('../../../',import.meta.url));const bytes=fs.readFileSync(root+'/tools/html-to-riv/tests/assets/Inter-Regular.ttf');
const b=await chromium.launch(),p=await b.newPage({viewport:{width:320,height:100}}),out=[];
try{for(const display of ['block','flex'])for(const sizes of [[40,20],[20,40]])for(const thickness of ['auto','from-font','10%','0.1em','3px']){
 await p.setContent(`<style>@font-face{font-family:Inter;src:url(data:font/ttf;base64,${bytes.toString('base64')})}body{margin:0;background:white}div{display:${display};text-decoration-line:underline;text-decoration-color:red;text-decoration-thickness:${thickness};text-decoration-skip-ink:none;font:${sizes[0]}px/60px Inter}p{margin:0;color:transparent;font-size:${sizes[1]}px}span{display:inline-block;width:0;height:0}</style><div><p id=a>agypqj M<span id=baseline></span></p></div>`);await p.evaluate(()=>document.fonts.ready);const baseline=await p.locator('#baseline').evaluate(e=>e.getBoundingClientRect().top);const png=PNG.sync.read(await p.screenshot());const rows=[];for(let y=0;y<png.height;y++){let n=0;for(let x=0;x<png.width;x++){let i=(y*png.width+x)*4;if(png.data[i]>200&&png.data[i+1]<200&&png.data[i+2]<200)n++;}if(n)rows.push({y,n});}out.push({display,sizes,thickness,baseline,rows});}
const expected=JSON.parse(fs.readFileSync(new URL('./underline-origin-reference.json',import.meta.url)));assert.equal(b.version(),expected.browser);assert.deepEqual(out,expected.cases);console.log(`${out.length} Chromium decoration-origin references match`);
}finally{await b.close();}
