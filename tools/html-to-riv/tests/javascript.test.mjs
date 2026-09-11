import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {execFileSync, spawnSync} from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createCompiler} from '@nuxie/html-to-riv';

const nativeCompiler = process.env.NUXIE_NATIVE_COMPILER
  ? path.resolve(process.env.NUXIE_NATIVE_COMPILER)
  : path.join(process.env.CARGO_TARGET_DIR ? path.resolve(fileURLToPath(new URL('../../../',import.meta.url)),process.env.CARGO_TARGET_DIR) : fileURLToPath(new URL('../../../target',import.meta.url)),'debug/html-to-riv');
const wasm = fs.readFileSync(new URL('../dist/html-to-riv.wasm', import.meta.url));
const input = JSON.parse(fs.readFileSync(new URL('../examples/box.json', import.meta.url)));

test('JavaScript compiles the same bytes and source map as the native publisher', async () => {
  const compiler = await createCompiler(wasm);
  const result = compiler.compile({languageVersion:'nuxie-html-v1', ...input});
  assert.equal(result.ok, true);
  assert.ok(result.riv instanceof Uint8Array);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'html-riv-'));
  try {
    fs.writeFileSync(path.join(directory,'input.json'), JSON.stringify(input));
    execFileSync(nativeCompiler, [path.join(directory,'input.json'),path.join(directory,'scene.riv')]);
    assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(path.join(directory,'scene.riv')));
    assert.deepEqual(result.sourceMap, JSON.parse(fs.readFileSync(path.join(directory,'scene.map.json'))));
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('Node asset buffers and browser byte arrays produce the same embedded asset', async () => {
  const compiler = await createCompiler(wasm);
  const bytes = fs.readFileSync(new URL('assets/quadrants.png',import.meta.url));
  const document = {languageVersion:'nuxie-html-v1',html:'<img src=asset:photo style="width:32px;height:32px">',css:'',width:100,height:100,assets:{photo:{kind:'image',bytes}}};
  const result = compiler.compile(document);
  assert.equal(result.ok,true,JSON.stringify(result));
  document.assets.photo.bytes = new Uint8Array(bytes);
  assert.deepEqual(compiler.compile(document),result);
});

test('failed editor edits return diagnostics without poisoning later compilation', async () => {
  const compiler = await createCompiler(wasm);
  const document = {languageVersion:'nuxie-html-v1',...input};
  for (const invalid of [
    {...document,languageVersion:'future-version'},
    {...document,css:'div {display:grid}'},
    {...document,html:'<script>alert(1)</script>'},
    {...document,width:NaN},
    {...document,unexpected:true},
  ]) {
    const result = compiler.compile(invalid);
    assert.equal(result.ok,false);
    assert.ok(result.diagnostics.length > 0);
    assert.equal('riv' in result,false);
    assert.equal(compiler.compile(document).ok,true);
  }
  const circular = {...document}; circular.assets = {cycle:circular};
  assert.equal(compiler.compile(circular).ok,false);
  assert.equal(compiler.compile(document).ok,true);
});

test('returned bytes and source maps survive repeated calls and independent instances', async () => {
  const module = await WebAssembly.compile(wasm);
  const first = await createCompiler(module);
  const second = await createCompiler(module);
  const document = {languageVersion:'nuxie-html-v1',...input};
  const original = first.compile(document);
  const copy = structuredClone(original);
  for (let i=0;i<40;i++) {
    assert.equal(first.compile({...document,width:240+i}).ok,true);
    assert.equal(second.compile({...document,css:'div {display:grid}'}).ok,false);
  }
  assert.deepEqual(original,copy);
  assert.deepEqual(second.compile(document),copy);
});

