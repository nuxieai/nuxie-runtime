"""Freeze/build or capture direct fixed_paint primitives; never public CSS admission.
Usage: SCRIPT prepare|build|capture OUTPUT. Coordinate build/capture with root.
"""
import difflib,hashlib,json,shutil,subprocess,sys
from pathlib import Path
M=Path(__file__).resolve().parents[1];R=M.parents[1]
phase=sys.argv[1];out=Path(sys.argv[2]).resolve()
assert out.is_relative_to(M/'output/playwright'), 'browser evidence must stay under compiler output/playwright'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def run(cmd,log,cwd=R):
 result=subprocess.run(cmd,cwd=cwd,capture_output=True,text=True);log.write_text(result.stdout+result.stderr)
 assert result.returncode==0,f'{cmd}: {result.stderr}'
if phase=='prepare':
 out.mkdir(parents=True,exist_ok=False)
 original=M/'validation/rounding-scalar-build.py';source=original.read_text()
 effective=source.replace('M=Path(__file__).resolve().parents[1];R=M.parents[1]',f'M=Path({str(M)!r});R=M.parents[1]').replace('rounding-scalar','public-fixed-paint').replace('validation_rounding_recipe','validation_fixedpaint_recipe')
 assert effective!=source
 shutil.copy2(original,out/'original-builder.py');(out/'builder.py').write_text(effective)
 for name in ['public-fixed-paint-experiment.py','public-fixed-paint-bridge.rs','public-fixed-paint-capture.mjs','pixels.mjs']:
  shutil.copy2(M/'validation'/name,out/name)
 write(out/'builder-adaptation.json',{'originalSha256':sha(original),'effectiveSha256':sha(out/'builder.py'),'diff':''.join(difflib.unified_diff(source.splitlines(True),effective.splitlines(True)))})
 def rect(l,t,r,b,c):return dict(left=l,top=t,right=r,bottom=b,color=c)
 a=rect(-11,-7,80,51,0x80ee5533);b=rect(21,15,111,76,0x802167b1)
 groups=[('signed-clipped',[rect(-20,-10,41,31,0xffffcc22),rect(70,-5,171,55,0xff1f9d76)]),('half-centers',[rect(3,5,44,36,0xff2167b1),rect(50,9,79,62,0x80ee5533)]),('alpha-a-then-b',[a,b]),('alpha-b-then-a',[b,a]),('full-envelope',[rect(-16384,-16384,16384,16384,0xff1f9d76),rect(30,11,67,44,0x80ee5533)]),('invisible-and-empty',[rect(0,0,80,80,0xffffcc22),rect(10,10,60,60,0x002167b1),rect(4,0,4,80,0xffff0000),rect(40,40,20,60,0xffff0000),rect(0,20,60,20,0xffff0000)])]
 cases=[dict(name=n,width=160,height=120,rects=rs,viewports=[[160,120],[64,80],[240,40],[160,120]])for n,rs in groups];write(out/'cases.json',cases)
 write(out/'prepared.json',{'scope':'direct primitive descriptors; no CSS-folding or runtime change','files':{p.name:sha(p)for p in out.iterdir()if p.is_file()}})
 print('Prepared; build/capture require separate coordinated invocations.')
elif phase=='build':
 prepared=json.loads((out/'prepared.json').read_text())
 for name,h in prepared['files'].items():assert sha(out/name)==h
 run(['python3',str(out/'builder.py'),str(out/'constructor')],out/'build.log')
 print('Frozen compiler-only constructor build complete.')
