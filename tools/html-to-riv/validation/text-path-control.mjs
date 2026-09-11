// Replays the bounded text-diagnosis path corpus in Canvas. This is a diagnostic
// control, never a substitute for the acceptance suite's DOM reference.
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {comparePixels} from './pixels.mjs';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../..');
const dir=path.join(root,'output/playwright/html-to-riv/text-diagnosis');
const names=['minimal-1','minimal-2','font-shorthand-normal-resets','normal-line-height-cascade'];
const parsePath=line=>{
 const m=line.match(/fillRule=(\d+),path=\{verbs=\[([^\]]*)\],points=\[([^\]]*)\]/);
 if(!m || ![0,1,2].includes(Number(m[1]))) throw new Error('Unsupported path encoding');
 return {rule:Number(m[1]),verbs:m[2].split(',').filter(Boolean),points:[...m[3].matchAll(/\(([^,]+),([^\)]+)\)/g)].map(m=>[Number(m[1]),Number(m[2])])};
};
const browser=await chromium.launch();
try {
 const page=await browser.newPage({viewport:{width:390,height:320},deviceScaleFactor:1});
 const results=[];
 for(const name of names){
  const lines=fs.readFileSync(path.join(dir,name+'.stream'),'utf8').trim().split('\n');
  const commands=[];
  for(const line of lines){
   if(line==='save'||line==='restore') commands.push({op:line});
   else if(line.startsWith('transform ')) commands.push({op:'transform',matrix:line.match(/\[([^\]]+)\]/)[1].split(',').map(Number)});
   else if(line.startsWith('drawPath ')) {
    const color=line.match(/paint=\{[^}]*color=0x([a-f\d]+)/i);
    if(!color || !line.includes('style=fill') || !line.includes('feather=0') || !line.includes('shader=0') || !line.includes('blendMode=3')) throw new Error('Control only accepts solid unfeathered fills');
    commands.push({op:'fill',path:parsePath(line),color:parseInt(color[1],16)});
   } else if(line.startsWith('clipPath ')) commands.push({op:'clip',path:parsePath(line)});
   else if(!/^(rive-golden-stream-v1|makeRenderPaint |makeEmptyRenderPath |frameSize |clearColor value=0xffffffff|sample seconds=0)/.test(line)) throw new Error('Unsupported diagnostic command: '+line.slice(0,80));
  }
  await page.setContent('<style>body{margin:0;background:white}canvas{display:block}</style><canvas width=390 height=320></canvas>');
  await page.evaluate(commands=>{
   const c=document.querySelector('canvas').getContext('2d');
   const make=p=>{const q=new Path2D();let i=0;for(const v of p.verbs){switch(v){case'move':q.moveTo(...p.points[i++]);break;case'line':q.lineTo(...p.points[i++]);break;case'quad':q.quadraticCurveTo(...p.points[i++],...p.points[i++]);break;case'cubic':q.bezierCurveTo(...p.points[i++],...p.points[i++],...p.points[i++]);break;case'close':q.closePath();break;default:throw new Error(v)}}if(i!==p.points.length)throw new Error('Unconsumed points');return q};
   for(const command of commands){switch(command.op){case'save':c.save();break;case'restore':c.restore();break;case'transform':c.transform(...command.matrix);break;case'clip':c.clip(make(command.path),command.path.rule===1?'evenodd':'nonzero');break;case'fill':{const v=command.color;c.fillStyle=`rgba(${(v>>>16)&255},${(v>>>8)&255},${v&255},${(v>>>24)/255})`;c.fill(make(command.path),command.path.rule===1?'evenodd':'nonzero');break;}}}
  },commands);
  const file=path.join(dir,name+'.canvas-paths.png');await page.screenshot({path:file});
  const canvas=PNG.sync.read(fs.readFileSync(file)),native=PNG.sync.read(fs.readFileSync(path.join(dir,name+'.native.png'))),dom=PNG.sync.read(fs.readFileSync(path.join(dir,name+'.browser.png')));
  const boxes=JSON.parse(fs.readFileSync(path.join(dir,name+'.bounds.json')));
  const summarize=(a,b)=>{const {metrics,failures}=comparePixels(a,b,boxes,true);return {metrics,failures}};
  results.push({name,pathsVsNative:summarize(canvas,native),domVsPaths:summarize(dom,canvas)});
 }
 fs.writeFileSync(path.join(dir,'path-control-results.json'),JSON.stringify(results,null,2));
 console.log(JSON.stringify(results));
}finally{await browser.close()}