test('the accepted scene corpus has identical WASM and native publish artifacts', async () => {
  const compiler = await createCompiler(wasm);
  const cases = ['elliptical-radii-composition-cases.json','elliptical-radii-edge-cases.json','elliptical-radii-initial-cases.json','corner-radii-nested-cases.json','corner-radii-composition-cases.json','corner-radii-initial-cases.json','border-sides-composition-cases.json','border-sides-edge-cases.json','border-sides-initial-cases.json','content-box-border-deferred-cases.json','border-axis-margin-cases.json','border-content-cases.json','border-clip-margin-cases.json','border-image-corner-minimal-cases.json','border-solid-cases.json','border-paint-cases.json','border-plain-cases.json','cases.json','stacking-context-cases.json','stacking-composition-cases.json','stacking-overlap-cases.json','indefinite-auto-main-cases.json','content-box-edge-cases.json','content-box-image-cases.json','aspect-ratio-cases.json','aspect-ratio-auto-probe-cases.json','aspect-ratio-expanded-cases.json','aspect-ratio-text-cases.json','aspect-ratio-image-cases.json','aspect-ratio-image-stress-cases.json'].flatMap(file=>JSON.parse(fs.readFileSync(new URL('../validation/'+file,import.meta.url))));
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-math-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-clamp-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-approximation-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-nonfinite-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-saturation-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-saturation-stress-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-cascade-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-typed-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-unit-order-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-rounding-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-rounding-stress-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-nested-gap-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-font-math-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-font-boundary-oracle.json',import.meta.url))).cases);
  cases.push(...JSON.parse(fs.readFileSync(new URL('assets/aspect-ratio-pair-stress-oracle.json',import.meta.url))).cases);
  for (const file of ['intrinsic-image-initial-oracle.json','intrinsic-image-nonsquare-oracle.json','intrinsic-image-card-oracle.json','intrinsic-image-flex-oracle.json','intrinsic-image-stretch-edge-oracle.json','percentage-spacing-initial-oracle.json','percentage-spacing-root-oracle.json','percentage-spacing-composition-oracle.json','percentage-spacing-minimal-oracle.json','percentage-spacing-rounding-oracle.json','percentage-spacing-reverse-composition-oracle.json','percentage-spacing-interaction-oracle.json','percentage-spacing-basis-reduced-oracle.json','negative-margin-oracle.json','negative-margin-composition-oracle.json','negative-margin-interaction-oracle.json','relative-position-oracle.json','relative-position-composition-oracle.json','relative-position-interaction-oracle.json','absolute-position-oracle.json','absolute-position-containing-block-oracle.json','absolute-position-composition-oracle.json','absolute-position-interaction-oracle.json']) cases.push(...JSON.parse(fs.readFileSync(new URL('assets/'+file,import.meta.url))).cases);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-corpus-'));
  try {
    for (const fixture of cases) {
      const request = {html:fixture.html,css:fixture.css,width:390,height:320,assets:{}};
      if (fixture.font) request.assets.inter = {kind:'font',family:fixture.fontFamily||'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/'+(['OpenSans-Regular.ttf','NotoSansOgham-Regular.ttf','NuxieJapaneseFixture-Regular.otf'].includes(fixture.fontAsset)?fixture.fontAsset:'Inter-Regular.ttf'),import.meta.url))]};
      if (fixture.image) {
        const imageAsset=fixture.imageAsset||'quadrants.png';
        assert.equal(path.basename(imageAsset),imageAsset);
        request.assets.photo = {kind:'image',bytes:[...fs.readFileSync(new URL('assets/'+imageAsset,import.meta.url))]};
      }
      const result = compiler.compile({languageVersion:'nuxie-html-v1',...request});
      assert.equal(result.ok,true,`${fixture.name}: ${JSON.stringify(result.diagnostics)}`);
      fs.writeFileSync(path.join(directory,'input.json'),JSON.stringify(request));
      execFileSync(nativeCompiler,[path.join(directory,'input.json'),path.join(directory,'scene.riv')]);
      assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(path.join(directory,'scene.riv')),fixture.name);
      assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(path.join(directory,'scene.map.json'))),fixture.name);
      assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(path.join(directory,'scene.requirements.json'))),fixture.name);
    }
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('fractional runtime metrics retain native JSON numeric representation', async () => {
  const compiler=await createCompiler(wasm);
  const request={html:'<p>fractional underline</p>',css:'p{font-size:24px;line-height:40px;text-decoration:underline auto}',width:240,height:80,
    assets:{font:{kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/Inter-Regular.ttf',import.meta.url))]}}};
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-fractional-'));
  try {
    const inputFile=path.join(directory,'input.json'),outputFile=path.join(directory,'scene.riv');
    fs.writeFileSync(inputFile,JSON.stringify(request));execFileSync(nativeCompiler,[inputFile,outputFile]);
    const result=compiler.compile({languageVersion:'nuxie-html-v1',...request});
    assert.equal(result.ok,true);
    assert.equal(result.runtimeRequirements.text_underlines[0].lines[0].thickness,2.4);
    assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(path.join(directory,'scene.requirements.json'))));
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('strikethrough and combined decorations have identical native and WASM contracts', async () => {
  const compiler = await createCompiler(wasm);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-strike-'));
  try {
    for (const fontName of ['Inter','OpenSans']) for (const decoration of ['line-through 2px red','underline line-through from-font','line-through 10%']) {
      const request={html:'<div><p id=a>agypqj M</p><p id=b>second line</p></div>',css:`div{font:24px/40px ${fontName};text-decoration:${decoration}}#a{text-decoration:none}#b{text-decoration:line-through blue}`,width:390,height:320,assets:{font:{kind:'font',family:fontName,weight:400,bytes:[...fs.readFileSync(new URL(`assets/${fontName}-Regular.ttf`,import.meta.url))]}}};
      const result=compiler.compile({languageVersion:'nuxie-html-v1',...request});
      assert.equal(result.ok,true,JSON.stringify(result));
      assert.equal(result.runtimeRequirements.version,11);
      assert.equal(result.runtimeRequirements.text_strikethroughs.length,2);
      assert.equal(result.runtimeRequirements.text_strikethroughs[1].lines.length,2);
      fs.writeFileSync(path.join(directory,'input.json'),JSON.stringify(request));
      execFileSync(nativeCompiler,[path.join(directory,'input.json'),path.join(directory,'scene.riv')]);
      assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(path.join(directory,'scene.riv')));
      assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(path.join(directory,'scene.map.json'))));
      assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(path.join(directory,'scene.requirements.json'))));
    }
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});


