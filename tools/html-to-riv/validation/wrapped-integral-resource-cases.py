"""Construct fixed-size owner-count resource cases with the existing frozen compiler.
No native timing: durations here include constructor process startup and artifact IO.
"""
import hashlib,json,shutil,subprocess,sys,time
from pathlib import Path
module=Path(__file__).resolve().parents[1]
constructor=(module/'output/wrapped-integral-constructor-r1').resolve()
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else module/'output/wrapped-integral-resource-r1'
assert not out.exists(),f'Preserve existing evidence: {out}'
out.mkdir(parents=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for name in ['candidate','source-bindings.json','receipt.json','bridge.rs','patches.json']:
 shutil.copy2(constructor/name,out/name)
shutil.copy2(__file__,out/'cases.py')
bindings=json.loads((out/'source-bindings.json').read_text())
for b in bindings:assert sha(b['snapshot'])==b['sha256'],b['snapshot']
def cost(n):return {'base':7+6*n,'sizing':58*n-33,'paint':(27*n*n-n-14)//2,'total':(27*n*n+127*n-66)//2}
limit=100000
n=1
while cost(n+1)['total']<=limit:n+=1
assert n==83 and cost(n)['total']==98239 and cost(n+1)['total']==100557
def dim(value,unit='px'):return dict(value=value,unit=unit,source=str(value),min=None,max=None)
def recipe(n):return dict(initialViewport=[320,240],row=True,reverseMain=False,wrap=1,lineFraction=0,mainFraction=0,parent=dict(width=dim(100,'%'),height=dim(100,'%')),slots=[dict(name=f'v{i}',main=dim(20),cross=dim(20),visibleMain=dim(20),visibleCross=dim(20),slotAlignment=0,alignment=0,color=[0x80ff6030,0xcc00a896,0xffeeee22][i%3]) for i in range(n)])
rows=[]
for count in [1,3,8,34,35,n,n+1]:
 name=f'owners-{count}';d=out/'cases'/name;d.mkdir(parents=True);(d/'recipe.json').write_text(json.dumps(recipe(count),indent=2)+'\n')
 runs=[]
 for run in ['first','repeat']:
  command=[str(out/'candidate'),str(d/'recipe.json'),str(d/run)]
  start=time.perf_counter_ns();result=subprocess.run(command,capture_output=True,text=True);elapsed=time.perf_counter_ns()-start
  log=d/f'{run}.log';log.write_text(result.stdout+result.stderr)
  runs.append(dict(command=command,exitCode=result.returncode,elapsedNs=elapsed,log={'path':str(log),'sha256':sha(log)}))
 row=dict(name=name,owners=count,expectedCosts=cost(count),recipe={'path':str(d/'recipe.json'),'sha256':sha(d/'recipe.json')},runs=runs)
 if count<=n:
  assert all(r['exitCode']==0 for r in runs),[Path(r['log']['path']).read_text() for r in runs]
  row['artifacts']={}
  for name in ['base.riv','scene.riv','scene.map.json','trace.json','proof.json']:
   assert sha(d/'first'/name)==sha(d/'repeat'/name),name
   shutil.copy2(d/'first'/name,d/name);row['artifacts'][name]={'path':str(d/name),'sha256':sha(d/name),'bytes':(d/name).stat().st_size}
  actual=json.loads((d/'proof.json').read_text())['recordCosts'];assert actual==[cost(count)[key] for key in ['base','sizing','paint']],actual
  row['deterministic']=True
 else:
  assert all(r['exitCode']!=0 for r in runs)
  assert all(Path(r['log']['path']).read_text().strip()=='Derived: ResourceBudget' for r in runs)
  assert all(not(d/run).exists() for run in ['first','repeat'])
  row['expectedRejection']='Derived: ResourceBudget';row['noOutputArtifacts']=True
 rows.append(row)
receipt=dict(scope='Frozen private Derived construction; not public qualification or native performance evidence.',budget=limit,maxOwners=n,oneOverOwners=n+1,costFormula='base=7+6N; sizing=58N-33; paint=(27N^2-N-14)/2; total=(27N^2+127N-66)/2, for N>0 and one distinct painted geometry per owner',constructor={'path':str(out/'candidate'),'sha256':sha(out/'candidate'),'origin':str(constructor/'candidate')},sourceBindings={'path':str(out/'source-bindings.json'),'sha256':sha(out/'source-bindings.json'),'verifiedSnapshots':len(bindings)},script={'path':str(out/'cases.py'),'sha256':sha(out/'cases.py')},copiedInputs=[{'path':str(out/name),'sha256':sha(out/name)} for name in ['receipt.json','bridge.rs','patches.json']],cases=rows,timingLimit='Two sequential constructor subprocess timings include process startup, validation, encoding and artifact IO; not a benchmark distribution or native lifecycle timing.')
(out/'construction-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'output':str(out),'maxOwners':n,'cases':[(r['owners'],r['expectedCosts']['total'],r['runs'][0]['exitCode'])for r in rows]}))
