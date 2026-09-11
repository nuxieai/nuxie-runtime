// Precise paint-row regression for the isolated blue underline fixtures.
// Usage: node validation/underline-row-control.mjs TEST_RESULTS_DIRECTORY
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {PNG} from 'pngjs';
const directory=path.resolve(process.argv[2]);
const rows=[];
function stripeRows(file) {
 const image=PNG.sync.read(fs.readFileSync(file)), rows=[];
 for(let y=0;y<image.height;y++) {
  let blue=0;
  for(let x=0;x<image.width;x++) {
   const i=(y*image.width+x)*4;
   if(image.data[i]===0 && image.data[i+1]===0 && image.data[i+2]===255 && image.data[i+3]===255) blue++;
  }
  if(blue>=64) rows.push(y);
 }
 return rows;
}
for(const name of fs.readdirSync(directory)) {
 if(!/^browser-underline-(fractional-origin|integer-origin-control)-at-/.test(name)) continue;
 const prefix=path.join(directory,name,'scene');
 rows.push({name,browser:stripeRows(prefix+'.browser.png'),native:stripeRows(prefix+'.native.png')});
}
console.log(JSON.stringify(rows,null,2));
assert.equal(rows.length,6,'all two fixtures × three widths must be present');
for(const row of rows) {
 assert.equal(row.browser.length,1,row.name+' browser must have one solid stripe row');
 assert.deepEqual(row.native,row.browser,row.name+' native must paint the same solid stripe row');
}