test('strikethrough metric extremes retain native and WASM publish parity', async () => {
  const compiler=await createCompiler(wasm);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-strike-limits-'));
  const font={kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/Inter-Regular.ttf',import.meta.url))]};
  try {
    for (const size of [0.1,1,8,128,800000]) for (const thickness of ['auto','from-font','0px','1000000px']) {
      const request={html:'<p>agypqj M</p>',css:`p{font-size:${size}px;line-height:1000000px;text-decoration:line-through red ${thickness}}`,width:390,height:320,assets:{font}};
      const result=compiler.compile({languageVersion:'nuxie-html-v1',...request});
      assert.equal(result.ok,true,JSON.stringify(result));
      fs.writeFileSync(path.join(directory,'input.json'),JSON.stringify(request));
      execFileSync(nativeCompiler,[path.join(directory,'input.json'),path.join(directory,'scene.riv')]);
      assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(path.join(directory,'scene.riv')));
      assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(path.join(directory,'scene.map.json'))));
      assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(path.join(directory,'scene.requirements.json'))));
    }
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('layout paint requirements publish identically and reject invalid host targets', async () => {
  const compiler=await createCompiler(wasm);
  const request={html:'<div id="root"><div id="paint"></div></div>',css:'#root{width:100%}#paint{width:80px;height:40px;margin-top:.5px;background:coral}',width:390,height:320};
  const result=compiler.compile({languageVersion:'nuxie-html-v1',...request});
  assert.equal(result.ok,true,JSON.stringify(result));
  const id=result.sourceMap.find(n=>n.id==='paint').object_id;
  assert.equal(result.runtimeRequirements.version,5);
  assert.deepEqual(result.runtimeRequirements.layout_pixel_bounds,[id]);
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-layout-'));
  try {
    const source=path.join(directory,'input.json'),riv=path.join(directory,'scene.riv'),map=path.join(directory,'scene.map.json'),requirements=path.join(directory,'scene.requirements.json');
    fs.writeFileSync(source,JSON.stringify(request));execFileSync(nativeCompiler,[source,riv]);
    assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(requirements)));
    assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(riv));
    const probe=process.env.NUXIE_NATIVE_PROBE
      ? path.resolve(process.env.NUXIE_NATIVE_PROBE)
      : path.join(path.dirname(nativeCompiler),'examples/probe');
    const args=[riv,map,'240','320',path.join(directory,'frame')];
    execFileSync(probe,args);
    assert.throws(()=>execFileSync(probe,args,{env:{...process.env,NUXIE_DISABLE_CSS_PIXEL_BOUNDS:'1'},stdio:'pipe'}),/missing-runtime-capability/);
    // The same Rive bytes retain ordinary paint geometry without the policy.
    fs.writeFileSync(requirements,JSON.stringify({version:1,capabilities:[]}));
    execFileSync(probe,[riv,map,'240','320',path.join(directory,'raw')]);
    assert.deepEqual(JSON.parse(fs.readFileSync(path.join(directory,'raw.bounds.json'))),JSON.parse(fs.readFileSync(path.join(directory,'frame.bounds.json'))));
    assert.notDeepEqual(fs.readFileSync(path.join(directory,'raw.stream')),fs.readFileSync(path.join(directory,'frame.stream')));
    for(const wrong of [[id+1],[999999],[id,id]]) {
      fs.writeFileSync(requirements,JSON.stringify({...result.runtimeRequirements,layout_pixel_bounds:wrong}));
      assert.throws(()=>execFileSync(probe,args,{stdio:'pipe'}),/invalid-layout-pixel-bounds/);
    }
  } finally { fs.rmSync(directory,{recursive:true,force:true}); }
});


