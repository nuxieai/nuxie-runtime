"""Observe all positive predicate stages in immutable ordinary import/clone/resize."""
import json,hashlib,struct,subprocess,shutil,sys
from pathlib import Path
M=Path(__file__).resolve().parents[1];root=Path(sys.argv[1]).resolve()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
f32=lambda x:struct.unpack('<f',struct.pack('<f',x))[0]
bits=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
value=lambda b:struct.unpack('<f',struct.pack('<I',b))[0]
probe=M/'output/wrapped-snapped-gate-r1/node-probe';assert sha(probe)=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53';shutil.copy2(probe,root/'node-probe');shutil.copy2(__file__,root/'runner.py')
static=[('min-subnormal',1),('next-subnormal',2),('min-normal',0x00800000),('pow-minus80',bits(2**-80)),('pow-minus32',bits(2**-32)),('zero',0),('minus-min-subnormal',0x80000001),('minus-one',bits(-1.)),('one',bits(1.)),('max-finite',0x7f7fffff)]
rows=[];cases=[];failures=[];commands=[]
for axis in ['x','y']:
 for name,b,dynamic in [(n,b,False)for n,b in static]+[('dynamic-through-zero',0,True)]:
  name=axis+'-'+name;folder=root/'cases'/name;folder.mkdir(parents=True,exist_ok=False);recipe=dict(axis=axis,inputBits=b,dynamic=dynamic);p=folder/'recipe.json';p.write_text(json.dumps(recipe,indent=2)+'\n');cmd=[str(root/'candidate'),str(p),str(folder)];r=subprocess.run(cmd,capture_output=True,text=True);(folder/'constructor.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr;commands.append(cmd)
  sizes=[(100,100),(200,200),(40,40),(100,100),(101,101),(99,99),(100,100),(200,200)]
  cmd=[str(root/'node-probe'),str(folder/'scene.riv'),str(folder/'native'),*[f'{w}x{h}'for w,h in sizes]];r=subprocess.run(cmd,capture_output=True,text=True);(folder/'probe.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr;commands.append(cmd)
  trace=json.loads((folder/'trace.json').read_text());frames=json.loads((folder/'native/frames.json').read_text())['frames'];observations=[]
  for f in frames:
   objects=json.loads((folder/'native'/f['geometry']).read_text());a={o['objectId']:o for o in objects};index=4+(axis=='y');source=(sizes[f['step']][axis=='y']*.5-50)if dynamic else value(b);expected=[source,max(0.,min(1.,source))]
   for _ in range(3):expected.append(max(0.,min(1.,f32(expected[-1]*(2**64)))))
   ids=[trace['source']]+trace['stages'];got=[a[i]['worldMatrix'][index]for i in ids];expectedbits=[bits(v)for v in expected];gotbits=[bits(v)for v in got];errors=[dict(stage=i,expected=expected[i],actual=got[i],expectedBits=expectedbits[i],actualBits=gotbits[i])for i in range(5)if expectedbits[i]!=gotbits[i]]
   row=dict(case=name,frame=f['frame'],instance=f['instance'],step=f['step'],source=source,expected=expected,actual=got,expectedBits=expectedbits,actualBits=gotbits,errors=errors);rows.append(row);observations.append(gotbits)
   if errors:failures.append(row)
  clone=observations[:8]==observations[8:];repeat=all(observations[i]==observations[0]for i in[3,6,8,11,14]);cases.append(dict(name=name,recipe=recipe,frames=len(frames),originalCloneExact=clone,repeatExact=repeat,sceneSha256=sha(folder/'scene.riv')))
for b in json.loads((root/'source-bindings.json').read_text()):assert sha(b['source'])==sha(b['snapshot'])==b['sha256']
(root/'observations.json').write_text(json.dumps(rows,indent=2)+'\n');artifacts=[dict(path=str(p),sha256=sha(p))for p in sorted((root/'cases').rglob('*'))if p.is_file()]
(root/'native-receipt.json').write_text(json.dumps(dict(scope='Private positive predicate stage observation; no public qualification or rendering claim',cases=cases,frames=len(rows),failures=failures,commands=commands,probeSha256=sha(root/'node-probe'),constructorSha256=sha(root/'candidate'),runnerSha256=sha(root/'runner.py'),observationsSha256=sha(root/'observations.json'),artifacts=artifacts),indent=2)+'\n');print(json.dumps(dict(cases=len(cases),frames=len(rows),failedFrames=len(failures))))
