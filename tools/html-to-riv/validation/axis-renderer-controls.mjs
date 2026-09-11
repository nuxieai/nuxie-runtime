// Direct renderer diagnostics, not compiler admission or browser-baked layout.
// Usage: node axis-renderer-controls.mjs RENDERER NEW_OUTPUT
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {comparePixels} from './pixels.mjs';
const [rendererArg,outArg,mode='current']=process.argv.slice(2);
assert(['current','local'].includes(mode),'Expected current or local clip transform mode');
assert(outArg,'Expected RENDERER NEW_OUTPUT');
const renderer=path.resolve(rendererArg),out=path.resolve(outArg);
assert(!fs.existsSync(out),'Refusing to overwrite diagnostics');
fs.mkdirSync(out,{recursive:true});
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const rendererSha256=sha(fs.readFileSync(renderer));
const transforms={translate:[1,0,0,1,60,60],fractional:[1,0,0,1,60.25,60.75],scale:[1.25,0,0,.8,60,60],rotate:[.8660254,.5,-.5,.8660254,100,60],reflect:[-1,0,0,1,160,60],shear:[1,.25,.4,1,60,60]};
const rect=(id,x,y,w,h,color)=>`drawPath path={id=${id},fillRule=0,path={verbs=[move,line,line,line,close],points=[(${x},${y}),(${x+w},${y}),(${x+w},${y+h}),(${x},${y+h})]}} paint={id=${id},style=fill,color=${color},thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}`;
const browser=await chromium.launch();
const cases=[];
try {
  assert.equal(browser.version(),'153.0.8010.12');
  const page=await browser.newPage({deviceScaleFactor:1});
  for(const [transform,matrix] of Object.entries(transforms))for(const axis of ['x','y','both','none'])for(const fractional of [false,true]) {
    const name=`axis-renderer-${transform}-${axis}-${fractional?'fractional':'integer'}`;
    const w=fractional?100.25:100,h=fractional?80.75:80;
    const html='<div id="host"><div id="child"></div></div><div id="after"></div>';
    const css=`html,body{margin:0;background:white}#host{position:absolute;width:${w}px;height:${h}px;transform:matrix(${matrix});transform-origin:0 0;overflow-x:${['x','both'].includes(axis)?'clip':'visible'};overflow-y:${['y','both'].includes(axis)?'clip':'visible'}}#child{position:absolute;left:-40px;top:-30px;width:200px;height:160px;background:#318ea8}#after{position:absolute;left:10px;top:240px;width:80px;height:30px;background:#897198}`;
    for(const width of [240,390,768]) {
      const prefix=path.join(out,`${name}-${width}`);
      const commands=['rive-golden-stream-v1',`frameSize width=${width} height=320`,'clearColor value=0xffffffff','save'];
      if(mode==='current')commands.push(`transform matrix=[${matrix}]`);
      const local=mode==='local'?` matrix=[${matrix}]`:'';
      if(['x','both'].includes(axis))commands.push(`clipAxis axis=x range=[0,${w}]${local}`);
      if(['y','both'].includes(axis))commands.push(`clipAxis axis=y range=[0,${h}]${local}`);
      if(mode==='local')commands.push(`transform matrix=[${matrix}]`);
      commands.push(rect(1,-40,-30,200,160,'0xff318ea8'),'restore',rect(2,10,240,80,30,'0xff897198'),'frame');
      fs.writeFileSync(prefix+'.stream',commands.join('\n')+'\n');
      assert.equal(sha(fs.readFileSync(renderer)),rendererSha256,'Renderer changed during run');
      execFileSync(renderer,['--stream',prefix+'.stream','--output',prefix+'.native.png','--backend','rust-metal','--mode','clockwise-atomic'],{stdio:'pipe'});
      await page.setViewportSize({width,height:320});
      await page.setContent(`<!doctype html><style>${css}</style>${html}`);
      await page.screenshot({path:prefix+'.browser.png'});
      const bytes=fs.readFileSync(prefix+'.browser.png'),nativeBytes=fs.readFileSync(prefix+'.native.png');
      const {metrics,failures,diff}=comparePixels(PNG.sync.read(bytes),PNG.sync.read(nativeBytes),{scene:{x:0,y:0,width,height:320},restored:{x:10,y:240,width:80,height:30}},false);
      fs.writeFileSync(prefix+'.diff.png',PNG.sync.write(diff));
      cases.push({name,width,prefix,html,css,geometryFailures:[],geometryScope:'Direct renderer coordinates; no compiler geometry qualification',browserSha256:sha(bytes),nativeSha256:sha(nativeBytes),streamSha256:sha(fs.readFileSync(prefix+'.stream')),metrics,failures});
      fs.writeFileSync(path.join(out,'replay.json'),JSON.stringify({profile:'direct-axis-renderer-diagnostic',clipTransformMode:mode,browser:browser.version(),rendererSha256,cases},null,2)+'\n');
    }
    console.log(name);
  }
} finally { await browser.close(); }
const failures=cases.filter(c=>c.failures.length);
console.log(JSON.stringify({total:cases.length,failed:failures.length}));
if(failures.length)process.exitCode=1;