test('absolute positioning rejects missing capabilities and malformed imported targets', async () => {
  const compiler = await createCompiler(wasm);
  const request = {html:'<div id="root"><div id="card"></div></div>',
    css:'#root{position:relative;width:100%;height:200px}#card{position:absolute;left:10%;top:12px;width:80px;height:40px;background:coral}',
    width:390,height:320};
  const result = compiler.compile({languageVersion:'nuxie-html-v1',...request});
  assert.equal(result.ok,true,JSON.stringify(result));
  const card = result.sourceMap.find(n=>n.id==='card').object_id;
  const root = result.sourceMap.find(n=>n.id==='root').object_id;
  assert.equal(result.runtimeRequirements.version,18);
  assert.deepEqual(result.runtimeRequirements.layout_absolute,[card]);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-absolute-host-'));
  try {
    const source=path.join(directory,'input.json'), riv=path.join(directory,'scene.riv');
    const requirements=path.join(directory,'scene.requirements.json');
    fs.writeFileSync(source,JSON.stringify(request));
    execFileSync(nativeCompiler,[source,riv]);
    assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(requirements)));
    assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(riv));
    const probe = process.env.NUXIE_NATIVE_PROBE ? path.resolve(process.env.NUXIE_NATIVE_PROBE)
      : path.join(path.dirname(nativeCompiler),'examples/probe');
    const args=[riv,path.join(directory,'scene.map.json'),'240','320',path.join(directory,'frame')];
    execFileSync(probe,args);
    for (const missing of ['NUXIE_DISABLE_CSS_ABSOLUTE_POSITION','NUXIE_DISABLE_CSS_POSITIONED_PAINT']) {
      assert.throws(()=>execFileSync(probe,args,{env:{...process.env,[missing]:'1'},stdio:'pipe'}),/missing-runtime-capability/);
    }
    for (const wrong of [[],[0],[card,card],[999999]]) {
      fs.writeFileSync(requirements,JSON.stringify({...result.runtimeRequirements,layout_absolute:wrong}));
      assert.throws(()=>execFileSync(probe,args,{stdio:'pipe'}),/invalid-layout-absolute-position/);
    }
    // These lists pass schema validation but disagree with the imported wires.
    for (const wrong of [[root],[root,card]]) {
      fs.writeFileSync(requirements,JSON.stringify({...result.runtimeRequirements,layout_absolute:wrong}));
      assert.throws(()=>execFileSync(probe,args,{stdio:'pipe'}),/invalid-layout-absolute-position-target/);
    }
    fs.writeFileSync(requirements,JSON.stringify(result.runtimeRequirements));
    execFileSync(probe,args);
  } finally { fs.rmSync(directory,{recursive:true,force:true}); }
});

