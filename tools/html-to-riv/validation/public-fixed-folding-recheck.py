"""Offline exact r2 evaluator recheck against immutable retained r1 captures."""
from pathlib import Path
import json,hashlib,struct
M=Path(__file__).resolve().parents[1];captures=M/'output/playwright/public-fixed-folding-r1';out=M/'output/playwright/public-fixed-folding-r2'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
bind=lambda p:dict(path=str(Path(p).resolve()),sha256=sha(p))
def write(p,j):
 assert not p.exists(),str(p)
 p.write_text(json.dumps(j,indent=2)+'\n')
transfers=[]
for c in json.loads((out/'cases.json').read_text()):
 name=c['name']
 for rel in ['recipe.json','first/original.riv','first/scene.riv','first/base.riv','first/scene.map.json','first/folding.json']:
  a=captures/name/rel;b=out/name/rel
  assert a.read_bytes()==b.read_bytes(),(name,rel)
  transfers.append(dict(original=bind(a),current=bind(b)))
assert (captures/'cases.json').read_bytes()==(out/'cases.json').read_bytes()
write(out/'capture-transfer-r2.json',dict(scope='Exact r1/r2 source recipe and scene transfer, evaluation intentionally recomputed',artifacts=transfers,oldCases=bind(captures/'cases.json'),newCases=bind(out/'cases.json'),priorFailure=bind(captures/'verification-r1.log')))
def read(p):return json.loads(p.read_text(),parse_int=lambda s:-0.0 if s=='-0'else int(s))
def bits(x):return struct.unpack('<I',struct.pack('<f',x))[0]
rows=[];failures=[];count=0
for c in read(out/'cases.json'):
 name=c['name'];evaluation=read(out/name/'first/evaluation.json');values=evaluation['worlds'];assert len({v['objectId']for v in values})==len(values)
 om=captures/'original-capture'/name/'probe';fm=captures/'folded-capture'/name/'probe';oldframes=read(om/'frames.json')['frames'];newframes=read(fm/'frames.json')['frames'];assert len(oldframes)==len(newframes)==8
 source_ids={n['object_id']for n in read(out/name/'first/scene.map.json')}
 for i in range(8):
  a=oldframes[i];b=newframes[i];assert [a[k]for k in ['frame','instance','step','width','height']]==[b[k]for k in ['frame','instance','step','width','height']]
  old=read(om/a['geometry']);new=read(fm/b['geometry']);assert len({o['objectId']for o in old})==len(old);assert len({o['objectId']for o in new})==len(new)
  old={o['objectId']:o for o in old};new={o['objectId']:o for o in new};prefix={id for id in old if id<evaluation['sizingEnd']-1};assert prefix=={id for id in new if id<evaluation['sizingEnd']-1};assert source_ids<=prefix
  delta=[]
  image_bindings={}
  for kind in ['native','chrome']:
   pa=captures/'original-capture'/name/f'frame-{i}.{kind}.png';pb=captures/'folded-capture'/name/f'frame-{i}.{kind}.png'
   image_bindings[kind]=dict(original=bind(pa),folded=bind(pb),identical=pa.read_bytes()==pb.read_bytes())
   if pa.read_bytes()!=pb.read_bytes():delta.append(dict(kind=kind+'-image-bytes',original=sha(pa),folded=sha(pb)))
  for id in sorted(prefix):
   ao=old[id];bo=new[id];assert len(ao['worldMatrix'])==len(bo['worldMatrix'])==6
   av=[ao['width'],ao['height'],*ao['worldMatrix']];bv=[bo['width'],bo['height'],*bo['worldMatrix']]
   for j in range(8):
    count+=1
    if bits(av[j])!=bits(bv[j]):delta.append(dict(kind='prefix',objectId=id,field=j,original=bits(av[j]),folded=bits(bv[j])))
  assert {v['objectId']for v in values}==set(old), 'Evaluator/native owner set mismatch'
  for v in values:
   assert len(v['worldBits'])==6;id=v['objectId'];assert id in old
   for axis in range(6):
    count+=1;actual=bits(old[id]['worldMatrix'][axis])
    if v['worldBits'][axis]!=actual:delta.append(dict(kind='evaluator',objectId=id,axis=axis,expected=v['worldBits'][axis],actual=actual))
  row=dict(name=name,frame=i,prefixOwners=len(prefix),evaluatedOwners=len(values),images=image_bindings,differences=delta);rows.append(row)
  if delta:failures.append(row)
write(out/'verification-observations.json',rows);write(out/'verification.json',dict(scope='Exact unchanged prefix geometry and original-native/evaluator full world matrices; separate from pixel qualification',frames=len(rows),scalarChecks=count,failedFrames=len(failures),failures=failures,observations=bind(out/'verification-observations.json'),script=bind(__file__)))
print(json.dumps(dict(frames=len(rows),scalarChecks=count,failedFrames=len(failures))))
raise SystemExit(1 if failures else 0)
