"""Offline sign-preserving verification of retained native seed observations."""
from pathlib import Path
import hashlib,json,struct,sys
M=Path(__file__).resolve().parents[1];old=M/'output/public-fixed-geometry-native-r1';out=M/'output/public-fixed-geometry-native-r2';out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
def integer(text):return -0.0 if text=='-0'else int(text)
def read(p):return json.loads(Path(p).read_text(),parse_int=integer)
def bits(x):return struct.unpack('<I',struct.pack('<f',x))[0]
receipt=read(old/'receipt.json')
for a in receipt['artifacts']:assert sha(a['path'])==a['sha256'],a['path']
rows=[];failures=[];checks=0;negative_zero_count=0
for name,_ in read(old/'cases.json'):
 folder=old/name;seeds=read(folder/'first/seeds.json')['seeds'];ids=[s['objectId']for s in seeds];assert len(ids)==len(set(ids))
 for f in read(folder/'native/frames.json')['frames']:
  objects=read(folder/'native'/f['geometry']);nativeids=[o['objectId']for o in objects];assert len(nativeids)==len(set(nativeids))
  # These closed base recipes contain only Artboard plus the certified parent,
  # slots and visible layout nodes. Fill/style/color records are not probe nodes.
  assert set(nativeids)==set(ids)|{0},(name,f['frame'],nativeids,ids)
  byid={o['objectId']:o for o in objects};delta=[]
  for s in seeds:
   o=byid[s['objectId']]
   for label,expected,observed,required in [('size',s['sizeBits'],[o['width'],o['height']],2),('world',s['worldBits'],o['worldMatrix'],6)]:
    assert len(expected)==len(observed)==required
    for i in range(required):
     a=expected[i];b=bits(observed[i]);checks+=1;negative_zero_count+=int(b==0x80000000)
     if a!=b:delta.append(dict(objectId=s['objectId'],field=label,index=i,expectedBits=a,actualBits=b,signedZeroOnly={a,b}=={0,0x80000000}))
  row=dict(name=name,frame=f['frame'],differences=delta);rows.append(row)
  if delta:failures.append(row)
(out/'observations.json').write_text(json.dumps(rows,indent=2)+'\n')
summary=dict(scope='Offline sign-preserving exact initial-seed verification; no new construction or native execution',parentReceipt=bind(old/'receipt.json'),originalExperimentSnapshot=bind(old/'experiment.py'),verifier=bind(__file__),observations=bind(out/'observations.json'),cases=len(read(old/'cases.json')),frames=len(rows),scalarChecks=checks,nativeNegativeZeroScalars=negative_zero_count,failedFrames=len(failures),failures=failures,shapeAndOwnerSetsVerified=True,priorArtifactBindingsVerified=len(receipt['artifacts']))
(out/'receipt.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:summary[k]for k in ['cases','frames','scalarChecks','nativeNegativeZeroScalars','failedFrames','priorArtifactBindingsVerified']}))