test('unsupported ratio math diagnostics match native through substitution', async () => {
  const compiler = await createCompiler(wasm);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-math-errors-'));
  try {
    for (const expression of ['calc(1vw / 1px)', 'sin(1)']) {
      for (const value of [expression, 'var(--ratio)', `var(--missing, ${expression})`]) {
        const request = {html:'<div id=a></div>', css:`#a{--ratio:${expression};aspect-ratio:${value}}`,width:390,height:320};
        const result=compiler.compile({languageVersion:'nuxie-html-v1',...request});
        assert.equal(result.ok,false,request.css);
        const file=path.join(directory,'input.json');fs.writeFileSync(file,JSON.stringify(request));
        const native=spawnSync(nativeCompiler,[file,path.join(directory,'scene.riv')],{encoding:'utf8'});
        assert.equal(native.status,1,native.stderr);
        assert.deepEqual(result.diagnostics,JSON.parse(native.stderr),request.css);
      }
    }
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('percentage gaps remain rejected identically with CSS ratios', async () => {
  const compiler = await createCompiler(wasm);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-gap-errors-'));
  try {
    for (const property of ['gap','row-gap','column-gap']) {
      for (const value of ['5%','var(--space)','var(--missing,5%)']) {
        const request = {html:'<div id=root><div id=a></div></div>',
          css:`#root{--space:5%;${property}:${value}}#a{aspect-ratio:2}`,width:390,height:320};
        const result = compiler.compile({languageVersion:'nuxie-html-v1',...request});
        assert.equal(result.ok,false,request.css);
        const file = path.join(directory,'input.json'); fs.writeFileSync(file,JSON.stringify(request));
        const native = spawnSync(nativeCompiler,[file,path.join(directory,'scene.riv')],{encoding:'utf8'});
        assert.equal(native.status,1,native.stderr);
        assert.deepEqual(result.diagnostics,JSON.parse(native.stderr),request.css);
      }
    }
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('stacking host rejects malformed and non-layout targets before rendering', async () => {
  const compiler = await createCompiler(wasm);
  const request = {html:'<div id=root><div id=card></div></div>',
    css:'#root{width:100%;height:200px}#card{width:80px;height:60px;background:red;z-index:0}',width:390,height:320};
  const result = compiler.compile({languageVersion:'nuxie-html-v1',...request});
  assert.equal(result.ok,true,JSON.stringify(result));
  assert.equal(result.runtimeRequirements.version,19);
  assert.equal(result.runtimeRequirements.layout_positioned,undefined);
  const entry = result.runtimeRequirements.layout_stacking[0];
  const directory = fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-stacking-host-'));
  try {
    const source=path.join(directory,'input.json'), riv=path.join(directory,'scene.riv');
    const requirements=path.join(directory,'scene.requirements.json');
    fs.writeFileSync(source,JSON.stringify(request));
    execFileSync(nativeCompiler,[source,riv]);
    assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(requirements)));
    const probe = process.env.NUXIE_NATIVE_PROBE ? path.resolve(process.env.NUXIE_NATIVE_PROBE)
      : path.join(path.dirname(nativeCompiler),'examples/probe');
    const args=[riv,path.join(directory,'scene.map.json'),'240','320',path.join(directory,'frame')];
    execFileSync(probe,args);
    assert.throws(()=>execFileSync(probe,args,{env:{...process.env,NUXIE_DISABLE_CSS_STACKING:'1'},stdio:'pipe'}),/missing-runtime-capability/);
    for (const wrong of [[],[entry,entry],[{object_id:0,level:1}]]) {
      fs.writeFileSync(requirements,JSON.stringify({...result.runtimeRequirements,layout_stacking:wrong}));
      assert.throws(()=>execFileSync(probe,args,{stdio:'pipe'}),/invalid-layout-stacking/);
    }
    // Object 1 is the compiler's artboard LayoutComponentStyle, not a layout.
    for (const object_id of [1,999999]) {
      fs.writeFileSync(requirements,JSON.stringify({...result.runtimeRequirements,layout_stacking:[{object_id,level:0}]}));
      assert.throws(()=>execFileSync(probe,args,{stdio:'pipe'}),/invalid-layout-stacking-target/);
    }
    for (const level of [0.5,2147483648,-2147483649]) {
      fs.writeFileSync(requirements,JSON.stringify({...result.runtimeRequirements,layout_stacking:[{...entry,level}]}));
      assert.notEqual(spawnSync(probe,args,{stdio:'pipe'}).status,0);
    }
    fs.writeFileSync(requirements,JSON.stringify(result.runtimeRequirements));
    execFileSync(probe,args);
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('public group opacity retains native and WASM parity across browser-reference compositions', async()=>{
 const compiler=await createCompiler(wasm);
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'html-riv-opacity-parity-'));
 try {
  for(const name of ['initial','stacking','composition','overflow-composition','decoration']) {
   const oracle=JSON.parse(fs.readFileSync(new URL(`assets/group-opacity-${name}-oracle.json`,import.meta.url)));
   for(const fixture of oracle.cases) {
    const request={html:fixture.html,css:fixture.css,width:390,height:320,assets:{}};
    if(fixture.font)request.assets.inter={kind:'font',family:'Inter',weight:400,bytes:[...fs.readFileSync(new URL('assets/Inter-Regular.ttf',import.meta.url))]};
    if(fixture.image)request.assets.photo={kind:'image',bytes:[...fs.readFileSync(new URL('assets/quadrants.png',import.meta.url))]};
    const result=compiler.compile({languageVersion:'nuxie-html-v1',...request});
    assert.equal(result.ok,true,`${fixture.name}: ${JSON.stringify(result)}`);
    fs.writeFileSync(path.join(directory,'input.json'),JSON.stringify(request));
    execFileSync(nativeCompiler,[path.join(directory,'input.json'),path.join(directory,'scene.riv')]);
    assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(path.join(directory,'scene.riv')),fixture.name);
    assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(path.join(directory,'scene.map.json'))),fixture.name);
    assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(path.join(directory,'scene.requirements.json'))),fixture.name);
   }
  }
 } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('opacity boundary values clamp identically in native and WASM publishers',async()=>{
 const compiler=await createCompiler(wasm),dir=fs.mkdtempSync(path.join(os.tmpdir(),'opacity-boundary-parity-'));
 try {
  for(const value of ['0','.5','1','-1e100','1e100','1e100%','99.99999%','99.999994%','99.999997%','99.999999%','.99999994','.99999997','.99999999','100.000001%']) {
   for(const authored of [value,`var(--alpha, ${value})`]) {
    const request={html:'<div id=group><div id=child></div></div>',css:`#group{opacity:${authored}}#child{opacity:inherit}`,width:240,height:320};
    const result=compiler.compile({languageVersion:'nuxie-html-v1',...request});assert.equal(result.ok,true,authored);
    fs.writeFileSync(path.join(dir,'input.json'),JSON.stringify(request));
    execFileSync(nativeCompiler,[path.join(dir,'input.json'),path.join(dir,'scene.riv')]);
    assert.deepEqual(Buffer.from(result.riv),fs.readFileSync(path.join(dir,'scene.riv')),authored);
    assert.deepEqual(result.runtimeRequirements,JSON.parse(fs.readFileSync(path.join(dir,'scene.requirements.json'))),authored);
    assert.deepEqual(result.sourceMap,JSON.parse(fs.readFileSync(path.join(dir,'scene.map.json'))),authored);
   }
  }
 }finally{fs.rmSync(dir,{recursive:true,force:true});}
});
