import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync,spawnSync} from 'node:child_process';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const target=path.resolve(root,process.env.CARGO_TARGET_DIR||'target','debug');
const compiler=process.env.NUXIE_NATIVE_COMPILER||path.join(target,'html-to-riv');
const probe=process.env.NUXIE_NATIVE_PROBE||path.join(target,'examples/probe');

test('version26 installs responsive authored stops and rejects invalid manifests before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-gradient-host-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=a></div>',css:'#a{width:60%;height:100px}',width:390,height:320}));
  execFileSync(compiler,[scene+'.json',scene+'.riv']);
  const baseline=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  const map=JSON.parse(fs.readFileSync(scene+'.map.json'));
  const id=map.find(n=>n.id==='a').object_id;
  const valid={...baseline,version:26,capabilities:[...baseline.capabilities,'layout-css-linear-gradient-v1'],
   layout_linear_gradients:[{object_id:id,direction:{degrees:90},stops:[
    {color:0xffff0000,position:null},{color:0xff0000ff,position:null}]}]};
  const riv=fs.readFileSync(scene+'.riv');let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`frame-${sequence++}`);
   const result=spawnSync(probe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{
    encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_LINEAR_GRADIENT:disabled?'1':'0'}});
   if(result.status!==0) assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before drawing');
   return {...result,prefix};
  };
  for(const [width,expected] of [[240,144],[390,234],[768,460.8],[240,144]]) {
   const result=run(valid,width);assert.equal(result.status,0,result.stderr);
   const stream=fs.readFileSync(result.prefix+'.stream','utf8');
   const line=stream.split('\n').find(l=>l.startsWith('makePremultipliedLinearGradient'));
   assert.ok(line);assert.match(line,/start=\(0,50\)/);
   assert.ok(Math.abs(Number(line.match(/end=\(([^,]+),/)[1])-expected)<.001,line);
   assert.deepEqual(fs.readFileSync(scene+'.riv'),riv);
  }
  const disabled=run(valid,390,true);assert.notEqual(disabled.status,0);assert.match(disabled.stderr,/missing-runtime-capability/);
  const changes=[m=>m.version=25,m=>m.layout_linear_gradients=[],m=>m.layout_linear_gradients[0].object_id=0,
   m=>m.layout_linear_gradients[0].object_id=999999,m=>m.layout_linear_gradients.push(structuredClone(m.layout_linear_gradients[0])),
   m=>m.layout_linear_gradients[0].stops.pop(),m=>m.layout_linear_gradients[0].stops=Array(257).fill({color:0,position:null}),
   m=>m.layout_linear_gradients[0].direction={degrees:null},m=>m.layout_linear_gradients[0].direction={degrees:90,corner:{right:true,bottom:true}},
   m=>m.layout_linear_gradients[0].stops[0].position={pixels:null},m=>m.layout_linear_gradients[0].stops[0].color=-1,
   m=>m.layout_linear_gradients[0].extra=true,m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-linear-gradient-v1')];
  for(const change of changes){const bad=structuredClone(valid);change(bad);assert.notEqual(run(bad).status,0,JSON.stringify(bad));}
  assert.equal(run(valid).status,0,'valid installation remains usable after rejection controls');
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});
