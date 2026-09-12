"""Observe all signed rounding stages in immutable ordinary import/clone/resize."""
import json,hashlib,struct,subprocess,shutil,sys
from pathlib import Path
M=Path(__file__).resolve().parents[1];root=Path(sys.argv[1]).resolve()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
bits=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
value=lambda b:struct.unpack('<f',struct.pack('<I',b))[0]
probe=M/'output/wrapped-snapped-gate-r1/node-probe';assert sha(probe)=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53';shutil.copy2(probe,root/'node-probe');shutil.copy2(__file__,root/'runner.py')
static=[('bits-'+hex(b),b)for b in [0,0x80000000,1,0x80000001,0x00800000,0x80800000]]
for x in [.25,.5,.75,1.5,2.5,8191.5,16383.5,16384.]:
 for sign in [1,-1]:
  b=bits(sign*x)
  for d in [-1,0,1]:
   if abs(value(b+d))<=16384:static.append(('bits-'+hex(b+d),b+d))
rows=[];cases=[];failures=[];commands=[]
for axis in ['x','y']:
 for name,b,dynamic in [(n,b,False)for n,b in static]+[('dynamic-through-zero',0,True)]:
  name=axis+'-'+name;folder=root/'cases'/name;folder.mkdir(parents=True,exist_ok=False);recipe=dict(axis=axis,inputBits=b,dynamic=dynamic);p=folder/'recipe.json';p.write_text(json.dumps(recipe,indent=2)+'\n');cmd=[str(root/'candidate'),str(p),str(folder)];r=subprocess.run(cmd,capture_output=True,text=True);(folder/'constructor.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr;commands.append(cmd)
  sizes=[(100,100),(200,200),(40,40),(100,100),(101,101),(99,99),(100,100),(200,200)]
  cmd=[str(root/'node-probe'),str(folder/'scene.riv'),str(folder/'native'),*[f'{w}x{h}'for w,h in sizes]];r=subprocess.run(cmd,capture_output=True,text=True);(folder/'probe.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr;commands.append(cmd)
  trace=json.loads((folder/'trace.json').read_text());frames=json.loads((folder/'native/frames.json').read_text())['frames'];observations=[]
  for f in frames:
   objects=json.loads((folder/'native'/f['geometry']).read_text());a={o['objectId']:o for o in objects};index=4+(axis=='y');source=(sizes[f['step']][axis=='y']*.5-50)if dynamic else value(b)
   checks=[]
   def check(label,id,expected):
    actual=f32(a[id]['worldMatrix'][index]);checks.append(dict(label=label,id=id,expected=expected,actual=actual,pass_=actual==expected))
   check('source',trace['source'],source)
   # The reference uses exact rational values from the IEEE source bits and
   # integer half-thresholds, independent of emitted graph arithmetic.
   from fractions import Fraction
   exact=Fraction(source); results=[]
   for negative,label in [(False,'positive'),(True,'negative')]:
    accumulator=0
    for step in trace[label]:
     p=step['power'];threshold=Fraction(2*(accumulator+p)-1,2)
     difference=(-exact-threshold)if negative else(threshold-exact)
     expected=f32(float(difference));check(label+' threshold',step['threshold'],float(threshold));check(label+' difference',step['difference'],expected)
     stage=max(0.,min(1.,expected))
     for i,id in enumerate(step['stages']):
      if i:stage=max(0.,min(1.,f32(stage*2**64)))
      check(label+' predicate '+str(i),id,stage)
     bit=int((-exact>threshold)if negative else(exact>=threshold));accumulator+=p*bit
     check(label+' bit',step['bit'],bit);check(label+' accumulator',step['accumulator'],accumulator)
    results.append(accumulator)
   rounded=(exact+Fraction(1,2)).numerator//(exact+Fraction(1,2)).denominator
   assert results[0]-results[1]==rounded
   check('rounded',trace['rounded'],rounded)
   errors=[c for c in checks if not c['pass_']];row=dict(case=name,frame=f['frame'],instance=f['instance'],step=f['step'],source=source,checks=checks,errors=errors);rows.append(row);observations.append([c['actual']for c in checks])
   if errors:failures.append(row)
  clone=observations[:8]==observations[8:];repeat=all(observations[i]==observations[0]for i in[3,6,8,11,14]);cases.append(dict(name=name,recipe=recipe,frames=len(frames),originalCloneExact=clone,repeatExact=repeat,sceneSha256=sha(folder/'scene.riv')))
for b in json.loads((root/'source-bindings.json').read_text()):assert sha(b['source'])==sha(b['snapshot'])==b['sha256']
(root/'observations.json').write_text(json.dumps(rows,indent=2)+'\n');artifacts=[dict(path=str(p),sha256=sha(p))for p in sorted((root/'cases').rglob('*'))if p.is_file()]
(root/'native-receipt.json').write_text(json.dumps(dict(scope='Private signed rounding all-stage observation; no public qualification or rendering claim',cases=cases,frames=len(rows),failures=failures,commands=commands,probeSha256=sha(root/'node-probe'),constructorSha256=sha(root/'candidate'),runnerSha256=sha(root/'runner.py'),observationsSha256=sha(root/'observations.json'),artifacts=artifacts),indent=2)+'\n');print(json.dumps(dict(cases=len(cases),frames=len(rows),failedFrames=len(failures))))
