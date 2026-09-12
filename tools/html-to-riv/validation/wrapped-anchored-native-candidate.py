"""Direct ordinary-record semantic experiment; no Chrome or public qualification."""
from pathlib import Path
import hashlib,itertools,json,shutil,subprocess,sys
M=Path(__file__).resolve().parents[1]
root=Path(sys.argv[1]).resolve();root.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def write(p,v):Path(p).write_text(json.dumps(v,indent=2)+'\n')
bindings=[]
def freeze(source,target):
 source=Path(source);target=Path(target);target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target)
 bindings.append({'path':str(source.resolve()),'snapshot':str(target),'sha256':sha(source)});assert sha(source)==sha(target)
for p in sorted((M/'src').rglob('*')):
 if p.is_file():freeze(p,root/'inputs/src'/p.relative_to(M/'src'))
for n in ['Cargo.toml','Cargo.lock']:freeze(M/n,root/'inputs'/n)
for n in ['wrapping.rs','wire.rs']:freeze(M/'src'/n,root/'sources'/n)
freeze(M/'validation/wrapped-anchored-native-harness.rs',root/'sources/harness.rs');freeze(__file__,root/'candidate-builder.py')
probe=M/'output/wrapped-snapped-gate-r1/node-probe'
assert sha(probe)=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
freeze(probe,root/'node-probe')
command=['cargo','build','--manifest-path',str(M/'Cargo.toml'),'--locked','--lib','--message-format=json']
r=subprocess.run(command,cwd=M,capture_output=True,text=True);(root/'cargo-build.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
libs={}
for s in r.stdout.splitlines():
 if not s.startswith('{'):continue
 a=json.loads(s)
 if a.get('reason')!='compiler-artifact':continue
 for f in a['filenames']:
  if Path(f).suffix in ['.rlib','.rmeta','.dylib','.so']:
   dest=root/'libraries'/Path(f).name;freeze(f,dest)
   if Path(f).suffix=='.rlib':libs[a['target']['name']]=dest
cmd=['rustc','--edition=2024',str(root/'sources/harness.rs'),'--extern','nuxie_schema='+str(libs['nuxie_schema']),'--extern','nuxie_html_to_riv='+str(libs['nuxie_html_to_riv']),'-L','dependency='+str(root/'libraries'),'-C','debuginfo=0','-o',str(root/'candidate')]
r=subprocess.run(cmd,cwd=M,capture_output=True,text=True);(root/'harness-build.log').write_text(r.stdout+r.stderr);assert r.returncode==0,r.stderr
cases=[];failures=[];rows=[]
for axis,reverse,wrap,level,mode in itertools.product(['row','column'],[0,1],[1,2],[0,1,2],['fixed','percent']):
 name=f'{axis}-reverse{reverse}-wrap{wrap}-level{level}-{mode}';dest=root/'cases'/name;dest.mkdir(parents=True)
 args=[str(root/'candidate'),str(dest),axis,str(reverse),str(wrap),str(level),mode]
 run=subprocess.run(args,capture_output=True,text=True)
 (dest/"candidate.log").write_text(run.stdout+run.stderr);assert run.returncode==0,run.stderr
 # Main-axis widths force 1, 2 and 3 lines; both viewport dimensions vary.
 sizes=[(220,240),(130,180),(70,280),(220,240)]
 if axis=='column':sizes=[(h,w)for w,h in sizes]
 for kind in ['base','scene']:
  run=subprocess.run([str(root/'node-probe'),str(dest/f'{kind}.riv'),str(dest/kind),*[f'{w}x{h}'for w,h in sizes]],capture_output=True,text=True)
  (dest/f'{kind}.log').write_text(run.stdout+run.stderr);assert run.returncode==0,run.stderr
  assert sha(dest/f'{kind}.riv')==sha(dest/kind/'scene.riv')
 trace=json.loads((dest/'trace.json').read_text());frames=json.loads((dest/'scene/frames.json').read_text())['frames'];assert len(frames)==8
 main=0 if axis=='row' else 1;cross=1-main;extent=['width','height'][cross];mainextent=['width','height'][main];f=level/2 if wrap==1 else 1-level/2
 observations=[]
 for frame in frames:
  fi=frame['frame'];actual=json.loads((dest/'scene'/frame['geometry']).read_text());base=json.loads((dest/'base'/frame['geometry']).read_text());A={o['objectId']:o for o in actual};B={o['objectId']:o for o in base}
  groups=[[]];used=0
  for i,s in enumerate(trace['slots']):
   width=B[s][mainextent]
   if groups[-1] and used+width>sizes[frame['step']][main]:groups.append([]);used=0
   groups[-1].append(i);used+=width
  errors=[];observed=[]
  for group in groups:
   maximum=max(B[trace['slots'][i]][extent] for i in group)
   for i in group:
    s=trace['slots'][i];v=trace['visible'][i];a=[0,.5,1][i];a=a if wrap==1 else 1-a
    # Independent algebra from unaugmented native slots and child used sizes.
    origin=B[s]['worldMatrix'][4+cross]-(maximum-B[s][extent])*f
    expected=origin+(maximum-B[v][extent])*a
    got=A[v]['worldMatrix'][4+cross]
    if abs(expected-got)>.1:errors.append({'item':i,'expected':expected,'actual':got,'delta':got-expected})
    assert A[s]==B[s],(name,fi,'slot mutation')
    assert A[v]['width']==B[v]['width'] and A[v]['height']==B[v]['height'],(name,fi,'visible size mutation')
    assert A[v]['worldMatrix'][4+main]==B[v]['worldMatrix'][4+main],(name,fi,'main position mutation')
    observed.append({'item':i,'line':groups.index(group),'lineOrigin':origin,'lineMaximum':maximum,'visibleExtent':B[v][extent],'expected':expected,'actual':got})
  row={'case':name,'frame':fi,'instance':frame['instance'],'step':frame['step'],'lines':len(groups),'errors':errors,'observations':observed};rows.append(row);observations.append(actual)
  if errors:failures.append(row)
 assert observations[:4]==observations[4:];assert observations[0]==observations[3]
 cases.append({'name':name,'arguments':args[2:],'sizes':sizes,'frames':8})
assert all(sha(b['path'])==sha(b['snapshot'])==b['sha256']for b in bindings)
write(root/'observations.json',rows)
artifacts=[{'path':str(p),'sha256':sha(p)}for p in sorted(root.rglob('*'))if p.is_file()]
write(root/'receipt.json',{'scope':'Private ComponentOrigin ordinary-file native semantic probe; no Chrome, pixel or public support qualification','epsilon':1/64,'epsilonIsAdmissionCertificate':False,'geometryTolerance':.1,'cases':cases,'frames':len(rows),'failures':failures,'originalCloneRepeatExact':True,'baseSlotsVisibleSizesMainPositionsExact':True,'bindings':bindings,'artifacts':artifacts,'commands':[command,cmd]})
print(json.dumps({'root':str(root),'cases':len(cases),'frames':len(rows),'failures':len(failures)}))