elif phase=='capture':
 cases=json.loads((out/'cases.json').read_text());probe=M/'output/wrapped-snapped-gate-r1/node-probe';renderer=M/'output/immutable-baseline-toolchain-r2/renderer-replay';candidate=out/'constructor/candidate'
 expected=json.loads((M/'output/playwright/public-wrapping-r2/receipt.json').read_text())['toolHashes']
 assert sha(probe)==expected['probe'] and sha(renderer)==expected['renderer']
 for c in cases:
  p=out/'cases'/c['name'];p.mkdir(parents=True,exist_ok=False);write(p/'recipe.json',{k:c[k]for k in ['width','height','rects']})
  for trial in ['first','repeat']:run([str(candidate),str(p/'recipe.json'),str(p/trial)],p/(trial+'.log'))
  assert (p/'first/scene.riv').read_bytes()==(p/'repeat/scene.riv').read_bytes()
  assert (p/'first/binding.json').read_bytes()==(p/'repeat/binding.json').read_bytes()
  run([str(probe),str(p/'first/scene.riv'),str(p/'probe'),*[f'{w}x{h}'for w,h in c['viewports']]],p/'probe.log')
  frames=json.loads((p/'probe/frames.json').read_text())['frames'];assert len(frames)==8
  for f in frames:
   cmd=[str(renderer),'--stream',str(p/'probe'/f['stream']),'--output',str(p/f'frame-{f["frame"]}.native.png'),'--backend','rust-metal','--mode','clockwise-atomic','--frame',str(f['frame'])]
   run(cmd,p/f'frame-{f["frame"]}.native.log')
   for name,value in [('cyan','0xff00ffff'),('transparent','0x00000000')]:
    clear=cmd.copy();clear[clear.index('--output')+1]=str(p/f'frame-{f["frame"]}.{name}.png');run(clear+['--clear',value],p/f'frame-{f["frame"]}.{name}.log')
 # Run the repository helper so Node resolves the existing compiler-only dependencies;
 # its frozen bytes and unchanged pixel comparison are checked immediately before execution.
 for name in ['public-fixed-paint-capture.mjs','pixels.mjs']:assert sha(M/'validation'/name)==sha(out/name)
 run(['node',str(M/'validation/public-fixed-paint-capture.mjs'),str(out)],out/'capture.log',M)
 # Every returned/cloned image must exactly reproduce the corresponding first occurrence.
 visual=json.loads((out/'visual-receipt.json').read_text());rows={(r['name'],r['frame']):r for r in visual['rows']};repeats=[]
 for c in cases:
  for frame,first in [(3,0),(4,0),(5,1),(6,2),(7,0)]:
   a=rows[c['name'],frame];b=rows[c['name'],first]
   assert a['chromeSha256']==b['chromeSha256'] and a['nativeSha256']==b['nativeSha256']
   repeats.append(dict(name=c['name'],frame=frame,first=first,chrome=a['chromeSha256'],native=a['nativeSha256']))
 write(out/'repeat-identity.json',repeats)
 from PIL import Image,ImageDraw
 visualdir=out/'visual';visualdir.mkdir();sheets=[]
 for c in cases:
  triples=[]
  for frame in range(3):
   prefix=out/'cases'/c['name']/f'frame-{frame}';files=[Path(str(prefix)+'.'+kind+'.png')for kind in ['chrome','native','diff']];ims=[Image.open(f).convert('RGB')for f in files];triples.append((frame,files,ims))
  width=max(sum(im.width for im in ims)+32 for _,_,ims in triples);height=sum(ims[0].height+32 for _,_,ims in triples)
  sheet=Image.new('RGB',(width,height),'#ddd');draw=ImageDraw.Draw(sheet);y=0;members=[]
  for frame,files,ims in triples:
   draw.text((8,y+4),f"{c['name']} frame {frame}: Chrome / native / diff; unscaled",fill='black');x=8
   for im in ims:sheet.paste(im,(x,y+24));x+=im.width+8
   members.append(dict(frame=frame,files=[dict(path=str(f),sha256=sha(f))for f in files]));y+=ims[0].height+32
  target=visualdir/(c['name']+'.png');sheet.save(target);sheets.append(dict(path=str(target),sha256=sha(target),members=members))
 write(visualdir/'coverage.json',dict(scope='18 unscaled representative triples; direct review pending',sheets=sheets,repeats=repeats))
 (out/'gallery.html').write_text('<!doctype html><meta charset="utf-8"><title>Private fixed paint primitives</title><h1>Direct primitives; no public CSS folding</h1>'+''.join(f'<h2>{c["name"]}</h2><img src="visual/{c["name"]}.png">'for c in cases))
 write(out/'receipt.json',{'scope':'ordinary fixed_paint primitive import/clone/resize/render only; no source proof or public folding','tools':{str(p):sha(p)for p in [probe,renderer,candidate]},'artifacts':{str(p.relative_to(out)):sha(p)for p in sorted(out.rglob('*'))if p.is_file() and 'target' not in p.parts}})
 print('Direct primitive capture complete; inspect visual-receipt.json and PNGs.')
else:raise SystemExit('prepare|build|capture required')
