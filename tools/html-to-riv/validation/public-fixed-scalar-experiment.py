"""Compare compiler-owned f32 results to frozen native observations, never seed them."""
from pathlib import Path
import hashlib,itertools,json,shutil,struct,subprocess,sys
M=Path(__file__).resolve().parents[1]
root=Path(sys.argv[1]).resolve()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
bits=lambda v:struct.unpack('<I',struct.pack('<f',float(v)))[0]
value=lambda b:struct.unpack('<f',struct.pack('<I',b))[0]
# Rust Display emits roundtrip f32 decimals, including integer spelling '-0'.
# Preserve that sign when decoding JSON (default Python int parsing loses it).
def load(p):return json.loads(Path(p).read_text(),parse_int=lambda t:-0.0 if t=='-0'else int(t))
probe=M/'output/wrapped-snapped-gate-r1/node-probe'
assert sha(probe)=='2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53'
shutil.copy2(probe,root/'node-probe');shutil.copy2(__file__,root/'experiment.py')
recipes=[]
scalar_bits=[0,0x80000000,1,0x80000001,bits(-16384),bits(16384),bits(-2.5),bits(2.5)]
for signed in [-.5,.5]:scalar_bits.extend(bits(signed)+d for d in [-1,0,1])
for axis,b in itertools.product([0,1],scalar_bits):recipes.append(dict(kind='round',axis=axis,inputBits=b,sourceSpace=0,destSpace=0,clampSpace=0))
for axis,source,dest,clamp,kind in itertools.product([0,1],[0,1],[0,1],[0,1],['spaces','multiple']):
 recipes.append(dict(kind=kind,axis=axis,inputBits=bits(5.0703125),sourceSpace=source,destSpace=dest,clampSpace=clamp))
commands=[];cases=[];mismatches=[];rows=[];checks=0;zero_sign_differences=0
sizes=[(100,100),(60,160),(160,60),(100,100)]
def run(cmd,log):
 commands.append(cmd);r=subprocess.run(cmd,capture_output=True,text=True);Path(log).write_text(r.stdout+r.stderr)
 assert r.returncode==0,(cmd,r.returncode,r.stderr)
for i,q in enumerate(recipes):
 folder=root/'cases'/f'{i:03}-{q["kind"]}-{q["axis"]}';folder.mkdir(parents=True,exist_ok=False)
 p=folder/'recipe.json';p.write_text(json.dumps(q,indent=2)+'\n')
 for name in ['first','repeat']:run([str(root/'candidate'),str(p),str(folder/name)],folder/f'{name}.log')
 for name in ['scene.riv','evaluation.json']:assert sha(folder/'first'/name)==sha(folder/'repeat'/name)
 e=load(folder/'first/evaluation.json');expected={v['objectId']:v['bits']for v in e['values']}
 run([str(root/'node-probe'),str(folder/'first/scene.riv'),str(folder/'native'),*[f'{w}x{h}'for w,h in sizes]],folder/'native.log')
 frames=load(folder/'native/frames.json')['frames'];assert len(frames)==8;frame_bits=[]
 for j,frame in enumerate(frames):
  assert frame['instance']==j//4 and frame['step']==j%4
  assert [frame['width'],frame['height']]==list(sizes[j%4])
  objects=load(folder/'native'/frame['geometry']);actual={v['objectId']:[bits(x)for x in v['worldMatrix'][4:6]]for v in objects}
  assert actual.keys()==expected.keys(),(actual.keys(),expected.keys())
  differences=[]
  for id,pair in expected.items():
   for axis,b in enumerate(pair):
    checks+=1;a=actual[id][axis]
    if a!=b:
     zero_only=value(a)==value(b)==0;zero_sign_differences+=int(zero_only)
     differences.append(dict(objectId=id,axis=axis,expectedBits=b,actualBits=a,zeroSignOnly=zero_only))
  row=dict(case=folder.name,frame=j,instance=j//4,step=j%4,coordinates=len(expected)*2,mismatches=differences)
  rows.append(row)
  if differences:mismatches.append(row)
  frame_bits.append(actual)
 assert frame_bits[0]==frame_bits[3] and frame_bits[:4]==frame_bits[4:]
 cases.append(dict(name=folder.name,recipe=q,recordCount=e['recordCount'],nodes=len(expected),frames=8,deterministic=True,originalCloneExact=True,repeatExact=True,scene=bind(folder/'first/scene.riv'),evaluation=bind(folder/'first/evaluation.json')))
(root/'observations.json').write_text(json.dumps(rows,indent=2)+'\n')
for b in load(root/'source-bindings.json'):assert sha(b['snapshot'])==b['sha256']
artifacts=[bind(p)for p in sorted((root/'cases').rglob('*'))if p.is_file()]
receipt=dict(scope='Private compiler f32 evaluation against immutable native Node observations; no geometry seeds, browser or public folding qualification',status='completed-native-comparison',cases=cases,frames=len(rows),coordinates=checks,mismatchFrames=len(mismatches),mismatchCoordinates=sum(len(r['mismatches'])for r in mismatches),zeroSignDifferences=zero_sign_differences,mismatches=mismatches,commands=commands,constructor=bind(root/'candidate'),probe=bind(root/'node-probe'),probeSource=bind(M/'output/wrapped-snapped-gate-r1/node-probe.rs'),experiment=bind(root/'experiment.py'),observations=bind(root/'observations.json'),artifacts=artifacts)
(root/'native-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k]for k in ['frames','coordinates','mismatchFrames','mismatchCoordinates','zeroSignDifferences']}))
