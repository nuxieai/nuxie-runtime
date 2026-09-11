import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync,spawnSync} from 'node:child_process';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const target=path.resolve(root,process.env.CARGO_TARGET_DIR||'target','debug');
const nativeCompiler=process.env.NUXIE_NATIVE_COMPILER ? path.resolve(process.env.NUXIE_NATIVE_COMPILER) : path.join(target,'html-to-riv');
const nativeProbe=process.env.NUXIE_NATIVE_PROBE ? path.resolve(process.env.NUXIE_NATIVE_PROBE) : path.join(target,'examples/probe');
test('version21 clip margins reject incompatible hosts and invalid policies before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-clip-margin-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=a><div id=b></div></div>',css:'#a{width:75%;height:100px;padding:12px 18px;overflow:clip;overflow-clip-margin:content-box 8px;background:orange}#b{width:200px;height:160px;background:red}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const valid=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  const map=JSON.parse(fs.readFileSync(scene+'.map.json'));
  assert.equal(valid.version,21);
  const bytes=fs.readFileSync(scene+'.riv');let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`frame-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_CLIP_MARGIN:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before drawing');
   return {...result,prefix};
  };
  for(const width of [240,390,768,240]) {
   const result=run(valid,width);assert.equal(result.status,0,result.stderr);
   assert.match(fs.readFileSync(result.prefix+'.stream','utf8'),/clipPath .*points=\[\(10,4\)/);
   assert.deepEqual(fs.readFileSync(scene+'.riv'),bytes);
  }
  const disabled=run(valid,390,true);assert.notEqual(disabled.status,0);assert.match(disabled.stderr,/missing-runtime-capability/);
  for(const mutate of [m=>m.version=20,m=>m.version=22,m=>m.layout_overflow_clip_margins=[],m=>m.layout_overflow_clip_margins.push({...m.layout_overflow_clip_margins[0]}),m=>m.layout_overflow_clip_margins[0].object_id=0,m=>m.layout_overflow_clip_margins[0].object_id=999999,m=>m.layout_overflow_clip_margins[0].object_id=map.find(n=>n.id==='b').object_id,m=>m.layout_overflow_clip_margins[0].origin='content',m=>m.layout_overflow_clip_margins[0].pixels=null,m=>m.layout_overflow_clip_margins[0].extra=true,m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-overflow-clip-margin-v1')]) {
   const bad=structuredClone(valid);mutate(bad);assert.notEqual(run(bad).status,0,JSON.stringify(bad));
  }
  assert.equal(run(valid).status,0,'valid manifest still loads after rejected attempts');
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});
test('version20 axis overflow rejects incompatible hosts and malformed manifests before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-axis-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=a><div id=b></div></div>',css:'#a{width:75%;height:80px;overflow:clip visible}#b{width:200px;height:160px;background:red}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const valid=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(valid.version,20);
  const bytes=fs.readFileSync(scene+'.riv');let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`frame-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_AXIS_OVERFLOW:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before drawing');
   return {...result,prefix};
  };
  for(const width of [240,390,768,240]) {
   const result=run(valid,width);assert.equal(result.status,0,result.stderr);
   assert.match(fs.readFileSync(result.prefix+'.stream','utf8'),/clipAxis axis=x/);
   assert.deepEqual(fs.readFileSync(scene+'.riv'),bytes);
  }
  const disabled=run(valid,390,true);assert.notEqual(disabled.status,0);assert.match(disabled.stderr,/missing-runtime-capability/);
  for(const mutate of [m=>m.version=19,m=>m.version=21,m=>m.layout_axis_overflow=[],m=>m.layout_axis_overflow.push({...m.layout_axis_overflow[0]}),m=>m.layout_axis_overflow[0].object_id=0,m=>m.layout_axis_overflow[0].object_id=999999,m=>m.layout_axis_overflow[0].axis='both',m=>m.layout_axis_overflow[0].extra=true,m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-axis-overflow-v1')]) {
   const bad=structuredClone(valid);mutate(bad);assert.notEqual(run(bad).status,0,JSON.stringify(bad));
  }
  assert.equal(run(valid).status,0,'valid manifest still loads after rejected attempts');
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});
test('version15 exact aspect-ratio policy is validated before drawing and survives resizing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-ratio-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=a></div>',css:'#a{width:120px;aspect-ratio:auto 2;padding:5px 11px 13px 13px}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const valid=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  const bytes=fs.readFileSync(scene+'.riv');
  assert.equal(valid.version,15);assert.deepEqual(valid.layout_aspect_ratios[0].pair,[2,1]);assert.equal(valid.layout_aspect_ratios[0].content_box,true);
  let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`attempt-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_ASPECT_RATIO:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before drawing');
   return {...result,prefix};
  };
  for(const width of [240,390,768,240]) {
   const result=run(valid,width);assert.equal(result.status,0,result.stderr);
   const bounds=JSON.parse(fs.readFileSync(result.prefix+'.bounds.json'));
   assert.ok(Math.abs(bounds.a.width-120)<=.1);assert.ok(Math.abs(bounds.a.height-66)<=.1);
   assert.deepEqual(fs.readFileSync(scene+'.riv'),bytes);
  }
  const legacy=structuredClone(valid);legacy.version=14;
  legacy.capabilities=legacy.capabilities.filter(c=>c!=='layout-css-aspect-ratio-pair-v1');
  for(const entry of legacy.layout_aspect_ratios)delete entry.pair;
  assert.equal(run(legacy).status,0,'legacy scalar contract remains loadable');
  for(const pair of [undefined,null,[0,1],[1,0],[-1,1],[1.5,1],[4294967295,1],[1,4294967295],[1],[1,2,3],'2/1']) {
   const bad=structuredClone(valid);bad.layout_aspect_ratios[0].pair=pair;
   assert.notEqual(run(bad).status,0,'reject malformed pair before drawing');
  }
  const missing=structuredClone(valid);missing.capabilities=missing.capabilities.filter(c=>c!=='layout-css-aspect-ratio-pair-v1');
  assert.notEqual(run(missing).status,0,'pair requires its capability');
  const disabled=run(valid,390,true);assert.notEqual(disabled.status,0);assert.match(disabled.stderr,/missing-runtime-capability/);
  for(const mutate of [m=>m.version=13,m=>m.version=14,m=>m.version=16,m=>m.layout_aspect_ratios=[],m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-aspect-ratio-v1'),m=>m.layout_aspect_ratios.push({...m.layout_aspect_ratios[0]}),m=>m.layout_aspect_ratios[0].object_id=999999,m=>delete m.layout_aspect_ratios[0].content_box,m=>m.layout_aspect_ratios[0].content_box='yes']) {
   const bad=structuredClone(valid);mutate(bad);assert.notEqual(run(bad).status,0);
  }
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});
test('version12 indefinite-basis capability is checked before rendering',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-indefinite-contract-'));
 try {
  const scene=path.join(dir,'scene');
  const fixture=JSON.parse(fs.readFileSync(new URL('../validation/indefinite-basis-cases.json',import.meta.url)))[0];
  fs.writeFileSync(scene+'.json',JSON.stringify({html:fixture.html,css:fixture.css,width:240,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const original=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(original.version,12);
  assert(original.capabilities.includes('layout-css-indefinite-basis-v1'));
  const valid=original;
  let sequence=0;
  const run=(manifest,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`attempt-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','320',prefix],{
    encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_INDEFINITE_BASIS:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before producing a render stream');
   return result;
  };
  assert.equal(run(valid).status,0);
  assert.match(run(valid,true).stderr,/missing-runtime-capability/);
  for(const invalid of [{...valid,version:11},{...valid,version:13},{...valid,capabilities:original.capabilities.filter(c=>c!=='layout-css-indefinite-basis-v1')}])
   assert.notEqual(run(invalid).status,0);
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});
test('version13 content-box capability is checked before rendering',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-content-box-contract-'));
 try {
  const scene=path.join(dir,'scene');
  const fixture=JSON.parse(fs.readFileSync(new URL('../validation/content-box-cases.json',import.meta.url)))[0];
  fs.writeFileSync(scene+'.json',JSON.stringify({html:fixture.html,css:fixture.css,width:240,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const original=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(original.version,13);
  assert(original.capabilities.includes('layout-css-content-box-v1'));
  const valid=original;
  let sequence=0;
  const run=(manifest,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`attempt-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','320',prefix],{
    encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_CONTENT_BOX:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before producing a render stream');
   return result;
  };
  assert.equal(run(valid).status,0);
  assert.match(run(valid,true).stderr,/missing-runtime-capability/);
  for(const invalid of [{...valid,version:12},{...valid,version:14},{...valid,layout_content_box:[]},{...valid,layout_content_box:[999999]},{...valid,layout_content_box:[...valid.layout_content_box,...valid.layout_content_box]},{...valid,capabilities:original.capabilities.filter(c=>c!=='layout-css-content-box-v1')}])
   assert.notEqual(run(invalid).status,0);
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});
test('the checked native host requires the manifest and honors capability availability',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-requirements-'));
 try {
  const scene=path.join(dir,'scene');
  const source={html:'<p>ASCII can later become combining text</p>',css:'p{letter-spacing:2px}',width:240,height:320,assets:{inter:{kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/Inter-Regular.ttf',import.meta.url))]}}};
  const compile=()=>{fs.writeFileSync(scene+'.json',JSON.stringify(source));execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);};
  let sequence=0;
  const run=(disabled=false,disablePrecision=false)=>{const env={...process.env,NUXIE_NATIVE_GLYPHS:"0",NUXIE_DISABLE_CLUSTER_SPACING:disabled?"1":"0",NUXIE_DISABLE_CSS_SHAPING_PRECISION:disablePrecision?"1":"0"};delete env.NUXIE_CSS_SHAPING_PRECISION;const prefix=path.join(dir,`attempt-${sequence++}`);const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','320',prefix],{encoding:'utf8',env});if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before drawing');return {...result,prefix};};
  compile();const manifest=fs.readFileSync(scene+'.requirements.json');
  const normal=JSON.parse(manifest);assert.equal(normal.version,11);assert.deepEqual(normal.capabilities,['layout-css-intrinsic-sizing-v1','text-css-shaping-precision-v1','text-css-normal-wrap-v1','text-css-letter-spacing-v1']);assert.equal(normal.text_policies.length,1);assert.equal(normal.text_policies[0].policy,'css-normal-wrap-v1');
  assert.equal(run().status,0);
  const noPrecision=run(false,true);assert.notEqual(noPrecision.status,0);assert.match(noPrecision.stderr,/missing-runtime-capability/);
  const unsupported=run(true);assert.notEqual(unsupported.status,0);assert.match(unsupported.stderr,/missing-runtime-capability/);
  for(const invalid of [{version:3,capabilities:[]},{version:1,capabilities:['unknown']},{version:1,capabilities:[],extra:true}]){
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(invalid));assert.notEqual(run().status,0);
  }
  fs.unlinkSync(scene+'.requirements.json');assert.notEqual(run().status,0);
  fs.writeFileSync(scene+'.requirements.json',manifest);assert.equal(run().status,0);
  source.css='p{letter-spacing:normal}';compile();assert.match(run(true).stderr,/missing-runtime-capability/,'normal scenes now require their wrap policy');
  assert.deepEqual(JSON.parse(fs.readFileSync(scene+'.requirements.json')).capabilities,['layout-css-intrinsic-sizing-v1','text-css-shaping-precision-v1','text-css-normal-wrap-v1']);
  assert.match(run(false,true).stderr,/missing-runtime-capability/);
  source.html='<div></div>';compile();assert.deepEqual(JSON.parse(fs.readFileSync(scene+'.requirements.json')).capabilities,[]);assert.equal(run(true,true).status,0);
  source.html='<p>one&#x2003;two</p>';compile();
  const preserved=JSON.parse(fs.readFileSync(scene+'.requirements.json'));assert.equal(preserved.version,11);assert.deepEqual(preserved.capabilities,['layout-css-intrinsic-sizing-v1','text-css-shaping-precision-v1','text-css-normal-wrap-v1','text-preserved-space-breaks-v1']);assert.equal(preserved.text_policies[0].policy,'css-normal-wrap-v1');
  assert.equal(run().status,0);assert.match(run(true).stderr,/missing-runtime-capability/);
  source.html='<p>long nowrap text</p>';source.css='p{white-space:nowrap;text-align:center}';compile();
  const nowrap=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(nowrap.version,11);assert.deepEqual(nowrap.capabilities,['layout-css-intrinsic-sizing-v1','text-css-shaping-precision-v1','text-css-nowrap-alignment-v1']);
  assert.equal(nowrap.text_policies.length,1);assert.equal(nowrap.text_policies[0].policy,'css-nowrap-alignment-v1');
  assert.equal(run().status,0);assert.match(run(true).stderr,/missing-runtime-capability/);
  source.html='<p>i\tX</p>';source.css='p{white-space:pre}';compile();
  const tabs=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(tabs.version,11);assert.deepEqual(tabs.capabilities,['layout-css-intrinsic-sizing-v1','text-css-shaping-precision-v1','text-css-tabs-v1','text-css-nowrap-alignment-v1']);
  assert.equal(tabs.text_policies.length,1);
  source.css='p{white-space:pre-wrap}';compile();
  const wrappedTabs=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.ok(wrappedTabs.capabilities.includes('text-css-wrapped-tabs-v1'));
  assert.ok(wrappedTabs.capabilities.includes('text-css-pre-wrap-v1'));
  assert.equal(wrappedTabs.text_policies[0].policy,'css-pre-wrap-v1');
  assert.equal(run().status,0);assert.match(run(true).stderr,/missing-runtime-capability/);
  source.html='<p> one \t two \n three </p>';source.css='p{white-space:pre-line}';compile();
  const preline=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.deepEqual(preline.capabilities,['layout-css-intrinsic-sizing-v1','text-css-shaping-precision-v1','text-css-pre-line-v1']);
  assert.equal(preline.text_policies[0].policy,'css-pre-line-v1');
  assert.equal(run().status,0);assert.match(run(true).stderr,/missing-runtime-capability/);
  for(const invalid of [{...preline,version:1,layout_intrinsic_sizing:[],capabilities:preline.capabilities.filter(c=>c!=='layout-css-intrinsic-sizing-v1')},{...preline,text_policies:[]},{...preline,capabilities:preline.capabilities.filter(c=>!c.startsWith('text-'))}]) {
    fs.writeFileSync(scene+'.requirements.json',JSON.stringify(invalid));
    assert.match(run().stderr,/invalid-text-policies/);
  }
  source.html='<div><p id=a>long nowrap words that overflow</p><p id=b>normal wrapped words</p><p id=c>pre words</p><p id=d>pre-wrap words</p></div>';
  source.css='div{width:120px}p{display:block;text-align:center}#a{white-space:nowrap}#c{white-space:pre}#d{white-space:normal}';compile();
  let mapped=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(mapped.text_policies.length,4);assert.equal(mapped.text_policies.filter(p=>p.policy==='css-normal-wrap-v1').length,2);
  const original=run();assert.equal(original.status,0,original.stderr);
  const save=(manifest)=>fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
  save({version:1,capabilities:mapped.capabilities.filter(c=>c!=='layout-css-intrinsic-sizing-v1')});
  assert.match(run().stderr,/invalid-text-policies/,'normal wrap cannot be silently downgraded to version1');
  save({version:1,capabilities:mapped.capabilities.filter(c=>c!=='text-css-normal-wrap-v1'&&c!=='layout-css-intrinsic-sizing-v1')});
  const legacy=run();assert.equal(legacy.status,0,legacy.stderr);
  source.css+=' #d{white-space:pre-wrap}';compile();
  mapped=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(mapped.text_policies.length,4);
  assert.equal(mapped.text_policies[3].policy,'css-pre-wrap-v1');
  assert.ok(mapped.capabilities.includes('text-css-pre-wrap-v1'));
  assert.equal(run().status,0);assert.match(run(true).stderr,/missing-runtime-capability/);
  const wrongType=JSON.parse(fs.readFileSync(scene+'.map.json'))[0].object_id;
  for(const id of [wrongType,4294967295]) {
    save({...mapped,text_policies:mapped.text_policies.map((entry,i)=>i===0?{...entry,object_id:id}:entry)});
    const rejected=run();assert.notEqual(rejected.status,0);assert.match(rejected.stderr,/invalid-text-policy-target/);
  }
  for(const invalid of [
    {...mapped,text_policies:[]},
    {...mapped,text_policies:mapped.text_policies.filter(entry=>entry.policy!=='css-pre-wrap-v1')},
    {...mapped,text_policies:mapped.text_policies.map((entry,i)=>i===2?{...entry,object_id:mapped.text_policies[0].object_id}:entry)},
    {...mapped,text_policies:[...mapped.text_policies,mapped.text_policies[0]]},
    {...mapped,capabilities:[]},
    {...mapped,version:1},
    {...mapped,text_policies:[{...mapped.text_policies[0],object_id:-1}]},
    {...mapped,text_policies:[{...mapped.text_policies[0],policy:'future-policy'}]},
    {...mapped,text_policies:[{...mapped.text_policies[0],extra:true}]},
  ]) {
    save(invalid);const rejected=run();assert.notEqual(rejected.status,0,JSON.stringify(invalid));
  }
  const decorated={...mapped,version:11,
    capabilities:[...mapped.capabilities,'text-solid-underlines-v1'],
    text_underlines:[{object_id:mapped.text_policies[0].object_id,lines:[
      {color:0xffff0000,thickness:2,offset:1,skip_ink:'auto'},
      {color:0xff0000ff,thickness:1,offset:4,skip_ink:'none'},
    ]}]};
  save(decorated);const drawn=run();assert.equal(drawn.status,0,drawn.stderr);
  const stream=fs.readFileSync(drawn.prefix+'.stream','utf8');
  assert.match(stream,/drawPath[^\n]*color=0xffff0000/);
  assert.match(stream,/drawPath[^\n]*color=0xff0000ff/);
  assert.match(run(true).stderr,/missing-runtime-capability/);
  for(const invalid of [
    {...decorated,version:2},
    {...decorated,text_underlines:[]},
    {...decorated,capabilities:mapped.capabilities},
    {...decorated,text_underlines:[...decorated.text_underlines,...decorated.text_underlines]},
    ...[wrongType,4294967295].map(object_id=>({...decorated,text_underlines:[{...decorated.text_underlines[0],object_id}]})),
    ...[0,-1,1000001,null].map(thickness=>({...decorated,text_underlines:[{...decorated.text_underlines[0],lines:[{...decorated.text_underlines[0].lines[0],thickness}]}]})),
  ]) {
    save(invalid);const rejected=run();assert.notEqual(rejected.status,0,JSON.stringify(invalid));
  }
  save(mapped);assert.equal(run().status,0);
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('ellipsis installer validates occurrences and matches the diagnostic renderer without flags',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-ellipsis-contract-'));
 try {
  const scene=path.join(dir,'scene');
  const source={html:'<p>ffi long title</p>',css:'p{display:block;white-space:nowrap;overflow:hidden;width:40px;font:24px/40px Inter;letter-spacing:2px}',width:240,height:200,assets:{inter:{kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/Inter-Regular.ttf',import.meta.url))]}}};
  const compile=()=>{
   fs.writeFileSync(scene+'.json',JSON.stringify(source));
   execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
   return JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  };
  const original=compile();
  const textId=original.text_policies[0].object_id;
  const installManifest=manifest=>{
   const capabilities=manifest.capabilities.filter(c=>c!=='text-css-nowrap-alignment-v1'&&c!=='text-css-normal-wrap-v1');
   capabilities.push('text-css-single-line-ellipsis-v1');
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify({...manifest,version:Math.max(manifest.version,2),capabilities,text_policies:[{object_id:textId,policy:'css-single-line-ellipsis-v1'}]}));
  };
  const run=(name,extra={})=>{
   const prefix=path.join(dir,name),env={...process.env,NUXIE_NATIVE_GLYPHS:'1',NUXIE_EXPERIMENTAL_TEXT_ELLIPSIS:'0',NUXIE_EXPERIMENTAL_CSS_ELLIPSIS:'0',...extra};
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','200',prefix],{encoding:'utf8',env});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false);
   return {...result,prefix};
  };
  const diagnostic=run('diagnostic',{NUXIE_EXPERIMENTAL_TEXT_ELLIPSIS:'1',NUXIE_EXPERIMENTAL_CSS_ELLIPSIS:'1'});
  assert.equal(diagnostic.status,0,diagnostic.stderr);
  installManifest(original);
  const installed=run('installed');
  assert.equal(installed.status,0,installed.stderr);
  assert.deepEqual(fs.readFileSync(installed.prefix+'.stream'),fs.readFileSync(diagnostic.prefix+'.stream'));
  const unsupported=run('unsupported',{NUXIE_DISABLE_CLUSTER_SPACING:'1'});
  assert.notEqual(unsupported.status,0);assert.match(unsupported.stderr,/missing-runtime-capability/);
  for(const [name,html,whiteSpace] of [['wrapped','<p>one two</p>','normal'],['break','<p>one<br>two</p>','nowrap'],['tab','<p>one\ttwo</p>','pre']]){
   source.html=html;source.css=`p{display:block;white-space:${whiteSpace};overflow:hidden;width:40px;font:24px/40px Inter}`;
   installManifest(compile());
   const rejected=run(name);
   assert.notEqual(rejected.status,0,name);assert.match(rejected.stderr,/invalid-text-ellipsis-target/,rejected.stderr);
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('normal wrapping is opt-in and a host without it rejects before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-normal-wrap-'));
 try {
  const scene=path.join(dir,'scene');
  const source={html:'<div id="word">Second</div>',css:'#word{display:block;width:48px;font:16px/24px Inter}',width:240,height:200,assets:{inter:{kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/Inter-Regular.ttf',import.meta.url))]}}};
  fs.writeFileSync(scene+'.json',JSON.stringify(source));execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  const run=(name,extra={})=>{
   const prefix=path.join(dir,name);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','200',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'1',...extra}});
   return {...result,prefix};
  };
  const css=run('css');assert.equal(css.status,0,css.stderr);assert.equal(JSON.parse(fs.readFileSync(css.prefix+'.bounds.json')).word.height,24);
  const missing=run('missing',{NUXIE_DISABLE_CSS_NORMAL_WRAP:'1'});assert.notEqual(missing.status,0);assert.match(missing.stderr,/missing-runtime-capability/);assert.equal(fs.existsSync(missing.prefix+'.stream'),false);
  fs.writeFileSync(scene+'.requirements.json',JSON.stringify({version:1,capabilities:manifest.capabilities.filter(c=>c!=='text-css-normal-wrap-v1')}));
  const raw=run('raw');assert.equal(raw.status,0,raw.stderr);assert.equal(JSON.parse(fs.readFileSync(raw.prefix+'.bounds.json')).word.height,48);
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('CSS paint order is installed from the manifest and unsupported hosts reject before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-paint-order-'));
 try {
  const scene=path.join(dir,'scene');
  const source={html:'<div id="root"><div id="a"><div id="ap"></div></div><div id="b"><div id="bp"></div></div></div>',css:'#root{width:80px;flex-direction:row}#a,#b{width:40px;height:30px}#ap,#bp{width:80px;height:30px}#ap{background:#f44}#bp{background:#48f}',width:390,height:200};
  fs.writeFileSync(scene+'.json',JSON.stringify(source));execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.ok(manifest.capabilities.includes('layout-css-paint-order-v1'));
  const run=(name,extra={})=>{
   const prefix=path.join(dir,name);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','200',prefix],{encoding:'utf8',env:{...process.env,NUXIE_EXPERIMENTAL_CSS_PAINT_ORDER:'0',...extra}});
   return {...result,prefix};
  };
  const colors=r=>fs.readFileSync(r.prefix+'.stream','utf8').split('\n').filter(s=>s.startsWith('drawPath ')).map(s=>s.match(/color=([^,]+)/)[1]);
  const css=run('css');assert.equal(css.status,0,css.stderr);assert.deepEqual(colors(css),['0xffff4444','0xff4488ff']);
  const missing=run('missing',{NUXIE_DISABLE_CSS_PAINT_ORDER:'1'});assert.notEqual(missing.status,0);assert.match(missing.stderr,/missing-runtime-capability/);assert.equal(fs.existsSync(missing.prefix+'.stream'),false);
  fs.writeFileSync(scene+'.requirements.json',JSON.stringify({...manifest,capabilities:manifest.capabilities.filter(c=>c!=='layout-css-paint-order-v1')}));
  const raw=run('raw',{NUXIE_EXPERIMENTAL_CSS_PAINT_ORDER:'1'});assert.equal(raw.status,0,raw.stderr);assert.deepEqual(colors(raw),['0xff4488ff','0xffff4444']);
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('alignment requirements reject missing support and non-layout targets before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-align-self-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=root><div id=a></div></div>',css:'#root{width:100%;height:120px;padding:10px;flex-direction:row;align-items:flex-end}#a{width:40px;height:30px;align-self:center;background:red}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));assert.equal(manifest.version,6);
  const run=(name,extra={})=>{const prefix=path.join(dir,name);const r=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','320',prefix],{encoding:'utf8',env:{...process.env,...extra}});return {...r,prefix};};
  const good=run('good');assert.equal(good.status,0,good.stderr);assert.equal(JSON.parse(fs.readFileSync(good.prefix+'.bounds.json')).a.y,45);
  const missing=run('missing',{NUXIE_DISABLE_CSS_ALIGN_SELF:'1'});assert.notEqual(missing.status,0);assert.match(missing.stderr,/missing-runtime-capability/);assert.equal(fs.existsSync(missing.prefix+'.stream'),false);
  for(const [name,change] of [['old',m=>m.version=5],['duplicate',m=>m.layout_align_self.push({...m.layout_align_self[0]})],['style-target',m=>m.layout_align_self[0].object_id++],['absent-target',m=>m.layout_align_self[0].object_id=999999]]) {
   const bad=structuredClone(manifest);change(bad);fs.writeFileSync(scene+'.requirements.json',JSON.stringify(bad));const rejected=run(name);assert.notEqual(rejected.status,0,name);assert.match(rejected.stderr,/invalid-layout-align-self/);assert.equal(fs.existsSync(rejected.prefix+'.stream'),false);
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('line alignment requirements reject missing support and non-layout targets before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-align-content-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=root><div id=a></div></div>',css:'#root{width:100%;height:160px;padding:10px;flex-direction:row;flex-wrap:wrap;align-items:flex-start;align-content:center}#a{width:40px;height:20px;background:red}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));assert.equal(manifest.version,7);
  const run=(name,extra={})=>{const prefix=path.join(dir,name);const r=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','320',prefix],{encoding:'utf8',env:{...process.env,...extra}});return {...r,prefix};};
  const good=run('good');assert.equal(good.status,0,good.stderr);assert.equal(JSON.parse(fs.readFileSync(good.prefix+'.bounds.json')).a.y,70);
  const missing=run('missing',{NUXIE_DISABLE_CSS_ALIGN_CONTENT:'1'});assert.notEqual(missing.status,0);assert.match(missing.stderr,/missing-runtime-capability/);assert.equal(fs.existsSync(missing.prefix+'.stream'),false);
  for(const [name,change] of [['old',m=>m.version=6],['duplicate',m=>m.layout_align_content.push({...m.layout_align_content[0]})],['style-target',m=>m.layout_align_content[0].object_id++],['absent-target',m=>m.layout_align_content[0].object_id=999999]]) {
   const bad=structuredClone(manifest);change(bad);fs.writeFileSync(scene+'.requirements.json',JSON.stringify(bad));const rejected=run(name);assert.notEqual(rejected.status,0,name);assert.match(rejected.stderr,/invalid-layout-align-content/);assert.equal(fs.existsSync(rejected.prefix+'.stream'),false);
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('distributed spacing requirements reject missing support and non-layout targets before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-distribution-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=root><div id=a></div></div>',css:'#root{width:100%;height:160px;padding:10px;flex-direction:row;flex-wrap:wrap;align-items:flex-start;align-content:space-evenly;justify-content:space-evenly}#a{width:40px;height:20px;background:red}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));assert.equal(manifest.version,8);
  const run=(name,extra={})=>{const prefix=path.join(dir,name);const r=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','240','320',prefix],{encoding:'utf8',env:{...process.env,...extra}});return {...r,prefix};};
  const good=run('good');assert.equal(good.status,0,good.stderr);assert.equal(JSON.parse(fs.readFileSync(good.prefix+'.bounds.json')).a.x,100);
  const missing=run('missing',{NUXIE_DISABLE_CSS_DISTRIBUTED_SPACING:'1'});assert.notEqual(missing.status,0);assert.match(missing.stderr,/missing-runtime-capability/);assert.equal(fs.existsSync(missing.prefix+'.stream'),false);
  for(const [name,change] of [['old',m=>m.version=7],['duplicate',m=>m.layout_justify_content.push({...m.layout_justify_content[0]})],['style-target',m=>m.layout_justify_content[0].object_id++],['absent-target',m=>m.layout_justify_content[0].object_id=999999]]) {
   const bad=structuredClone(manifest);change(bad);fs.writeFileSync(scene+'.requirements.json',JSON.stringify(bad));const rejected=run(name);assert.notEqual(rejected.status,0,name);assert.match(rejected.stderr,/invalid-layout-(distributed-spacing|justify-content-target)/);assert.equal(fs.existsSync(rejected.prefix+'.stream'),false);
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('wrap-reverse bytes reflow in the checked host at three widths',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-wrap-reverse-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=root><div id=a></div><div id=b></div><div id=c></div><div id=d></div></div>',css:'#root{width:100%;height:160px;padding:10px;gap:10px;flex-direction:row;flex-wrap:wrap-reverse}#root>div{width:90px;height:20px;background:red}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));assert.equal(manifest.version,7);
  for(const [width,y] of [[240,100],[390,130],[768,130]]) {
   const prefix=path.join(dir,String(width));
   const r=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8'});
   assert.equal(r.status,0,r.stderr);assert.equal(JSON.parse(fs.readFileSync(prefix+'.bounds.json')).c.y,y);
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('auto margins consume free space before distributed alignment in the checked host',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-auto-margin-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=root><div id=a></div><div id=b></div><div id=c></div></div>',css:'#root{width:100%;height:180px;padding:10px;gap:10px;flex-direction:row;justify-content:space-evenly}#root>div{width:85px;height:50px;background:red}#b{margin-left:auto}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  for(const [width,bx,cx] of [[240,105,200],[390,200,295],[768,578,673]]) {
   const prefix=path.join(dir,String(width));
   const r=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8'});
   assert.equal(r.status,0,r.stderr);
   const bounds=JSON.parse(fs.readFileSync(prefix+'.bounds.json'));
   assert.equal(bounds.a.x,10);assert.equal(bounds.b.x,bx);assert.equal(bounds.c.x,cx);
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('independent flex factors resize and reject unsupported or invalid host contracts',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-independent-flex-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=root><div id=a></div><div id=b></div></div>',css:'#root{width:100%;height:100px;padding:10px;flex-direction:row}#a,#b{width:100px;height:20px}#a{flex:1 0 100px}#b{flex:0 1 100px}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(manifest.version,9);assert.equal(manifest.layout_flex_factors.length,2);
  const run=(name,width=390,extra={})=>{
   const prefix=path.join(dir,name);const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,...extra}});return {...result,prefix};
  };
  for(const [width,a,b] of [[390,270,100],[240,120,100],[180,100,60]]) {
   const result=run(String(width),width);assert.equal(result.status,0,result.stderr);
   const bounds=JSON.parse(fs.readFileSync(result.prefix+'.bounds.json'));assert.equal(bounds.a.width,a);assert.equal(bounds.b.width,b);
  }
  const absent=run('unsupported',390,{NUXIE_DISABLE_CSS_FLEX_FACTORS:'1'});assert.notEqual(absent.status,0);assert.match(absent.stderr,/missing-runtime-capability/);assert.equal(fs.existsSync(absent.prefix+'.stream'),false);
  for(const [name,mutate] of [
   ['old-version',m=>m.version=8],['negative',m=>m.layout_flex_factors[0].shrink=-1],
   ['duplicate',m=>m.layout_flex_factors.push({...m.layout_flex_factors[0]})],
   ['non-layout',m=>m.layout_flex_factors[0].object_id++],['absent-target',m=>m.layout_flex_factors[0].object_id=999999],
  ]) {
   const bad=structuredClone(manifest);mutate(bad);fs.writeFileSync(scene+'.requirements.json',JSON.stringify(bad));
   const result=run(name);assert.notEqual(result.status,0,name);assert.match(result.stderr,/invalid-layout-flex-factors/);assert.equal(fs.existsSync(result.prefix+'.stream'),false);
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('partial flex factors preserve unused space and require corrected host distribution',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-partial-flex-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=root><div id=a></div><div id=b></div></div>',css:'#root{width:100%;height:100px;padding:10px;gap:8px;flex-direction:row}#a,#b{height:40px;flex:.25 .25 20px}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(manifest.version,10);assert.equal(manifest.layout_flex_factors.length,2);
  assert.ok(manifest.capabilities.includes('layout-css-partial-flex-factors-v1'));
  const run=(name,width=390,extra={})=>{
   const prefix=path.join(dir,name);const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,...extra}});return {...result,prefix};
  };
  for(const [width,expected] of [[240,63],[390,100.5],[768,195]]) {
   const result=run(String(width),width);assert.equal(result.status,0,result.stderr);
   const bounds=JSON.parse(fs.readFileSync(result.prefix+'.bounds.json'));
   for(const id of ['a','b']) assert.ok(Math.abs(bounds[id].width-expected)<.1,`${width} ${id}`);
   assert.ok(bounds.b.x+bounds.b.width<width-10,'Partial growth must leave unused space');
  }
  const absent=run('unsupported',390,{NUXIE_DISABLE_CSS_PARTIAL_FLEX_FACTORS:'1'});
  assert.notEqual(absent.status,0);assert.match(absent.stderr,/missing-runtime-capability/);assert.equal(fs.existsSync(absent.prefix+'.stream'),false);
  for(const [name,mutate] of [
   ['old-version',m=>m.version=9],
   ['missing-capability',m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-partial-flex-factors-v1')],
   ['no-partial-factors',m=>m.layout_flex_factors.forEach(e=>{e.grow=1;e.shrink=1})],
  ]) {
   const bad=structuredClone(manifest);mutate(bad);fs.writeFileSync(scene+'.requirements.json',JSON.stringify(bad));
   const result=run(name);assert.notEqual(result.status,0,name);assert.match(result.stderr,/invalid-layout-partial-flex-factors/);assert.equal(fs.existsSync(result.prefix+'.stream'),false);
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});

test('intrinsic auto basis resizes unequal text and rejects incompatible hosts before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-intrinsic-'));
 try {
  const scene=path.join(dir,'scene');
  const source={html:'<div id=root><p id=a>Short</p><p id=b>Longer text</p></div>',css:'#root{width:100%;flex-direction:row;padding:10px;gap:8px}#a,#b{flex:auto;padding:8px}',width:390,height:320,assets:{inter:{kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/Inter-Regular.ttf',import.meta.url))]}}};
  fs.writeFileSync(scene+'.json',JSON.stringify(source));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const bytes=fs.readFileSync(scene+'.riv');
  const manifest=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(manifest.version,11);assert.equal(manifest.layout_intrinsic_sizing.length,2);
  assert.ok(manifest.capabilities.includes('layout-css-intrinsic-sizing-v1'));
  const run=(name,width=390,extra={})=>{
   const prefix=path.join(dir,name);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',...extra}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'Reject before drawing');
   return {...result,prefix};
  };
  // Independent Chromium references in content-auto-oracle/oracle.json.
  for(const [width,a,b] of [[240,83.796875,128.203125],[390,158.796875,203.203125],[768,347.796875,392.203125]]) {
   const result=run('width-'+width,width);assert.equal(result.status,0,result.stderr);
   const bounds=JSON.parse(fs.readFileSync(result.prefix+'.bounds.json'));
   assert.ok(Math.abs(bounds.a.width-a)<.1);assert.ok(Math.abs(bounds.b.width-b)<.1);
   assert.deepEqual(fs.readFileSync(scene+'.riv'),bytes);
  }
  const absent=run('unsupported',390,{NUXIE_DISABLE_CSS_INTRINSIC_SIZING:'1'});
  assert.notEqual(absent.status,0);assert.match(absent.stderr,/missing-runtime-capability/);
  for(const [name,mutate,diagnostic] of [
   ['old-version',m=>m.version=9,/invalid-layout-intrinsic-sizing/],
   ['missing-capability',m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-intrinsic-sizing-v1'),/invalid-layout-intrinsic-sizing/],
   ['duplicate-target',m=>m.layout_intrinsic_sizing.push(m.layout_intrinsic_sizing[0]),/invalid-layout-intrinsic-sizing/],
   ['unknown-target',m=>m.layout_intrinsic_sizing[0]=999999,/invalid-layout-intrinsic-sizing-target/],
  ]) {
   const bad=structuredClone(manifest);mutate(bad);fs.writeFileSync(scene+'.requirements.json',JSON.stringify(bad));
   const result=run(name);assert.notEqual(result.status,0,name);assert.match(result.stderr,diagnostic);
  }
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('version16 percentage spacing rejects unsupported hosts and malformed targets before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-spacing-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=host><div id=card><div id=inner></div></div></div>',css:'#host{width:100%;height:200px;padding:10px}#card{width:100px;height:90px;padding-top:3%;margin-top:2%}#inner{width:30px;height:20px}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const valid=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(valid.version,16);assert.equal(valid.layout_percentage_spacing.length,1);
  const bytes=fs.readFileSync(scene+'.riv');let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`frame-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_PERCENTAGE_SPACING:disabled?'1':'0'}});
   return {result,prefix};
  };
  for(const width of [240,390,768,240]) {
   const {result,prefix}=run(valid,width);assert.equal(result.status,0,result.stderr);
   const actual=JSON.parse(fs.readFileSync(prefix+'.bounds.json')).inner.y;
   const expected=10+Math.floor((width-20)*.02*64)/64+Math.floor((width-20)*.03*64)/64;
   assert.ok(Math.abs(actual-expected)<.001,`${width}: ${actual} vs ${expected}`);
   assert.deepEqual(fs.readFileSync(scene+'.riv'),bytes);
  }
  const disabled=run(valid,390,true);assert.notEqual(disabled.result.status,0);assert.match(disabled.result.stderr,/missing-runtime-capability/);assert.equal(fs.existsSync(disabled.prefix+'.stream'),false);
  for(const mutate of [m=>m.version=15,m=>m.layout_percentage_spacing=[],m=>m.layout_percentage_spacing.push(m.layout_percentage_spacing[0]),m=>m.layout_percentage_spacing=[999999],m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-percentage-spacing-v1')]) {
   const bad=structuredClone(valid);mutate(bad);const {result,prefix}=run(bad);
   assert.notEqual(result.status,0);assert.equal(fs.existsSync(prefix+'.stream'),false);
  }
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('version17 positioned paint installs before drawing and rejects invalid contracts',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-positioned-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=host><div id=a><div id=inner></div></div><div id=b></div></div>',css:'#host{width:100%;height:260px;padding:20px;flex-direction:row;gap:10px}#a{width:80px;height:100px;background:#e9a344}#inner{width:70px;height:50px;background:#318ea8;position:relative;left:80px;top:10px}#b{width:90px;height:80px;background:#86ae83}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const valid=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  const map=JSON.parse(fs.readFileSync(scene+'.map.json'));
  assert.equal(valid.version,17);assert.deepEqual(valid.layout_positioned,[map.find(n=>n.id==='inner').object_id]);
  assert.ok(valid.capabilities.includes('layout-css-positioned-paint-v1'));
  const bytes=fs.readFileSync(scene+'.riv');let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`frame-${sequence++}`);
   const env={...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_POSITIONED_PAINT:disabled?'1':'0'};
   delete env.NUXIE_CSS_POSITIONED_SOURCE_IDS;
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'Reject before drawing');
   return {result,prefix};
  };
  for(const width of [240,390,768,240]) {
   const {result,prefix}=run(valid,width);assert.equal(result.status,0,result.stderr);
   const bounds=JSON.parse(fs.readFileSync(prefix+'.bounds.json'));
   assert.equal(bounds.inner.x,100);assert.equal(bounds.inner.y,30);
   const colors=fs.readFileSync(prefix+'.stream','utf8').split('\n').filter(l=>l.startsWith('drawPath ')).map(l=>l.split('color=')[1].split(',')[0]);
   assert.deepEqual(colors,['0xffe9a344','0xff86ae83','0xff318ea8']);
   assert.deepEqual(fs.readFileSync(scene+'.riv'),bytes);
  }
  const disabled=run(valid,390,true);assert.notEqual(disabled.result.status,0);assert.match(disabled.result.stderr,/missing-runtime-capability/);
  for(const mutate of [m=>m.version=16,m=>m.version=18,m=>m.layout_positioned=[],m=>m.layout_positioned.push(m.layout_positioned[0]),m=>m.layout_positioned=[0],m=>m.layout_positioned=[999999],m=>m.layout_positioned=[map.find(n=>n.id==='a').object_id+1],m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-positioned-paint-v1')]) {
   const bad=structuredClone(valid);mutate(bad);const {result}=run(bad);assert.notEqual(result.status,0,JSON.stringify(bad));
  }
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('version22 borders reject incompatible hosts and malformed targets before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-border-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=a></div>',css:'#a{width:60%;height:100px;border:8px solid #563cab80;border-radius:18px}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const valid=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(valid.version,22); assert.equal(valid.layout_borders.length,1);
  assert.equal(valid.layout_borders[0].color,0x80563cab);
  let sequence=0;
  const run=(manifest,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`frame-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json','390','320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_BORDERS:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before drawing');
   return {...result,prefix};
  };
  const accepted=run(valid);assert.equal(accepted.status,0,accepted.stderr);
  assert.match(fs.readFileSync(accepted.prefix+'.stream','utf8'),/drawPath .*80563cab/);
  const disabled=run(valid,true);assert.notEqual(disabled.status,0);assert.match(disabled.stderr,/missing-runtime-capability/);
  for(const mutate of [m=>m.version=21,m=>m.version=23,m=>m.layout_borders=[],m=>m.layout_borders.push({...m.layout_borders[0]}),m=>m.layout_borders[0].object_id=0,m=>m.layout_borders[0].object_id=999999,m=>m.layout_borders[0].color=-1,m=>m.layout_borders[0].color=4294967296,m=>m.layout_borders[0].color=1.5,m=>m.layout_borders[0].color=null,m=>m.layout_borders[0].extra=true,m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-solid-borders-v1')]) {
   const bad=structuredClone(valid);mutate(bad);assert.notEqual(run(bad).status,0,JSON.stringify(bad));
  }
  assert.equal(run(valid).status,0,'valid manifest survives rejected attempts');
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('version23 border sides validate mixed policies before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-border-sides-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=a><div id=b></div></div>',css:'#a{width:60%;height:140px;border-style:solid;border-width:2px 8px 14px 20px;border-color:#c44747 #3a8752 #386fc4 #b58129;border-radius:18px}#b{width:30px;height:20px;border:3px solid #563cab80}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const valid=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  const map=JSON.parse(fs.readFileSync(scene+'.map.json'));
  assert.equal(valid.version,23);
  assert.equal(valid.layout_borders.length,1);
  assert.equal(valid.layout_border_sides.length,1);
  assert.deepEqual(valid.layout_border_sides[0].colors,[0xffc44747,0xff3a8752,0xff386fc4,0xffb58129]);
  let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`frame-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_BORDERS:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before drawing');
   return {...result,prefix};
  };
  for(const width of [240,390,768,240]) {
   const result=run(valid,width);assert.equal(result.status,0,result.stderr);
   const stream=fs.readFileSync(result.prefix+'.stream','utf8');
   for(const color of ['ffc44747','ff3a8752','ff386fc4','ffb58129','80563cab'])
    assert.match(stream,new RegExp('drawPath .*'+color));
  }
  const disabled=run(valid,390,true);assert.notEqual(disabled.status,0);assert.match(disabled.stderr,/missing-runtime-capability/);
  const mutations=[
   m=>m.version=22,m=>m.version=24,m=>m.layout_border_sides=[],
   m=>m.layout_border_sides.push({...m.layout_border_sides[0]}),
   m=>m.layout_border_sides[0].object_id=0,m=>m.layout_border_sides[0].object_id=999999,
   m=>m.layout_border_sides[0].object_id=map.find(n=>n.id==='b').object_id,
   m=>m.layout_border_sides[0].colors=[],m=>m.layout_border_sides[0].colors=[1,2,3],
   m=>m.layout_border_sides[0].colors=[1,2,3,4,5],m=>m.layout_border_sides[0].colors[0]=-1,
   m=>m.layout_border_sides[0].colors[0]=4294967296,m=>m.layout_border_sides[0].colors[0]=1.5,
   m=>m.layout_border_sides[0].colors[0]=null,m=>m.layout_border_sides[0].extra=true,
   m=>delete m.layout_border_sides[0].colors,
   m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-border-sides-v1'),
  ];
  for(const mutate of mutations) {
   const bad=structuredClone(valid);mutate(bad);assert.notEqual(run(bad).status,0,JSON.stringify(bad));
  }
  assert.equal(run(valid).status,0,'valid manifest survives rejected attempts');
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('version24 corner radii reject malformed policies and incompatible hosts before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-corner-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=a><div id=b></div></div>',css:'#a{width:60%;height:140px;overflow:clip;background:orange;border:4px solid red;border-radius:25% / 12px}#b{width:200px;height:160px;background:teal}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const valid=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  assert.equal(valid.version,24);assert.equal(valid.layout_corner_radii.length,1);
  assert.deepEqual(valid.layout_corner_radii[0].radii,Array.from({length:4},()=>[{percent:25},{pixels:12}]));
  let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`view-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_CORNER_RADII:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before drawing');
   return {...result,prefix};
  };
  for(const width of [240,390,768,240]) {
   const result=run(valid,width);assert.equal(result.status,0,result.stderr);
   assert.match(fs.readFileSync(result.prefix+'.stream','utf8'),/drawPath/);
  }
  const disabled=run(valid,390,true);assert.notEqual(disabled.status,0);assert.match(disabled.stderr,/missing-runtime-capability/);
  const mutations=[m=>m.version=23,m=>m.version=25,m=>m.layout_corner_radii=[],
   m=>m.layout_corner_radii.push(structuredClone(m.layout_corner_radii[0])),
   m=>m.layout_corner_radii[0].object_id=0,m=>m.layout_corner_radii[0].object_id=999999,
   m=>m.layout_corner_radii[0].radii=[],m=>m.layout_corner_radii[0].radii[0]=[{pixels:2}],
   m=>m.layout_corner_radii[0].radii[0][0]={pixels:1,percent:2},
   m=>m.layout_corner_radii[0].radii[0][0]={pixels:-1},
   m=>m.layout_corner_radii[0].radii[0][0]={percent:-1},
   m=>m.layout_corner_radii[0].radii[0][0]={percent:null},
   m=>m.layout_corner_radii[0].radii[0][0]={em:2},
   m=>m.layout_corner_radii[0].extra=true,
   m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-corner-radii-v1')];
  for(const mutate of mutations) {
   const bad=structuredClone(valid);mutate(bad);assert.notEqual(run(bad).status,0,JSON.stringify(bad));
  }
  assert.equal(run(valid).status,0,'valid manifest survives rejected attempts');
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});

test('version25 opacity installs recorded groups and rejects incompatible hosts before drawing',()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-opacity-contract-'));
 try {
  const scene=path.join(dir,'scene');
  fs.writeFileSync(scene+'.json',JSON.stringify({html:'<div id=a><div id=b></div></div>',css:'#a{width:75%;height:120px;background:blue}#b{width:80px;height:80px;background:red}',width:390,height:320}));
  execFileSync(nativeCompiler,[scene+'.json',scene+'.riv']);
  const baseline=JSON.parse(fs.readFileSync(scene+'.requirements.json'));
  const map=JSON.parse(fs.readFileSync(scene+'.map.json'));
  const a=map.find(n=>n.id==='a').object_id,b=map.find(n=>n.id==='b').object_id;
  // Transport integration test until public opacity CSS emission is admitted.
  const valid={...baseline,version:25,capabilities:[...baseline.capabilities,'layout-css-group-opacity-v1'],layout_group_opacity:[{object_id:a,opacity:.5},{object_id:b,opacity:.25}]};
  let sequence=0;
  const run=(manifest,width=390,disabled=false)=>{
   fs.writeFileSync(scene+'.requirements.json',JSON.stringify(manifest));
   const prefix=path.join(dir,`frame-${sequence++}`);
   const result=spawnSync(nativeProbe,[scene+'.riv',scene+'.map.json',String(width),'320',prefix],{encoding:'utf8',env:{...process.env,NUXIE_NATIVE_GLYPHS:'0',NUXIE_DISABLE_CSS_GROUP_OPACITY:disabled?'1':'0'}});
   if(result.status!==0)assert.equal(fs.existsSync(prefix+'.stream'),false,'reject before output');
   return {...result,prefix};
  };
  for(const width of [240,390,768,240]) {
   const result=run(valid,width);assert.equal(result.status,0,result.stderr);
   const stream=fs.readFileSync(result.prefix+'.stream','utf8');
   assert.deepEqual(stream.split('\n').filter(line=>/^(beginOpacity|endOpacity)/.test(line)),['beginOpacity opacity=0.5','beginOpacity opacity=0.25','endOpacity','endOpacity']);
  }
  const disabled=run(valid,390,true);assert.notEqual(disabled.status,0);assert.match(disabled.stderr,/missing-runtime-capability/);
  for(const mutate of [m=>m.version=24,m=>m.layout_group_opacity=[],m=>m.layout_group_opacity[0].object_id=0,m=>m.layout_group_opacity[0].object_id=999999,m=>m.layout_group_opacity.push({...m.layout_group_opacity[0]}),m=>m.layout_group_opacity[0].opacity=1,m=>m.layout_group_opacity[0].opacity=-1,m=>m.layout_group_opacity[0].opacity=null,m=>m.capabilities=m.capabilities.filter(c=>c!=='layout-css-group-opacity-v1')]) {
   const bad=structuredClone(valid);mutate(bad);assert.notEqual(run(bad).status,0,JSON.stringify(bad));
  }
  assert.equal(run(valid).status,0,'valid manifest still loads');
 } finally {fs.rmSync(dir,{recursive:true,force:true});}
});
