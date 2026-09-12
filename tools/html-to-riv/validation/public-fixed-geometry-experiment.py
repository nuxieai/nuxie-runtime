"""Validate source-only initial seeds against frozen native observations.
Measurements are outputs only; they never enter the recipe or constructor.
"""
from pathlib import Path
import copy,hashlib,json,shutil,struct,subprocess,sys
M=Path(__file__).resolve().parents[1];O=Path(sys.argv[1]).resolve();B=Path(sys.argv[2]).resolve();O.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
bits=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
receipt=json.loads((B/'receipt.json').read_text());assert sha(B/'candidate')==receipt['candidateSha256']
probe=M/'output/wrapped-snapped-gate-r1/node-probe';assert sha(probe)=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
shutil.copy2(probe,O/'node-probe');shutil.copy2(B/'candidate',O/'candidate');shutil.copy2(__file__,O/'experiment.py')
def dim(x):return dict(value=x,source=str(x),unit='px',min=None,max=None)
def num(x):return dict(value=x,source=str(x),unit='px')
def recipe(row,reverse,wrap,line,main=0):
 return dict(initialViewport=[320,240],row=row,reverseMain=reverse,wrap=wrap,lineFraction=line,mainFraction=main,parent=dict(width=dim(40 if row else 100),height=dim(100 if row else 40)),slots=[dict(name=f'v{i}',main=dim(20),cross=dim(h),visibleMain=dim(20),visibleCross=dim(h),slotAlignment=0,alignment=0,color=0xff22aa99)for i,h in enumerate([10,30,20])])
cases=[]
for row in [True,False]:
 for reverse in [False,True]:
  for wrap in [1,2]:
   for f in [0,.5,1]:cases.append((f'axes-{int(row)}-{int(reverse)}-{wrap}-{f}',recipe(row,reverse,wrap,f)))
for row in [True,False]:
 q=recipe(row,True,2,.5,.5);q['slots'][0]['main']=dim(50);q['slots'][0]['visibleMain']=dim(15);cases.append((f'oversized-{row}',q))
 q=recipe(row,False,1,0);q['parent']['width'if row else'height']['min']=num(60);q['parent']['width'if row else'height']['max']=num(50);q['slots'][0]['main']['min']=num(30);q['slots'][0]['main']['max']=num(25);q['slots'][0]['visibleMain']=dim(10);q['slots'][0]['visibleCross']=dim(5);cases.append((f'minwins-{row}',q))
q=recipe(True,False,1,.5);q['parent']['height']=dim(1);q['slots']=q['slots'][:1];q['slots'][0]['cross']=dim(1/64);q['slots'][0]['visibleCross']=dim(1/64);cases.append(('odd-unit-center',q))
q=recipe(True,False,2,0);q['parent']['height']=dim(10);q['slots']=q['slots'][:1];q['slots'][0]['cross']=dim(30);q['slots'][0]['visibleCross']=dim(20);cases.append(('cross-overflow',q))
(O/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
sizes=[(320,240),(19,31),(16384,16384),(320,240)];results=[];failures=[];commands=[];checks=0
for name,q in cases:
 folder=O/name;folder.mkdir();r=folder/'recipe.json';r.write_text(json.dumps(q,indent=2)+'\n')
 for run in ['first','repeat']:
  command=[str(O/'candidate'),str(r),str(folder/run)];p=subprocess.run(command,capture_output=True);(folder/f'{run}.log').write_bytes(p.stdout+p.stderr);commands.append(dict(command=command,exitCode=p.returncode));assert p.returncode==0,(name,p.stderr)
 for artifact in ['scene.riv','seeds.json']:assert sha(folder/'first'/artifact)==sha(folder/'repeat'/artifact)
 command=[str(O/'node-probe'),str(folder/'first/scene.riv'),str(folder/'native'),*[f'{w}x{h}'for w,h in sizes]];p=subprocess.run(command,capture_output=True);(folder/'native.log').write_bytes(p.stdout+p.stderr);commands.append(dict(command=command,exitCode=p.returncode));assert p.returncode==0,(name,p.stderr)
 assert sha(folder/'first/scene.riv')==sha(folder/'native/scene.riv')
 seeds=json.loads((folder/'first/seeds.json').read_text())['seeds'];frames=json.loads((folder/'native/frames.json').read_text())['frames'];assert len(frames)==8
 for f in frames:
  assert f['instance']==f['frame']//4 and f['step']==f['frame']%4 and [f['width'],f['height']]==list(sizes[f['step']])
  objects={o['objectId']:o for o in json.loads((folder/'native'/f['geometry']).read_text())};delta=[]
  for s in seeds:
   actual=objects[s['objectId']]
   for label,expected,observed in [('size',s['sizeBits'],[actual['width'],actual['height']]),('world',s['worldBits'],actual['worldMatrix'])]:
    for i,(a,b)in enumerate(zip(expected,map(bits,observed))):
     checks+=1
     if a!=b:delta.append(dict(objectId=s['objectId'],field=label,index=i,expectedBits=a,actualBits=b,actual=observed[i]))
  entry=dict(name=name,frame=f['frame'],instance=f['instance'],step=f['step'],differences=delta);results.append(entry)
  if delta:failures.append(entry)
(O/'observations.json').write_text(json.dumps(results,indent=2)+'\n')
artifacts=[bind(p)for p in sorted(O.rglob('*'))if p.is_file()]
summary=dict(scope='Private initial fixed base geometry validation only; no final-constraint, paint or public qualification',cases=len(cases),frames=len(results),scalarChecks=checks,failedFrames=len(failures),failures=failures,constructor=bind(O/'candidate'),constructorReceipt=bind(B/'receipt.json'),probe=bind(O/'node-probe'),commands=commands,artifacts=artifacts)
(O/'receipt.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:summary[k]for k in ['cases','frames','scalarChecks','failedFrames']}))
